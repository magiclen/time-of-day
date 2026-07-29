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
fn sqlite_round_trips_storage_strategies() {
    use sqlx_core::{connection::Connection, query::query, query_scalar::query_scalar};
    use sqlx_sqlite::{Sqlite, SqliteConnection};
    use time_of_day::{Minute, Nanosecond, TimeOfDay};
    use time_of_day_sqlx::sqlite::{SqliteIntegerTimeOfDay, SqliteTextTimeOfDay};

    let expected_integer =
        SqliteIntegerTimeOfDay::from(TimeOfDay::<Minute>::from_hour_minute(12, 0).unwrap());
    let expected_text =
        SqliteTextTimeOfDay::from(TimeOfDay::<Nanosecond>::from_hms_nano(12, 0, 0, 123).unwrap());

    futures_executor::block_on(async {
        let mut connection = SqliteConnection::connect(":memory:").await.unwrap();

        query::<Sqlite>(
            "CREATE TABLE time_values (native_ticks INTEGER NOT NULL, canonical_text TEXT NOT NULL)",
        )
        .execute(&mut connection)
        .await
        .unwrap();

        query::<Sqlite>("INSERT INTO time_values VALUES (?, ?)")
            .bind(expected_integer)
            .bind(expected_text)
            .execute(&mut connection)
            .await
            .unwrap();

        let actual_integer = query_scalar::<Sqlite, SqliteIntegerTimeOfDay<Minute>>(
            "SELECT native_ticks FROM time_values",
        )
        .fetch_one(&mut connection)
        .await
        .unwrap();
        let actual_text = query_scalar::<Sqlite, SqliteTextTimeOfDay<Nanosecond>>(
            "SELECT canonical_text FROM time_values",
        )
        .fetch_one(&mut connection)
        .await
        .unwrap();

        assert_eq!(expected_integer, actual_integer);
        assert_eq!(expected_text, actual_text);
    });
}
