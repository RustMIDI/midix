use std::fmt::Debug;

use crate::prelude::*;

#[doc = r#"
Identifies some event emitted by a track in a MIDI file.

# Overview
All MIDI track events have an associated `delta_time`. This
identifies the amount of time since the previous event.

"#]
#[derive(Clone, PartialEq)]
pub struct TrackEvent<'a> {
    /// Variable length quantity
    /// Delta-time is in some fraction of a beat
    /// (or a second, for recording a track with SMPTE times),
    /// as specified in the header chunk.
    delta_time: u32,
    event: TrackMessage<'a>,
}

impl Debug for TrackEvent<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Track Event {{ delta_time: 0x{:02X}, event: {:?} }}",
            self.delta_time, self.event
        )
    }
}

impl<'a> TrackEvent<'a> {
    pub(crate) fn new(delta_time: u32, event: TrackMessage<'a>) -> Self {
        Self { delta_time, event }
    }

    /// Update the running status here.
    pub(crate) fn read<'slc, 'r, R>(
        reader: &'r mut Reader<R>,
        running_status: &mut Option<u8>,
    ) -> ReadResult<Self>
    where
        R: MidiSource<'slc>,
        'slc: 'a,
    {
        let delta_time = crate::reader::decode_varlen(reader)?;

        let next_event = reader.read_next()?;

        let message = match next_event {
            0xF0 => {
                let mut data = reader.read_varlen_slice()?;
                if !data.is_empty() {
                    //discard the last 0xF7
                    data.truncate(1);
                }
                TrackMessage::SystemExclusive(SystemExclusiveMessage::new(data))
            }
            0xFF => TrackMessage::Meta(MetaMessage::read(reader)?),
            byte => {
                //status if the byte has a leading 1, otherwise it's
                //a running status

                let status = if byte >> 7 == 1 {
                    *running_status = Some(byte);
                    byte
                } else if let Some(prev_status) = running_status {
                    //Hack: decrementing the buffer position should not be done
                    reader.state.decrement_offset(1);
                    *prev_status
                } else {
                    return Err(inv_data(reader, "Invalid MIDI event triggered"));
                };
                let status = StatusByte::try_from(status).unwrap();

                TrackMessage::ChannelVoice(ChannelVoiceMessage::read(status, reader)?)
            }
        };

        Ok(Self {
            delta_time,
            event: message,
        })
    }

    /// Get the difference in time from the last event
    ///
    /// The actual value should be interpreted by the MIDI file's
    /// [`Timing`] event.
    pub fn delta_time(&self) -> u32 {
        self.delta_time
    }
    /// Get a refrence to the message for the track event
    pub fn event(&self) -> &TrackMessage<'a> {
        &self.event
    }
}
