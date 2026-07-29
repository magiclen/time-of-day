use core::{fmt, marker::PhantomData};

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Visitor};

use crate::{DynamicTimeOfDay, Resolution, TimeOfDay};

impl<R: Resolution> Serialize for TimeOfDay<R> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de, R: Resolution> Deserialize<'de> for TimeOfDay<R> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(TimeOfDayVisitor(PhantomData))
    }
}

struct TimeOfDayVisitor<R>(PhantomData<R>);

impl<'de, R: Resolution> Visitor<'de> for TimeOfDayVisitor<R> {
    type Value = TimeOfDay<R>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a time-of-day string")
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        value.parse().map_err(E::custom)
    }
}

impl Serialize for DynamicTimeOfDay {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for DynamicTimeOfDay {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(DynamicTimeOfDayVisitor)
    }
}

struct DynamicTimeOfDayVisitor;

impl<'de> Visitor<'de> for DynamicTimeOfDayVisitor {
    type Value = DynamicTimeOfDay;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a time-of-day string")
    }

    fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
        value.parse().map_err(E::custom)
    }
}
