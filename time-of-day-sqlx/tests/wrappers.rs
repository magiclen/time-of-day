#[cfg(feature = "postgres")]
#[test]
fn postgres_checks_microsecond_resolution() {
    use time_of_day::{Nanosecond, TimeOfDay};
    use time_of_day_sqlx::{StorageError, postgres::PgTimeOfDay};

    let exact = TimeOfDay::<Nanosecond>::from_hms_nano(12, 0, 0, 1_000).unwrap();
    let lossy = TimeOfDay::<Nanosecond>::from_hms_nano(12, 0, 0, 1).unwrap();

    assert!(PgTimeOfDay::try_from(exact).is_ok());
    assert!(matches!(
        PgTimeOfDay::try_from(lossy),
        Err(StorageError::BackendPrecisionLoss { .. })
    ));
}

#[cfg(feature = "mysql")]
#[test]
fn mysql_preserves_end_of_day() {
    use time_of_day::{Microsecond, TimeOfDay};
    use time_of_day_sqlx::mysql::MySqlTimeOfDay;

    let wrapper = MySqlTimeOfDay::from(TimeOfDay::<Microsecond>::END_OF_DAY);

    assert_eq!(TimeOfDay::<Microsecond>::END_OF_DAY, wrapper.into_inner());
}

#[cfg(feature = "sqlite")]
#[test]
fn sqlite_keeps_storage_strategies_separate() {
    use time_of_day::{Minute, Nanosecond, TimeOfDay};
    use time_of_day_sqlx::sqlite::{SqliteIntegerTimeOfDay, SqliteTextTimeOfDay};

    let integer =
        SqliteIntegerTimeOfDay::from(TimeOfDay::<Minute>::from_hour_minute(12, 0).unwrap());
    let text =
        SqliteTextTimeOfDay::from(TimeOfDay::<Nanosecond>::from_hms_nano(12, 0, 0, 123).unwrap());

    assert_eq!(720, integer.as_inner().ticks());
    assert_eq!("12:00:00.000000123", text.as_inner().to_string());
}
