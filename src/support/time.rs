#![cfg(feature = "time")]

use time::{Date, Duration, Month, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset, Weekday};

impl_total_size_childless! {
    Date,
    Time,
    Month,
    Weekday,
    Duration,
    UtcOffset,
    OffsetDateTime,
    PrimitiveDateTime,
}

#[cfg(feature = "time-std")]
mod time_std {
    use crate::{Context, SizeOf};
    use time::Instant;

    impl SizeOf for Instant {
        fn size_of_children(&self, context: &mut Context) {
            self.0.size_of_children(context);
        }
    }
}
