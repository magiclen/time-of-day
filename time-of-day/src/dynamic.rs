use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    str::FromStr,
};

use crate::{
    FormatOptionError, FormatOptions, HundredNanosecond, Microsecond, Millisecond, Minute,
    Nanosecond, ParseTimeOfDayError, QuantizeMode, Resolution, ResolutionKind, ResolutionLoss,
    Second, TimeOfDay, format::format_options, parse::parse_raw,
};

#[derive(Clone, Copy, Debug)]
/// Stores a time of day together with its runtime resolution.
///
/// Equality, ordering, and hashing use only the nanosecond position and ignore the enum variant.
pub enum DynamicTimeOfDay {
    /// A minute-resolution value.
    Minute(TimeOfDay<Minute>),
    /// A second-resolution value.
    Second(TimeOfDay<Second>),
    /// A millisecond-resolution value.
    Millisecond(TimeOfDay<Millisecond>),
    /// A microsecond-resolution value.
    Microsecond(TimeOfDay<Microsecond>),
    /// A one-hundred-nanosecond-resolution value.
    HundredNanosecond(TimeOfDay<HundredNanosecond>),
    /// A nanosecond-resolution value.
    Nanosecond(TimeOfDay<Nanosecond>),
}

impl DynamicTimeOfDay {
    /// Returns the runtime resolution stored by this value.
    #[inline]
    pub const fn resolution(self) -> ResolutionKind {
        match self {
            Self::Minute(_) => ResolutionKind::Minute,
            Self::Second(_) => ResolutionKind::Second,
            Self::Millisecond(_) => ResolutionKind::Millisecond,
            Self::Microsecond(_) => ResolutionKind::Microsecond,
            Self::HundredNanosecond(_) => ResolutionKind::HundredNanosecond,
            Self::Nanosecond(_) => ResolutionKind::Nanosecond,
        }
    }

    /// Returns variant-native ticks since the start of day.
    ///
    /// The tick unit depends on the value returned by `resolution`.
    #[inline]
    pub fn ticks(self) -> u64 {
        match self {
            Self::Minute(value) => value.ticks(),
            Self::Second(value) => value.ticks(),
            Self::Millisecond(value) => value.ticks(),
            Self::Microsecond(value) => value.ticks(),
            Self::HundredNanosecond(value) => value.ticks(),
            Self::Nanosecond(value) => value.ticks(),
        }
    }

    /// Returns nanoseconds since the start of day independently of the stored resolution.
    #[inline]
    pub fn nanoseconds_since_start_of_day(self) -> u64 {
        match self {
            Self::Minute(value) => value.nanoseconds_since_start_of_day(),
            Self::Second(value) => value.nanoseconds_since_start_of_day(),
            Self::Millisecond(value) => value.nanoseconds_since_start_of_day(),
            Self::Microsecond(value) => value.nanoseconds_since_start_of_day(),
            Self::HundredNanosecond(value) => value.nanoseconds_since_start_of_day(),
            Self::Nanosecond(value) => value.nanoseconds_since_start_of_day(),
        }
    }

    /// Returns `true` when the value is `00:00`.
    #[inline]
    pub fn is_start_of_day(self) -> bool {
        self.nanoseconds_since_start_of_day() == 0
    }

    /// Returns `true` when the value is the distinct `24:00` boundary.
    #[inline]
    pub fn is_end_of_day(self) -> bool {
        self.nanoseconds_since_start_of_day() == crate::value::NANOSECONDS_PER_DAY
    }

    /// Converts exactly to a typed resolution.
    pub fn to_typed<R: Resolution>(self) -> Result<TimeOfDay<R>, ResolutionLoss> {
        TimeOfDay::from_nanoseconds_since_start_of_day(self.nanoseconds_since_start_of_day())
            .map_err(|_| ResolutionLoss {
                nanoseconds:             self.nanoseconds_since_start_of_day(),
                target_tick_nanoseconds: R::NANOSECONDS_PER_TICK,
            })
    }

