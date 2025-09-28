use crate::prelude::*;

/// An iterator returned from [`ParsedMidiFile::into_events`].
pub enum OptTimedEventIterator<'a> {
    /// No tracks in the file
    None,
    /// Iterator over tracks in the file
    Some(TimedEventIterator<'a>),
}
impl<'a> Iterator for OptTimedEventIterator<'a> {
    type Item = Timed<LiveEvent<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            OptTimedEventIterator::Some(iter) => iter.next(),
            OptTimedEventIterator::None => None,
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            OptTimedEventIterator::None => (0, Some(0)),
            OptTimedEventIterator::Some(iter) => iter.size_hint(),
        }
    }
}

/// An iterator returned from [`ParsedMidiFile::into_events`]
pub struct TimedEventIterator<'a> {
    len_remaining: usize,
    header: Header,
    tracks: alloc::vec::IntoIter<Track<'a>>,
    cur_track: CurrentTrack<'a>,
    file_tempo: Option<Tempo>,
}
impl<'a> TimedEventIterator<'a> {
    pub(super) fn new(file: ParsedMidiFile<'a>) -> Option<Self> {
        let header = file.header;

        let (size, tracks, next, file_tempo) = match file.format {
            Format::SequentiallyIndependent(t) => {
                let size = t.iter().fold(0, |acc, b| acc + b.events.len());

                let mut iter = t.into_iter();
                let cur_track = iter.next()?;
                (size, iter, cur_track, None)
            }

            Format::Simultaneous(t) => {
                let size = t.iter().fold(0, |acc, b| acc + b.events.len());
                let mut iter = t.into_iter();
                let cur_track = iter.next()?;
                let tempo = cur_track.info().tempo;
                (size, iter, cur_track, Some(tempo))
            }
            Format::SingleMultiChannel(track) => {
                let size = track.events.len();
                let tempo = track.info().tempo;
                (size, alloc::vec::Vec::new().into_iter(), track, Some(tempo))
            }
        };
        let cur_track = CurrentTrack::new(next, file_tempo, header.timing());

        Some(Self {
            len_remaining: size,
            header,
            tracks,
            cur_track,
            file_tempo,
        })
    }
}

impl<'a> Iterator for TimedEventIterator<'a> {
    type Item = Timed<LiveEvent<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.cur_track.next() {
            Some(event) => {
                self.len_remaining -= 1;
                Some(event)
            }
            None => {
                let next_track = self.tracks.next()?;
                let mut next_track =
                    CurrentTrack::new(next_track, self.file_tempo, self.header.timing());
                let next_ev = next_track.next()?;
                self.len_remaining -= 1;
                self.cur_track = next_track;
                Some(next_ev)
            }
        }
    }
}

struct CurrentTrack<'a> {
    micros_per_tick: f64,
    offset_in_micros: f64,
    event: alloc::vec::IntoIter<Ticked<LiveEvent<'a>>>,
}

impl<'a> CurrentTrack<'a> {
    fn new(track: Track<'a>, file_tempo: Option<Tempo>, timing: &Timing) -> Self {
        let track_tempo = file_tempo.unwrap_or(track.info().tempo);
        let micros_per_quarter_note = track_tempo.micros_per_quarter_note();

        let (micros_per_tick, offset_in_micros) = match timing {
            Timing::Smpte(v) => {
                //µs_per_tick = 1 000 000 / (fps × ticks_per_frame)
                //FPS is −24/−25/−29/−30 in the high byte of division;
                // ticks per frame is the low byte.

                let frames_per_second = v.fps().as_division() as u32;
                let ticks_per_frame = v.ticks_per_frame() as u32;
                let ticks_per_second = frames_per_second * ticks_per_frame;
                let micros_per_tick = 1_000_000. / ticks_per_second as f64;

                //NOTE: if the file header uses smpte, that overrides any track smpte offset.
                let offset_in_micros = track
                    .info()
                    .smpte_offset
                    .as_ref()
                    .map(|offset| {
                        if offset.fps != v.fps() {
                            #[cfg(feature = "tracing")]
                            tracing::warn!(
                                "Header's fps({}) does not align with track's fps({}). \
                                            The file's fps will override the track's!",
                                v.fps().as_f64(),
                                offset.fps.as_f64()
                            );
                        }
                        offset.as_micros_with_override(v.fps())
                    })
                    .unwrap_or(0.);

                (micros_per_tick, offset_in_micros)
            }
            Timing::TicksPerQuarterNote(tpqn) => {
                // µs_per_tick = tempo_meta / TPQN
                // micro_seconds/quarternote * quarternote_per_tick (1/ticks per qn)
                let micros_per_tick =
                    micros_per_quarter_note as f64 / tpqn.ticks_per_quarter_note() as f64;

                let offset_in_micros = track
                    .info()
                    .smpte_offset
                    .as_ref()
                    .map(|offset| offset.as_micros())
                    .unwrap_or(0.);

                (micros_per_tick, offset_in_micros)
            }
        };

        Self {
            micros_per_tick,
            offset_in_micros,
            event: track.events.into_iter(),
        }
    }
}

