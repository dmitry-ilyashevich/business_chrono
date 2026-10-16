# Changelog

All notable changes to this project will be documented in this file.
## [0.3.0] - 2026-10-16

### 🐛 Bug Fixes

- *(api)* Fixed api design for methods business_chrono_between, add_business_chrono, subtract_business_chrono
- *(calendar)* [**breaking**] Keep all holidays sharing a date and add Calendar::holidays()
- *(calendar)* Return DateNotCovered from roll_forward/roll_backward outside covered years
- *(calendar)* Return DateNotCovered when add/subtract_business_days result leaves covered years
- *(builder)* Fail when a country has no cached data instead of emitting a non-compiling crate
- *(api)* [**breaking**] Require explicit end date in business_days_between, add business_days_until_today
- *(builder)* Do not retry non-retryable HTTP statuses and drop the extra sleep between retries
- *(builder)* Run rustfmt with edition 2024 on generated files

### 🚜 Refactor

- *(data)* [**breaking**] Use BTreeMap for HolidayPerYearMap for deterministic year order

### ⚡ Performance

- *(calendar)* Count business days arithmetically and jump whole weeks in add/subtract
- *(calendar)* Store weekend as a bitmask

### 🧪 Testing

- Fix misnamed tests, cover Fri/Sat weekend arithmetic and case-insensitive country parsing
- Check data consistency for all contries, assert exact error variants
- *(calendar)* Cover business_days_until_today without depending on the current date

### ⚙️ Miscellaneous Tasks

- *(config)* Fix Git cliff config for release notes
## [0.2.1] - 2026-10-09

### ⚙️ Miscellaneous Tasks

- *(doc)* Fix small issues in examples
## [0.2.0] - 2026-08-22

### 🐛 Bug Fixes

- *(errors)* Correct way to deal with errors using thiserror
## [0.1.0] - 2026-08-16

### 🚀 Features

- *(calendar)* Initial support of calendar logic
- *(builder)* Load available countries list
- *(builder)* Validate before caching so a truncated response never poisons the cache
- *(builder)* Ability to force refetching data even if cache exists
- *(builder)* Weekend days loader for each country
- *(builder)* Generate country holidays data
- *(builder)* Generate country enum module
- *(builder)* Weekdata builder
- *(builder)* Generate mod.rs with all generated countries
- *(builder)* Updater for Cargo.toml features section
- *(data)* Update data with holidays data for all available countries
- *(calendar)* Business days between function
- *(calendar)* Extended business days calculations
- *(builder)* Added possibility to change start year

### 🐛 Bug Fixes

- *(builder)* Filter holidays to only global & public
- *(builder)* Mod generator does not escaped country codes, that collide with rust keywords
- *(builder)* Sort country codes in generated mods alphabetically

### ⚙️ Miscellaneous Tasks

- *(tests)* Skip tests for excluded countries
- *(tests)* Better test coverage, depending of features configuration
- Added LICENSE file
- Updated README
- Fixed formatting
- Fixed typos in tests
- Added github actions for CI checks
- More tests
- Updated GitHub checkout action to v5
- Add git-cliff config
- Correct crate keywords
- Rename crate
- Fixed formatting
