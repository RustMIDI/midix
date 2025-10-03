#![doc = r#"
SMPTE Offset - Precise time positioning for MIDI events

# What is SMPTE Offset?

SMPTE Offset is a MIDI meta-event that specifies an exact starting time for a track
using SMPTE time code format. This allows MIDI sequences to be precisely synchronized
with video, film, or other time-based media.

# Why use SMPTE Offset?

SMPTE Offset is essential for:
- **Post-production**: Aligning MIDI tracks with specific video frames
- **Broadcasting**: Ensuring music cues hit exact broadcast timecodes
- **Film scoring**: Synchronizing musical events with on-screen action
- **Multi-track recording**: Maintaining sync across different recording sessions

When a MIDI file uses SMPTE-based timing (instead of tempo-based), the SMPTE Offset
tells sequencers exactly where in absolute time the track should begin playing.

# Format

The SMPTE Offset meta-event contains:
- Frame rate (24, 25, 29.97, or 30 fps)
- Hours (0-23)
- Minutes (0-59)
- Seconds (0-59)
- Frames (0-29/24 depending on fps)
- Subframes (0-99, for additional precision)

This provides frame-accurate positioning for professional audio/video work.
"#]

use crate::{SmpteError, prelude::SmpteFps};

/// A representation of a MIDI track's starting position in SMPTE time code.
///
/// This structure holds an absolute time position using SMPTE (Society of Motion Picture
/// and Television Engineers) time code format. When present in a MIDI file, it indicates
/// that the track should begin playback at this specific time position rather than at
/// the beginning of the sequence.
///
/// # Use Cases
/// - Synchronizing MIDI with video where the music doesn't start at 00:00:00:00
/// - Aligning multiple MIDI files that represent different sections of a larger work
/// - Post-production workflows requiring frame-accurate synchronization
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "bevy", derive(bevy::reflect::Reflect))]
pub struct SmpteOffset {
    /// The frame rate for this offset.
    ///
    /// **Important**: This should match the file's frame rate when the MIDI file
    /// uses SMPTE-based timing. Mismatched rates will cause synchronization errors.
    pub fps: SmpteFps,
    /// Hour component of the time code (0-23).
    ///
    /// Represents hours in 24-hour format. Values above 23 are invalid
    /// and will be rejected during parsing.
    pub hour: u8,
    /// Minute component of the time code (0-59).
    pub minute: u8,
    /// Second component of the time code (0-59).
    pub second: u8,
    /// Frame number within the current second.
    ///
    /// Valid range depends on the frame rate:
    /// - 24 fps: 0-23
    /// - 25 fps: 0-24
    /// - 29.97 fps: 0-29 (with drop-frame rules)
    /// - 30 fps: 0-29
    pub frame: u8,
    /// Subframe component for additional precision (0-99).
    ///
    /// Each subframe represents 1/100th of a frame, allowing for
    /// sub-frame accuracy in positioning. This is particularly useful
    /// for sample-accurate synchronization in digital audio workstations.
    pub subframe: u8,
}

impl SmpteOffset {
    /// Calculate the offset in microseconds using a different frame rate.
    ///
    /// This is useful when the MIDI file's timing uses a different SMPTE
    /// rate than the offset itself. The provided `fps` parameter overrides
    /// the offset's internal frame rate for the calculation.
    ///
    /// # Parameters
    /// - `fps`: The frame rate to use for the calculation
    ///
    /// # Returns
    /// The time offset in microseconds as a floating-point value
    pub const fn as_micros_with_override(&self, fps: SmpteFps) -> f64 {
        ((((self.hour as u64 * 3600) + (self.minute as u64) * 60 + self.second as u64) * 1_000_000)
            as f64)
            + ((self.frame as u64) * 1_000_000) as f64 / fps.as_f64()
            + ((self.subframe as u32) * 10_000) as f64 / fps.as_f64()
    }
    /// Convert this SMPTE offset to microseconds.
    ///
    /// Calculates the absolute time position represented by this offset
    /// using its internal frame rate. The calculation accounts for hours,
    /// minutes, seconds, frames, and subframes to provide a precise
    /// microsecond value.
    pub const fn as_micros(&self) -> f64 {
        ((((self.hour as u64 * 3600) + (self.minute as u64) * 60 + self.second as u64) * 1_000_000)
            as f64)
            + ((self.frame as u64) * 1_000_000) as f64 / self.fps.as_f64()
            + ((self.subframe as u32) * 10_000) as f64 / self.fps.as_f64()
    }

