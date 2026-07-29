use core::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
    marker::PhantomData,
    str::FromStr,
    time::Duration,
};

use crate::{
    ArithmeticError, ComponentRangeError, FormatOptions, FormattedTimeOfDay, ParseMode,
    ParseTimeOfDayError, QuantizeMode, Resolution, ResolutionLoss, TrailingZeros,
    format::format_options,
    parse::parse_for,
    resolution::{storage_from_u64, storage_into_u64},
};

const NANOSECONDS_PER_SECOND: u64 = 1_000_000_000;
pub(crate) const NANOSECONDS_PER_DAY: u64 = 86_400 * NANOSECONDS_PER_SECOND;

/// A time-of-day value at resolution `R` in the inclusive range `00:00..=24:00`.
pub struct TimeOfDay<R: Resolution> {
    ticks:  R::Storage,
    marker: PhantomData<R>,
}

impl<R: Resolution> TimeOfDay<R> {
    /// The distinct end-of-day boundary `24:00`.
    pub const END_OF_DAY: Self = Self {
        ticks: R::END_STORAGE, marker: PhantomData
    };
    /// The start-of-day value `00:00`.
    pub const START_OF_DAY: Self = Self {
        ticks: R::START_STORAGE, marker: PhantomData
    };

    /// Creates a value from resolution-native ticks since the start of day.
    #[inline]
    pub fn from_ticks(ticks: u64) -> Result<Self, ComponentRangeError> {
        if ticks > R::TICKS_PER_DAY {
            return Err(ComponentRangeError::Ticks {
                ticks,
                maximum: R::TICKS_PER_DAY,
            });
        }

        Ok(Self::from_valid_ticks(ticks))
    }

    /// Creates a value from nanoseconds since the start of day without losing resolution.
    pub fn from_nanoseconds_since_start_of_day(
        nanoseconds: u64,
    ) -> Result<Self, ComponentRangeError> {
        if nanoseconds > NANOSECONDS_PER_DAY {
            return Err(ComponentRangeError::NanosecondsSinceStartOfDay(nanoseconds));
        }

        let remainder = nanoseconds % R::NANOSECONDS_PER_TICK;

        if remainder != 0 {
            return Err(ResolutionLoss {
                nanoseconds,
                target_tick_nanoseconds: R::NANOSECONDS_PER_TICK,
            }
            .into());
        }

        Ok(Self::from_valid_ticks(nanoseconds / R::NANOSECONDS_PER_TICK))
    }

    /// Creates a value from hour and minute components.
    #[inline]
    pub fn from_hour_minute(hour: u8, minute: u8) -> Result<Self, ComponentRangeError> {
        Self::from_hms_nanos(hour, minute, 0, 0)
    }

    /// Returns resolution-native ticks since the start of day.
    #[inline]
    pub fn ticks(self) -> u64 {
        storage_into_u64::<R>(self.ticks)
    }

    /// Returns nanoseconds since the start of day.
    #[inline]
    pub fn nanoseconds_since_start_of_day(self) -> u64 {
        self.ticks() * R::NANOSECONDS_PER_TICK
    }

    /// Returns seconds since the start of day.
    #[inline]
    pub fn seconds_since_start_of_day(self) -> u32 {
        (self.nanoseconds_since_start_of_day() / NANOSECONDS_PER_SECOND) as u32
    }

    /// Returns the hour component in `0..=24`.
    #[inline]
    pub fn hour(self) -> u8 {
        (self.seconds_since_start_of_day() / 3_600) as u8
    }

    /// Returns the minute component in `0..=59`.
    #[inline]
    pub fn minute(self) -> u8 {
        ((self.seconds_since_start_of_day() / 60) % 60) as u8
    }

    /// Returns the second component in `0..=59`.
    #[inline]
    pub fn second(self) -> u8 {
        (self.seconds_since_start_of_day() % 60) as u8
    }

