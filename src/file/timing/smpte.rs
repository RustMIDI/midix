#![doc = r#"
SMPTE (Society of Motion Picture and Television Engineers) Time Code support for MIDI

# What is SMPTE?

SMPTE Time Code is a standard for labeling individual frames of video or film with a time code.
It was developed by the Society of Motion Picture and Television Engineers in the 1960s to provide
accurate synchronization between audio and video equipment.

# Why does SMPTE exist in MIDI?

MIDI supports two timing methods:

1. **Musical Time** - Based on beats and tempo (ticks per quarter note)
2. **Absolute Time** - Based on SMPTE time code (frames per second)

SMPTE timing is essential for:
- Synchronizing MIDI with video/film production
- Professional audio/video post-production work
- Maintaining precise timing relationships independent of tempo
- Broadcasting and theatrical applications

When MIDI uses SMPTE timing, events are timestamped with absolute time positions rather than
musical beats, making them ideal for scenarios where the timing must match external media
exactly, regardless of tempo changes.

# SMPTE Frame Rates in MIDI

The MIDI specification supports four standard SMPTE frame rates, each serving different
video/broadcast standards:
- 24 fps: Film standard
- 25 fps: PAL/SECAM video standard (Europe, Asia, Africa)
- 29.97 fps: NTSC color video (North America, Japan) - "drop frame"
- 30 fps: NTSC black & white video, some digital formats
"#]

/// The possible FPS (Frames Per Second) for MIDI tracks and files
///
/// The MIDI specification defines only four possible frame types:
/// - 24 fps: Standard film rate
/// - 25 fps: PAL/SECAM television standard
/// - 29.97 fps: NTSC color television (drop-frame timecode)
/// - 30 fps: NTSC black & white, some digital video formats
///
/// # Drop-Frame Timecode
///
/// The "TwentyNine" variant represents 29.97 fps, also known as "drop-frame" timecode.
/// This rate (30000/1001 fps) was introduced for NTSC color television to maintain
/// backward compatibility. Despite the name, no actual frames are dropped - the time
/// code numbering skips certain values to keep the timecode aligned with real time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "bevy", derive(bevy::reflect::Reflect))]
pub enum SmpteFps {
    /// 24 frames per second - Standard film rate
    TwentyFour,
    /// 25 frames per second - PAL/SECAM television standard
    TwentyFive,
    /// 29.97 frames per second (30000/1001) - NTSC color television drop-frame rate
    TwentyNine,
    /// 30 frames per second - NTSC black & white, some digital formats
    Thirty,
}

impl SmpteFps {
    /// Get the nominal frame rate as an integer division value.
    ///
    /// This returns the simplified integer representation used in MIDI timing calculations.
    /// Note that drop-frame 29.97 fps returns 30 here, as MIDI uses the nominal rate
    /// for division calculations.
    ///
    /// # Example
    /// ```ignore
    /// assert_eq!(SmpteFps::TwentyNine.as_division(), 30); // Not 29!
    /// ```
    pub const fn as_division(&self) -> u8 {
        match self {
            Self::TwentyFour => 24,
            Self::TwentyFive => 25,
            Self::TwentyNine => 30,
            Self::Thirty => 30,
        }
    }
    /// Get the actual frame rate as a floating-point value.
    ///
    /// This returns the precise frame rate, including the fractional rate for
    /// drop-frame timecode (29.97 fps = 30000/1001).
    ///
    /// Use this method when you need precise time calculations, especially
    /// for synchronization with actual video playback.
    ///
    /// # Drop-Frame Note
    ///
    /// The 29.97 fps rate doesn't actually drop frames from the video.
    /// Instead, the timecode numbering skips certain values (frames 0 and 1
    /// of every minute except multiples of 10) to keep the timecode aligned
    /// with real time over long durations.
    pub const fn as_f64(&self) -> f64 {
        match self {
            Self::TwentyFour => 24.,
            Self::TwentyFive => 25.,
            Self::TwentyNine => DROP_FRAME,
            Self::Thirty => 30.,
        }
    }
}

/// The precise value for NTSC drop-frame rate: 29.97002997... fps
/// This fractional rate ensures color NTSC video stays synchronized with its audio
const DROP_FRAME: f64 = 30_000. / 1001.;
