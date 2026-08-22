pub mod calendar;
pub mod country;
pub mod data;

mod error;
mod holiday;

pub use calendar::Calendar;
pub use country::Country;
pub use error::Error;
pub use holiday::{Holiday, HolidayPerYearMap, Year};

pub type Result<T> = std::result::Result<T, Error>;

/// Build a map of holidays for a given country and optional range of years.
pub fn build(country: Country, years: Option<&std::ops::Range<Year>>) -> Result<HolidayPerYearMap> {
    data::build(country, years)
}
