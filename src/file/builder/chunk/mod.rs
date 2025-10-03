#![doc = r#"
Contains types for MIDI file chunks

# Overview

MIDI files are organized into chunks, each identified by a 4-character ASCII type identifier
followed by a 32-bit length field and then the chunk data. The Standard MIDI File (SMF)
specification defines two chunk types, though files may contain additional proprietary chunks.

MIDI defines anything that does not fall into the standard chunk types as unknown chunks,
which can be safely ignored or processed based on application needs.

## [`RawHeaderChunk`]

The header chunk (identified by "MThd") must be the first chunk in a MIDI file. This chunk
type contains meta information about the MIDI file, such as:

- [`RawFormat`](crate::file::builder::RawFormat), which identifies how tracks should be played
  (single track, simultaneous tracks, or independent tracks) and the number of tracks in the file
- [`Timing`](crate::prelude::Timing), which defines how delta-ticks (timestamps) are to be
  interpreted - either as ticks per quarter note or in SMPTE time code format

The header chunk always has a fixed length of 6 bytes.

## Track Chunks

Track chunks (identified by "MTrk") contain the actual MIDI events and timing information:

- [`TrackChunkHeader`] - Contains only the length in bytes of the track data
- [`RawTrackChunk`] - Contains the complete track data which can be parsed into a sequence
  of [`TrackEvent`](crate::prelude::TrackEvent)s, each with delta-time and event data

Track chunks appear after the header chunk, and the number of track chunks should match
the track count specified in the header (though this is not strictly enforced by all
MIDI software).

## [`UnknownChunk`]

Any chunk with a type identifier other than "MThd" or "MTrk" is treated as an unknown chunk.
These chunks preserve their type identifier and data, allowing applications to either:
- Ignore them (the most common approach)
- Process them if they understand the proprietary format
- Preserve them when reading and writing files to maintain compatibility

# Example Structure

A typical MIDI file structure looks like:
```text
[Header Chunk: "MThd"]
[Track Chunk 1: "MTrk"]
[Track Chunk 2: "MTrk"]
...
[Track Chunk N: "MTrk"]
[Optional Unknown Chunks]
```
"#]

mod unknown_chunk;
pub use unknown_chunk::*;

mod header;
pub use header::*;

mod track;
pub use track::*;
