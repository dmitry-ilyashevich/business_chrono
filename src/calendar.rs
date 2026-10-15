use std::collections::BTreeMap;
use std::ops::{Bound, Range, RangeInclusive};

use chrono::{Datelike, Days, NaiveDate, Weekday};

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
    /// Bit `n` is set when the weekday `n` days after Monday is a rest day.
    weekend_mask: u8,
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

        let (Some(&first_year), Some(&last_year)) =
            (per_year.keys().next(), per_year.keys().next_back())
        else {
            return Err(Error::NoHolidays { country });
        };

        let mut holidays: BTreeMap<NaiveDate, Vec<Holiday>> = BTreeMap::new();
        for (date, same_day) in per_year.into_values().flatten() {
            holidays.entry(date).or_default().extend(same_day);
        }

        Ok(Self {
            country,
            weekend_mask: country
                .weekend()
                .iter()
                .fold(0, |mask, day| mask | weekday_bit(*day)),
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
        self.weekend_mask & weekday_bit(date.weekday()) != 0
    }

    /// Check if a given date is a business day (not a holiday and not a weekend).
    pub fn is_business_day(&self, date: NaiveDate) -> bool {
        !self.is_holiday(date) && !self.is_weekend(date)
    }

    /// Number of business days from `start` to `end`, inclusive.
    ///
    /// Returns an error if `start` is after `end` or the range is not fully
    /// within [`covered_years()`](Self::covered_years).
    pub fn business_days_between(&self, start: NaiveDate, end: NaiveDate) -> Result<u32> {
        if start > end {
            return Err(Error::StartAfterEnd { start, end });
        }

        if !self.years.contains(&start.year()) || !self.years.contains(&end.year()) {
            return Err(Error::RangeNotCovered {
                start,
                end,
                covered: self.covered_years(),
            });
        }

        let total_days = (end - start).num_days() as u32 + 1;
        let mut count = total_days / 7 * self.business_days_per_week();
        let mut day = start;
        for _ in 0..total_days % 7 {
            if !self.is_weekend(day) {
                count += 1;
            }
            day = day.succ_opt().ok_or(Error::DateOverflow)?;
        }

        Ok(count - self.weekday_holidays((Bound::Included(start), Bound::Included(end))))
    }

    /// Number of business days from `start` through today (local time), inclusive.
    ///
    /// Same errors as [`business_days_between()`](Self::business_days_between).
    pub fn business_days_until_today(&self, start: NaiveDate) -> Result<u32> {
        self.business_days_between(start, chrono::Local::now().date_naive())
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
    /// The start date is rolled forward to a business day first. Returns an
    /// error if the start date or the result is not within
    /// [`covered_years()`](Self::covered_years).
    pub fn add_business_days(&self, start: NaiveDate, days: u32) -> Result<NaiveDate> {
        let mut day = self.roll_forward(start)?;
        let mut remaining = days;

        // Jump whole weeks, always leaving at least one day to step, so the
        // result is found by the loop below and is a business day.
        let per_week = self.business_days_per_week();
        while remaining > per_week {
            let weeks = (remaining - 1) / per_week;
            let target = day
                .checked_add_days(Days::new(u64::from(weeks) * 7))
                .ok_or(Error::DateOverflow)?;
            self.ensure_covered(target)?;

            let skipped = self.weekday_holidays((Bound::Excluded(day), Bound::Included(target)));
            remaining -= weeks * per_week - skipped;
            day = target;
        }

        while remaining > 0 {
            day = day.succ_opt().ok_or(Error::DateOverflow)?;
            if self.is_business_day(day) {
                remaining -= 1;
            }
        }

        self.ensure_covered(day)?;
        Ok(day)
    }

    /// Subtract a number of business days from a given start date.
    ///
    /// The start date is rolled backward to a business day first. Returns an
    /// error if the start date or the result is not within
    /// [`covered_years()`](Self::covered_years).
    pub fn subtract_business_days(&self, start: NaiveDate, days: u32) -> Result<NaiveDate> {
        let mut day = self.roll_backward(start)?;
        let mut remaining = days;

        // Mirror of the week jumps in `add_business_days`.
        let per_week = self.business_days_per_week();
        while remaining > per_week {
            let weeks = (remaining - 1) / per_week;
            let target = day
                .checked_sub_days(Days::new(u64::from(weeks) * 7))
                .ok_or(Error::DateOverflow)?;
            self.ensure_covered(target)?;

            let skipped = self.weekday_holidays((Bound::Included(target), Bound::Excluded(day)));
            remaining -= weeks * per_week - skipped;
            day = target;
        }

        while remaining > 0 {
            day = day.pred_opt().ok_or(Error::DateOverflow)?;
            if self.is_business_day(day) {
                remaining -= 1;
            }
        }

        self.ensure_covered(day)?;
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

    fn business_days_per_week(&self) -> u32 {
        7 - self.weekend_mask.count_ones()
    }

    /// Number of holiday dates in `range` that do not fall on a weekend.
    fn weekday_holidays(&self, range: (Bound<NaiveDate>, Bound<NaiveDate>)) -> u32 {
        self.holidays
            .range(range)
            .filter(|(date, _)| !self.is_weekend(**date))
            .count() as u32
    }
}

fn weekday_bit(day: Weekday) -> u8 {
    1 << day.num_days_from_monday()
}
