#![doc = r"
# Identifier for a MIDI Channel
"]

use core::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use crate::message::{ChannelVoiceMessage, VoiceEvent};

/// Identifies a channel for MIDI.
///
/// To get this channel from a `u8`, use [`Channel::try_from_primitive`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, PartialOrd, Ord)]
#[cfg_attr(
    feature = "bevy",
    derive(bevy::prelude::Component, bevy::prelude::Reflect)
)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum Channel {
    /// 0bxxxx0000
    One = 0,
    /// 0bxxxx0001
    Two,
    /// 0bxxxx0010
    Three,
    /// 0bxxxx0011
    Four,
    /// 0bxxxx0100
    Five,
    /// 0bxxxx0101
    Six,
    /// 0bxxxx0110
    Seven,
    /// 0bxxxx0111
    Eight,
    /// 0bxxxx1000
    Nine,
    /// 0bxxxx1001
    ///
    /// Note: MIDI gives Channel Ten a special role (drums).
    ///
    /// Therefore, this channel may have different properties than you would expect!
    Ten,
    /// 0bxxxx1010
    Eleven,
    /// 0bxxxx1011
    Twelve,
    /// 0bxxxx1100
    Thirteen,
    /// 0bxxxx1101
    Fourteen,
    /// 0bxxxx1110
    Fifteen,
    /// 0bxxxx1111
    Sixteen,
}

impl Channel {
    /// Return an array of all channels ordered [`Channel::One`] through [`Channel::Sixteen`]
    pub const fn all() -> [Channel; 16] {
        use Channel::*;
        [
            One, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Eleven, Twelve, Thirteen,
            Fourteen, Fifteen, Sixteen,
        ]
    }

    /// Send a voice event to this channel
    pub const fn send_event(self, event: VoiceEvent) -> ChannelVoiceMessage {
        ChannelVoiceMessage::new(self, event)
    }
    /// Create a `Channel` from a byte.
    ///
    /// 0 -> `Channel::One`
    /// ..
    /// 15 -> `Channel::Sixteen`
    pub const fn try_from_byte(byte: u8) -> Option<Channel> {
        let channel = match byte {
            0 => Channel::One,
            1 => Channel::Two,
            2 => Channel::Three,
            3 => Channel::Four,
            4 => Channel::Five,
            5 => Channel::Six,
            6 => Channel::Seven,
            7 => Channel::Eight,
            8 => Channel::Nine,
            9 => Channel::Ten,
            10 => Channel::Eleven,
            11 => Channel::Twelve,
            12 => Channel::Thirteen,
            13 => Channel::Fourteen,
            14 => Channel::Fifteen,
            15 => Channel::Sixteen,
            _ => return None,
        };
        Some(channel)
    }

    /// Given a status byte from some [`ChannelVoiceMessage`], perform bitwise ops
    /// to get the channel
    #[must_use]
    pub const fn from_status(status: u8) -> Self {
        let channel = status & 0b0000_1111;
        // SAFETY: every produced value must be a valid discriminant.
        //
        // Channel will ALWAYS be between 0 and 15 here and is owned.
        unsafe { core::mem::transmute(channel) }
    }

    /// Returns the 4-bit channel number (0-15)
    #[must_use]
    pub const fn to_byte(&self) -> u8 {
        *self as u8
    }
}

impl fmt::Display for Channel {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let res: u8 = *self as u8;
        res.fmt(f)
    }
}

impl Add<u8> for Channel {
    type Output = Channel;
    fn add(self, rhs: u8) -> Self::Output {
        // convert to the raw repr, add, then map back
        let mut next = (self as u8).saturating_add(rhs);
        if next > 15 {
            next = 15;
        }

        assert!((0..16).contains(&next));
        // SAFETY: every produced value must be a valid discriminant.
        //
        // We check that next is not greater than 15.
        unsafe { core::mem::transmute(next) }
    }
}

impl AddAssign<u8> for Channel {
    fn add_assign(&mut self, rhs: u8) {
        *self = *self + rhs;
    }
}

#[test]
fn test_add_channel() {
    let channel = Channel::Two;
    assert_eq!(channel + 0, Channel::Two);
    assert_eq!(channel + 1, Channel::Three);
    assert_eq!(channel + 28, Channel::Sixteen);
    assert_eq!(channel + 140, Channel::Sixteen);
}

impl Sub<u8> for Channel {
    type Output = Channel;
    fn sub(self, rhs: u8) -> Self::Output {
        // wrapping behaviour; pick `checked_sub` or `overflowing_sub` if you prefer
        let next = (self as u8).saturating_sub(rhs);

        assert!((0..16).contains(&next));
        // SAFETY: all values map to valid discriminants here
        unsafe { core::mem::transmute(next) }
    }
}

impl SubAssign<u8> for Channel {
    fn sub_assign(&mut self, rhs: u8) {
        *self = *self - rhs;
    }
}

#[test]
fn test_sub_channel() {
    let channel = Channel::Five;
    assert_eq!(channel - 0, Channel::Five);
    assert_eq!(channel - 1, Channel::Four);
    assert_eq!(channel - 5, Channel::One);
    assert_eq!(channel - 8, Channel::One);
}

#[test]
fn channel_from_status() {
    use pretty_assertions::assert_eq;
    assert_eq!(Channel::Eight, Channel::from_status(0b1011_0111));
    assert_eq!(Channel::One, Channel::from_status(0b1011_0000));
    assert_eq!(Channel::Sixteen, Channel::from_status(0b0101_1111));
}
