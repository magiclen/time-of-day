use core::error::Error;
use core::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Reports a SQL Server domain, resolution, or storage-policy failure.
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
    /// A Rust value cannot be represented by the SQL Server storage resolution.
    BackendPrecisionLoss {
        /// The source position in nanoseconds since the start of day.
        nanoseconds: u64,
        /// The backend tick duration in nanoseconds.
        backend_tick_nanoseconds: u64,
    },
    /// A decoded SQL Server `TIME` value has an unexpected fractional scale.
    SchemaScaleMismatch {
        /// The required fractional scale.
        expected: u8,
        /// The decoded fractional scale.
        actual: u8,
    },
    /// A literal maximum `TIME(7)` value collides with the reserved `24:00` sentinel.
    SentinelCollision,
    /// The TDS column data variant does not match the selected storage policy.
    UnexpectedSqlType,
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
            Self::SchemaScaleMismatch { expected, actual } => write!(
                f,
                "SQL Server TIME scale {actual} does not match required scale {expected}"
            ),
            Self::SentinelCollision => {
                f.write_str("the literal maximum TIME(7) value is reserved for 24:00")
            }
            Self::UnexpectedSqlType => f.write_str("unexpected SQL Server column type"),
        }
    }
}

impl Error for StorageError {}

pub(crate) fn conversion_error(error: StorageError) -> tiberius::error::Error {
    tiberius::error::Error::Conversion(error.to_string().into())
}
