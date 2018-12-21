//! Provides a method `unix_time` which returns elapsed time since *UNIX EPOCH*. The returned time
//! can be faked in each thread separately.
//!
//! ## Compilation Time Switch
//!
//! Faketime can be disabled at compilation time to avoid runtime overhead and potential
//! vulnerability of manipulating the process time.
//!
//! It is disabled via rust cfg `disable_faketime`, e.g.:
//!
//! ```shell
//! RUSTFLAGS="--cfg disable_faketime" cargo build
//! ```
//!
//! By default, without setting the cfg, faketime is available for all the threads.
//!
//! ## Usage
//!
//! The faketime setting is per thread. If it is enabled in a thread, a file path is also
//! configured. The file should store the milliseconds since UNIX EPOCH. This function will first
//! try to read the time from this file, and fallback to the system time when an error occurs.
//!
//! The most straightforward way to enable faketime is calling `faketime::enable(path)` in the
//! thread, and `path` is the configured timestamp file. It also overrides the auto-detected
//! settings, as mentioned below.
//!
//! ```
//! assert_ne!(faketime::unix_time().as_secs(), 100);
//! let faketime_file = faketime::millis_tempfile(100_000).expect("create faketime file");
//! faketime::enable(&faketime_file);
//! assert_eq!(faketime::unix_time().as_secs(), 100);
//! ```
//!
//! In each thread, when this function is first invoked, and neither `faketime::enable(path)` nor
//! `faketime::disable()` has been invoked in the thread already, it will detect whether faketime
//! should be enabled and which timestamp file should be used.
//!
//! First, if the environment variable `FAKETIME` exists, faketime is enabled, and the timestamp
//! file path is the environment variable value.
//!
//! ```
//! use std::env;
//! use std::thread;
//!
//! let faketime_file = faketime::millis_tempfile(123_456).expect("create faketime file");
//! env::set_var("FAKETIME", faketime_file.as_os_str());
//!
//! thread::spawn(|| assert_eq!(123, faketime::unix_time().as_secs()))
//!     .join()
//!     .expect("join thread");
//! ```
//!
//! If the environment variable `FAKETIME` is missing, but this thread has a name and the name
//! starts with `FAKETIME=` literally, faketime is also enabled, and the timestamp file is the
//! portion of the thread name after `FAKETIME=`.
//!
//! ```
//! use std::thread;
//!
//! let faketime_file = faketime::millis_tempfile(123_456).expect("create faketime file");
//!
//! thread::Builder::new()
//!     .name(format!("FAKETIME={}", faketime_file.display()))
//!     .spawn(|| assert_eq!(123, faketime::unix_time().as_secs()))
//!     .expect("spawn thread")
//!     .join()
//!     .expect("join thread");
//! ```
//!
//! Otherwise, faketime is disabled in the thread, until it is manually enabled via
//! `faketime::enable(path)`.
//!
//! ```
//! use std::thread;
//!
//! let start = faketime::system::unix_time();
//! thread::spawn(move || assert!((faketime::unix_time() - start).as_secs() < 60))
//!     .join()
//!     .expect("join thread");
//! ```
//!
//! ## Atomic Write
//!
//! This function reads timestamp from the file when faketime is enabled. To ensure the written
//! content is read in a whole, it is recommended to write the file atomically. One solution is
//! writing to a temporary file and then moving to the target path, as implemented in
//! `faketime::write_millis(path, millis)`.
//!
//! Following snippet demonstrates how to write time to file /tmp/faketime.
//!
//! ```shell
//! cat 100 > /tmp/faketime_
//! mv /tmp/faketime_ /tmp/faketime
//! ```

use crate::system::unix_time as system_unix_time;
use std::cell::{Cell, RefCell};
use std::env;
use std::fs;
use std::io::{Error, ErrorKind, Write};
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tempfile::{NamedTempFile, TempPath};

pub use std::io::Result;

thread_local! {
    /// Some(true): Enabled
    /// Some(false): Disabled
    /// None: Undecided
    static FAKETIME_ENABLED: Cell<Option<bool>> = Cell::new(None);
    static FAKETIME_PATH: RefCell<PathBuf> = Default::default();
}

