#[allow(unused_imports)]
use business_chrono::{Calendar, Country, Error, build};
#[allow(unused_imports)]
use chrono::NaiveDate;

#[test]
#[cfg(feature = "ua")]
fn ua_new_year_2026() {
    let map = build(Country::UA, None).unwrap();
    let date = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
    let holidays = &map[&2026][&date];
    assert_eq!(holidays.len(), 1);
    let holiday = &holidays[0];

    assert_eq!(holiday.name, "Новий Рік");
    assert_eq!(holiday.name_en, "New Year's Day");
    assert_eq!(holiday.country, Country::UA);
    assert_eq!(holiday.country_name, "Ukraine");
}

#[test]
#[cfg(feature = "us")]
fn us_covers_2000_through_2031() {
    let map = build(Country::US, None).unwrap();

    assert!(map.contains_key(&2000));
    assert!(map.contains_key(&2031));
}

#[test]
#[cfg(feature = "us")]
fn year_range_filter() {
    let map = build(Country::US, Some(&(2020..2022))).unwrap();

    assert!(map.contains_key(&2020));
    assert!(map.contains_key(&2021));
    assert!(!map.contains_key(&2022));
    assert!(!map.contains_key(&2019));
}

#[test]
#[cfg(feature = "us")]
fn country_roundtrip() {
    let country: Country = "US".parse().unwrap();

    assert_eq!(country, Country::US);
    assert_eq!(country.code(), "US");
    assert_eq!(country.name(), "United States");
    assert_eq!(country.to_string(), "US");

    assert!(Country::ALL.contains(&country));
    assert!(matches!(
        "XX".parse::<Country>(),
        Err(Error::UnknownCountryCode(code)) if code == "XX"
    ));

    assert_eq!("us".parse::<Country>().unwrap(), Country::US);
    assert_eq!(" Us\n".parse::<Country>().unwrap(), Country::US);
}

#[test]
#[cfg(feature = "za")]
fn duplicate_dates_are_kept_in_raw_map() {
    let map = build(Country::ZA, None).unwrap();
    let date = NaiveDate::from_ymd_opt(2008, 3, 21).unwrap();

    assert_eq!(map[&2008][&date].len(), 2);

    // years iterate in order, without gaps
    let years: Vec<_> = map.keys().copied().collect();
    assert_eq!(years.first(), Some(&2000));
    assert!(years.windows(2).all(|pair| pair[1] == pair[0] + 1));
}

#[test]
#[cfg(feature = "all")]
fn every_country_has_consistent_data() {
    for country in Country::ALL {
        assert_eq!(country.code().parse::<Country>().unwrap(), country);
        assert_eq!(
            country.code().to_lowercase().parse::<Country>().unwrap(),
            country
        );
        assert!(!country.name().is_empty(), "{country}");
        assert!((1..=2).contains(&country.weekend().len()), "{country}");

        let map = build(country, None).unwrap();
        let years: Vec<_> = map.keys().copied().collect();
        assert_eq!(years.first(), Some(&2000), "{country}");
        assert!(
            years.windows(2).all(|pair| pair[1] == pair[0] + 1),
            "{country}: years have gaps"
        );

        for (date, holidays) in map.values().flatten() {
            assert!(!holidays.is_empty(), "{country} {date}");
            for holiday in holidays {
                assert_eq!(holiday.date, *date, "{country}");
                assert_eq!(holiday.country, country);
                assert_eq!(holiday.country_name, country.name(), "{country}");
                assert!(!holiday.name.is_empty(), "{country} {date}");
                assert!(!holiday.name_en.is_empty(), "{country} {date}");
            }
        }

        let cal = Calendar::new(country).unwrap();
        assert_eq!(cal.country(), country);
        assert_eq!(cal.covered_years(), years[0]..=years[years.len() - 1]);
    }
}

#[test]
#[cfg(not(feature = "ua"))]
fn disabled_country_is_unsupported() {
    let error = Calendar::new(Country::UA).unwrap_err();

    assert!(matches!(
        error,
        Error::UnsupportedCountry {
            country: Country::UA
        }
    ));
    assert!(
        error.to_string().contains("enable the `ua` cargo feature"),
        "{error}"
    );
    assert!(matches!(
        build(Country::UA, None),
        Err(Error::UnsupportedCountry { .. })
    ));
}
