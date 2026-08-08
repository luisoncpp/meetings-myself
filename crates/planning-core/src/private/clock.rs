use chrono::{DateTime, Duration, Utc};
use std::sync::Mutex;

/// The single source of "now". Production code never reads the wall clock
/// directly — see `tests/architecture.test.ts`.
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// Test double. Public because every crate's tests need it.
pub struct FixedClock {
    instant: Mutex<DateTime<Utc>>,
}

impl FixedClock {
    pub fn at(instant: DateTime<Utc>) -> Self {
        Self { instant: Mutex::new(instant) }
    }

    pub fn set(&self, instant: DateTime<Utc>) {
        *self.instant.lock().expect("clock mutex poisoned") = instant;
    }

    pub fn advance(&self, delta: Duration) {
        let mut guard = self.instant.lock().expect("clock mutex poisoned");
        *guard += delta;
    }
}

impl Clock for FixedClock {
    fn now(&self) -> DateTime<Utc> {
        *self.instant.lock().expect("clock mutex poisoned")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn fixed_clock_holds_and_advances_its_instant() {
        let start = Utc.with_ymd_and_hms(2026, 8, 6, 5, 0, 0).unwrap();
        let clock = FixedClock::at(start);
        assert_eq!(clock.now(), start);

        clock.advance(Duration::hours(3));
        assert_eq!(clock.now(), Utc.with_ymd_and_hms(2026, 8, 6, 8, 0, 0).unwrap());
    }
}
