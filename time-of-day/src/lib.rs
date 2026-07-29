/*!
# Time of Day

A compact time-of-day type that supports the inclusive range from `00:00` through `24:00`.

Most time crates represent midnight but cannot preserve `24:00` as a distinct end-of-day boundary.
This crate keeps `00:00` and `24:00` separate, which is useful for schedules, business hours, file formats, and database values where the end of a day must not become the start of the same day.

[`TimeOfDay`] stores a resolution in its type through one of the built-in resolution markers: [`Minute`], [`Second`], [`Millisecond`], [`Microsecond`], [`HundredNanosecond`], or [`Nanosecond`].
Use [`DynamicTimeOfDay`] when the resolution must be selected at runtime or inferred from text.

## Quick start

```rust,no_run
use time_of_day::{Minute, TimeOfDay};

let opens_at = "09:30".parse::<TimeOfDay<Minute>>().unwrap();
let closes_at = TimeOfDay::<Minute>::from_hour_minute(24, 0).unwrap();

assert_eq!("09:30", opens_at.to_string());
assert_eq!("24:00", closes_at.to_string());
assert!(closes_at.is_end_of_day());
```

Parsing with [`str::parse`] accepts an exactly representable value.
Use [`TimeOfDay::parse_with`] to select strict parsing, truncation, ceiling, or rounding explicitly.
Formatting preserves the type-level resolution by default, while [`TimeOfDay::format_compact`] removes unnecessary trailing zero components.

## Features

| Feature | Purpose |
| --- | --- |
| `std` *(default)* | Builds the crate with the standard library available. |
| `serde` | Serializes and deserializes typed and dynamic values as time strings. |
| `chrono` | Converts to and from Chrono time types and attaches values to Chrono dates. |
| `time` | Converts to and from `time` crate types and attaches values to `time::Date`. |
| `jiff` | Converts to and from Jiff civil time types and attaches values to Jiff dates. |

## `no_std`

Disable default features to build without the standard library:

```toml
[dependencies]
time-of-day = { version = "0.1", default-features = false }
```

The optional integration features are also configured without their dependencies' default features, so they can be enabled separately when the target environment supports them.
 */

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod dynamic;
mod errors;
mod format;
mod parse;
mod resolution;
mod value;

#[cfg(any(feature = "chrono", feature = "jiff", feature = "time"))]
mod datetime;
#[cfg(feature = "serde")]
mod serde_impl;

pub use dynamic::{DynamicTimeOfDay, FormattedDynamicTimeOfDay};
pub use errors::{
    ArithmeticError, ComponentRangeError, ConversionError, FormatOptionError, ParseTimeOfDayError,
    ResolutionLoss,
};
pub use format::FormattedTimeOfDay;
pub use resolution::{
    HundredNanosecond, Microsecond, Millisecond, Minute, Nanosecond, Resolution, ResolutionKind,
    Second,
};
pub use value::TimeOfDay;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Selects lexical validation and quantization behavior while parsing.
pub enum ParseMode {
    /// Requires the canonical syntax and precision for the target resolution.
    Strict,
    /// Accepts omitted finer components but requires an exactly representable value.
    Relaxed,
    /// Discards the portion below the target resolution.
    Truncate,
    /// Advances to the next target tick when a discarded portion is nonzero.
    Ceil,
    /// Selects the nearest target tick and resolves ties toward the end of day.
    Round,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Selects how a value is converted to another resolution.
pub enum QuantizeMode {
    /// Requires the source value to be exactly representable.
    Exact,
    /// Discards the portion below the target resolution.
    Truncate,
    /// Advances to the next target tick when a discarded portion is nonzero.
    Ceil,
    /// Selects the nearest target tick and resolves ties toward the end of day.
    Round,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Controls whether canonical trailing zero components are retained.
pub enum TrailingZeros {
    /// Keeps every component required by the type-level resolution.
    Keep,
    /// Trims zero components without producing output coarser than the given resolution.
    TrimAfter(ResolutionKind),
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Configures a display-compatible time-of-day formatter.
pub struct FormatOptions {
    /// Controls the removal of trailing zero components.
    pub trailing_zeros: TrailingZeros,
}

impl FormatOptions {
    /// Trims zero components down to minutes.
    pub const COMPACT: Self = Self {
        trailing_zeros: TrailingZeros::TrimAfter(ResolutionKind::Minute),
    };
    /// Keeps the canonical representation of the type-level resolution.
    pub const KEEP: Self = Self {
        trailing_zeros: TrailingZeros::Keep
    };

    /// Creates options that retain at least the given resolution.
    #[inline]
    pub const fn trim_after(minimum: ResolutionKind) -> Self {
        Self {
            trailing_zeros: TrailingZeros::TrimAfter(minimum)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Describes the calendar-day adjustment produced while normalizing `24:00`.
pub enum DayOffset {
    /// Keeps the associated time on the original date.
    SameDay,
    /// Moves the associated time to the next date.
    NextDay,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Holds an external time and the day adjustment needed to represent the source value exactly.
pub struct NormalizedTime<T> {
    /// The calendar-day adjustment associated with `time`.
    pub day_offset: DayOffset,
    /// The normalized external time value.
    pub time:       T,
}
