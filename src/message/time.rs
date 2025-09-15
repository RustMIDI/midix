//! Events that require some representation of time

use core::time::Duration;

#[doc = r#"
A wrapper around some type with an associated accumulated tick
"#]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Reflect))]
pub struct Ticked<T> {
    /// In ticks
    accumulated_ticks: u32,
    event: T,
}

impl<T> Ticked<T> {
    /// Create a new timed event based on *accumulated* ticks
    pub const fn new(accumulated_ticks: u32, event: T) -> Self {
        Self {
            accumulated_ticks,
            event,
        }
    }

    /// Returns the accumulated ticks since the beginning of the track
    pub const fn accumulated_ticks(&self) -> u32 {
        self.accumulated_ticks
    }

    /// Returns the timed event
    pub const fn event(&self) -> &T {
        &self.event
    }
}

/// A wrapper around some type with an associated timestamp in micros.
///
/// This differs from `Ticked`, which does not necessarily represent itself in time.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "bevy_resources", derive(bevy::prelude::Reflect))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Timed<T> {
    /// Micros
    pub timestamp: u64,
    /// The event that should happen at the timestamp
    pub event: T,
}
impl<T> Timed<T> {
    /// Create a command to do something at a time.
    ///
    /// Timestamp is delta micros from now.
    pub const fn new(timestamp: u64, event: T) -> Self {
        Self { timestamp, event }
    }

    /// Use a duration to create a timed type.
    pub const fn new_from_duration(duration: Duration, event: T) -> Self {
        Self {
            timestamp: duration.as_micros() as u64,
            event,
        }
    }
}