impl<'a> Iterator for CurrentTrack<'a> {
    type Item = Timed<LiveEvent<'a>>;
    fn next(&mut self) -> Option<Self::Item> {
        let event = self.event.next()?;
        let tick = event.accumulated_ticks();
        let micros = self.micros_per_tick * tick as f64 + self.offset_in_micros;
        Some(Timed::new(micros as u64, event.event))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.event.size_hint()
    }
}

#[cfg(test)]
fn channel_from_num(num: u8) -> Channel {
    match num {
        0 => Channel::One,
        1 => Channel::Two,
        2 => Channel::Three,
        3 => Channel::Four,
        4 => Channel::Five,
        5 => Channel::Six,
        6 => Channel::Seven,
        7 => Channel::Eight,
        8 => Channel::Nine,
        9 => Channel::Ten,
        10 => Channel::Eleven,
        11 => Channel::Twelve,
        12 => Channel::Thirteen,
        13 => Channel::Fourteen,
        14 => Channel::Fifteen,
        15 => Channel::Sixteen,
        _ => Channel::One,
    }
}

#[cfg(test)]
fn note_from_midi_number(num: u8) -> Note {
    Note::from_databyte(num).unwrap()
}

#[cfg(test)]
fn note_on_event(delta_ticks: u32, note: u8, velocity: u8, channel: u8) -> TrackEvent<'static> {
    TrackEvent::new(
        delta_ticks,
        TrackMessage::ChannelVoice(ChannelVoiceMessage::new(
            channel_from_num(channel),
            VoiceEvent::NoteOn {
                note: note_from_midi_number(note),
                velocity: Velocity::new(velocity).unwrap(),
            },
        )),
    )
}

#[cfg(test)]
fn note_off_event(delta_ticks: u32, note: u8, channel: u8) -> TrackEvent<'static> {
    TrackEvent::new(
        delta_ticks,
        TrackMessage::ChannelVoice(ChannelVoiceMessage::new(
            channel_from_num(channel),
            VoiceEvent::NoteOff {
                note: note_from_midi_number(note),
                velocity: Velocity::new(0).unwrap(),
            },
        )),
    )
}

#[cfg(test)]
fn tempo_event(delta_ticks: u32, micros_per_quarter: u32) -> TrackEvent<'static> {
    let bytes = [
        (micros_per_quarter >> 16) as u8,
        (micros_per_quarter >> 8) as u8,
        micros_per_quarter as u8,
    ];
    TrackEvent::new(
        delta_ticks,
        TrackMessage::Meta(MetaMessage::Tempo(Tempo::new_from_bytes(&bytes))),
    )
}

#[test]
fn test_empty_file_returns_none_iterator() {
    let header = Header::new(Timing::TicksPerQuarterNote(TicksPerQuarterNote {
        inner: [0x01, 0xE0],
    }));
    let format = Format::Simultaneous(alloc::vec![]);
    let file = ParsedMidiFile { header, format };

    let mut iter = file.into_events();
    assert_eq!(iter.next(), None);
}

#[test]
fn test_single_track_single_event() {
    let header = Header::new(Timing::TicksPerQuarterNote(TicksPerQuarterNote {
        inner: [0x01, 0xE0],
    }));
    let events = alloc::vec![tempo_event(0, 500_000), note_on_event(0, 60, 100, 0),];
    let track = Track::new(events);
    let format = Format::SingleMultiChannel(track);
    let file = ParsedMidiFile { header, format };

    let mut iter = file.into_events();
    let event = iter.next().unwrap();

    assert_eq!(event.timestamp, 0);
    assert!(matches!(
        event.event,
        LiveEvent::ChannelVoice(ChannelVoiceMessage {
            event: VoiceEvent::NoteOn { .. },
            ..
        })
    ));

    assert_eq!(iter.next(), None);
}

