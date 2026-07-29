use core::error::Error;
use core::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Reports a database domain, resolution, or storage-policy failure.
pub enum StorageError {
    /// A decoded numeric value is outside the time-of-day domain.
    DatabaseValueOutOfRange {
        /// The decoded database value.
        value: i128,
        /// The smallest accepted value.
        minimum: i128,
        /// The largest accepted value.
        maximum: i128,
    },
    /// A decoded value is not aligned to the requested Rust resolution.
    TargetResolutionLoss {
        /// The decoded position in nanoseconds since the start of day.
        nanoseconds: u64,
        /// The target Rust tick duration in nanoseconds.
        target_tick_nanoseconds: u64,
    },
    /// A Rust value cannot be represented by the backend wire resolution.
    BackendPrecisionLoss {
        /// The source position in nanoseconds since the start of day.
        nanoseconds: u64,
        /// The backend tick duration in nanoseconds.
        backend_tick_nanoseconds: u64,
    },
    /// A negative database time was decoded.
    NegativeDatabaseValue,
    /// A text database value does not contain an accepted time-of-day string.
    InvalidTextValue,
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DatabaseValueOutOfRange {
                value,
                minimum,
                maximum,
            } => write!(f, "database value {value} is outside {minimum}..={maximum}"),
            Self::TargetResolutionLoss {
                nanoseconds,
                target_tick_nanoseconds,
            } => write!(
                f,
                "{nanoseconds} nanoseconds is not divisible by the target tick of {target_tick_nanoseconds} nanoseconds"
            ),
            Self::BackendPrecisionLoss {
                nanoseconds,
                backend_tick_nanoseconds,
            } => write!(
                f,
                "{nanoseconds} nanoseconds is not divisible by the backend tick of {backend_tick_nanoseconds} nanoseconds"
            ),
            Self::NegativeDatabaseValue => {
                f.write_str("negative database time is outside the time-of-day domain")
            }
            Self::InvalidTextValue => f.write_str("database text is not a valid time of day"),
        }
    }
}

impl Error for StorageError {}
