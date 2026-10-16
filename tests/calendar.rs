#[allow(unused_imports)]
use business_chrono::{Calendar, Country, Error};
#[allow(unused_imports)]
use chrono::{Datelike, NaiveDate, Weekday};

#[allow(dead_code)]
fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

#[test]
#[cfg(feature = "pl")]
fn pl_business_chrono() {
    let cal = Calendar::new(Country::PL).unwrap();

    assert!(Calendar::with_years(Country::PL, 2020..2025).is_ok());
    assert!(matches!(
        Calendar::with_years(Country::PL, 3000..3100),
        Err(Error::NoHolidays {
            country: Country::PL
        })
    ));

    assert!(cal.is_holiday(date(2026, 1, 1))); // New Year's Day
    assert!(cal.is_holiday(date(2026, 5, 1))); // Labor Day
    assert!(cal.is_holiday(date(2026, 12, 25))); // Christmas Day

    assert!(!cal.is_holiday(date(2026, 1, 2))); // Not a holiday
    assert!(!cal.is_holiday(date(2026, 5, 2))); // Not a holiday

    assert!(cal.is_weekend(date(2026, 1, 3))); // Saturday
    assert!(cal.is_weekend(date(2026, 1, 4))); // Sunday

    assert!(!cal.is_weekend(date(2026, 1, 5))); // Monday
}

#[test]
#[cfg(feature = "us")]
fn us_business_chrono() {
    let cal = Calendar::new(Country::US).unwrap();

    assert_eq!(cal.country(), Country::US);

    assert!(Calendar::with_years(Country::US, 2020..2025).is_ok());
    assert!(matches!(
        Calendar::with_years(Country::US, 3000..3100),
        Err(Error::NoHolidays {
            country: Country::US
        })
    ));

    assert!(cal.is_holiday(date(2026, 7, 3))); // Independence Day 
    assert!(!cal.is_business_day(date(2026, 7, 3))); // Independence Day

    assert!(cal.is_weekend(date(2026, 7, 4))); // Saturday 

    assert_eq!(
        cal.holiday(date(2026, 7, 3)).unwrap().name_en,
        "Independence Day"
    );
    assert_eq!(
        cal.holiday(date(2025, 7, 4)).unwrap().name_en,
        "Independence Day"
    );
}

#[test]
#[cfg(feature = "us")]
fn us_count_between() {
    let cal = Calendar::new(Country::US).unwrap();

    // Jan 1 Thu = holiday, Jan 2 Fri = work, Jan 3/4 = weekend, Jan 5-7 = work
    assert_eq!(
        cal.business_days_between(date(2026, 1, 1), date(2026, 1, 7))
            .unwrap(),
        4
    );

    // single working day, inclusive on both ends
    assert_eq!(
        cal.business_days_between(date(2026, 1, 2), date(2026, 1, 2))
            .unwrap(),
        1
    );

    // single non-working days
    assert_eq!(
        cal.business_days_between(date(2026, 1, 1), date(2026, 1, 1))
            .unwrap(),
        0
    );
    assert_eq!(
        cal.business_days_between(date(2026, 1, 3), date(2026, 1, 4))
            .unwrap(),
        0
    );

    // Jul 4 2026 is a Saturday: a holiday on a weekend is not subtracted twice
    assert_eq!(
        cal.business_days_between(date(2026, 7, 4), date(2026, 7, 6))
            .unwrap(),
        1
    );
}

#[test]
#[cfg(feature = "us")]
fn count_errors() {
    let cal = Calendar::new(Country::US).unwrap();

    assert!(matches!(
        cal.business_days_between(date(2026, 1, 7), date(2026, 1, 1)),
        Err(Error::StartAfterEnd { .. })
    ));
    assert!(matches!(
        cal.business_days_between(date(1999, 1, 1), date(2026, 1, 1)),
        Err(Error::RangeNotCovered { .. })
    ));
    assert!(matches!(
        cal.business_days_between(date(2026, 1, 1), date(2090, 1, 1)),
        Err(Error::RangeNotCovered { .. })
    ));
}

