use core::fmt;

use tiberius::{ColumnData, FromSql, FromSqlOwned, IntoSql, ToSql};
use time_of_day::{ComponentRangeError, Resolution, TimeOfDay};

use crate::{StorageError, errors::conversion_error};

const NANOSECONDS_PER_DAY: i64 = 86_400_000_000_000;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// Stores nanoseconds since the start of day in a SQL Server `BIGINT`.
pub struct BigIntNanoseconds<R: Resolution>(TimeOfDay<R>);

impl<R: Resolution> BigIntNanoseconds<R> {
    /// Wraps a core value for lossless `BIGINT` nanosecond storage.
    #[inline]
    pub const fn new(value: TimeOfDay<R>) -> Self {
        Self(value)
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
}

impl<R: Resolution> fmt::Debug for BigIntNanoseconds<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BigIntNanoseconds").field(&self.0).finish()
    }
}

impl<R: Resolution> From<TimeOfDay<R>> for BigIntNanoseconds<R> {
    fn from(value: TimeOfDay<R>) -> Self {
        Self(value)
    }
}

impl<R: Resolution> ToSql for BigIntNanoseconds<R> {
    fn to_sql(&self) -> ColumnData<'_> {
        ColumnData::I64(Some(self.0.nanoseconds_since_start_of_day() as i64))
    }
}

impl<'a, R: Resolution> IntoSql<'a> for BigIntNanoseconds<R> {
    fn into_sql(self) -> ColumnData<'a> {
        ColumnData::I64(Some(self.0.nanoseconds_since_start_of_day() as i64))
    }
}

impl<'a, R: Resolution> FromSql<'a> for BigIntNanoseconds<R> {
    fn from_sql(value: &'a ColumnData<'static>) -> tiberius::Result<Option<Self>> {
        match value {
            ColumnData::I64(Some(value)) => decode(*value).map(Some),
            ColumnData::I64(None) => Ok(None),
            _ => Err(conversion_error(StorageError::UnexpectedSqlType)),
        }
    }
}

impl<R: Resolution> FromSqlOwned for BigIntNanoseconds<R> {
    fn from_sql_owned(value: ColumnData<'static>) -> tiberius::Result<Option<Self>> {
        match value {
            ColumnData::I64(Some(value)) => decode(value).map(Some),
            ColumnData::I64(None) => Ok(None),
            _ => Err(conversion_error(StorageError::UnexpectedSqlType)),
        }
    }
}

fn decode<R: Resolution>(nanoseconds: i64) -> tiberius::Result<BigIntNanoseconds<R>> {
    if !(0..=NANOSECONDS_PER_DAY).contains(&nanoseconds) {
        return Err(conversion_error(StorageError::DatabaseValueOutOfRange {
            value: i128::from(nanoseconds),
            minimum: 0,
            maximum: i128::from(NANOSECONDS_PER_DAY),
        }));
    }

    let value =
        TimeOfDay::from_nanoseconds_since_start_of_day(nanoseconds as u64).map_err(target_loss)?;

    Ok(BigIntNanoseconds(value))
}

fn target_loss(error: ComponentRangeError) -> tiberius::error::Error {
    match error {
        ComponentRangeError::ResolutionLoss(error) => {
            conversion_error(StorageError::TargetResolutionLoss {
                nanoseconds: error.nanoseconds,
                target_tick_nanoseconds: error.target_tick_nanoseconds,
            })
        }
        _ => unreachable!("BIGINT range is checked before core construction"),
    }
}
