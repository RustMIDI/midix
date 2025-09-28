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
                //todo
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
