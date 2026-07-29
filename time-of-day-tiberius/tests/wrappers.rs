use tiberius::{ColumnData, FromSqlOwned, IntoSql, time::Time};
use time_of_day::{HundredNanosecond, Microsecond, Nanosecond, Second, TimeOfDay};
use time_of_day_tiberius::{BigIntNanoseconds, StorageError, Time7EndOfDaySentinel};

const TDS_SENTINEL_INCREMENT: u64 = 863_999_999_999;

#[test]
fn bigint_round_trips_end_of_day_column_data() {
    let source = BigIntNanoseconds::from(TimeOfDay::<Nanosecond>::END_OF_DAY);
    let decoded = BigIntNanoseconds::<Nanosecond>::from_sql_owned(source.into_sql()).unwrap();

    assert_eq!(Some(source), decoded);
}

#[test]
fn sentinel_maps_end_of_day_both_ways() {
    let source = Time7EndOfDaySentinel::from(TimeOfDay::<Microsecond>::END_OF_DAY);
    let encoded = source.into_sql();

    assert!(matches!(
        encoded,
        ColumnData::Time(Some(value))
            if value.scale() == 7 && value.increments() == TDS_SENTINEL_INCREMENT
    ));

    assert_eq!(
        Some(source),
        Time7EndOfDaySentinel::<Microsecond>::from_sql_owned(encoded).unwrap()
    );
}

#[test]
fn sentinel_rejects_literal_collision() {
    let value = TimeOfDay::<HundredNanosecond>::from_ticks(TDS_SENTINEL_INCREMENT).unwrap();

    assert_eq!(
        Err(StorageError::SentinelCollision),
        Time7EndOfDaySentinel::try_from(value)
    );
}

#[test]
fn sentinel_requires_scale_seven() {
    assert!(
        Time7EndOfDaySentinel::<Second>::from_sql_owned(ColumnData::Time(Some(Time::new(
            86_399_999_999,
            6
        ),)))
        .is_err()
    );
}