    /// Returns the complete millisecond fraction within the current second.
    #[inline]
    pub fn millisecond(self) -> u16 {
        (self.subsecond_nanoseconds() / 1_000_000) as u16
    }

    /// Returns the complete microsecond fraction within the current second.
    #[inline]
    pub fn microsecond(self) -> u32 {
        self.subsecond_nanoseconds() / 1_000
    }

    /// Returns the complete nanosecond fraction within the current second.
    #[inline]
    pub fn nanosecond(self) -> u32 {
        self.subsecond_nanoseconds()
    }

    /// Returns the fractional second as nanoseconds.
    #[inline]
    pub fn subsecond_nanoseconds(self) -> u32 {
        (self.nanoseconds_since_start_of_day() % NANOSECONDS_PER_SECOND) as u32
    }

    /// Returns `true` when the value is `00:00`.
    #[inline]
    pub fn is_start_of_day(self) -> bool {
        self.ticks() == 0
    }

    /// Returns `true` when the value is the distinct `24:00` boundary.
    #[inline]
    pub fn is_end_of_day(self) -> bool {
        self.ticks() == R::TICKS_PER_DAY
    }

    /// Parses a value with explicit lexical and quantization behavior.
    pub fn parse_with(input: &str, mode: ParseMode) -> Result<Self, ParseTimeOfDayError> {
        parse_for::<R>(input, mode)
    }

    /// Creates a display wrapper using the given formatting options.
    pub fn format_with(
        &self,
        options: FormatOptions,
    ) -> Result<FormattedTimeOfDay<'_, R>, crate::FormatOptionError> {
        format_options::<R>(options)?;

