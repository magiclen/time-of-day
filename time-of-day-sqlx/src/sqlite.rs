//! SQLite support with separate `INTEGER` and `TEXT` storage policies.

use core::fmt;

use sqlx_core::{
    decode::Decode,
    encode::{Encode, IsNull},
    error::BoxDynError,
    types::Type,
};
use sqlx_sqlite::{Sqlite, SqliteArgumentsBuffer, SqliteTypeInfo, SqliteValueRef};
use time_of_day::{ParseMode, Resolution, TimeOfDay};

use crate::StorageError;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// Stores resolution-native ticks in an SQLite `INTEGER`.
pub struct SqliteIntegerTimeOfDay<R: Resolution>(TimeOfDay<R>);

impl<R: Resolution> SqliteIntegerTimeOfDay<R> {
    /// Wraps a core value for native-tick integer storage.
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

impl<R: Resolution> fmt::Debug for SqliteIntegerTimeOfDay<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SqliteIntegerTimeOfDay")
            .field(&self.0)
            .finish()
    }
}

impl<R: Resolution> From<TimeOfDay<R>> for SqliteIntegerTimeOfDay<R> {
    fn from(value: TimeOfDay<R>) -> Self {
        Self(value)
    }
}

impl<R: Resolution> Type<Sqlite> for SqliteIntegerTimeOfDay<R> {
    fn type_info() -> SqliteTypeInfo {
        <i64 as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <i64 as Type<Sqlite>>::compatible(ty)
    }
}

impl<R: Resolution> Encode<'_, Sqlite> for SqliteIntegerTimeOfDay<R> {
    fn encode_by_ref(&self, buf: &mut SqliteArgumentsBuffer) -> Result<IsNull, BoxDynError> {
        <i64 as Encode<Sqlite>>::encode(self.0.ticks() as i64, buf)
    }
}

impl<'r, R: Resolution> Decode<'r, Sqlite> for SqliteIntegerTimeOfDay<R> {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let ticks = <i64 as Decode<Sqlite>>::decode(value)?;

        if ticks < 0 || ticks as u64 > R::TICKS_PER_DAY {
            return Err(StorageError::DatabaseValueOutOfRange {
                value: i128::from(ticks),
                minimum: 0,
                maximum: i128::from(R::TICKS_PER_DAY),
            }
            .into());
        }

        Ok(Self(TimeOfDay::from_ticks(ticks as u64).expect(
            "SQLite tick range is checked before core construction",
        )))
    }
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// Stores canonical time text in an SQLite `TEXT` value.
pub struct SqliteTextTimeOfDay<R: Resolution>(TimeOfDay<R>);

impl<R: Resolution> SqliteTextTimeOfDay<R> {
    /// Wraps a core value for canonical text storage.
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

impl<R: Resolution> fmt::Debug for SqliteTextTimeOfDay<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SqliteTextTimeOfDay").field(&self.0).finish()
    }
}

impl<R: Resolution> From<TimeOfDay<R>> for SqliteTextTimeOfDay<R> {
    fn from(value: TimeOfDay<R>) -> Self {
        Self(value)
    }
}

impl<R: Resolution> Type<Sqlite> for SqliteTextTimeOfDay<R> {
    fn type_info() -> SqliteTypeInfo {
        <String as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <String as Type<Sqlite>>::compatible(ty)
    }
}

impl<R: Resolution> Encode<'_, Sqlite> for SqliteTextTimeOfDay<R> {
    fn encode_by_ref(&self, buf: &mut SqliteArgumentsBuffer) -> Result<IsNull, BoxDynError> {
        <String as Encode<Sqlite>>::encode(self.0.to_string(), buf)
    }
}

impl<'r, R: Resolution> Decode<'r, Sqlite> for SqliteTextTimeOfDay<R> {
    fn decode(value: SqliteValueRef<'r>) -> Result<Self, BoxDynError> {
        let value = <String as Decode<Sqlite>>::decode(value)?;
        let inner = TimeOfDay::parse_with(&value, ParseMode::Relaxed)
            .map_err(|_| StorageError::InvalidTextValue)?;

        Ok(Self(inner))
    }
}