#[test]
fn test_single_track_multiple_events_with_delta_time() {
    let header = Header::new(Timing::TicksPerQuarterNote(TicksPerQuarterNote {
        inner: [0x01, 0xE0],
    }));
    let events = alloc::vec![
        tempo_event(0, 500_000),
        note_on_event(0, 60, 100, 0),
        note_off_event(480, 60, 0),
        note_on_event(240, 62, 80, 0),
    ];
    let track = Track::new(events);
    let format = Format::SingleMultiChannel(track);
    let file = ParsedMidiFile { header, format };

    let events: alloc::vec::Vec<_> = file.into_events().collect();
    assert_eq!(events.len(), 3);

    assert_eq!(events[0].timestamp, 0);

    assert_eq!(events[1].timestamp, 500_000);

    assert_eq!(events[2].timestamp, 750_000);
}

#[test]
fn test_simultaneous_format_multiple_tracks() {
    let header = Header::new(Timing::TicksPerQuarterNote(TicksPerQuarterNote {
        inner: [0x01, 0xE0],
    }));

    let track1_events = alloc::vec![
        tempo_event(0, 500_000),
        note_on_event(0, 60, 100, 0),
        note_off_event(480, 60, 0),
    ];
    let track1 = Track::new(track1_events);

    let track2_events = alloc::vec![note_on_event(240, 36, 80, 1), note_off_event(480, 36, 1),];
    let track2 = Track::new(track2_events);

    let format = Format::Simultaneous(alloc::vec![track1, track2]);
    let file = ParsedMidiFile { header, format };

    let events: alloc::vec::Vec<_> = file.into_events().collect();
    assert_eq!(events.len(), 4);

    assert_eq!(events[0].timestamp, 0);
    assert_eq!(events[1].timestamp, 500_000);
    assert_eq!(events[2].timestamp, 250_000);
    assert_eq!(events[3].timestamp, 750_000);
}

#[test]
fn test_sequentially_independent_format() {
    let header = Header::new(Timing::TicksPerQuarterNote(TicksPerQuarterNote {
        inner: [0x03, 0xC0],
    }));

    let track1_events = alloc::vec![
        tempo_event(0, 1_000_000),
        note_on_event(0, 60, 100, 0),
        note_off_event(960, 60, 0),
    ];
    let track1 = Track::new(track1_events);

    let track2_events = alloc::vec![
        tempo_event(0, 500_000),
        note_on_event(0, 48, 90, 1),
        note_off_event(480, 48, 1),
    ];
    let track2 = Track::new(track2_events);

    let format = Format::SequentiallyIndependent(alloc::vec![track1, track2]);
    let file = ParsedMidiFile { header, format };

    let events: alloc::vec::Vec<_> = file.into_events().collect();
    assert_eq!(events.len(), 4);

    assert_eq!(events[0].timestamp, 0);
    assert_eq!(events[1].timestamp, 1_000_000);
    assert_eq!(events[2].timestamp, 0);
    assert_eq!(events[3].timestamp, 250_000);
}

#[test]
fn test_smpte_timing() {
    let smpte = SmpteHeader {
        fps: SmpteFps::Thirty,
        ticks_per_frame: DataByte::new(40).unwrap(),
    };
    let header = Header::new(Timing::Smpte(smpte));

    let events = alloc::vec![
        note_on_event(0, 60, 100, 0),
        note_off_event(1200, 60, 0),
        note_on_event(600, 62, 80, 0),
    ];
    let track = Track::new(events);
    let format = Format::SingleMultiChannel(track);
    let file = ParsedMidiFile { header, format };

    let events: alloc::vec::Vec<_> = file.into_events().collect();
    assert_eq!(events.len(), 3);

    assert_eq!(events[0].timestamp, 0);
    assert_eq!(events[1].timestamp, 1_000_000);
    assert_eq!(events[2].timestamp, 1_500_000);
}

