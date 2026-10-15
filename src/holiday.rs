use std::collections::BTreeMap;

use chrono::NaiveDate;

use crate::country::Country;

pub type Year = i32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Holiday {
    pub date: NaiveDate,
    pub country: Country,
    pub country_name: &'static str,
    pub name: &'static str,
    pub name_en: &'static str,
}

/// Holidays per year, keyed by date. Several holidays can share one date.
pub type HolidayPerYearMap = BTreeMap<Year, BTreeMap<NaiveDate, Vec<Holiday>>>;

/// Convert year, month, day to NaiveDate, returning an error if the date is invalid.
pub(crate) fn from_ymd_res(year: Year, month: u32, day: u32) -> crate::Result<NaiveDate> {
    NaiveDate::from_ymd_opt(year, month, day).ok_or(crate::Error::InvalidDate { year, month, day })
}
