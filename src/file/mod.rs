#![doc = r#"
Rusty representation of a [`MidiFile`]
"#]

/// Contains the [`MidiFileBuilder`] and assocaited
///
/// MIDI file parsing events.
pub mod builder;

mod format;
pub use format::*;

mod track;
pub use track::*;

mod timed_event_iter;
pub use timed_event_iter::*;

mod timing;
pub use timing::*;

mod meta;
pub use meta::*;

use crate::{
    ParseError,
    events::LiveEvent,
    file::builder::MidiFileBuilder,
    message::Timed,
    reader::{ReadResult, Reader, ReaderError, ReaderErrorKind},
};
use alloc::{borrow::Cow, vec::Vec};

#[doc = r#"
TODO
"#]
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "bevy", derive(bevy::reflect::Reflect))]
pub struct MidiFile<'a> {
    timing: Timing,
    format: Format<'a>,
}
#[cfg(feature = "bevy_asset")]
impl bevy::asset::Asset for MidiFile<'static> {}

#[cfg(feature = "bevy_asset")]
impl bevy::asset::VisitAssetDependencies for MidiFile<'static> {
    fn visit_dependencies(&self, _visit: &mut impl FnMut(bevy::asset::UntypedAssetId)) {}
}

impl<'a> MidiFile<'a> {
    /// Parse a set of bytes into a file struct
    pub fn parse<B>(bytes: B) -> ReadResult<Self>
    where
        B: Into<Cow<'a, [u8]>>,
    {
        let mut reader = Reader::from_bytes(bytes);
        let mut builder = MidiFileBuilder::default();

        loop {
            let val = reader.read_chunk().unwrap();

            if val.is_eof() {
                break;
            }
            builder
                .handle_chunk(val)
                .map_err(|k| ReaderError::new(reader.buffer_position(), k))?;
        }

        builder.build().map_err(|k| {
            ReaderError::new(
                reader.buffer_position(),
                ReaderErrorKind::ParseError(ParseError::File(k)),
            )
        })
    }

    /// Returns header info
    pub fn timing(&self) -> Timing {
        self.timing
    }

    /// Executes the provided function for all the tracks in the format.
    ///
    /// Useful if you don't want to allocate more data on the stack.
    pub fn for_each_track<F>(&self, mut func: F)
    where
        F: FnMut(&Track),
    {
        match &self.format {
            Format::SequentiallyIndependent(t) => t.iter().for_each(func),
            Format::Simultaneous(s) => s.iter().for_each(func),
            Format::SingleMultiChannel(c) => func(c),
        }
    }

    /// Returns a track list
    pub fn tracks(&self) -> Vec<&Track<'a>> {
        match &self.format {
            Format::SequentiallyIndependent(t) => t.iter().collect(),
            Format::Simultaneous(s) => s.iter().collect(),
            Format::SingleMultiChannel(c) => [c].to_vec(),
        }
    }
    /// Returns the format type for the file.
    pub fn format_type(&self) -> FormatType {
        match &self.format {
            Format::SequentiallyIndependent(_) => FormatType::SequentiallyIndependent,
            Format::Simultaneous(_) => FormatType::Simultaneous,
            Format::SingleMultiChannel(_) => FormatType::SingleMultiChannel,
        }
    }

    /// Returns a set of timed events from the midi file.
    pub fn into_events(self) -> impl Iterator<Item = Timed<LiveEvent<'a>>> {
        match TimedEventIterator::new(self) {
            Some(iter) => OptTimedEventIterator::Some(iter),
            None => OptTimedEventIterator::None,
        }
    }
}
