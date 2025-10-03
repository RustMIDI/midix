use crate::file::FormatType;

#[doc = r#"

    FF 00 02 Sequence Number
    This optional event, which must occur at the beginning of a track,
    before any nonzero delta-times, and before any transmittable MIDI
    events, specifies the number of a sequence. In a format 2 MIDI File,
    it is used to identify each "pattern" so that a "song" sequence using
    the Cue message can refer to the patterns. If the ID numbers are
    omitted, the sequences' locations in order in the file are used as
    defaults. In a format 0 or 1 MIDI File, which only contain one
    sequence, this number should be contained in the first (or only)
    track. If transfer of several multitrack sequences is required,
    this must be done as a group of format 1 files, each with a different
    sequence number.
"#]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RawFormat {
    /// Format 0
    SingleMultiChannel,
    /// Format 1
    Simultaneous([u8; 2]),
    /// Format 2
    SequentiallyIndependent([u8; 2]),
}
impl RawFormat {
    /// Create a [`RawFormat::SingleMultiChannel`]
    pub const fn single_multichannel() -> Self {
        Self::SingleMultiChannel
    }

    /// Create a [`Format::Simultaneous`]
    pub(crate) const fn simultaneous_from_byte_slice(bytes: [u8; 2]) -> Self {
        Self::Simultaneous(bytes)
    }

    /// Create a [`Format::SequentiallyIndependent`]
    pub(crate) const fn sequentially_independent_from_byte_slice(bytes: [u8; 2]) -> Self {
        Self::SequentiallyIndependent(bytes)
    }

    /// Returns the number of tracks identified by the format.
    ///
    /// [`RawFormat::SingleMultiChannel`] will always return 1.
    pub const fn num_tracks(&self) -> u16 {
        use RawFormat::*;
        match &self {
            SingleMultiChannel => 1,
            Simultaneous(num) | SequentiallyIndependent(num) => u16::from_be_bytes(*num),
        }
    }

    /// Returns the format type of the format.
    pub const fn format_type(&self) -> FormatType {
        use RawFormat::*;
        match self {
            SingleMultiChannel => FormatType::SingleMultiChannel,
            Simultaneous(_) => FormatType::Simultaneous,
            SequentiallyIndependent(_) => FormatType::SequentiallyIndependent,
        }
    }
}
