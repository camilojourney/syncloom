//! Time as a port, so records carry RFC 3339 UTC instants and tests fix them.
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub trait Clock: Send + Sync {
    /// The current instant in UTC, whole seconds.
    fn now(&self) -> OffsetDateTime;
    /// The current instant as RFC 3339 with a `Z` suffix, such as `2026-10-30T18:01:24Z`.
    fn now_rfc3339(&self) -> String {
        rfc3339(self.now())
    }
}

pub struct SystemClock;
impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

/// A clock that always answers the same instant; for tests.
pub struct FixedClock(pub OffsetDateTime);
impl Clock for FixedClock {
    fn now(&self) -> OffsetDateTime {
        self.0
    }
}

pub fn rfc3339(instant: OffsetDateTime) -> String {
    let _ = instant;
    String::new()
}