const KEY_FAKETIME: &str = "FAKETIME";
const PREFIX_FAKETIME_EQ: &str = "FAKETIME=";

/// Gets elapsed time since *UNIX EPOCH*.
///
/// ## Panics
///
/// Panics if the time is before *UNIX EPOCH*.
pub fn unix_time() -> Duration {
    FAKETIME_ENABLED.with(|enabled_cell| match enabled_cell.get() {
        Some(true) => FAKETIME_PATH.with(|path_cell| read_or_system(path_cell.borrow().deref())),
        Some(false) => system_unix_time(),
        None => auto_detect(&enabled_cell),
    })
}

fn auto_detect(enabled_cell: &Cell<Option<bool>>) -> Duration {
    if let Some(path) = match env::var(KEY_FAKETIME) {
        Ok(val) => Some(PathBuf::from(val)),
        _ => match thread::current().name() {
            Some(name) if name.starts_with(PREFIX_FAKETIME_EQ) => {
                Some(PathBuf::from(&name[PREFIX_FAKETIME_EQ.len()..]))
            }
            _ => None,
        },
    } {
        let duration = read_or_system(&path);
        FAKETIME_PATH.with(|file_cell| file_cell.replace(path));
        enabled_cell.set(Some(true));
        duration
    } else {
        enabled_cell.set(Some(false));
        system_unix_time()
    }
}

/// Enables faketime in current thread and use the specified timestamp file.
pub fn enable<T: AsRef<Path>>(path: T) {
    let path_buf = path.as_ref().to_path_buf();
    FAKETIME_PATH.with(|cell| cell.replace(path_buf));
    FAKETIME_ENABLED.with(|cell| cell.set(Some(true)));
}

/// Disables faketime in current thread.
pub fn disable() {
    FAKETIME_ENABLED.with(|cell| cell.set(Some(false)));
}

fn read_millis<T: AsRef<Path>>(path: T) -> Result<u64> {
    fs::read_to_string(path).and_then(|text| {
        text.trim()
            .parse()
            .map_err(|err| Error::new(ErrorKind::Other, err))
    })
}

fn read_or_system<T: AsRef<Path>>(path: T) -> Duration {
    read_millis(path)
        .ok()
        .map_or_else(system_unix_time, Duration::from_millis)
}

/// Writes time as milliseconds since *UNIX EPOCH* into the specified timestamp file.
pub fn write_millis<T: AsRef<Path>>(path: T, millis: u64) -> Result<()> {
    let mut file = NamedTempFile::new()?;
    file.write_all(millis.to_string().as_bytes())?;
    file.into_temp_path().persist(path)?;
    Ok(())
}

/// Writes time into a temporary file and return the file.
///
///
/// It returns the handle to the timestamp file on success.
///
/// ```
/// use std::fs;
///
/// let faketime_file = faketime::millis_tempfile(123).expect("create faketime file");
/// assert_eq!(fs::read_to_string(&faketime_file).ok(), Some("123".to_string()));
/// ```
///
/// The file is deleted when the handle goes out of scope.
///
/// ```
/// let path = {
///     let faketime_file = faketime::millis_tempfile(123).expect("create faketime file");
///     faketime_file.to_path_buf()
/// };
/// assert!(!path.exists());
/// ```
pub fn millis_tempfile(millis: u64) -> Result<TempPath> {
    let path = NamedTempFile::new()?.into_temp_path();
    write_millis(&path, millis)?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_file_io() {
        {
            let path = NamedTempFile::new()
                .expect("create faketime file")
                .into_temp_path();
            println!("write {:?}", path);
            write_millis(&path, 0).expect("write millis");
        }

        let faketime_file = NamedTempFile::new()
            .expect("create faketime file")
            .into_temp_path();

        assert_eq!(None, read_millis(&faketime_file).ok());
        println!("write {:?}", faketime_file);
        fs::write(&faketime_file, "x").expect("write millis");
        assert_eq!(None, read_millis(&faketime_file).ok());
        fs::write(&faketime_file, "12345\n").expect("write millis");
        assert_eq!(12345, read_millis(&faketime_file).expect("read millis"));
        write_millis(&faketime_file, 54321).expect("write millis");
        assert_eq!(54321, read_millis(&faketime_file).expect("read millis"));
    }
}
