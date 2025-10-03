mod smpte;
pub use smpte::*;

use crate::{prelude::*, reader::ReaderError};

/// The header timing type.
///
/// This is either the number of ticks per quarter note or
/// the alternative SMTPE format. See the [`RawHeaderChunk`](crate::file::builder::chunk::RawHeaderChunk) docs for more information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "bevy", derive(bevy::reflect::Reflect))]
pub enum Timing {
    /// The midi file's delta times are defined using a tick rate per quarter note
    TicksPerQuarterNote(TicksPerQuarterNote),

    /// The midi file's delta times are defined using an SMPTE and MIDI Time Code
    Smpte(SmpteHeader),
}

impl Timing {
    /// The tickrate per quarter note defines what a "quarter note" means.
    ///
    /// The leading bit of the u16 is disregarded, so 1-32767
    pub const fn new_ticks_per_quarter_note(tpqn: u16) -> Self {
        let msb = (tpqn >> 8) as u8;
        let lsb = (tpqn & 0x00FF) as u8;
        Self::TicksPerQuarterNote(TicksPerQuarterNote { inner: [msb, lsb] })
    }

    /// Define the timing in terms of fps and ticks per frame
    pub const fn new_smpte(fps: SmpteFps, ticks_per_frame: DataByte) -> Self {
        Self::Smpte(SmpteHeader {
            fps,
            ticks_per_frame,
        })
    }

    pub(crate) fn read<'slc, 'r, R: MidiSource<'slc>>(
        reader: &'r mut Reader<R>,
    ) -> ReadResult<Self> {
        let bytes = reader.read_exact_size()?;
        match bytes[0] >> 7 {
            0 => {
                //this is ticks per quarter_note
                Ok(Timing::TicksPerQuarterNote(TicksPerQuarterNote {
                    inner: bytes,
                }))
            }
            1 => Ok(Timing::Smpte(SmpteHeader::new(bytes).map_err(|e| {
                ReaderError::new(reader.buffer_position(), e.into())
            })?)),
            t => Err(inv_data(reader, HeaderError::InvalidTiming(t))),
        }
    }
    /// Returns Some if the midi timing is defined
    /// as ticks per quarter note
    pub const fn ticks_per_quarter_note(&self) -> Option<u16> {
        match self {
            Self::TicksPerQuarterNote(t) => Some(t.ticks_per_quarter_note()),
            _ => None,
        }
    }
}

/// A representation of the `tpqn` timing for a MIDI file
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "bevy", derive(bevy::reflect::Reflect))]
pub struct TicksPerQuarterNote {
    pub(crate) inner: [u8; 2],
}
impl TicksPerQuarterNote {
    /// Returns the ticks per quarter note for the file.
    pub const fn ticks_per_quarter_note(&self) -> u16 {
        let v = u16::from_be_bytes(self.inner);
        v & 0x7FFF
    }
}

/// A representation of the `smpte` timing for a MIDI file
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[cfg_attr(feature = "bevy", derive(bevy::reflect::Reflect))]
pub struct SmpteHeader {
    pub(crate) fps: SmpteFps,
    pub(crate) ticks_per_frame: DataByte,
}

impl SmpteHeader {
    fn new(bytes: [u8; 2]) -> Result<Self, ParseError> {
        //first byte is known to be 1 when calling this
        //Bits 14 thru 8 contain one of the four values -24, -25, -29, or -30
        let byte = bytes[0] as i8;

        let frame = match byte {
            -24 => SmpteFps::TwentyFour,
            -25 => SmpteFps::TwentyFive,
            -29 => {
                //drop frame (29.997)
                SmpteFps::TwentyNine
            }
            -30 => SmpteFps::Thirty,
            _ => return Err(ParseError::Smpte(SmpteError::HeaderFrameTime(byte))),
        };
        let ticks_per_frame = DataByte::new(bytes[1])?;
        Ok(Self {
            fps: frame,
            ticks_per_frame,
        })
    }

    /// Returns the frames per second
    pub const fn fps(&self) -> SmpteFps {
        self.fps
    }

    /// Returns the ticks per frame
    pub const fn ticks_per_frame(&self) -> u8 {
        self.ticks_per_frame.0
    }
}
