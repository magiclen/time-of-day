Time of Day
====================

[![CI](https://github.com/magiclen/time-of-day/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/time-of-day/actions/workflows/ci.yml)

A time-of-day type supporting the inclusive range from 00:00:00 through 24:00:00.

Most time types cannot preserve `24:00` as a value distinct from `00:00`. This crate is intended for schedules, business hours, file formats, and database values where the end of a day must remain an explicit boundary.

`TimeOfDay<R>` stores its resolution in the type. The built-in resolutions are minute, second, millisecond, microsecond, one hundred nanoseconds, and nanosecond. `DynamicTimeOfDay` is available when the resolution must be inferred from input or selected at runtime.

## Quick Start

```rust
use time_of_day::{Minute, TimeOfDay};

let opens_at = "09:30".parse::<TimeOfDay<Minute>>().unwrap();
let closes_at = TimeOfDay::<Minute>::from_hour_minute(24, 0).unwrap();

assert_eq!("09:30", opens_at.to_string());
assert_eq!("24:00", closes_at.to_string());
assert!(closes_at.is_end_of_day());
```

Parsing with `str::parse` accepts exactly representable values. `TimeOfDay::parse_with` provides explicit strict, truncating, ceiling, and rounding modes. Display output keeps the selected resolution by default, and `format_compact` removes unnecessary trailing zero components.

## Features

| Feature | Purpose |
| --- | --- |
| `std` *(default)* | Builds the crate with the standard library available. |
| `serde` | Serializes and deserializes values as time strings. |
| `chrono` | Integrates with Chrono time and date types. |
| `time` | Integrates with the `time` crate's time and date types. |
| `jiff` | Integrates with Jiff civil time and date types. |

## `no_std`

Disable default features to build without the standard library:

```toml
[dependencies]
time-of-day = { version = "0.1", default-features = false }
```

Optional integration features can be enabled separately without enabling this crate's `std` feature.

## Related Crates

- [time-of-day-sqlx](../time-of-day-sqlx/README.md) provides SQLx database adapters.
- [time-of-day-tiberius](../time-of-day-tiberius/README.md) provides Tiberius and SQL Server adapters.

## Crates.io

https://crates.io/crates/time-of-day

## Documentation

https://docs.rs/time-of-day

## License

[MIT](LICENSE)