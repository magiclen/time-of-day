use core::fmt;

use tiberius::{ColumnData, FromSql, FromSqlOwned, IntoSql, ToSql, time::Time};
use time_of_day::{
    ComponentRangeError, HundredNanosecond, Microsecond, Millisecond, Minute, Nanosecond,
    Resolution, Second, TimeOfDay,
};

use crate::{StorageError, errors::conversion_error};

const TDS_SCALE: u8 = 7;
const TDS_TICK_NANOSECONDS: u64 = 100;
const TDS_SENTINEL_INCREMENT: u64 = 863_999_999_999;
const SENTINEL_NANOSECONDS: u64 = 86_400_000_000_000 - TDS_TICK_NANOSECONDS;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// Stores values in SQL Server `TIME(7)` with its maximum value reserved for `24:00`.
pub struct Time7EndOfDaySentinel<R: Resolution>(TimeOfDay<R>);

impl<R: Resolution> Time7EndOfDaySentinel<R> {
    /// Validates alignment and rejects a literal value that collides with the sentinel.
    #[inline]
    pub fn try_new(value: TimeOfDay<R>) -> Result<Self, StorageError> {
        let nanoseconds = value.nanoseconds_since_start_of_day();

        if value.is_end_of_day() {
            return Ok(Self(value));
        }

        if !nanoseconds.is_multiple_of(TDS_TICK_NANOSECONDS) {
            return Err(StorageError::BackendPrecisionLoss {
                nanoseconds,
                backend_tick_nanoseconds: TDS_TICK_NANOSECONDS,
            });
        }

        if nanoseconds == SENTINEL_NANOSECONDS {
            return Err(StorageError::SentinelCollision);
        }

        Ok(Self(value))
    }

    /// Borrows the wrapped core value.
    #[inline]
    pub const fn as_inner(&self) -> &TimeOfDay<R> {
        &self.0
    }

    /// Returns the wrapped core value.
    #[inline]
    pub const fn into_inner(self) -> TimeOfDay<R> {
        self.0
    }

    fn tds_time(self) -> Time {
        // SQL Server cannot encode 24:00, so the maximum TIME(7) increment is reserved as a sentinel.
        let increments = if self.0.is_end_of_day() {
            TDS_SENTINEL_INCREMENT
        } else {
            self.0.nanoseconds_since_start_of_day() / TDS_TICK_NANOSECONDS
        };

        Time::new(increments, TDS_SCALE)
    }
}

impl<R: Resolution> fmt::Debug for Time7EndOfDaySentinel<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Time7EndOfDaySentinel")
            .field(&self.0)
            .finish()
    }
}

impl<R: Resolution> ToSql for Time7EndOfDaySentinel<R> {
    fn to_sql(&self) -> ColumnData<'_> {
        ColumnData::Time(Some(self.tds_time()))
    }
}

impl<'a, R: Resolution> IntoSql<'a> for Time7EndOfDaySentinel<R> {
    fn into_sql(self) -> ColumnData<'a> {
        ColumnData::Time(Some(self.tds_time()))
    }
}

impl<'a, R: Resolution> FromSql<'a> for Time7EndOfDaySentinel<R> {
    fn from_sql(value: &'a ColumnData<'static>) -> tiberius::Result<Option<Self>> {
        match value {
            ColumnData::Time(Some(value)) => decode(*value).map(Some),
            ColumnData::Time(None) => Ok(None),
            _ => Err(conversion_error(StorageError::UnexpectedSqlType)),
        }
    }
}

impl<R: Resolution> FromSqlOwned for Time7EndOfDaySentinel<R> {
    fn from_sql_owned(value: ColumnData<'static>) -> tiberius::Result<Option<Self>> {
        match value {
            ColumnData::Time(Some(value)) => decode(value).map(Some),
            ColumnData::Time(None) => Ok(None),
            _ => Err(conversion_error(StorageError::UnexpectedSqlType)),
        }
    }
}

fn decode<R: Resolution>(value: Time) -> tiberius::Result<Time7EndOfDaySentinel<R>> {
    // Scale validation prevents the maximum value of a coarser TIME column from being treated as the sentinel.
    if value.scale() != TDS_SCALE {
        return Err(conversion_error(StorageError::SchemaScaleMismatch {
            expected: TDS_SCALE,
            actual: value.scale(),
        }));
    }

    if value.increments() > TDS_SENTINEL_INCREMENT {
        return Err(conversion_error(StorageError::DatabaseValueOutOfRange {
            value: i128::from(value.increments()),
            minimum: 0,
            maximum: i128::from(TDS_SENTINEL_INCREMENT),
        }));
    }

    if value.increments() == TDS_SENTINEL_INCREMENT {
        return Ok(Time7EndOfDaySentinel(TimeOfDay::END_OF_DAY));
    }

    let nanoseconds = value.increments() * TDS_TICK_NANOSECONDS;
    let inner = TimeOfDay::from_nanoseconds_since_start_of_day(nanoseconds).map_err(target_loss)?;

    Ok(Time7EndOfDaySentinel(inner))
}

fn target_loss(error: ComponentRangeError) -> tiberius::error::Error {
    match error {
        ComponentRangeError::ResolutionLoss(error) => {
            conversion_error(StorageError::TargetResolutionLoss {
                nanoseconds: error.nanoseconds,
                target_tick_nanoseconds: error.target_tick_nanoseconds,
            })
        }
        _ => unreachable!("TIME(7) range is checked before core construction"),
    }
}

macro_rules! impl_infallible {
    ($($resolution:ty),+ $(,)?) => {
        $(
            impl From<TimeOfDay<$resolution>> for Time7EndOfDaySentinel<$resolution> {
                fn from(value: TimeOfDay<$resolution>) -> Self {
                    Self(value)
                }
            }
        )+
    };
}

impl_infallible!(Minute, Second, Millisecond, Microsecond);

macro_rules! impl_fallible {
    ($($resolution:ty),+ $(,)?) => {
        $(
            impl TryFrom<TimeOfDay<$resolution>> for Time7EndOfDaySentinel<$resolution> {
                type Error = StorageError;

                fn try_from(value: TimeOfDay<$resolution>) -> Result<Self, Self::Error> {
                    Self::try_new(value)
                }
            }
        )+
    };
}

impl_fallible!(HundredNanosecond, Nanosecond);
