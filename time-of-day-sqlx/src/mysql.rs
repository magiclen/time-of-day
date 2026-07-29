//! MySQL and MariaDB `TIME` support with a time-of-day domain restriction.
//!
//! MySQL and MariaDB use fractional precision zero when `TIME` has no explicit precision.
//! Use a `TIME(6)` column when every value accepted by [`MySqlTimeOfDay`] must round-trip unchanged.

use core::{
    fmt,
    hash::{Hash, Hasher},
};

use sqlx_core::{
    decode::Decode,
    encode::{Encode, IsNull},
    error::BoxDynError,
    types::Type,
};
use sqlx_mysql::{
    MySql, MySqlTypeInfo, MySqlValueRef,
    types::{MySqlTime, MySqlTimeSign},
};
use time_of_day::{
    ComponentRangeError, HundredNanosecond, Microsecond, Millisecond, Minute, Nanosecond,
    Resolution, Second, TimeOfDay,
};

use crate::StorageError;

const MICROSECOND_NANOSECONDS: u64 = 1_000;

#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
/// Stores a value validated for the nonnegative `00:00..=24:00` subset of MySQL `TIME`.
pub struct MySqlTimeOfDay<R: Resolution> {
    inner: TimeOfDay<R>,
    encoded: MySqlTime,
}

impl<R: Resolution> MySqlTimeOfDay<R> {
    /// Validates a value against MySQL's microsecond wire resolution.
    #[inline]
    pub fn try_new(value: TimeOfDay<R>) -> Result<Self, StorageError> {
        let nanoseconds = value.nanoseconds_since_start_of_day();

        if !nanoseconds.is_multiple_of(MICROSECOND_NANOSECONDS) {
            return Err(StorageError::BackendPrecisionLoss {
                nanoseconds,
                backend_tick_nanoseconds: MICROSECOND_NANOSECONDS,
            });
        }

        let encoded = MySqlTime::new(
            MySqlTimeSign::Positive,
            u32::from(value.hour()),
            value.minute(),
            value.second(),
            value.microsecond(),
        )
        .expect("TimeOfDay values within microsecond precision are valid MySQL TIME values");

        Ok(Self {
            inner: value,
            encoded,
        })
    }

    /// Borrows the wrapped core value.
    #[inline]
    pub const fn as_inner(&self) -> &TimeOfDay<R> {
        &self.inner
    }

    /// Returns the wrapped core value.
    #[inline]
    pub const fn into_inner(self) -> TimeOfDay<R> {
        self.inner
    }
}

impl<R: Resolution> fmt::Debug for MySqlTimeOfDay<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("MySqlTimeOfDay").field(&self.inner).finish()
    }
}

impl<R: Resolution> Hash for MySqlTimeOfDay<R> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl<R: Resolution> Type<MySql> for MySqlTimeOfDay<R> {
    fn type_info() -> MySqlTypeInfo {
        <MySqlTime as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        <MySqlTime as Type<MySql>>::compatible(ty)
    }
}

impl<R: Resolution> Encode<'_, MySql> for MySqlTimeOfDay<R> {
    fn encode_by_ref(
        &self,
        buf: &mut <MySql as sqlx_core::database::Database>::ArgumentBuffer,
    ) -> Result<IsNull, BoxDynError> {
        <MySqlTime as Encode<MySql>>::encode_by_ref(&self.encoded, buf)
    }

    fn size_hint(&self) -> usize {
        <MySqlTime as Encode<MySql>>::size_hint(&self.encoded)
    }
}

impl<'r, R: Resolution> Decode<'r, MySql> for MySqlTimeOfDay<R> {
    fn decode(value: MySqlValueRef<'r>) -> Result<Self, BoxDynError> {
        let value = <MySqlTime as Decode<MySql>>::decode(value)?;

        // MySQL TIME also represents signed durations, so the wrapper must enforce its smaller domain.
        if value.sign().is_negative() {
            return Err(StorageError::NegativeDatabaseValue.into());
        }

        if value.hours() > 24
            || (value.hours() == 24
                && (value.minutes() != 0 || value.seconds() != 0 || value.microseconds() != 0))
        {
            return Err(StorageError::DatabaseValueOutOfRange {
                value: i128::from(value.hours()) * 3_600_000_000
                    + i128::from(value.minutes()) * 60_000_000
                    + i128::from(value.seconds()) * 1_000_000
                    + i128::from(value.microseconds()),
                minimum: 0,
                maximum: 86_400_000_000,
            }
            .into());
        }

        let nanoseconds = u64::from(value.hours()) * 3_600_000_000_000
            + u64::from(value.minutes()) * 60_000_000_000
            + u64::from(value.seconds()) * 1_000_000_000
            + u64::from(value.microseconds()) * MICROSECOND_NANOSECONDS;

        let inner = TimeOfDay::<R>::from_nanoseconds_since_start_of_day(nanoseconds)
            .map_err(target_loss)?;

        Ok(Self {
            inner,
            encoded: value,
        })
    }
}

fn target_loss(error: ComponentRangeError) -> StorageError {
    match error {
        ComponentRangeError::ResolutionLoss(error) => StorageError::TargetResolutionLoss {
            nanoseconds: error.nanoseconds,
            target_tick_nanoseconds: error.target_tick_nanoseconds,
        },
        _ => unreachable!("MySQL range is checked before core construction"),
    }
}

macro_rules! impl_infallible {
    ($($resolution:ty),+ $(,)?) => {
        $(
            impl From<TimeOfDay<$resolution>> for MySqlTimeOfDay<$resolution> {
                fn from(value: TimeOfDay<$resolution>) -> Self {
                    Self::try_new(value).expect("this resolution is supported by MySQL")
                }
            }
        )+
    };
}

impl_infallible!(Minute, Second, Millisecond, Microsecond);

macro_rules! impl_fallible {
    ($($resolution:ty),+ $(,)?) => {
        $(
            impl TryFrom<TimeOfDay<$resolution>> for MySqlTimeOfDay<$resolution> {
                type Error = StorageError;

                fn try_from(value: TimeOfDay<$resolution>) -> Result<Self, Self::Error> {
                    Self::try_new(value)
                }
            }
        )+
    };
}

impl_fallible!(HundredNanosecond, Nanosecond);