        Ok(FormattedTimeOfDay::new(self, options))
    }

    /// Creates a display wrapper that removes trailing zero components down to minutes.
    #[inline]
    pub fn format_compact(&self) -> FormattedTimeOfDay<'_, R> {
        FormattedTimeOfDay::new(self, FormatOptions {
            trailing_zeros: TrailingZeros::TrimAfter(crate::ResolutionKind::Minute),
        })
    }

    /// Converts exactly to resolution `T`.
    pub fn cast<T: Resolution>(self) -> Result<TimeOfDay<T>, ResolutionLoss> {
        let nanoseconds = self.nanoseconds_since_start_of_day();

        if !nanoseconds.is_multiple_of(T::NANOSECONDS_PER_TICK) {
            return Err(ResolutionLoss {
                nanoseconds,
                target_tick_nanoseconds: T::NANOSECONDS_PER_TICK,
            });
        }

        Ok(TimeOfDay::<T>::from_valid_ticks(nanoseconds / T::NANOSECONDS_PER_TICK))
    }

    /// Converts to resolution `T` using an explicit quantization mode.
    pub fn quantize<T: Resolution>(
        self,
        mode: QuantizeMode,
    ) -> Result<TimeOfDay<T>, ResolutionLoss> {
        let nanoseconds = self.nanoseconds_since_start_of_day();
        let tick = T::NANOSECONDS_PER_TICK;
        let quotient = nanoseconds / tick;
        let remainder = nanoseconds % tick;

        let ticks = match mode {
            QuantizeMode::Exact if remainder != 0 => {
                return Err(ResolutionLoss {
                    nanoseconds,
                    target_tick_nanoseconds: tick,
                });
            },
            QuantizeMode::Exact | QuantizeMode::Truncate => quotient,
            QuantizeMode::Ceil => quotient + u64::from(remainder != 0),
            QuantizeMode::Round => quotient + u64::from(remainder >= tick - remainder),
        };

        Ok(TimeOfDay::<T>::from_valid_ticks(ticks))
    }

    /// Adds resolution-native ticks and returns `None` when the result is after `24:00`.
    #[inline]
    pub fn checked_add_ticks(self, ticks: u64) -> Option<Self> {
        self.ticks()
            .checked_add(ticks)
            .filter(|value| *value <= R::TICKS_PER_DAY)
            .map(Self::from_valid_ticks)
    }

    /// Subtracts resolution-native ticks and returns `None` when the result is before `00:00`.
    #[inline]
    pub fn checked_sub_ticks(self, ticks: u64) -> Option<Self> {
        self.ticks().checked_sub(ticks).map(Self::from_valid_ticks)
    }

    /// Adds resolution-native ticks and clamps the result to `24:00`.
    #[inline]
    pub fn saturating_add_ticks(self, ticks: u64) -> Self {
        Self::from_valid_ticks(self.ticks().saturating_add(ticks).min(R::TICKS_PER_DAY))
    }

    /// Subtracts resolution-native ticks and clamps the result to `00:00`.
    #[inline]
    pub fn saturating_sub_ticks(self, ticks: u64) -> Self {
        Self::from_valid_ticks(self.ticks().saturating_sub(ticks))
    }

    /// Adds an exactly aligned duration without wrapping across the day boundary.
    pub fn try_add_duration(self, duration: Duration) -> Result<Self, ArithmeticError> {
        let ticks = duration_ticks::<R>(duration)?;
        self.checked_add_ticks(ticks).ok_or(ArithmeticError::OutOfRange)
    }

    /// Subtracts an exactly aligned duration without wrapping across the day boundary.
    pub fn try_sub_duration(self, duration: Duration) -> Result<Self, ArithmeticError> {
        let ticks = duration_ticks::<R>(duration)?;
        self.checked_sub_ticks(ticks).ok_or(ArithmeticError::OutOfRange)
    }

    /// Adds an exactly aligned duration and clamps range overflow to `24:00`.
    pub fn saturating_add_duration(self, duration: Duration) -> Result<Self, ArithmeticError> {
        let ticks = duration_ticks_saturating::<R>(duration)?;

        Ok(self.saturating_add_ticks(ticks))
    }

    /// Subtracts an exactly aligned duration and clamps range overflow to `00:00`.
    pub fn saturating_sub_duration(self, duration: Duration) -> Result<Self, ArithmeticError> {
        let ticks = duration_ticks_saturating::<R>(duration)?;

        Ok(self.saturating_sub_ticks(ticks))
    }

    /// Returns the duration since `earlier`, or `None` when `earlier` is later than this value.
    pub fn duration_since<T: Resolution>(self, earlier: TimeOfDay<T>) -> Option<Duration> {
        self.nanoseconds_since_start_of_day()
            .checked_sub(earlier.nanoseconds_since_start_of_day())
            .map(Duration::from_nanos)
    }

    /// Returns the absolute duration between values of any built-in resolution.
    pub fn abs_diff<T: Resolution>(self, other: TimeOfDay<T>) -> Duration {
        Duration::from_nanos(
            self.nanoseconds_since_start_of_day().abs_diff(other.nanoseconds_since_start_of_day()),
        )
    }

    #[inline]
    pub(crate) fn from_valid_ticks(ticks: u64) -> Self {
        debug_assert!(ticks <= R::TICKS_PER_DAY);

        Self {
            ticks: storage_from_u64::<R>(ticks), marker: PhantomData
        }
    }

    pub(crate) fn from_hms_nanos(
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Result<Self, ComponentRangeError> {
        validate_components(hour, minute, second, nanosecond)?;

        let nanoseconds = u64::from(hour) * 3_600 * NANOSECONDS_PER_SECOND
            + u64::from(minute) * 60 * NANOSECONDS_PER_SECOND
            + u64::from(second) * NANOSECONDS_PER_SECOND
            + u64::from(nanosecond);

        Self::from_nanoseconds_since_start_of_day(nanoseconds)
    }
}

