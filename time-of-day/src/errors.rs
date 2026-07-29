use core::{error::Error, fmt};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Describes a value that is not exactly aligned to a target resolution.
pub struct ResolutionLoss {
    /// The source position in nanoseconds since the start of day.
    pub nanoseconds:             u64,
    /// The target tick duration in nanoseconds.
    pub target_tick_nanoseconds: u64,
}

impl fmt::Display for ResolutionLoss {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} nanoseconds is not divisible by the target tick of {} nanoseconds",
            self.nanoseconds, self.target_tick_nanoseconds
        )
    }
}

impl Error for ResolutionLoss {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Reports invalid components supplied to a validated constructor.
pub enum ComponentRangeError {
    /// The hour is outside `0..=24`.
    Hour(u8),
    /// The minute is outside `0..=59`.
    Minute(u8),
    /// The second is outside `0..=59`.
    Second(u8),
    /// The millisecond component is outside `0..=999`.
    Millisecond(u16),
    /// The microsecond component is outside `0..=999_999`.
    Microsecond(u32),
    /// The one-hundred-nanosecond component is outside `0..=9_999_999`.
    HundredNanosecond(u32),
    /// The nanosecond component is outside `0..=999_999_999`.
    Nanosecond(u32),
    /// The tick count is outside the target resolution range.
    Ticks {
        /// The supplied tick count.
        ticks:   u64,
        /// The largest accepted tick count.
        maximum: u64,
    },
    /// The nanosecond position is after the end of day.
    NanosecondsSinceStartOfDay(u64),
    /// A nonzero component follows hour 24.
    NonZeroEndOfDayComponent,
    /// The value is not aligned to the target resolution.
    ResolutionLoss(ResolutionLoss),
}

impl fmt::Display for ComponentRangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hour(value) => write!(f, "hour {value} is outside 0..=24"),
            Self::Minute(value) => write!(f, "minute {value} is outside 0..=59"),
            Self::Second(value) => write!(f, "second {value} is outside 0..=59"),
            Self::Millisecond(value) => write!(f, "millisecond {value} is outside 0..=999"),
            Self::Microsecond(value) => write!(f, "microsecond {value} is outside 0..=999999"),
            Self::HundredNanosecond(value) => {
                write!(f, "100-nanosecond component {value} is outside 0..=9999999")
            },
            Self::Nanosecond(value) => write!(f, "nanosecond {value} is outside 0..=999999999"),
            Self::Ticks {
                ticks,
                maximum,
            } => write!(f, "tick count {ticks} is outside 0..={maximum}"),
            Self::NanosecondsSinceStartOfDay(value) => {
                write!(f, "nanosecond position {value} is outside the current day")
            },
            Self::NonZeroEndOfDayComponent => {
                f.write_str("hour 24 requires every later component to be zero")
            },
            Self::ResolutionLoss(error) => error.fmt(f),
        }
    }
}

impl Error for ComponentRangeError {}

impl From<ResolutionLoss> for ComponentRangeError {
    fn from(value: ResolutionLoss) -> Self {
        Self::ResolutionLoss(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Reports a syntax, range, or resolution failure while parsing.
pub enum ParseTimeOfDayError {
    /// The input does not follow the accepted grammar.
    InvalidSyntax,
    /// A parsed component is outside its valid range.
    ComponentOutOfRange(ComponentRangeError),
    /// The parsed source value is after `24:00`.
    AfterEndOfDay,
    /// The parsed value is not aligned to the target resolution.
    ResolutionLoss(ResolutionLoss),
    /// Quantization unexpectedly produced a value after `24:00`.
    RoundingOverflow,
}

impl fmt::Display for ParseTimeOfDayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSyntax => f.write_str("invalid time-of-day syntax"),
            Self::ComponentOutOfRange(error) => error.fmt(f),
            Self::AfterEndOfDay => f.write_str("time is after 24:00"),
            Self::ResolutionLoss(error) => error.fmt(f),
            Self::RoundingOverflow => f.write_str("rounding exceeded 24:00"),
        }
    }
}

impl Error for ParseTimeOfDayError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Reports a failure while converting to another time or date representation.
pub enum ConversionError {
    /// The source value is not aligned to the target resolution.
    ResolutionLoss(ResolutionLoss),
    /// The target time type cannot represent `24:00`.
    EndOfDayNotRepresentable,
    /// The source uses a leap-second representation outside this crate's domain.
    UnsupportedLeapSecond,
    /// Moving `24:00` to the next date exceeded the external date range.
    DateOverflow,
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResolutionLoss(error) => error.fmt(f),
            Self::EndOfDayNotRepresentable => {
                f.write_str("24:00 is not representable by the target type")
            },
            Self::UnsupportedLeapSecond => f.write_str("leap seconds are not supported"),
            Self::DateOverflow => f.write_str("normalizing 24:00 overflowed the date"),
        }
    }
}

impl Error for ConversionError {}

impl From<ResolutionLoss> for ConversionError {
    fn from(value: ResolutionLoss) -> Self {
        Self::ResolutionLoss(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Reports a resolution or range failure during arithmetic.
pub enum ArithmeticError {
    /// The duration is not aligned to the value resolution.
    ResolutionLoss(ResolutionLoss),
    /// The result is outside `00:00..=24:00`.
    OutOfRange,
}

impl fmt::Display for ArithmeticError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResolutionLoss(error) => error.fmt(f),
            Self::OutOfRange => f.write_str("arithmetic result is outside 00:00..=24:00"),
        }
    }
}

impl Error for ArithmeticError {}

impl From<ResolutionLoss> for ArithmeticError {
    fn from(value: ResolutionLoss) -> Self {
        Self::ResolutionLoss(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Reports a formatter minimum that is finer than the value resolution.
pub struct FormatOptionError {
    /// The requested minimum output resolution.
    pub minimum:          crate::ResolutionKind,
    /// The type-level resolution of the formatted value.
    pub value_resolution: crate::ResolutionKind,
}

impl fmt::Display for FormatOptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "minimum {:?} is finer than value resolution {:?}",
            self.minimum, self.value_resolution
        )
    }
}

impl Error for FormatOptionError {}
