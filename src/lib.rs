//! This module just reexports `faketime::system` or `faketime::faketime`, depending on whether on
//! not the rust cfg `disable_faketime` is set. See details in the module document of
//! [faketime::faketime](faketime/index.html).

#[cfg(not(disable_faketime))]
pub mod faketime;
pub mod system;

#[cfg(not(disable_faketime))]
pub use crate::faketime::{disable, enable, millis_tempfile, unix_time, write_millis};
#[cfg(disable_faketime)]
pub use crate::system::unix_time;

/// Gets elapsed time in milliseconds since *UNIX EPOCH*.
///
/// ```
/// let now = faketime::unix_time();
/// let millis = faketime::unix_time_as_millis();
/// assert!(millis / 1000 - now.as_secs() < 60);
/// ```
///
/// This function depends on the return result from `unix_time`. If `unix_time` is faked, this
/// function is also faked.
///
/// ## Panics
///
/// Panics if the time is before *UNIX EPOCH*, or `u64` is not enough to store the number of
/// milliseconds.
pub fn unix_time_as_millis() -> u64 {
    let duration = unix_time();
    duration.as_secs() * 1000 + u64::from(duration.subsec_millis())
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[cfg(disable_faketime)]
    #[test]
    fn test_system() {
        let system_now = system::unix_time();
        let now = unix_time();
        assert!((now - system_now).as_secs() < 60);
    }

    #[cfg(not(disable_faketime))]
    #[test]
    fn test_faketime() {
        let faketime_file = millis_tempfile(123_456).expect("create faketime file");
        enable(&faketime_file);

        let now = unix_time();
        assert_eq!(123, now.as_secs());
        assert_eq!(456, now.subsec_millis());
    }
}