#[test]
#[cfg(feature = "ua")]
fn with_years_restricts_coverage() {
    let cal = Calendar::with_years(Country::UA, 2020..2022).unwrap();
    assert_eq!(cal.covered_years(), 2020..=2021);
    assert!(
        cal.business_days_between(date(2021, 12, 1), date(2021, 12, 31))
            .is_ok()
    );
    assert!(matches!(
        cal.business_days_between(date(2021, 12, 1), date(2022, 1, 1)),
        Err(Error::RangeNotCovered { .. })
    ));
}

#[test]
#[cfg(all(feature = "us", feature = "eg", feature = "dz"))]
fn weekend_days_per_country() {
    assert_eq!(Country::US.weekend(), &[Weekday::Sat, Weekday::Sun]);
    assert_eq!(Country::EG.weekend(), &[Weekday::Fri, Weekday::Sat]);
    assert_eq!(Country::DZ.weekend(), &[Weekday::Fri, Weekday::Sat]);
}

#[test]
#[cfg(feature = "eg")]
fn egypt_friday_is_not_business_day() {
    let cal = Calendar::new(Country::EG).unwrap();

    assert!(cal.is_weekend(date(2026, 7, 3))); // Friday
    assert!(!cal.is_business_day(date(2026, 7, 3)));
    assert!(cal.is_business_day(date(2026, 7, 5))); // Sunday is a workday
}

#[test]
#[cfg(feature = "gb")]
fn gb_substitute_day_is_in_data() {
    let cal = Calendar::new(Country::GB).unwrap();

    // Boxing Day 2026 falls on Saturday; the API reports the substitute
    // Monday as the holiday - no shifting logic in this crate.
    assert!(cal.is_holiday(date(2026, 12, 28)));
    assert!(!cal.is_business_day(date(2026, 12, 28)));
    assert!(!cal.is_holiday(date(2026, 12, 26)));
}

#[test]
#[cfg(feature = "us")]
fn add_business_days() {
    let cal = Calendar::new(Country::US).unwrap();

    // Jan 1 Thu = New Year, Jan 2 Fri = work, Jan 3/4 = weekend, Jan 5-6 = work
    assert_eq!(
        cal.add_business_days(date(2026, 1, 1), 0).unwrap(),
        date(2026, 1, 2)
    );
    assert_eq!(
        cal.add_business_days(date(2026, 1, 1), 1).unwrap(),
        date(2026, 1, 5)
    );
    assert_eq!(
        cal.add_business_days(date(2026, 1, 1), 2).unwrap(),
        date(2026, 1, 6)
    );
    assert_eq!(
        cal.add_business_days(date(2026, 1, 1), 3).unwrap(),
        date(2026, 1, 7)
    );

    assert!(matches!(
        cal.add_business_days(date(1999, 1, 1), 1),
        Err(Error::DateNotCovered { .. })
    ));
}

#[test]
#[cfg(feature = "us")]
fn subtract_business_days() {
    let cal = Calendar::new(Country::US).unwrap();

    // Jan 1 Thu = New Year, Jan 2 Fri = work, Jan 3/4 = weekend, Jan 5-7 = work
    assert_eq!(
        cal.subtract_business_days(date(2026, 1, 7), 1).unwrap(),
        date(2026, 1, 6)
    );
    assert_eq!(
        cal.subtract_business_days(date(2026, 1, 7), 2).unwrap(),
        date(2026, 1, 5)
    );
    assert_eq!(
        cal.subtract_business_days(date(2026, 1, 7), 3).unwrap(),
        date(2026, 1, 2)
    );
    assert_eq!(
        cal.subtract_business_days(date(2026, 1, 7), 5).unwrap(),
        date(2025, 12, 30)
    );
    assert_eq!(
        cal.subtract_business_days(date(2026, 1, 1), 0).unwrap(),
        date(2025, 12, 31)
    );

    assert!(matches!(
        cal.subtract_business_days(date(2090, 1, 1), 0),
        Err(Error::DateNotCovered { .. })
    ));
}

