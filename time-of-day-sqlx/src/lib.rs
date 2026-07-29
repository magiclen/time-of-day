/*!
# Time of Day for SQLx

SQLx database adapters for [`time_of_day::TimeOfDay`].

The core time type deliberately does not implement one universal SQL representation.
This crate provides wrapper types with explicit storage policies, backend validation, and SQLx [`Type`], [`Encode`], and [`Decode`] implementations.

[`Type`]: sqlx_core::types::Type
[`Encode`]: sqlx_core::encode::Encode
[`Decode`]: sqlx_core::decode::Decode

## Quick start

Enable the feature for the selected backend, then wrap a core value before binding it to a query or use the same wrapper type when decoding a row:

```rust,no_run
# #[cfg(feature = "postgres")]
# fn main() {
use time_of_day::{Microsecond, TimeOfDay};
use time_of_day_sqlx::postgres::PgTimeOfDay;

let value = TimeOfDay::<Microsecond>::from_hms_micro(12, 30, 15, 123_456).unwrap();
let stored = PgTimeOfDay::from(value);

assert_eq!(&value, stored.as_inner());
# }
# #[cfg(not(feature = "postgres"))]
# fn main() {}
```

## Backend features

| Feature | Wrapper and storage policy |
| --- | --- |
| `postgres` | `postgres::PgTimeOfDay` uses PostgreSQL `TIME` with microsecond wire resolution. |
| `mysql` | `mysql::MySqlTimeOfDay` accepts the nonnegative `00:00..=24:00` subset of MySQL or MariaDB `TIME` with microsecond wire resolution. |
| `sqlite` | `sqlite::SqliteIntegerTimeOfDay` stores resolution-native ticks, while `sqlite::SqliteTextTimeOfDay` stores canonical text. The bundled SQLite library is enabled. |

No backend feature is enabled by default.
[`StorageError`] remains available without selecting a backend.

## Schema precision

Wire-format validation cannot inspect a column's declared fractional precision.
Use PostgreSQL `TIME(6)` or MySQL and MariaDB `TIME(6)` when every value accepted by the corresponding wrapper must round-trip unchanged.
MySQL and MariaDB use fractional precision zero when `TIME` has no explicit precision.
A lower column precision can round or truncate an encoded value.

## `no_std`

This adapter crate requires the standard library through SQLx.
Use the underlying `time-of-day` crate with `default-features = false` when a `no_std` time-of-day type is needed without database integration.
*/

#![cfg_attr(docsrs, feature(doc_cfg))]

mod errors;

#[cfg(feature = "mysql")]
pub mod mysql;
#[cfg(feature = "postgres")]
pub mod postgres;
#[cfg(feature = "sqlite")]
pub mod sqlite;

pub use errors::StorageError;
