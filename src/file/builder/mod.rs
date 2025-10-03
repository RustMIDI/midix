mod format;
pub use format::*;

pub mod chunk;

pub mod event;

use super::MidiFile;
use crate::{
    file::builder::{chunk::UnknownChunk, event::ChunkEvent},
    prelude::*,
    reader::ReaderErrorKind,
};
use alloc::vec::Vec;

#[derive(Default)]
enum FormatStage<'a> {
    #[default]
    Unknown,
    KnownFormat(RawFormat),
    KnownTracks(Vec<Track<'a>>),
    Formatted(Format<'a>),
}

/// A builder used to create a new [`MidiFile`].
#[derive(Default)]
pub struct MidiFileBuilder<'a> {
    format: FormatStage<'a>,
    timing: Option<Timing>,
    //TODO
    unknown_chunks: Vec<UnknownChunk<'a>>,
}

impl<'a> MidiFileBuilder<'a> {
    /// Handles a chunk of a midi file.
    pub fn handle_chunk<'b: 'a>(&mut self, chunk: ChunkEvent<'b>) -> Result<(), ReaderErrorKind> {
        use ChunkEvent::*;
        match chunk {
            Header(h) => {
                if self.timing.is_some() {
                    return Err(ReaderErrorKind::chunk(ChunkError::DuplicateHeader));
                }

                match &self.format {
                    FormatStage::Unknown => {
                        self.format = FormatStage::KnownFormat(h.format().clone());
                    }
                    FormatStage::KnownFormat(_) | FormatStage::Formatted(_) => {
                        return Err(ReaderErrorKind::chunk(ChunkError::DuplicateFormat));
                    }
                    FormatStage::KnownTracks(tracks) => match h.format_type() {
                        FormatType::Simultaneous => {
                            self.format =
                                FormatStage::Formatted(Format::Simultaneous(tracks.clone()))
                        }
                        FormatType::SingleMultiChannel => {
                            // this shouldn't even happen...but we will support headers that aren't at the top of the file, so it *could*
                            if tracks.len() != 1 {
                                return Err(ReaderErrorKind::chunk(
                                    ChunkError::MultipleTracksForSingleMultiChannel,
                                ));
                            }
                            let track = tracks.first().unwrap().clone();
                            self.format = FormatStage::Formatted(Format::SingleMultiChannel(track))
                        }
                        FormatType::SequentiallyIndependent => {
                            self.format = FormatStage::Formatted(Format::SequentiallyIndependent(
                                tracks.clone(),
                            ))
                        }
                    },
                };

                self.timing = Some(h.timing());

                Ok(())
            }
            Track(t) => {
                let events = t.events()?;

                let track = super::Track::new(events);
                let mut track_vec = Vec::new();
                match &mut self.format {
                    FormatStage::Unknown => {
                        track_vec.push(track);
                        self.format = FormatStage::KnownTracks(track_vec);
                    }
                    FormatStage::KnownFormat(t) => match t.format_type() {
                        FormatType::Simultaneous => {
                            track_vec.push(track);

                            self.format = FormatStage::Formatted(Format::Simultaneous(track_vec))
                        }
                        FormatType::SingleMultiChannel => {
                            self.format = FormatStage::Formatted(Format::SingleMultiChannel(track))
                        }
                        FormatType::SequentiallyIndependent => {
                            track_vec.push(track);
                            self.format =
                                FormatStage::Formatted(Format::SequentiallyIndependent(track_vec))
                        }
                    },
                    FormatStage::KnownTracks(tracks) => tracks.push(track),
                    FormatStage::Formatted(format) => match format {
                        Format::SequentiallyIndependent(tracks) => tracks.push(track),
                        Format::SingleMultiChannel(_) => {
                            return Err(ReaderErrorKind::chunk(
                                ChunkError::MultipleTracksForSingleMultiChannel,
                            ));
                        }
                        Format::Simultaneous(tracks) => tracks.push(track),
                    },
                }
                Ok(())
            }
            Unknown(data) => {
                self.unknown_chunks.push(data);
                Ok(())
            }
            Eof => Err(ReaderErrorKind::Eof),
        }
    }
    /// Attempts to finish the midifile from the provided chunks.
    pub fn build(self) -> Result<MidiFile<'a>, FileError> {
        let FormatStage::Formatted(format) = self.format else {
            return Err(FileError::NoFormat);
        };
        let Some(timing) = self.timing else {
            return Err(FileError::NoTiming);
        };

        Ok(MidiFile { format, timing })
    }
}