#[test]
fn test_mixed_event_types() {
    let header = Header::new(Timing::TicksPerQuarterNote(TicksPerQuarterNote {
        inner: [0x01, 0xE0],
    }));

    let events = alloc::vec![
        tempo_event(0, 500_000),
        note_on_event(0, 60, 100, 0),
        TrackEvent::new(
            240,
            TrackMessage::ChannelVoice(ChannelVoiceMessage::new(
                Channel::One,
                VoiceEvent::ControlChange(Controller::other(
                    DataByte::new(64).unwrap(),
                    DataByte::new(127).unwrap(),
                )),
            )),
        ),
        TrackEvent::new(
            240,
            TrackMessage::ChannelVoice(ChannelVoiceMessage::new(
                Channel::One,
                VoiceEvent::ProgramChange {
                    program: Program::new(1).unwrap(),
                },
            )),
        ),
        note_off_event(480, 60, 0),
    ];

    let track = Track::new(events);
    let format = Format::SingleMultiChannel(track);
    let file = ParsedMidiFile { header, format };

    let events: alloc::vec::Vec<_> = file.into_events().collect();
    assert_eq!(events.len(), 4);

    assert!(matches!(
        &events[0].event,
        LiveEvent::ChannelVoice(ChannelVoiceMessage {
            event: VoiceEvent::NoteOn { .. },
            ..
        })
    ));
    assert!(matches!(
        &events[1].event,
        LiveEvent::ChannelVoice(ChannelVoiceMessage {
            event: VoiceEvent::ControlChange(_),
            ..
        })
    ));
    assert!(matches!(
        &events[2].event,
        LiveEvent::ChannelVoice(ChannelVoiceMessage {
            event: VoiceEvent::ProgramChange { .. },
            ..
        })
    ));
    assert!(matches!(
        &events[3].event,
        LiveEvent::ChannelVoice(ChannelVoiceMessage {
            event: VoiceEvent::NoteOff { .. },
            ..
        })
    ));

    assert_eq!(events[0].timestamp, 0);
    assert_eq!(events[1].timestamp, 250_000);
    assert_eq!(events[2].timestamp, 500_000);
    assert_eq!(events[3].timestamp, 1_000_000);
}

#[test]
fn test_system_exclusive_events() {
    let header = Header::new(Timing::TicksPerQuarterNote(TicksPerQuarterNote {
        inner: [0x01, 0xE0],
    }));

    let sysex_data = alloc::vec![0xF0, 0x43, 0x12, 0x00, 0xF7];
    let events = alloc::vec![
        tempo_event(0, 500_000),
        note_on_event(0, 60, 100, 0),
        TrackEvent::new(
            480,
            TrackMessage::SystemExclusive(SystemExclusiveMessage::new(sysex_data.as_slice())),
        ),
        note_off_event(480, 60, 0),
    ];

    let track = Track::new(events);
    let format = Format::SingleMultiChannel(track);
    let file = ParsedMidiFile { header, format };

    let events: alloc::vec::Vec<_> = file.into_events().collect();
    assert_eq!(events.len(), 3);

    assert!(matches!(
        &events[1].event,
        LiveEvent::SysCommon(SystemCommonMessage::SystemExclusive(_))
    ));
    assert_eq!(events[1].timestamp, 500_000);
}

#[test]
fn test_file_tempo_override_in_simultaneous_format() {
    let header = Header::new(Timing::TicksPerQuarterNote(TicksPerQuarterNote {
        inner: [0x01, 0xE0],
    }));

    let track1_events = alloc::vec![tempo_event(0, 600_000), note_on_event(0, 60, 100, 0),];
    let track1 = Track::new(track1_events);

    let track2_events = alloc::vec![tempo_event(0, 400_000), note_on_event(480, 48, 90, 1),];
    let track2 = Track::new(track2_events);

    let format = Format::Simultaneous(alloc::vec![track1, track2]);
    let file = ParsedMidiFile { header, format };

    let events: alloc::vec::Vec<_> = file.into_events().collect();
    assert_eq!(events.len(), 2);

    assert_eq!(events[0].timestamp, 0);
    assert_eq!(events[1].timestamp, 600_000);
}

#[test]
fn test_empty_track_handling() {
    let header = Header::new(Timing::TicksPerQuarterNote(TicksPerQuarterNote {
        inner: [0x01, 0xE0],
    }));

    let track1_events = alloc::vec![
        tempo_event(0, 500_000),
        note_on_event(0, 60, 100, 0),
        note_off_event(480, 60, 0),
    ];
    let track1 = Track::new(track1_events);

    let track2 = Track::new(alloc::vec![]);

    let track3_events = alloc::vec![note_on_event(0, 48, 90, 2)];
    let track3 = Track::new(track3_events);

    let format = Format::Simultaneous(alloc::vec![track1, track2, track3]);
    let file = ParsedMidiFile { header, format };

    let events: alloc::vec::Vec<_> = file.into_events().collect();

    assert_eq!(events.len(), 3);

    if let LiveEvent::ChannelVoice(msg) = &events[0].event {
        assert_eq!(msg.channel(), Channel::One);
    }
    if let LiveEvent::ChannelVoice(msg) = &events[2].event {
        assert_eq!(msg.channel(), Channel::Three);
    }
}
