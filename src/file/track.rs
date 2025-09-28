use alloc::vec::Vec;

use crate::{
    channel::Channel,
    events::LiveEvent,
    message::Ticked,
    prelude::{BytesText, SmpteOffset, Tempo, TimeSignature, TrackEvent, TrackMessage},
};

#[doc = r#"
A set of track events
"#]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "bevy", derive(bevy::reflect::Reflect))]
pub struct Track<'a> {
    info: TrackInfo<'a>,
    pub(crate) events: Vec<Ticked<LiveEvent<'a>>>,
}

impl<'a> Track<'a> {
    /// Create a new track
    pub fn new(events: Vec<TrackEvent<'a>>) -> Self {
        let mut info = TrackInfo::default();
        let mut track_events = Vec::with_capacity(events.len());

        let mut time_accumulated = None;

        for event in events {
            let delta_ticks = event.delta_ticks();

            let accumulated_ticks = if let Some(tick_acc) = &mut time_accumulated {
                *tick_acc += delta_ticks;
                *tick_acc
            } else {
                time_accumulated = Some(delta_ticks);
                delta_ticks
            };
            let event: LiveEvent = match event.into_event() {
                TrackMessage::ChannelVoice(cvm) => cvm.into(),
                TrackMessage::SystemExclusive(sysex) => sysex.into(),
                TrackMessage::Meta(meta) => {
                    meta.adjust_track_info(&mut info);
                    continue;
                }
            };
            track_events.push(Ticked::new(accumulated_ticks, event));
        }

        // update track_event's time_since_start, since it currently
        // holds delta_time, which is fractions of a beat.
        // So we want to convert fractions of a beat to microseconds
        // The conversion is
        // Track (us / quarter note) *
        // Header (quarter notes / tick )^-1
        // Ticks (delta time)

        Self {
            info,
            events: track_events,
        }
    }

    /// Get information about the track
    pub fn info(&self) -> &TrackInfo<'a> {
        &self.info
    }
    /// Get the timed events for the track
    pub fn events(&self) -> &[Ticked<LiveEvent<'a>>] {
        self.events.as_slice()
    }
}

/// Provides information about the track
#[allow(missing_docs)]
#[derive(Default, Debug, Clone, PartialEq)]
#[cfg_attr(feature = "bevy", derive(bevy::reflect::Reflect))]
pub struct TrackInfo<'a> {
    pub time_signature: TimeSignature,
    pub name: Option<BytesText<'a>>,
    pub device: Option<BytesText<'a>>,
    pub track_info: Option<u16>,
    pub channel: Option<Channel>,
    pub tempo: Tempo,
    /// this is intentionally allowed if the file doesn't identify as using smpte.
    pub smpte_offset: Option<SmpteOffset>,
}

#[test]
fn get_accumulated_ticks() {}
