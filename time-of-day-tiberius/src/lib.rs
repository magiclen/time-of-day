/*!
# Time of Day for Tiberius

Tiberius storage adapters for [`time_of_day::TimeOfDay`] and SQL Server.

SQL Server `TIME` cannot represent the distinct `24:00` boundary directly.
This crate provides two explicit storage policies so applications can choose between a lossless integer representation and a conventional `TIME(7)` column with a reserved sentinel.

## Quick start

[`BigIntNanoseconds`] stores nanoseconds since the start of day in a SQL Server `BIGINT`:

```rust,no_run
use time_of_day::{Nanosecond, TimeOfDay};
use time_of_day_tiberius::BigIntNanoseconds;

let value = TimeOfDay::<Nanosecond>::END_OF_DAY;
let stored = BigIntNanoseconds::from(value);

assert_eq!(&value, stored.as_inner());
```

[`Time7EndOfDaySentinel`] uses the maximum `TIME(7)` value as a sentinel for `24:00`.
This preserves the SQL `TIME` type but makes the literal `23:59:59.9999999` unavailable.
Construction rejects values that collide with the sentinel or cannot be represented at one-hundred-nanosecond resolution.

This crate has no optional features.

## `no_std`

This adapter crate requires the standard library through Tiberius.
Use the underlying `time-of-day` crate with `default-features = false` when a `no_std` time-of-day type is needed without database integration.
*/

mod bigint;
mod errors;
mod sentinel;

pub use bigint::BigIntNanoseconds;
pub use errors::StorageError;
pub use sentinel::Time7EndOfDaySentinel;
