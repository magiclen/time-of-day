use jiff::civil::{Date, DateTime, Time};

use crate::{
    ComponentRangeError, ConversionError, DayOffset, DynamicTimeOfDay, NormalizedTime, Resolution,
    TimeOfDay,
};

fn from_component_error(error: ComponentRangeError) -> ConversionError {
    match error {
        ComponentRangeError::ResolutionLoss(error) => ConversionError::ResolutionLoss(error),
        _ => unreachable!("Jiff already validates ordinary time components"),
    }
}

impl<R: Resolution> TryFrom<Time> for TimeOfDay<R> {
    type Error = ConversionError;

    fn try_from(value: Time) -> Result<Self, Self::Error> {
        Self::from_hms_nanos(
            value.hour() as u8,
            value.minute() as u8,
            value.second() as u8,
            value.subsec_nanosecond() as u32,
        )
        .map_err(from_component_error)
    }
}

impl<R: Resolution> TryFrom<TimeOfDay<R>> for Time {
    type Error = ConversionError;

    fn try_from(value: TimeOfDay<R>) -> Result<Self, Self::Error> {
        if value.is_end_of_day() {
            return Err(ConversionError::EndOfDayNotRepresentable);
        }

        Ok(Time::new(
            value.hour() as i8,
            value.minute() as i8,
            value.second() as i8,
            value.subsecond_nanoseconds() as i32,
        )
        .expect("TimeOfDay components are valid for Jiff"))
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
    /// Converts to Jiff civil time while preserving `24:00` as a next-day offset.
    #[inline]
    pub fn normalize_jiff(self) -> NormalizedTime<Time> {
        if self.is_end_of_day() {
            NormalizedTime {
                day_offset: DayOffset::NextDay, time: Time::MIN
            }
        } else {
            NormalizedTime {
                day_offset: DayOffset::SameDay,
                time:       self.try_into().expect("non-end-of-day TimeOfDay is valid for Jiff"),
            }
        }
    }

    /// Attaches this value to a Jiff civil date and moves `24:00` to the next date.
    pub fn attach_to_jiff_date(self, date: Date) -> Result<DateTime, ConversionError> {
        let normalized = self.normalize_jiff();

        let date = match normalized.day_offset {
            DayOffset::SameDay => date,
            DayOffset::NextDay => date.tomorrow().map_err(|_| ConversionError::DateOverflow)?,
        };

        Ok(DateTime::from_parts(date, normalized.time))
    }
}
