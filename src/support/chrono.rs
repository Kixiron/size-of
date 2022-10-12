#![cfg(feature = "chrono")]

use crate::{Context, SizeOf};
use chrono::{
    format::{
        Fixed, InternalFixed, InternalNumeric, Item, Numeric, Pad, ParseError, ParseErrorKind,
        Parsed,
    },
    Date, DateTime, Duration, FixedOffset, IsoWeek, Local, LocalResult, Month, Months, NaiveDate,
    NaiveDateTime, NaiveTime, NaiveWeek, ParseMonthError, ParseWeekdayError, RoundingError,
    SecondsFormat, TimeZone, Utc, Weekday,
};

impl_total_size_childless! {
    Pad,
    Utc,
    Local,
    Fixed,
    Month,
    Months,
    Parsed,
    Numeric,
    IsoWeek,
    Weekday,
    Duration,
    NaiveDate,
    NaiveTime,
    NaiveWeek,
    ParseError,
    FixedOffset,
    InternalFixed,
    NaiveDateTime,
    RoundingError,
    SecondsFormat,
    ParseErrorKind,
    ParseMonthError,
    InternalNumeric,
    ParseWeekdayError,
}

impl<Tz> SizeOf for Date<Tz>
where
    Tz: TimeZone,
    Tz::Offset: SizeOf,
{
    fn size_of_children(&self, context: &mut Context) {
        self.offset().size_of_children(context);
    }
}

impl<Tz> SizeOf for DateTime<Tz>
where
    Tz: TimeZone,
    Tz::Offset: SizeOf,
{
    fn size_of_children(&self, context: &mut Context) {
        self.offset().size_of_children(context);
    }
}

impl<T> SizeOf for LocalResult<T>
where
    T: SizeOf,
{
    fn size_of_children(&self, context: &mut Context) {
        match self {
            Self::None => {}
            Self::Single(result) => T::size_of_children(result, context),
            Self::Ambiguous(first, second) => {
                T::size_of_children(first, context);
                T::size_of_children(second, context);
            }
        }
    }
}

impl SizeOf for Item<'_> {
    fn size_of_children(&self, context: &mut Context) {
        match self {
            &Self::Literal(string) | &Self::Space(string) => string.size_of_children(context),
            Self::OwnedLiteral(string) | Self::OwnedSpace(string) => {
                string.size_of_children(context)
            }
            Self::Numeric(numeric, pad) => {
                numeric.size_of_children(context);
                pad.size_of_children(context);
            }
            Self::Fixed(fixed) => fixed.size_of_children(context),
            Self::Error => {}
        }
    }
}
