use core::ops::{Add, AddAssign, Mul, Sub, SubAssign};

/// Signed Microseconds
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "bevy", derive(bevy::reflect::Reflect))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Micros(i64);

impl Micros {
    /// Zero.
    pub const ZERO: Self = Self(0);
    /// Creates a new instance of microseconds
    pub const fn new(microseconds: i64) -> Self {
        Self(microseconds)
    }
    /// Returns the microseconds as an i64
    pub const fn us(&self) -> i64 {
        self.0
    }
    /// ms -> us
    pub const fn from_ms(ms: i64) -> Self {
        Self(ms * 1_000)
    }
    /// s -> us
    pub const fn from_seconds(secs: f32) -> Self {
        Self((secs * 1_000_000.) as i64)
    }
    /// Returns seconds
    pub const fn as_secs_f32(&self) -> f32 {
        self.0 as f32 / 1_000_000.
    }
    /// Returns seconds
    pub const fn as_secs_f64(&self) -> f64 {
        self.0 as f64 / 1_000_000.
    }
    /// Returns unsigned microseconds
    /// IF I am greater than or equal to zero.
    pub const fn to_unsigned(&self) -> Option<UMicros> {
        if self.0 < 0 {
            return None;
        }
        Some(UMicros(self.0 as u64))
    }

    /// Returns unsigned microseconds as an absolute value
    pub fn abs_unsigned(&self) -> UMicros {
        if self.0 < 0 {
            UMicros(-self.0 as u64)
        } else {
            UMicros(self.0 as u64)
        }
    }
}

/// Unsigned Microseconds
#[derive(PartialEq, Eq, PartialOrd, Ord, Debug, Clone, Copy, Default)]
#[cfg_attr(feature = "bevy", derive(bevy::reflect::Reflect))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UMicros(u64);

impl UMicros {
    /// Zero.
    pub const ZERO: Self = Self(0);

    /// Creates a new instance of microseconds
    #[inline]
    pub const fn new(microseconds: u64) -> Self {
        Self(microseconds)
    }

    /// Returns the microseconds as an i64
    #[inline]
    pub const fn us(&self) -> u64 {
        self.0
    }

    /// ms -> us
    #[inline]
    pub const fn from_ms(ms: u64) -> Self {
        Self(ms * 1_000)
    }

    /// Returns seconds
    #[inline]
    pub const fn as_secs_f32(&self) -> f32 {
        self.0 as f32 / 1_000_000.
    }

    /// Converts self into microseconds.
    #[inline]
    pub const fn to_micros(&self) -> Micros {
        Micros(self.0 as i64)
    }

    /// Returns no time if I am less than other.
    pub const fn saturating_sub(&self, other: Self) -> UMicros {
        if self.0 < other.0 {
            UMicros(0)
        } else {
            UMicros(self.0 - other.0)
        }
    }
}

impl Add for Micros {
    type Output = Micros;
    fn add(self, rhs: Self) -> Self::Output {
        Micros(self.0 + rhs.0)
    }
}
impl Add<UMicros> for Micros {
    type Output = Micros;
    fn add(self, rhs: UMicros) -> Self::Output {
        Micros(self.0 + rhs.0 as i64)
    }
}

impl AddAssign for Micros {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Micros {
    type Output = Micros;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Micros {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul<i64> for Micros {
    type Output = Micros;
    fn mul(self, rhs: i64) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl Add for UMicros {
    type Output = UMicros;
    fn add(self, rhs: Self) -> Self::Output {
        UMicros(self.0 + rhs.0)
    }
}
impl Add<Micros> for UMicros {
    type Output = Micros;
    fn add(self, rhs: Micros) -> Self::Output {
        Micros(self.0 as i64 + rhs.0)
    }
}

impl AddAssign for UMicros {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}
impl Sub for UMicros {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        UMicros(self.0 - rhs.0)
    }
}

impl Sub<Micros> for UMicros {
    type Output = Micros;
    fn sub(self, rhs: Micros) -> Self::Output {
        Micros(self.0 as i64 - rhs.0)
    }
}

impl From<UMicros> for Micros {
    fn from(value: UMicros) -> Self {
        Self(value.0 as i64)
    }
}
