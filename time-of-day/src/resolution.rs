use core::{fmt::Debug, hash::Hash};

mod private {
    pub trait Sealed {}

    impl Sealed for super::Minute {}
    impl Sealed for super::Second {}
    impl Sealed for super::Millisecond {}
    impl Sealed for super::Microsecond {}
    impl Sealed for super::HundredNanosecond {}
    impl Sealed for super::Nanosecond {}

    pub trait Storage:
        Copy + Clone + core::fmt::Debug + Eq + Ord + core::hash::Hash + Send + Sync + 'static {
        fn from_u64(value: u64) -> Self;
        fn into_u64(self) -> u64;
    }

    impl Storage for u16 {
        fn from_u64(value: u64) -> Self {
            value as Self
        }

        fn into_u64(self) -> u64 {
            self.into()
        }
    }

    impl Storage for u32 {
        fn from_u64(value: u64) -> Self {
            value as Self
        }

        fn into_u64(self) -> u64 {
            self.into()
        }
    }

    impl Storage for u64 {
        fn from_u64(value: u64) -> Self {
            value
        }

        fn into_u64(self) -> u64 {
            self
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// Identifies one of the built-in resolutions at runtime.
pub enum ResolutionKind {
    /// One-minute resolution.
    Minute,
    /// One-second resolution.
    Second,
    /// One-millisecond resolution.
    Millisecond,
    /// One-microsecond resolution.
    Microsecond,
    /// One-hundred-nanosecond resolution.
    HundredNanosecond,
    /// One-nanosecond resolution.
    Nanosecond,
}

impl ResolutionKind {
    /// Returns the number of nanoseconds represented by one tick.
    #[inline]
    pub const fn nanoseconds_per_tick(self) -> u64 {
        match self {
            Self::Minute => 60_000_000_000,
            Self::Second => 1_000_000_000,
            Self::Millisecond => 1_000_000,
            Self::Microsecond => 1_000,
            Self::HundredNanosecond => 100,
            Self::Nanosecond => 1,
        }
    }

    /// Returns the canonical number of fractional-second digits.
    #[inline]
    pub const fn fractional_digits(self) -> u8 {
        match self {
            Self::Minute | Self::Second => 0,
            Self::Millisecond => 3,
            Self::Microsecond => 6,
            Self::HundredNanosecond => 7,
            Self::Nanosecond => 9,
        }
    }
}

/// Defines the storage and tick properties of a built-in time resolution.
///
/// This trait is sealed because parsing, dynamic dispatch, and database adapters require an exhaustive resolution set.
pub trait Resolution:
    private::Sealed + Copy + Clone + Debug + Eq + Ord + Hash + Send + Sync + 'static {
    /// The compact integer type used to store ticks.
    type Storage: private::Storage;

    /// The number of fractional-second digits in canonical text.
    const CANONICAL_FRACTIONAL_DIGITS: u8;
    /// The storage value used by `TimeOfDay::END_OF_DAY`.
    const END_STORAGE: Self::Storage;
    /// The runtime identifier for this resolution.
    const KIND: ResolutionKind;
    /// The duration of one tick in nanoseconds.
    const NANOSECONDS_PER_TICK: u64;
    /// The storage value used by `TimeOfDay::START_OF_DAY`.
    const START_STORAGE: Self::Storage;
    /// The inclusive tick value representing the end of day.
    const TICKS_PER_DAY: u64;
    /// The number of ticks per second, or `None` for minute resolution.
    const TICKS_PER_SECOND: Option<u64>;
}

macro_rules! define_resolution {
    ($name:ident, $storage:ty, $kind:ident, $ns:expr, $day:expr, $second:expr, $digits:expr) => {
        #[doc = concat!("The ", stringify!($name), " resolution marker.")]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name;

        impl Resolution for $name {
            type Storage = $storage;

            const CANONICAL_FRACTIONAL_DIGITS: u8 = $digits;
            const END_STORAGE: Self::Storage = $day;
            const KIND: ResolutionKind = ResolutionKind::$kind;
            const NANOSECONDS_PER_TICK: u64 = $ns;
            const START_STORAGE: Self::Storage = 0;
            const TICKS_PER_DAY: u64 = $day;
            const TICKS_PER_SECOND: Option<u64> = $second;
        }
    };
}

define_resolution!(Minute, u16, Minute, 60_000_000_000, 1_440, None, 0);
define_resolution!(Second, u32, Second, 1_000_000_000, 86_400, Some(1), 0);
define_resolution!(Millisecond, u32, Millisecond, 1_000_000, 86_400_000, Some(1_000), 3);
define_resolution!(Microsecond, u64, Microsecond, 1_000, 86_400_000_000, Some(1_000_000), 6);
define_resolution!(
    HundredNanosecond,
    u64,
    HundredNanosecond,
    100,
    864_000_000_000,
    Some(10_000_000),
    7
);
define_resolution!(Nanosecond, u64, Nanosecond, 1, 86_400_000_000_000, Some(1_000_000_000), 9);

#[inline]
pub(crate) fn storage_from_u64<R: Resolution>(value: u64) -> R::Storage {
    <R::Storage as private::Storage>::from_u64(value)
}

#[inline]
pub(crate) fn storage_into_u64<R: Resolution>(value: R::Storage) -> u64 {
    <R::Storage as private::Storage>::into_u64(value)
}
