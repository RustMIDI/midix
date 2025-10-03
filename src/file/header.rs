use crate::prelude::*;

#[doc = r#"
   Information about the timing of the MIDI file
"#]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "bevy", derive(bevy::reflect::Reflect))]
pub struct MidiFileHeader {
    timing: Timing,
}

impl MidiFileHeader {
    /// Create a new header from timing
    pub fn new(timing: Timing) -> Self {
        Self { timing }
    }
    /// Get the timing props
    pub fn timing(&self) -> &Timing {
        &self.timing
    }
}
