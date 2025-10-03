use super::Reader;
use crate::{ChunkError, ParseError};
use thiserror::Error;

#[doc = r#"
A set of errors that can occur while reading data into the midi representation
"#]
#[derive(Debug, Error)]
#[error("Reading at Position {position}, {kind}")]
pub struct ReaderError {
    position: usize,
    pub(crate) kind: ReaderErrorKind,
}

/// A kind of error that a reader can produce
#[derive(Debug, Error)]
pub enum ReaderErrorKind {
    /// Parsing errors
    #[error("Parsing {0}")]
    ParseError(#[from] ParseError),
    /// Reading out of bounds.
    #[error("Read out of bounds!")]
    OutOfBounds,
}

impl ReaderErrorKind {
    pub(crate) const fn chunk(chunk_err: ChunkError) -> Self {
        Self::ParseError(ParseError::Chunk(chunk_err))
    }
}

impl ReaderError {
    /// Create a reader error from a position and kind
    pub const fn new(position: usize, kind: ReaderErrorKind) -> Self {
        Self { position, kind }
    }
    /// True if out of bounds or unexpected end of file
    pub const fn is_out_of_bounds(&self) -> bool {
        matches!(self.kind, ReaderErrorKind::OutOfBounds)
    }
    /// Returns the error kind of the reader.
    pub fn error_kind(&self) -> &ReaderErrorKind {
        &self.kind
    }
    /// Returns the position where the read error occurred.
    pub fn position(&self) -> usize {
        self.position
    }

    /// Create a new invalid data error
    pub const fn parse_error(position: usize, error: ParseError) -> Self {
        Self {
            position,
            kind: ReaderErrorKind::ParseError(error),
        }
    }

    /// Create a new out of bounds error
    pub const fn oob(position: usize) -> Self {
        Self {
            position,
            kind: ReaderErrorKind::OutOfBounds,
        }
    }
}

/// The Read Result type (see [`ReaderError`])
///
/// This may change in a future release if `midix`
/// should support `no-std` environments.
pub type ReadResult<T> = Result<T, ReaderError>;

pub(crate) fn inv_data<R>(reader: &mut Reader<R>, v: impl Into<ParseError>) -> ReaderError {
    reader.set_last_error_offset(reader.buffer_position());
    ReaderError::parse_error(reader.buffer_position(), v.into())
}