    /// Converts to a typed resolution using an explicit quantization mode.
    pub fn to_typed_quantized<R: Resolution>(
        self,
        mode: QuantizeMode,
    ) -> Result<TimeOfDay<R>, ResolutionLoss> {
        TimeOfDay::<Nanosecond>::from_valid_ticks(self.nanoseconds_since_start_of_day())
            .quantize(mode)
    }

    /// Converts to the coarsest built-in resolution that represents the value exactly.
    pub fn minimize_resolution(self) -> Self {
        Self::from_nanoseconds_minimized(self.nanoseconds_since_start_of_day())
    }

    /// Parses relaxed syntax and infers the variant from the declared textual precision.
    ///
    /// Trailing zeros do not cause automatic resolution minimization.
    pub fn parse(input: &str) -> Result<Self, ParseTimeOfDayError> {
        let parsed = parse_raw(input)?;

        let value = match (parsed.has_seconds, parsed.fractional_digits) {
            (false, 0) => Self::Minute(
                TimeOfDay::from_nanoseconds_since_start_of_day(parsed.nanoseconds)
                    .map_err(component_to_parse)?,
            ),
            (true, 0) => Self::Second(
                TimeOfDay::from_nanoseconds_since_start_of_day(parsed.nanoseconds)
                    .map_err(component_to_parse)?,
            ),
            (_, 1..=3) => Self::Millisecond(
                TimeOfDay::from_nanoseconds_since_start_of_day(parsed.nanoseconds)
                    .map_err(component_to_parse)?,
            ),
            (_, 4..=6) => Self::Microsecond(
                TimeOfDay::from_nanoseconds_since_start_of_day(parsed.nanoseconds)
                    .map_err(component_to_parse)?,
            ),
            (_, 7) => Self::HundredNanosecond(
                TimeOfDay::from_nanoseconds_since_start_of_day(parsed.nanoseconds)
                    .map_err(component_to_parse)?,
            ),
            (_, 8..=9) => Self::Nanosecond(
                TimeOfDay::from_nanoseconds_since_start_of_day(parsed.nanoseconds)
                    .map_err(component_to_parse)?,
            ),
            _ => return Err(ParseTimeOfDayError::InvalidSyntax),
        };

        Ok(value)
    }

    /// Creates a display wrapper using the given formatting options.
    pub fn format_with(
        &self,
        options: FormatOptions,
    ) -> Result<FormattedDynamicTimeOfDay<'_>, FormatOptionError> {
        match self {
            Self::Minute(_) => format_options::<Minute>(options)?,
            Self::Second(_) => format_options::<Second>(options)?,
            Self::Millisecond(_) => format_options::<Millisecond>(options)?,
            Self::Microsecond(_) => format_options::<Microsecond>(options)?,
            Self::HundredNanosecond(_) => format_options::<HundredNanosecond>(options)?,
            Self::Nanosecond(_) => format_options::<Nanosecond>(options)?,
        }

