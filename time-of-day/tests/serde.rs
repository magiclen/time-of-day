#![cfg(feature = "serde")]

use time_of_day::{DynamicTimeOfDay, Microsecond, TimeOfDay};

#[test]
fn typed_values_use_canonical_strings() {
    let value = TimeOfDay::<Microsecond>::from_hms_micro(12, 30, 15, 123_000).unwrap();

    assert_eq!("\"12:30:15.123000\"", serde_json::to_string(&value).unwrap());
    assert_eq!(value, serde_json::from_str::<TimeOfDay<Microsecond>>("\"12:30:15.123\"").unwrap());
}

#[test]
fn dynamic_values_preserve_variants() {
    let value = "12:00:00.000000".parse::<DynamicTimeOfDay>().unwrap();
    let encoded = serde_json::to_string(&value).unwrap();

    assert_eq!(
        value.resolution(),
        serde_json::from_str::<DynamicTimeOfDay>(&encoded).unwrap().resolution()
    );
}
