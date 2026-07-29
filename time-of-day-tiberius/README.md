Time of Day For Tiberius
====================

[![CI](https://github.com/magiclen/time-of-day/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/time-of-day/actions/workflows/ci.yml)

Tiberius and SQL Server integrations for time-of-day.

SQL Server `TIME` cannot directly represent the distinct `24:00` boundary. This crate provides two explicit storage policies for Tiberius:

- `BigIntNanoseconds` stores nanoseconds since the start of day in a SQL Server `BIGINT`. It preserves every built-in resolution and the complete `00:00..=24:00` domain.
- `Time7EndOfDaySentinel` stores values in `TIME(7)` and reserves `23:59:59.9999999` as the sentinel for `24:00`. The literal maximum `TIME(7)` value is therefore unavailable.

Both wrappers implement the Tiberius conversion traits needed for query parameters and row values.

## Quick Start

```rust
use time_of_day::{Nanosecond, TimeOfDay};
use time_of_day_tiberius::BigIntNanoseconds;

let value = TimeOfDay::<Nanosecond>::END_OF_DAY;
let stored = BigIntNanoseconds::from(value);

assert_eq!(&value, stored.as_inner());
```

Use `Time7EndOfDaySentinel::try_new` for finer-resolution values because construction can fail on a sentinel collision or precision loss.

## Features

This crate has no optional features. Tiberius support for SQL Server `TIME` is enabled internally through its `tds73` feature.

## `no_std`

This adapter crate requires the standard library through Tiberius. For a `no_std` time-of-day type without database integration, use `time-of-day` with `default-features = false`.

## Crates.io

https://crates.io/crates/time-of-day-tiberius

## Documentation

https://docs.rs/time-of-day-tiberius

## License

[MIT](LICENSE)