fn validate_components(
    hour: u8,
    minute: u8,
    second: u8,
    nanosecond: u32,
) -> Result<(), ComponentRangeError> {
    if hour > 24 {
        return Err(ComponentRangeError::Hour(hour));
    }

    if minute > 59 {
        return Err(ComponentRangeError::Minute(minute));
    }

    if second > 59 {
        return Err(ComponentRangeError::Second(second));
    }

    if nanosecond > 999_999_999 {
        return Err(ComponentRangeError::Nanosecond(nanosecond));
    }

    if hour == 24 && (minute != 0 || second != 0 || nanosecond != 0) {
        return Err(ComponentRangeError::NonZeroEndOfDayComponent);
    }

    Ok(())
}

fn duration_ticks<R: Resolution>(duration: Duration) -> Result<u64, ArithmeticError> {
    let ticks = duration_ticks_u128::<R>(duration)?;

    u64::try_from(ticks).map_err(|_| ArithmeticError::OutOfRange)
}

fn duration_ticks_saturating<R: Resolution>(duration: Duration) -> Result<u64, ArithmeticError> {
    let ticks = duration_ticks_u128::<R>(duration)?;

    Ok(u64::try_from(ticks).unwrap_or(u64::MAX))
}

fn duration_ticks_u128<R: Resolution>(duration: Duration) -> Result<u128, ArithmeticError> {
    let nanoseconds = duration.as_nanos();
    let tick = u128::from(R::NANOSECONDS_PER_TICK);

    if !nanoseconds.is_multiple_of(tick) {
        return Err(ResolutionLoss {
            nanoseconds:             u64::try_from(nanoseconds).unwrap_or(u64::MAX),
            target_tick_nanoseconds: R::NANOSECONDS_PER_TICK,
        }
        .into());
    }

    Ok(nanoseconds / tick)
}

impl<R: Resolution> Copy for TimeOfDay<R> {}

impl<R: Resolution> Clone for TimeOfDay<R> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<R: Resolution> fmt::Debug for TimeOfDay<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TimeOfDay")
            .field("resolution", &R::KIND)
            .field("ticks", &self.ticks())
            .finish()
    }
}

impl<R: Resolution> fmt::Display for TimeOfDay<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        crate::format::format_time(*self, FormatOptions::KEEP, f)
    }
}

impl<R: Resolution> FromStr for TimeOfDay<R> {
    type Err = ParseTimeOfDayError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse_with(input, ParseMode::Relaxed)
    }
}

impl<R: Resolution, T: Resolution> PartialEq<TimeOfDay<T>> for TimeOfDay<R> {
    fn eq(&self, other: &TimeOfDay<T>) -> bool {
        self.nanoseconds_since_start_of_day() == other.nanoseconds_since_start_of_day()
    }
}

impl<R: Resolution> Eq for TimeOfDay<R> {}

impl<R: Resolution, T: Resolution> PartialOrd<TimeOfDay<T>> for TimeOfDay<R> {
    fn partial_cmp(&self, other: &TimeOfDay<T>) -> Option<Ordering> {
        Some(self.nanoseconds_since_start_of_day().cmp(&other.nanoseconds_since_start_of_day()))
    }
}

impl<R: Resolution> Ord for TimeOfDay<R> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ticks().cmp(&other.ticks())
    }
}

impl<R: Resolution> Hash for TimeOfDay<R> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.nanoseconds_since_start_of_day().hash(state);
    }
}

macro_rules! impl_hms {
    ($($resolution:ty),+ $(,)?) => {
        $(
            impl TimeOfDay<$resolution> {
                /// Creates a value from hour, minute, and second components.
                #[inline]
                pub fn from_hms(
                    hour: u8,
                    minute: u8,
                    second: u8,
                ) -> Result<Self, ComponentRangeError> {
                    Self::from_hms_nanos(hour, minute, second, 0)
                }
            }
        )+
    };
}

