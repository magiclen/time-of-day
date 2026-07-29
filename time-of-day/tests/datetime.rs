#[cfg(feature = "chrono")]
mod chrono_tests {
    use chrono::{NaiveDate, NaiveTime};
    use time_of_day::{DayOffset, Microsecond, Minute, TimeOfDay};

    #[test]
    fn normalizes_end_of_day() {
        let normalized = TimeOfDay::<Minute>::END_OF_DAY.normalize_chrono();

        assert_eq!(DayOffset::NextDay, normalized.day_offset);
        assert_eq!(NaiveTime::from_hms_opt(0, 0, 0).unwrap(), normalized.time);

        assert_eq!(
            NaiveDate::from_ymd_opt(2026, 7, 30).unwrap().and_hms_opt(0, 0, 0).unwrap(),
            TimeOfDay::<Minute>::END_OF_DAY
                .attach_to_chrono_date(NaiveDate::from_ymd_opt(2026, 7, 29).unwrap())
                .unwrap()
        );
    }

    #[test]
    fn converts_exact_values() {
        let source = NaiveTime::from_hms_micro_opt(12, 30, 15, 123_456).unwrap();
        let value = TimeOfDay::<Microsecond>::try_from(source).unwrap();

        assert_eq!(source, NaiveTime::try_from(value).unwrap());
    }
}

#[cfg(feature = "time")]
mod time_tests {
    use time::{Date, Month, PrimitiveDateTime, Time};
    use time_of_day::{Nanosecond, Second, TimeOfDay};

    #[test]
    fn normalizes_end_of_day() {
        assert_eq!(
            PrimitiveDateTime::new(
                Date::from_calendar_date(2026, Month::July, 30).unwrap(),
                Time::MIDNIGHT,
            ),
            TimeOfDay::<Second>::END_OF_DAY
                .attach_to_time_date(Date::from_calendar_date(2026, Month::July, 29).unwrap())
                .unwrap()
        );
    }

    #[test]
    fn converts_exact_values() {
        let source = Time::from_hms_nano(12, 30, 15, 123_456_789).unwrap();
        let value = TimeOfDay::<Nanosecond>::try_from(source).unwrap();

        assert_eq!(source, Time::try_from(value).unwrap());
    }
}

#[cfg(feature = "jiff")]
mod jiff_tests {
    use jiff::civil::{Date, DateTime, Time};
    use time_of_day::{Minute, Nanosecond, TimeOfDay};

    #[test]
    fn normalizes_end_of_day() {
        assert_eq!(
            DateTime::new(2026, 7, 30, 0, 0, 0, 0).unwrap(),
            TimeOfDay::<Minute>::END_OF_DAY
                .attach_to_jiff_date(Date::new(2026, 7, 29).unwrap())
                .unwrap()
        );
    }

    #[test]
    fn converts_exact_values() {
        let source = Time::new(12, 30, 15, 123_456_789).unwrap();
        let value = TimeOfDay::<Nanosecond>::try_from(source).unwrap();

        assert_eq!(source, Time::try_from(value).unwrap());
    }
}
