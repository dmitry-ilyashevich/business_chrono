use std::collections::BTreeMap;
use std::ops::{Range, RangeInclusive};

use chrono::{Datelike, NaiveDate, Weekday};

use crate::Result;
use crate::country::Country;
use crate::error::Error;
use crate::holiday::{Holiday, Year};

/// Business-day calendar for one country.
///
/// Combines the embedded public-holiday data (date.nager.at) with the
/// country's weekly rest days (Unicode CLDR week data - Sat/Sun for most
/// countries, e.g. Fri/Sat for Egypt).
///
/// Observed holidays are already part of the data: where a country moves a
/// weekend holiday to the next working day (US "in lieu" days, UK substitute
/// bank holidays, …), the API reports the substitute date as the holiday, so
/// no extra shifting is done here.
#[derive(Debug, Clone)]
pub struct Calendar {
    country: Country,
    weekend: &'static [Weekday],
    holidays: BTreeMap<NaiveDate, Vec<Holiday>>,
    years: RangeInclusive<Year>,
}

impl Calendar {
    pub fn new(country: Country) -> Result<Self> {
        Self::build(country, None)
    }

    /// Create a calendar for a specific country and a range of years.
    pub fn with_years(country: Country, years: Range<Year>) -> Result<Self> {
        Self::build(country, Some(&years))
    }

    pub fn build(country: Country, years: Option<&Range<Year>>) -> Result<Self> {
        let per_year = crate::data::build(country, years)?;

        let (Some(&first_year), Some(&last_year)) = (per_year.keys().min(), per_year.keys().max())
        else {
            return Err(Error::NoHolidays { country });
        };

        let mut holidays: BTreeMap<NaiveDate, Vec<Holiday>> = BTreeMap::new();
        for (date, same_day) in per_year.into_values().flatten() {
            holidays.entry(date).or_default().extend(same_day);
        }

        Ok(Self {
            country,
            weekend: country.weekend(),
            holidays,
            years: first_year..=last_year,
        })
    }

    /// Get the country this calendar is for.
    pub fn country(&self) -> Country {
        self.country
    }

    /// Years this calendar has holidays data for.
    pub fn covered_years(&self) -> RangeInclusive<Year> {
        self.years.clone()
    }

    /// Get the holiday for a given date, if it exists.
    ///
    /// When several holidays share the date this returns the first one;
    /// use [`holidays()`](Self::holidays) to get all of them.
    pub fn holiday(&self, date: NaiveDate) -> Option<&Holiday> {
        self.holidays(date).first()
    }

    /// Get all holidays for a given date (empty if the date is not a holiday).
    pub fn holidays(&self, date: NaiveDate) -> &[Holiday] {
        self.holidays.get(&date).map_or(&[], Vec::as_slice)
    }

    /// Check if a given date is a holiday.
    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        self.holidays.contains_key(&date)
    }

    /// Check if a given date is a weekend day.
    pub fn is_weekend(&self, date: NaiveDate) -> bool {
        self.weekend.contains(&date.weekday())
    }

    /// Check if a given date is a business day (not a holiday and not a weekend).
    pub fn is_business_day(&self, date: NaiveDate) -> bool {
        !self.is_holiday(date) && !self.is_weekend(date)
    }

    /// Number of business days from `start` to `end`, inclusive.
    ///
    /// `end` defaults to today if not provided. Returns an error if
    /// `start` is after `end` or the range is not fully within [`covered_years()`](Self::covered_years).
    pub fn business_days_between(&self, start: NaiveDate, end: Option<NaiveDate>) -> Result<u32> {
        let end = end.unwrap_or_else(|| chrono::Local::now().naive_local().date());

        if start > end {
            return Err(Error::StartAfterEnd { start, end });
        }

        if !self.covered_years().contains(&start.year())
            || !self.covered_years().contains(&end.year())
        {
            return Err(Error::RangeNotCovered {
                start,
                end,
                covered: self.covered_years(),
            });
        }

        let mut count = 0;
        let mut day = start;
        while day <= end {
            if self.is_business_day(day) {
                count += 1;
            }
            day = day.succ_opt().ok_or(Error::DateOverflow)?;
        }

        Ok(count)
    }

    /// Roll a date forward to the next business day.
    ///
    /// Returns an error if the date or the resulting business day is not
    /// within [`covered_years()`](Self::covered_years).
    pub fn roll_forward(&self, mut day: NaiveDate) -> Result<NaiveDate> {
        self.ensure_covered(day)?;
        while !self.is_business_day(day) {
            day = day.succ_opt().ok_or(Error::DateOverflow)?;
        }
        self.ensure_covered(day)?;
        Ok(day)
    }

    /// Roll a date backward to the previous business day.
    ///
    /// Returns an error if the date or the resulting business day is not
    /// within [`covered_years()`](Self::covered_years).
    pub fn roll_backward(&self, mut day: NaiveDate) -> Result<NaiveDate> {
        self.ensure_covered(day)?;
        while !self.is_business_day(day) {
            day = day.pred_opt().ok_or(Error::DateOverflow)?;
        }
        self.ensure_covered(day)?;
        Ok(day)
    }

    /// Add a number of business days to a given start date.
    ///
    /// Returns an error if the start date is not within [`covered_years()`](Self::covered_years).
    pub fn add_business_days(&self, start: NaiveDate, days: u32) -> Result<NaiveDate> {
        let start_business_day = self.roll_forward(start)?;

        if !self.covered_years().contains(&start_business_day.year()) {
            return Err(Error::DateNotCovered {
                date: start_business_day,
                covered: self.covered_years(),
            });
        }

        let mut count = 0;
        let mut day = start_business_day;
        while count < days {
            day = day.succ_opt().ok_or(Error::DateOverflow)?;
            if self.is_business_day(day) {
                count += 1;
            }
        }

        Ok(day)
    }

    /// Subtract a number of business days from a given start date.
    ///
    /// Returns an error if the start date is not within [`covered_years()`](Self::covered_years).
    pub fn subtract_business_days(&self, start: NaiveDate, days: u32) -> Result<NaiveDate> {
        let start_business_day = self.roll_backward(start)?;

        if !self.covered_years().contains(&start_business_day.year()) {
            return Err(Error::DateNotCovered {
                date: start_business_day,
                covered: self.covered_years(),
            });
        }

        let mut count = 0;
        let mut day = start_business_day;
        while count < days {
            day = day.pred_opt().ok_or(Error::DateOverflow)?;

            if self.is_business_day(day) {
                count += 1;
            }
        }

        Ok(day)
    }

    fn ensure_covered(&self, date: NaiveDate) -> Result<()> {
        if self.years.contains(&date.year()) {
            Ok(())
        } else {
            Err(Error::DateNotCovered {
                date,
                covered: self.covered_years(),
            })
        }
    }
}