macro_rules! impl_milli {
    ($($resolution:ty),+ $(,)?) => {
        $(
            impl TimeOfDay<$resolution> {
                /// Creates a value from hour, minute, second, and millisecond components.
                #[inline]
                pub fn from_hms_milli(
                    hour: u8,
                    minute: u8,
                    second: u8,
                    millisecond: u16,
                ) -> Result<Self, ComponentRangeError> {
                    if millisecond > 999 {
                        return Err(ComponentRangeError::Millisecond(millisecond));
                    }
                    Self::from_hms_nanos(
                        hour,
                        minute,
                        second,
                        u32::from(millisecond) * 1_000_000,
                    )
                }
            }
        )+
    };
}

macro_rules! impl_micro {
    ($($resolution:ty),+ $(,)?) => {
        $(
            impl TimeOfDay<$resolution> {
                /// Creates a value from hour, minute, second, and microsecond components.
                #[inline]
                pub fn from_hms_micro(
                    hour: u8,
                    minute: u8,
                    second: u8,
                    microsecond: u32,
                ) -> Result<Self, ComponentRangeError> {
                    if microsecond > 999_999 {
                        return Err(ComponentRangeError::Microsecond(microsecond));
                    }
                    Self::from_hms_nanos(hour, minute, second, microsecond * 1_000)
                }
            }
        )+
    };
}

macro_rules! impl_hundred_nano {
    ($($resolution:ty),+ $(,)?) => {
        $(
            impl TimeOfDay<$resolution> {
                /// Creates a value from hour, minute, second, and one-hundred-nanosecond components.
                #[inline]
                pub fn from_hms_hundred_nano(
                    hour: u8,
                    minute: u8,
                    second: u8,
                    hundred_nanosecond: u32,
                ) -> Result<Self, ComponentRangeError> {
                    if hundred_nanosecond > 9_999_999 {
                        return Err(ComponentRangeError::HundredNanosecond(hundred_nanosecond));
                    }
                    Self::from_hms_nanos(hour, minute, second, hundred_nanosecond * 100)
                }
            }
        )+
    };
}

impl_hms!(
    crate::Second,
    crate::Millisecond,
    crate::Microsecond,
    crate::HundredNanosecond,
    crate::Nanosecond,
);
impl_milli!(crate::Millisecond, crate::Microsecond, crate::HundredNanosecond, crate::Nanosecond,);
impl_micro!(crate::Microsecond, crate::HundredNanosecond, crate::Nanosecond,);
impl_hundred_nano!(crate::HundredNanosecond, crate::Nanosecond);

impl TimeOfDay<crate::Nanosecond> {
    /// Creates a value from hour, minute, second, and nanosecond components.
    #[inline]
    pub fn from_hms_nano(
        hour: u8,
        minute: u8,
        second: u8,
        nanosecond: u32,
    ) -> Result<Self, ComponentRangeError> {
        if nanosecond > 999_999_999 {
            return Err(ComponentRangeError::Nanosecond(nanosecond));
        }

        Self::from_hms_nanos(hour, minute, second, nanosecond)
    }
}

macro_rules! impl_widening_from {
    ($source:ty => $($target:ty),+ $(,)?) => {
        $(
            impl From<TimeOfDay<$source>> for TimeOfDay<$target> {
                fn from(value: TimeOfDay<$source>) -> Self {
                    value.cast().expect("a finer resolution represents every source value")
                }
            }
        )+
    };
}

impl_widening_from!(crate::Minute => crate::Second, crate::Millisecond, crate::Microsecond, crate::HundredNanosecond, crate::Nanosecond);
impl_widening_from!(crate::Second => crate::Millisecond, crate::Microsecond, crate::HundredNanosecond, crate::Nanosecond);
impl_widening_from!(crate::Millisecond => crate::Microsecond, crate::HundredNanosecond, crate::Nanosecond);
impl_widening_from!(crate::Microsecond => crate::HundredNanosecond, crate::Nanosecond);
impl_widening_from!(crate::HundredNanosecond => crate::Nanosecond);

impl<R: Resolution> From<TimeOfDay<R>> for u64 {
    fn from(value: TimeOfDay<R>) -> Self {
        value.ticks()
    }
}
