use time::{Date, PrimitiveDateTime, Time};

use crate::{
    ComponentRangeError, ConversionError, DayOffset, DynamicTimeOfDay, NormalizedTime, Resolution,
    TimeOfDay,
};

fn from_component_error(error: ComponentRangeError) -> ConversionError {
    match error {
        ComponentRangeError::ResolutionLoss(error) => ConversionError::ResolutionLoss(error),
        _ => unreachable!("time already validates ordinary time components"),
    }
}

impl<R: Resolution> TryFrom<Time> for TimeOfDay<R> {
    type Error = ConversionError;

    fn try_from(value: Time) -> Result<Self, Self::Error> {
        Self::from_hms_nanos(value.hour(), value.minute(), value.second(), value.nanosecond())
            .map_err(from_component_error)
    }
}

impl<R: Resolution> TryFrom<TimeOfDay<R>> for Time {
    type Error = ConversionError;

    fn try_from(value: TimeOfDay<R>) -> Result<Self, Self::Error> {
        if value.is_end_of_day() {
            return Err(ConversionError::EndOfDayNotRepresentable);
        }

        Ok(Time::from_hms_nano(
            value.hour(),
            value.minute(),
            value.second(),
            value.subsecond_nanoseconds(),
        )
        .expect("TimeOfDay components are valid for time"))
    }
}

impl TryFrom<Time> for DynamicTimeOfDay {
    type Error = ConversionError;

    fn try_from(value: Time) -> Result<Self, Self::Error> {
        let value = TimeOfDay::<crate::Nanosecond>::try_from(value)?;
        Ok(Self::from_nanoseconds_minimized(value.nanoseconds_since_start_of_day()))
    }
}

impl<R: Resolution> TimeOfDay<R> {
    /// Converts to `time::Time` while preserving `24:00` as a next-day offset.
    #[inline]
    pub fn normalize_time(self) -> NormalizedTime<Time> {
        if self.is_end_of_day() {
            NormalizedTime {
                day_offset: DayOffset::NextDay, time: Time::MIDNIGHT
            }
        } else {
            NormalizedTime {
                day_offset: DayOffset::SameDay,
                time:       self.try_into().expect("non-end-of-day TimeOfDay is valid for time"),
            }
        }
    }

    /// Attaches this value to a `time::Date` and moves `24:00` to the next date.
    pub fn attach_to_time_date(self, date: Date) -> Result<PrimitiveDateTime, ConversionError> {
        let normalized = self.normalize_time();

        let date = match normalized.day_offset {
            DayOffset::SameDay => date,
            DayOffset::NextDay => date.next_day().ok_or(ConversionError::DateOverflow)?,
        };

        Ok(PrimitiveDateTime::new(date, normalized.time))
    }
}
