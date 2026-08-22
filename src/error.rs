use std::ops::RangeInclusive;

use chrono::NaiveDate;
use thiserror::Error;

use crate::country::Country;
use crate::holiday::Year;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid date: {year}-{month:02}-{day:02}")]
    InvalidDate { year: Year, month: u32, day: u32 },

    #[error("unknown country code: {0}")]
    UnknownCountryCode(String),

    #[error(
        "country {country:?} is not supported. Please enable the `{}` cargo feature.",
        .country.code().to_ascii_lowercase()
    )]
    UnsupportedCountry { country: Country },

    #[error("no holidays found for country {country:?} in the requested years")]
    NoHolidays { country: Country },

    #[error("start date {start} is after end date {end}")]
    StartAfterEnd { start: NaiveDate, end: NaiveDate },

    #[error("date range {start} to {end} is not fully within covered years {covered:?}")]
    RangeNotCovered {
        start: NaiveDate,
        end: NaiveDate,
        covered: RangeInclusive<Year>,
    },

    #[error("date {date} is not within covered years {covered:?}")]
    DateNotCovered {
        date: NaiveDate,
        covered: RangeInclusive<Year>,
    },

    #[error("date out of range")]
    DateOverflow,
}