#[test]
#[cfg(feature = "us")]
fn roll_forward_to_next_business_day() {
    let cal = Calendar::new(Country::US).unwrap();

    // Jan 1 Thu = New Year, Jan 2 Fri = work, Jan 3/4 = weekend, Jan 5-7 = work
    assert_eq!(
        cal.roll_forward(date(2026, 1, 1)).unwrap(),
        date(2026, 1, 2)
    );
    assert_eq!(
        cal.roll_forward(date(2026, 1, 2)).unwrap(),
        date(2026, 1, 2)
    );
    assert_eq!(
        cal.roll_forward(date(2026, 1, 3)).unwrap(),
        date(2026, 1, 5)
    );
    assert_eq!(
        cal.roll_forward(date(2026, 1, 4)).unwrap(),
        date(2026, 1, 5)
    );
}

#[test]
#[cfg(feature = "us")]
fn roll_backward_to_next_business_day() {
    let cal = Calendar::new(Country::US).unwrap();

    // Jan 1 Thu = New Year, Jan 2 Fri = work, Jan 3/4 = weekend, Jan 5-7 = work
    assert_eq!(
        cal.roll_backward(date(2026, 1, 1)).unwrap(),
        date(2025, 12, 31)
    );
    assert_eq!(
        cal.roll_backward(date(2026, 1, 2)).unwrap(),
        date(2026, 1, 2)
    );
    assert_eq!(
        cal.roll_backward(date(2026, 1, 3)).unwrap(),
        date(2026, 1, 2)
    );
    assert_eq!(
        cal.roll_backward(date(2026, 1, 4)).unwrap(),
        date(2026, 1, 2)
    );
}
#[test]
#[cfg(feature = "za")]
fn holidays_sharing_a_date_are_all_kept() {
    let cal = Calendar::new(Country::ZA).unwrap();

    // 2008-03-21 was both Human Rights Day and Good Friday
    let day = date(2008, 3, 21);
    let names: Vec<_> = cal.holidays(day).iter().map(|h| h.name_en).collect();
    assert_eq!(names, ["Human Rights Day", "Good Friday"]);
    assert!(cal.holidays(day).iter().all(|h| h.date == day));

    assert!(cal.is_holiday(day));
    assert_eq!(cal.holiday(day), cal.holidays(day).first());

    // a single holiday and a plain working day
    assert_eq!(cal.holidays(date(2008, 1, 1)).len(), 1);
    assert!(cal.holidays(date(2008, 1, 2)).is_empty());
    assert!(cal.holiday(date(2008, 1, 2)).is_none());
}

#[test]
#[cfg(feature = "us")]
fn roll_errors_outside_coverage() {
    let cal = Calendar::new(Country::US).unwrap();

    assert!(matches!(
        cal.roll_forward(date(1999, 1, 1)),
        Err(Error::DateNotCovered { .. })
    ));
    assert!(matches!(
        cal.roll_backward(date(2090, 1, 1)),
        Err(Error::DateNotCovered { .. })
    ));

    // 2000-01-01 is a Saturday: rolling backward leaves the covered years
    assert!(matches!(
        cal.roll_backward(date(2000, 1, 1)),
        Err(Error::DateNotCovered { date: d, .. }) if d < date(2000, 1, 1)
    ));
    assert_eq!(
        cal.roll_forward(date(2000, 1, 1)).unwrap(),
        date(2000, 1, 3)
    );
}

#[test]
#[cfg(feature = "ua")]
fn roll_forward_errors_when_result_leaves_coverage() {
    let cal = Calendar::with_years(Country::UA, 2020..2023).unwrap();

    // 2022-12-31 is a Saturday in the last covered year: the next business
    // day is in 2023, which has no holiday data
    assert!(matches!(
        cal.roll_forward(date(2022, 12, 31)),
        Err(Error::DateNotCovered { date: d, .. }) if d.year() == 2023
    ));
    assert_eq!(
        cal.roll_backward(date(2022, 12, 31)).unwrap(),
        date(2022, 12, 30)
    );
}

#[test]
#[cfg(feature = "ua")]
fn add_subtract_error_when_result_leaves_coverage() {
    let cal = Calendar::with_years(Country::UA, 2020..2022).unwrap();

    assert!(cal.add_business_days(date(2020, 1, 2), 100).is_ok());
    assert!(matches!(
        cal.add_business_days(date(2021, 6, 1), 300),
        Err(Error::DateNotCovered { .. })
    ));
    assert!(matches!(
        cal.add_business_days(date(2021, 12, 30), 5),
        Err(Error::DateNotCovered { .. })
    ));
    assert!(matches!(
        cal.subtract_business_days(date(2020, 6, 1), 300),
        Err(Error::DateNotCovered { .. })
    ));
    assert!(matches!(
        cal.subtract_business_days(date(2020, 1, 3), 5),
        Err(Error::DateNotCovered { .. })
    ));
    assert!(cal.add_business_days(date(2021, 6, 1), u32::MAX).is_err());
    assert!(
        cal.subtract_business_days(date(2021, 6, 1), u32::MAX)
            .is_err()
    );
}

