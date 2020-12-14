use std::time::Duration;

#[derive(Copy, Clone)]
pub struct SystemTime(f64);

impl SystemTime {
    pub const UNIX_EPOCH: SystemTime = SystemTime(0.0);

    pub fn now() -> SystemTime {
        SystemTime(js_sys::Date::now())
    }

    pub fn duration_since(&self, earlier: SystemTime) -> Result<Duration, ()> {
        let dur_ms = self.0 - earlier.0;
        if dur_ms < 0.0 {
            return Err(());
        }
        Ok(Duration::from_millis(dur_ms as u64))
    }
}
