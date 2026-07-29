use core::{mem::size_of, time::Duration};

use time_of_day::{
    ComponentRangeError, DynamicTimeOfDay, FormatOptions, HundredNanosecond, Microsecond,
    Millisecond, Minute, Nanosecond, ParseMode, ParseTimeOfDayError, ResolutionKind, Second,
    TimeOfDay,
};

#[test]
fn storage_sizes_follow_resolution() {
    assert_eq!(2, size_of::<TimeOfDay<Minute>>());
    assert_eq!(4, size_of::<TimeOfDay<Second>>());
    assert_eq!(4, size_of::<TimeOfDay<Millisecond>>());
    assert_eq!(8, size_of::<TimeOfDay<Microsecond>>());
    assert_eq!(8, size_of::<TimeOfDay<HundredNanosecond>>());
    assert_eq!(8, size_of::<TimeOfDay<Nanosecond>>());
}

#[test]
fn start_and_end_are_distinct() {
    assert_ne!(TimeOfDay::<Minute>::START_OF_DAY, TimeOfDay::<Minute>::END_OF_DAY);
    assert!(TimeOfDay::<Minute>::START_OF_DAY < TimeOfDay::<Minute>::END_OF_DAY);
    assert_eq!("00:00", TimeOfDay::<Minute>::START_OF_DAY.to_string());
    assert_eq!("24:00", TimeOfDay::<Minute>::END_OF_DAY.to_string());
}

#[test]
fn constructors_enforce_end_of_day() {
    assert_eq!(
        TimeOfDay::<Nanosecond>::END_OF_DAY,
        TimeOfDay::<Nanosecond>::from_hms_nano(24, 0, 0, 0).unwrap()
    );

    assert_eq!(
        Err(ComponentRangeError::NonZeroEndOfDayComponent),
        TimeOfDay::<Nanosecond>::from_hms_nano(24, 0, 0, 1)
    );
}

#[test]
fn parse_modes_quantize_as_specified() {
    assert_eq!(
        "12:34:56.987",
        TimeOfDay::<Millisecond>::parse_with("12:34:56.987654", ParseMode::Truncate)
            .unwrap()
            .to_string()
    );

    assert_eq!(
        "12:34:56.001",
        TimeOfDay::<Millisecond>::parse_with("12:34:56.000001", ParseMode::Ceil)
            .unwrap()
            .to_string()
    );

    assert_eq!(
        "12:00:01",
        TimeOfDay::<Second>::parse_with("12:00:00.500", ParseMode::Round).unwrap().to_string()
    );

    assert_eq!(
        TimeOfDay::<Second>::END_OF_DAY,
        TimeOfDay::<Second>::parse_with("23:59:59.999999999", ParseMode::Ceil).unwrap()
    );
}

#[test]
fn relaxed_parse_requires_exact_value() {
    assert_eq!(
        "12:30:15.123",
        "12:30:15.123000".parse::<TimeOfDay<Millisecond>>().unwrap().to_string()
    );

    assert!(matches!(
        "12:30:15.123001".parse::<TimeOfDay<Millisecond>>(),
        Err(ParseTimeOfDayError::ResolutionLoss(_))
    ));
}

#[test]
fn formatter_trims_to_minimum_resolution() {
    let zero = TimeOfDay::<Nanosecond>::from_hms_nano(12, 34, 0, 0).unwrap();
    let fraction = TimeOfDay::<Nanosecond>::from_hms_nano(12, 34, 56, 120_000_000).unwrap();

    assert_eq!("12:34", zero.format_compact().to_string());

    assert_eq!(
        "12:34:56.120",
        fraction
            .format_with(FormatOptions::trim_after(ResolutionKind::Millisecond))
            .unwrap()
            .to_string()
    );

    assert_eq!(
        "12:34:56.12",
        fraction
            .format_with(FormatOptions::trim_after(ResolutionKind::Second))
            .unwrap()
            .to_string()
    );
}

#[test]
fn cross_resolution_comparison_uses_value() {
    let minute = TimeOfDay::<Minute>::from_hour_minute(12, 30).unwrap();
    let second = TimeOfDay::<Second>::from_hms(12, 30, 0).unwrap();

    assert_eq!(minute, second);
}

#[test]
fn dynamic_parse_preserves_declared_resolution() {
    assert!(matches!(
        "12:00:00.000000".parse::<DynamicTimeOfDay>().unwrap(),
        DynamicTimeOfDay::Microsecond(_)
    ));

    assert_eq!(
        DynamicTimeOfDay::Minute(TimeOfDay::from_hour_minute(12, 0).unwrap()),
        "12:00:00.000000".parse::<DynamicTimeOfDay>().unwrap().minimize_resolution()
    );
}

#[test]
fn arithmetic_does_not_wrap() {
    assert_eq!(None, TimeOfDay::<Minute>::END_OF_DAY.checked_add_ticks(1));

    assert_eq!(
        TimeOfDay::<Minute>::END_OF_DAY,
        TimeOfDay::<Minute>::from_hour_minute(23, 59)
            .unwrap()
            .try_add_duration(Duration::from_secs(60))
            .unwrap()
    );
}