        Ok(FormattedDynamicTimeOfDay {
            value: self,
            options,
        })
    }

    /// Creates a display wrapper that removes trailing zero components down to minutes.
    #[inline]
    pub fn format_compact(&self) -> FormattedDynamicTimeOfDay<'_> {
        FormattedDynamicTimeOfDay {
            value: self, options: FormatOptions::COMPACT
        }
    }

    pub(crate) fn from_nanoseconds_minimized(nanoseconds: u64) -> Self {
        // Checking from coarse to fine guarantees the first match is the minimal exact resolution.
        if nanoseconds.is_multiple_of(Minute::NANOSECONDS_PER_TICK) {
            Self::Minute(TimeOfDay::from_valid_ticks(nanoseconds / Minute::NANOSECONDS_PER_TICK))
        } else if nanoseconds.is_multiple_of(Second::NANOSECONDS_PER_TICK) {
            Self::Second(TimeOfDay::from_valid_ticks(nanoseconds / Second::NANOSECONDS_PER_TICK))
        } else if nanoseconds.is_multiple_of(Millisecond::NANOSECONDS_PER_TICK) {
            Self::Millisecond(TimeOfDay::from_valid_ticks(
                nanoseconds / Millisecond::NANOSECONDS_PER_TICK,
            ))
        } else if nanoseconds.is_multiple_of(Microsecond::NANOSECONDS_PER_TICK) {
            Self::Microsecond(TimeOfDay::from_valid_ticks(
                nanoseconds / Microsecond::NANOSECONDS_PER_TICK,
            ))
        } else if nanoseconds.is_multiple_of(HundredNanosecond::NANOSECONDS_PER_TICK) {
            Self::HundredNanosecond(TimeOfDay::from_valid_ticks(
                nanoseconds / HundredNanosecond::NANOSECONDS_PER_TICK,
            ))
        } else {
            Self::Nanosecond(TimeOfDay::from_valid_ticks(nanoseconds))
        }
    }
}

fn component_to_parse(error: crate::ComponentRangeError) -> ParseTimeOfDayError {
    match error {
        crate::ComponentRangeError::ResolutionLoss(error) => {
            ParseTimeOfDayError::ResolutionLoss(error)
        },
        error => ParseTimeOfDayError::ComponentOutOfRange(error),
    }
}

impl fmt::Display for DynamicTimeOfDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Minute(value) => value.fmt(f),
            Self::Second(value) => value.fmt(f),
            Self::Millisecond(value) => value.fmt(f),
            Self::Microsecond(value) => value.fmt(f),
            Self::HundredNanosecond(value) => value.fmt(f),
            Self::Nanosecond(value) => value.fmt(f),
        }
    }
}

impl FromStr for DynamicTimeOfDay {
    type Err = ParseTimeOfDayError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}

impl PartialEq for DynamicTimeOfDay {
    fn eq(&self, other: &Self) -> bool {
        self.nanoseconds_since_start_of_day() == other.nanoseconds_since_start_of_day()
    }
}

impl Eq for DynamicTimeOfDay {}

impl PartialOrd for DynamicTimeOfDay {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for DynamicTimeOfDay {
    fn cmp(&self, other: &Self) -> Ordering {
        self.nanoseconds_since_start_of_day().cmp(&other.nanoseconds_since_start_of_day())
    }
}

impl Hash for DynamicTimeOfDay {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nanoseconds_since_start_of_day().hash(state);
    }
}

macro_rules! impl_from_typed {
    ($($resolution:ident),+ $(,)?) => {
        $(
            impl From<TimeOfDay<$resolution>> for DynamicTimeOfDay {
                fn from(value: TimeOfDay<$resolution>) -> Self {
                    Self::$resolution(value)
                }
            }
        )+
    };
}

impl_from_typed!(Minute, Second, Millisecond, Microsecond, HundredNanosecond, Nanosecond,);

/// A borrowing display wrapper for a dynamically resolved time of day.
pub struct FormattedDynamicTimeOfDay<'a> {
    value:   &'a DynamicTimeOfDay,
    options: FormatOptions,
}

impl fmt::Display for FormattedDynamicTimeOfDay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            DynamicTimeOfDay::Minute(value) => crate::format::format_time(*value, self.options, f),
            DynamicTimeOfDay::Second(value) => crate::format::format_time(*value, self.options, f),
            DynamicTimeOfDay::Millisecond(value) => {
                crate::format::format_time(*value, self.options, f)
            },
            DynamicTimeOfDay::Microsecond(value) => {
                crate::format::format_time(*value, self.options, f)
            },
            DynamicTimeOfDay::HundredNanosecond(value) => {
                crate::format::format_time(*value, self.options, f)
            },
            DynamicTimeOfDay::Nanosecond(value) => {
                crate::format::format_time(*value, self.options, f)
            },
        }
    }
}
