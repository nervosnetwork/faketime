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
//! let _faketime_file = faketime::enable_and_write_millis(100_000).expect("enable faketime");
//! assert_eq!(faketime::unix_time().as_secs(), 100);
//! ```
//!
//! In each thread, when this function is first invoked, and neither `faketime::enable(path)` nor
//! `faketime::disable()` has been invoked in the thread already, it will detect whether faketime
//! should be enabled and which timestamp file should be used.
//!
//! First, if the environment variable `FAKETIME_DIR` exists, and this thread has a name
//! `THREAD_NAME` (see `std::thread::Thread::name`), faketime is enabled, and the timestamp file
//! path is `FAKETIME_DIR/THREAD_NAME`.
//!
//! ```
//! use std::env;
//! use std::fs;
//! use std::thread;
//! use tempfile::TempDir;
//!
//! let dir = TempDir::new().expect("create temp dir");
//! env::set_var("FAKETIME_DIR", dir.path().as_os_str());
//! let file = dir.path().join("faketime");
//! faketime::write_millis(&file, 123_456);
//!
//! thread::Builder::new()
//!     .name("faketime".to_string())
//!     .spawn(|| assert_eq!(123, faketime::unix_time().as_secs()))
//!     .expect("spawn thread")
//!     .join()
//!     .expect("join thread");
//! ```
//!
//! If the environment variable `FAKETIME_DIR` is missing, but this thread has a name and the name
//! starts with `FAKETIME=` literally, faketime is also enabled, and the timestamp file is the
//! portion of the thread name after `FAKETIME=`.
//!
//! ```
//! use std::fs;
//! use std::thread;
//! use tempfile::NamedTempFile;
//!
//! let file = NamedTempFile::new().expect("create temp file");
//! faketime::write_millis(file.path(), 123_456);
//!
//! thread::Builder::new()
//!     .name(format!("FAKETIME={}", file.path().display()))
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
//!     .expect("spawn thread");
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
use std::io::Write;
use std::ops::Deref;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;
use tempfile::NamedTempFile;

pub use std::io::Result;

thread_local! {
    /// Some(true): Enabled
    /// Some(false): Disabled
    /// None: Undecided
    static FAKETIME_ENABLED: Cell<Option<bool>> = Cell::new(None);
    static FAKETIME_PATH: RefCell<PathBuf> = Default::default();
}

const KEY_FAKETIME_DIR: &str = "FAKETIME_DIR";
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
    let path_option = match thread::current().name() {
        Some(name) => match env::var(KEY_FAKETIME_DIR) {
            Ok(val) => {
                let mut path = PathBuf::from(val);
                path.push(name);
                Some(path)
            }
            _ => {
                if name.starts_with(PREFIX_FAKETIME_EQ) {
                    Some(PathBuf::from(&name[PREFIX_FAKETIME_EQ.len()..]))
                } else {
                    None
                }
            }
        },
        None => None,
    };

    match path_option {
        Some(path) => {
            let duration = read_or_system(&path);
            enabled_cell.set(Some(true));
            FAKETIME_PATH.with(|file_cell| file_cell.replace(path));
            duration
        }
        None => {
            enabled_cell.set(Some(false));
            system_unix_time()
        }
    }
}

/// Enables faketime in current thread and use the specified timestamp file.
pub fn enable(path: PathBuf) {
    FAKETIME_PATH.with(|cell| cell.replace(path));
    FAKETIME_ENABLED.with(|cell| cell.set(Some(true)));
}

/// Disables faketime in current thread.
pub fn disable() {
    FAKETIME_ENABLED.with(|cell| cell.set(Some(false)));
}

fn read_millis<T: AsRef<Path>>(path: T) -> Option<u64> {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| text.trim().parse().ok())
}

fn read_or_system<T: AsRef<Path>>(path: T) -> Duration {
    read_millis(path).map_or_else(system_unix_time, Duration::from_millis)
}

/// Writes time as milliseconds since *UNIX EPOCH* into the specified timestamp file.
pub fn write_millis<T: AsRef<Path>>(path: T, millis: u64) -> Result<()> {
    let mut file = NamedTempFile::new()?;
    file.write_all(millis.to_string().as_bytes())?;
    file.into_temp_path().persist(path)?;
    Ok(())
}

/// Enables and writes time into the timestamp file.
///
/// It returns the handle to the timestamp file. The file is deleted when the handle goes out of
/// scope.
///
/// **Attention**: if the return result is assigned to an unnamed variable, the timestamp file is
/// deleted immediately, which may not be what you mean. Always assign it to a named variable, and
/// keep it until faketime is no longer needed.
pub fn enable_and_write_millis(millis: u64) -> Result<NamedTempFile> {
    let file = NamedTempFile::new()?;
    let path = file.path();
    write_millis(&path, millis)?;

    enable(path.to_path_buf());
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_file_io() {
        let file = NamedTempFile::new().unwrap();
        let path = file.path();

        assert_eq!(None, read_millis(&path));
        let _ = fs::write(&path, "x");
        assert_eq!(None, read_millis(&path));
        let _ = fs::write(&path, "12345\n");
        assert_eq!(Some(12345), read_millis(&path));
        let _ = write_millis(&path, 54321);
        assert_eq!(Some(54321), read_millis(&path));
    }

    #[test]
    fn format_code() {
        // thread::Builder::new()
        //     .name("faketime".to_string())
        //     .spawn(|| assert_eq!(123, faketime::unix_time().as_secs()))
        //     .expect("spawn thread")
        //     .join()
        //     .expect("join thread");
    }
}