#[test]
#[cfg(feature = "eg")]
fn egypt_add_subtract_with_fri_sat_weekend() {
    let cal = Calendar::new(Country::EG).unwrap();

    // Jul 1 Wed = work, Jul 2 Thu = holiday, Jul 3/4 Fri/Sat = weekend, Jul 5 Sun = work
    assert_eq!(
        cal.add_business_days(date(2026, 7, 1), 1).unwrap(),
        date(2026, 7, 5)
    );
    assert_eq!(
        cal.add_business_days(date(2026, 7, 3), 0).unwrap(),
        date(2026, 7, 5)
    );
    assert_eq!(
        cal.subtract_business_days(date(2026, 7, 5), 1).unwrap(),
        date(2026, 7, 1)
    );
    assert_eq!(
        cal.subtract_business_days(date(2026, 7, 4), 0).unwrap(),
        date(2026, 7, 1)
    );
    assert_eq!(
        cal.business_days_between(date(2026, 7, 1), date(2026, 7, 5))
            .unwrap(),
        2
    );
}

/// Counting, adding and subtracting must agree with a plain day-by-day walk
/// for Sat/Sun, Fri/Sat and Sun-only weekends.
#[test]
#[cfg(all(feature = "us", feature = "eg", feature = "ug", feature = "za"))]
fn arithmetic_matches_day_by_day_walk() {
    for country in [Country::US, Country::EG, Country::UG, Country::ZA] {
        let cal = Calendar::new(country).unwrap();

        for offset in (0..400).step_by(13) {
            let start = date(2024, 1, 1) + chrono::Days::new(offset);

            let mut day = start;
            let mut count = 0;
            for span in 0..800 {
                if cal.is_business_day(day) {
                    count += 1;
                }
                assert_eq!(
                    cal.business_days_between(start, day).unwrap(),
                    count,
                    "{country} {start}..={day} ({span} days)"
                );
                day = day.succ_opt().unwrap();
            }

            let forward_start = cal.roll_forward(start).unwrap();
            let backward_start = cal.roll_backward(start).unwrap();
            let (mut forward, mut backward) = (forward_start, backward_start);
            for days in 0..600 {
                assert_eq!(
                    cal.add_business_days(start, days).unwrap(),
                    forward,
                    "{country} {start} + {days}"
                );
                assert_eq!(
                    cal.subtract_business_days(start, days).unwrap(),
                    backward,
                    "{country} {start} - {days}"
                );

                forward = forward.succ_opt().unwrap();
                while !cal.is_business_day(forward) {
                    forward = forward.succ_opt().unwrap();
                }
                backward = backward.pred_opt().unwrap();
                while !cal.is_business_day(backward) {
                    backward = backward.pred_opt().unwrap();
                }
            }
        }
    }
}

#[test]
#[cfg(feature = "us")]
fn business_days_until_today_ends_today() {
    let cal = Calendar::new(Country::US).unwrap();

    // Retry if the clock crosses midnight between reading "today" and the call.
    loop {
        let today = chrono::Local::now().date_naive();
        let start = today - chrono::Days::new(30);
        let until_today = cal.business_days_until_today(start);
        let tomorrow = cal.business_days_until_today(today + chrono::Days::new(1));
        if chrono::Local::now().date_naive() != today {
            continue;
        }

        // Stays valid after the embedded data stops covering the current year.
        match cal.business_days_between(start, today) {
            Ok(expected) => {
                assert_eq!(until_today.unwrap(), expected);
                assert!(matches!(tomorrow, Err(Error::StartAfterEnd { .. })));
            }
            Err(_) => assert!(matches!(until_today, Err(Error::RangeNotCovered { .. }))),
        }
        break;
    }
}
