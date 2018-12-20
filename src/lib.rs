//! This module just reexports `faketime::system` or `faketime::faketime`, depending on whether on
//! not the rust cfg `disable_faketime` is set. See details in the module document of
//! [faketime::faketime](faketime/index.html).

#[cfg(not(disable_faketime))]
pub mod faketime;
pub mod system;

#[cfg(not(disable_faketime))]
pub use crate::faketime::{disable, enable, enable_and_write_millis, unix_time, write_millis};
#[cfg(disable_faketime)]
pub use crate::system::unix_time;

#[cfg(test)]
mod tests {
    use crate::*;

    #[cfg(disable_faketime)]
    #[test]
    fn test_system_time() {
        let system_now = system::unix_time();
        let now = unix_time();
        assert!((now - system_now).as_secs() < 60);
    }

    #[cfg(not(disable_faketime))]
    #[test]
    fn test_mock_constant_time() {
        let _faketime_file = enable_and_write_millis(123456).expect("enable faketime");

        let now = unix_time();
        assert_eq!(123, now.as_secs());
        assert_eq!(456, now.subsec_millis());
    }
}
