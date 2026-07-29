Time of Day For SQLx
====================

[![CI](https://github.com/magiclen/time-of-day/actions/workflows/ci.yml/badge.svg)](https://github.com/magiclen/time-of-day/actions/workflows/ci.yml)

SQLx integrations for time-of-day.

The core `TimeOfDay` type does not assume one universal SQL representation. This crate provides wrapper types with explicit storage policies, backend validation, and SQLx `Type`, `Encode`, and `Decode` implementations.

## Quick Start

Enable the feature for the selected backend:

```toml
[dependencies]
time-of-day = "0.1"
time-of-day-sqlx = { version = "0.1", features = ["postgres"] }
```

Wrap a value before binding it to a query, and use the same wrapper type when decoding a row:

```rust
use time_of_day::{Microsecond, TimeOfDay};
use time_of_day_sqlx::postgres::PgTimeOfDay;

let value = TimeOfDay::<Microsecond>::from_hms_micro(12, 30, 15, 123_456).unwrap();
let stored = PgTimeOfDay::from(value);

assert_eq!(&value, stored.as_inner());
```

## Features

No backend feature is enabled by default.

| Feature | Wrapper and storage policy |
| --- | --- |
| `postgres` | `PgTimeOfDay` uses PostgreSQL `TIME` with microsecond wire resolution. |
| `mysql` | `MySqlTimeOfDay` accepts the nonnegative `00:00..=24:00` subset of MySQL or MariaDB `TIME` with microsecond wire resolution. |
| `sqlite` | `SqliteIntegerTimeOfDay` stores resolution-native ticks, while `SqliteTextTimeOfDay` stores canonical text. This feature uses bundled SQLite. |

The PostgreSQL and MySQL wrappers reject values that would lose precision in their microsecond wire formats. The SQLite integer policy depends on the selected Rust resolution, while the text policy uses the canonical `TimeOfDay` display format.

## Schema Precision

Wire-format validation cannot detect the fractional precision declared by a database column.

- For PostgreSQL, use `TIME(6)` when every value accepted by `PgTimeOfDay` must round-trip unchanged, or select a lower `TIME(p)` only when the application does not store finer values.
- MySQL and MariaDB use fractional precision zero when `TIME` has no explicit precision. Use `TIME(6)` when every value accepted by `MySqlTimeOfDay` must round-trip unchanged.

A column with lower fractional precision can round or truncate a value after the wrapper has validated and encoded it.

## `no_std`

This adapter crate requires the standard library through SQLx. For a `no_std` time-of-day type without database integration, use `time-of-day` with `default-features = false`.

## Crates.io

https://crates.io/crates/time-of-day-sqlx

## Documentation

https://docs.rs/time-of-day-sqlx

## License

[MIT](LICENSE)