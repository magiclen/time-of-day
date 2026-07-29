//! PostgreSQL `TIME WITHOUT TIME ZONE` support.
//!
//! Use a `TIME(6)` column when every value accepted by [`PgTimeOfDay`] must round-trip unchanged.
//! A lower column precision can round an encoded value after wire-format validation.

use core::fmt;

use sqlx_core::{
    decode::Decode,
    encode::{Encode, IsNull},
    error::BoxDynError,
    types::Type,
};

use sqlx_postgres::{
    PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueFormat, PgValueRef, Postgres,
};

use time_of_day::{
    ComponentRangeError, HundredNanosecond, Microsecond, Millisecond, Minute, Nanosecond,
    ParseMode, Resolution, Second, TimeOfDay,
};

use crate::StorageError;

const MICROSECOND_NANOSECONDS: u64 = 1_000;
const MICROSECONDS_PER_DAY: i64 = 86_400_000_000;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// Stores a value validated for PostgreSQL's microsecond `TIME` wire format.
pub struct PgTimeOfDay<R: Resolution>(TimeOfDay<R>);

impl<R: Resolution> PgTimeOfDay<R> {
    /// Validates a value against PostgreSQL's microsecond wire resolution.
    #[inline]
    pub fn try_new(value: TimeOfDay<R>) -> Result<Self, StorageError> {
        let nanoseconds = value.nanoseconds_since_start_of_day();
        if !nanoseconds.is_multiple_of(MICROSECOND_NANOSECONDS) {
            return Err(StorageError::BackendPrecisionLoss {
                nanoseconds,
                backend_tick_nanoseconds: MICROSECOND_NANOSECONDS,
            });
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
}

impl<R: Resolution> fmt::Debug for PgTimeOfDay<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("PgTimeOfDay").field(&self.0).finish()
    }
}

impl<R: Resolution> Type<Postgres> for PgTimeOfDay<R> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("TIME")
    }

    fn compatible(ty: &PgTypeInfo) -> bool {
        Self::type_info() == *ty
    }
}

impl<R: Resolution> PgHasArrayType for PgTimeOfDay<R> {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::array_of("TIME")
    }

    fn array_compatible(ty: &PgTypeInfo) -> bool {
        Self::array_type_info() == *ty
    }
}

impl<R: Resolution> Encode<'_, Postgres> for PgTimeOfDay<R> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> Result<IsNull, BoxDynError> {
        // PostgreSQL encodes TIME as signed microseconds since midnight.
        let microseconds =
            (self.0.nanoseconds_since_start_of_day() / MICROSECOND_NANOSECONDS) as i64;

        <i64 as Encode<Postgres>>::encode(microseconds, buf)
    }

    fn size_hint(&self) -> usize {
        core::mem::size_of::<i64>()
    }
}

impl<'r, R: Resolution> Decode<'r, Postgres> for PgTimeOfDay<R> {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let microseconds = match value.format() {
            PgValueFormat::Binary => <i64 as Decode<Postgres>>::decode(value)?,
            PgValueFormat::Text => {
                let parsed =
                    TimeOfDay::<Microsecond>::parse_with(value.as_str()?, ParseMode::Relaxed)
                        .map_err(|_| StorageError::InvalidTextValue)?;
                (parsed.nanoseconds_since_start_of_day() / MICROSECOND_NANOSECONDS) as i64
            }
        };

        if !(0..=MICROSECONDS_PER_DAY).contains(&microseconds) {
            return Err(StorageError::DatabaseValueOutOfRange {
                value: i128::from(microseconds),
                minimum: 0,
                maximum: i128::from(MICROSECONDS_PER_DAY),
            }
            .into());
        }

        let nanoseconds = microseconds as u64 * MICROSECOND_NANOSECONDS;
        let value = TimeOfDay::<R>::from_nanoseconds_since_start_of_day(nanoseconds)
            .map_err(target_loss)?;

        Ok(Self(value))
    }
}

fn target_loss(error: ComponentRangeError) -> StorageError {
    match error {
        ComponentRangeError::ResolutionLoss(error) => StorageError::TargetResolutionLoss {
            nanoseconds: error.nanoseconds,
            target_tick_nanoseconds: error.target_tick_nanoseconds,
        },
        _ => unreachable!("PostgreSQL range is checked before core construction"),
    }
}

macro_rules! impl_infallible {
    ($($resolution:ty),+ $(,)?) => {
        $(
            impl From<TimeOfDay<$resolution>> for PgTimeOfDay<$resolution> {
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
            impl TryFrom<TimeOfDay<$resolution>> for PgTimeOfDay<$resolution> {
                type Error = StorageError;

                fn try_from(value: TimeOfDay<$resolution>) -> Result<Self, Self::Error> {
                    Self::try_new(value)
                }
            }
        )+
    };
}

impl_fallible!(HundredNanosecond, Nanosecond);
