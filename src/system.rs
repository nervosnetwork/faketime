//! This is the fallback implementation when cfg `disable_faketime` is set.

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
use crate::wasm::SystemTime;
use std::time::Duration;
#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
use std::time::SystemTime;

/// Gets elapsed time since *UNIX EPOCH*.
///
/// ## Panics
///
/// Panics if the time is before *UNIX EPOCH*.
pub fn unix_time() -> Duration {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("SystemTime before UNIX EPOCH!")
}
