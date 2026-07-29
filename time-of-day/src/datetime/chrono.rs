use chrono::{NaiveDate, NaiveDateTime, NaiveTime, Timelike};

use crate::{
    ComponentRangeError, ConversionError, DayOffset, DynamicTimeOfDay, NormalizedTime, Resolution,
    TimeOfDay,
};

fn from_component_error(error: ComponentRangeError) -> ConversionError {
    match error {
        ComponentRangeError::ResolutionLoss(error) => ConversionError::ResolutionLoss(error),
        _ => unreachable!("Chrono already validates ordinary time components"),
    }
}

impl<R: Resolution> TryFrom<NaiveTime> for TimeOfDay<R> {
    type Error = ConversionError;

    fn try_from(value: NaiveTime) -> Result<Self, Self::Error> {
        // Chrono represents leap seconds with a nanosecond value at or above one billion.
        if value.nanosecond() >= 1_000_000_000 {
            return Err(ConversionError::UnsupportedLeapSecond);
        }

        Self::from_hms_nanos(
            value.hour() as u8,
            value.minute() as u8,
            value.second() as u8,
            value.nanosecond(),
        )
        .map_err(from_component_error)
    }
}

impl<R: Resolution> TryFrom<TimeOfDay<R>> for NaiveTime {
    type Error = ConversionError;

    fn try_from(value: TimeOfDay<R>) -> Result<Self, Self::Error> {
        if value.is_end_of_day() {
            return Err(ConversionError::EndOfDayNotRepresentable);
        }

        Ok(NaiveTime::from_hms_nano_opt(
            u32::from(value.hour()),
            u32::from(value.minute()),
            u32::from(value.second()),
            value.subsecond_nanoseconds(),
        )
        .expect("TimeOfDay components are valid for Chrono"))
    }
}

impl TryFrom<NaiveTime> for DynamicTimeOfDay {
    type Error = ConversionError;

    fn try_from(value: NaiveTime) -> Result<Self, Self::Error> {
        let value = TimeOfDay::<crate::Nanosecond>::try_from(value)?;
        Ok(Self::from_nanoseconds_minimized(value.nanoseconds_since_start_of_day()))
    }
}

impl<R: Resolution> TimeOfDay<R> {
    /// Converts to Chrono time while preserving `24:00` as a next-day offset.
    #[inline]
    pub fn normalize_chrono(self) -> NormalizedTime<NaiveTime> {
        if self.is_end_of_day() {
            NormalizedTime {
                day_offset: DayOffset::NextDay,
                time:       NaiveTime::from_hms_opt(0, 0, 0).expect("midnight is valid"),
            }
        } else {
            NormalizedTime {
                day_offset: DayOffset::SameDay,
                time:       self.try_into().expect("non-end-of-day TimeOfDay is valid for Chrono"),
            }
        }
    }

    /// Attaches this value to a Chrono date and moves `24:00` to the next date.
    pub fn attach_to_chrono_date(self, date: NaiveDate) -> Result<NaiveDateTime, ConversionError> {
        let normalized = self.normalize_chrono();

        let date = match normalized.day_offset {
            DayOffset::SameDay => date,
            DayOffset::NextDay => date.succ_opt().ok_or(ConversionError::DateOverflow)?,
        };

        Ok(date.and_time(normalized.time))
    }
}