    /// Parse a SMPTE offset from a 5-byte MIDI data array.
    ///
    /// The MIDI specification defines the SMPTE offset format as:
    /// - Byte 0: `0rrhhhhh` where `rr` is frame rate type, `hhhhh` is hours
    /// - Byte 1: Minutes (0-59)
    /// - Byte 2: Seconds (0-59)
    /// - Byte 3: Frames (depends on frame rate)
    /// - Byte 4: Fractional frames in 100ths (0-99)
    ///
    /// # Frame Rate Encoding
    /// The frame rate is encoded in bits 5-6 of the first byte:
    /// - `00`: 24 fps
    /// - `01`: 25 fps
    /// - `10`: 29.97 fps (drop frame)
    /// - `11`: 30 fps
    ///
    /// # Errors
    /// - `SmpteError::Length` if data is not exactly 5 bytes
    /// - `SmpteError::TrackFrame` if frame rate type is invalid
    /// - `SmpteError::HourOffset` if hours > 23
    /// - `SmpteError::MinuteOffset` if minutes > 59
    /// - `SmpteError::SecondOffset` if seconds > 59
    /// - `SmpteError::Subframe` if fractional frames > 99
    pub const fn parse(data: &[u8]) -> Result<Self, SmpteError> {
        if data.len() != 5 {
            return Err(SmpteError::Length(data.len()));
        }

        // 0 rr hhhhh
        let frame_type = match data[0] >> 5 {
            0 => SmpteFps::TwentyFour,
            1 => SmpteFps::TwentyFive,
            2 => SmpteFps::TwentyNine,
            3 => SmpteFps::Thirty,
            v => return Err(SmpteError::TrackFrame(v)),
        };
        let hour = data[0] & 0b0001_1111;
        if hour > 23 {
            return Err(SmpteError::HourOffset(hour));
        }
        let minute = data[1];
        if minute > 59 {
            return Err(SmpteError::MinuteOffset(minute));
        }
        let second = data[2];
        if second > 59 {
            return Err(SmpteError::SecondOffset(second));
        }

        let frame = data[3];
        // always 1/100 of frame
        let subframe = data[4];
        if subframe > 99 {
            return Err(SmpteError::Subframe(subframe));
        }
        Ok(Self {
            fps: frame_type,
            hour,
            minute,
            second,
            frame,
            subframe,
        })
    }
}

#[test]
fn parse_smpte_offset() {
    use pretty_assertions::assert_eq;
    // this are the bytes after 00 FF 54 05
    // where 54 is smpte offset, and 05 is length five.
    let bytes = [0x41, 0x17, 0x2D, 0x0C, 0x22];
    let offset = SmpteOffset::parse(&bytes).unwrap();

    assert_eq!(offset.fps, SmpteFps::TwentyNine);
    assert_eq!(offset.hour, 1);
    assert_eq!(offset.minute, 23);
    assert_eq!(offset.second, 45);
    assert_eq!(offset.frame, 12);
    assert_eq!(offset.subframe, 34);
}

#[test]
fn parse_invalid_smpte_offset() {
    use pretty_assertions::assert_eq;
    // this are the bytes after 00 FF 54 05
    // where 54 is smpte offset, and 05 is length five.
    let bytes = [0x7F, 0x17, 0x2D, 0x0C, 0x22];
    let err = SmpteOffset::parse(&bytes).unwrap_err();
    assert_eq!(err, SmpteError::HourOffset(31));

    let bytes = [0x41, 0x50, 0x2D, 0x0C, 0x22];
    let err = SmpteOffset::parse(&bytes).unwrap_err();
    assert_eq!(err, SmpteError::MinuteOffset(80));
}
