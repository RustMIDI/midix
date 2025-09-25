use midix::{Dynamic, Key, Octave, events::LiveEvent, prelude::*};

#[test]
fn test_parse() {
    let parsed = ParsedMidiFile::parse(include_bytes!("./simple.mid")).unwrap();

    assert_eq!(parsed.tracks().len(), 1);

    let track = parsed.tracks()[0];

    let mut events = track.events().iter().skip(3);

    use Channel::*;
    note_on(events.next().unwrap(), 0, Three, Key::C, 3, Dynamic::ff());
    note_on(events.next().unwrap(), 0, Three, Key::C, 4, Dynamic::ff());
    note_on(events.next().unwrap(), 96, Two, Key::G, 4, Dynamic::mf());
    note_on(events.next().unwrap(), 192, One, Key::E, 5, Dynamic::p());
    note_off(events.next().unwrap(), 384, Three, Key::C, 3);
    note_off(events.next().unwrap(), 384, Three, Key::C, 4);
    note_off(events.next().unwrap(), 384, Two, Key::G, 4);
    note_off(events.next().unwrap(), 384, One, Key::E, 5);
}
fn note_on(
    e: &Ticked<LiveEvent<'_>>,
    accumulated_ticks: u32,
    channel: Channel,
    note: Key,
    octave: i8,
    dynamic: Dynamic,
) {
    assert_eq!(e.accumulated_ticks(), accumulated_ticks);
    let LiveEvent::ChannelVoice(cv) = e.event() else {
        panic!();
    };

    assert_eq!(cv.channel(), channel);
    let VoiceEvent::NoteOn {
        note: key,
        velocity,
    } = cv.event()
    else {
        panic!();
    };
    assert_eq!(key.key(), note);
    assert_eq!(key.octave(), Octave::new(octave));
    assert_eq!(velocity.dynamic(), dynamic);
}

fn note_off(
    e: &Ticked<LiveEvent<'_>>,
    accumulated_ticks: u32,
    channel: Channel,
    note: Key,
    octave: i8,
) {
    assert_eq!(e.accumulated_ticks(), accumulated_ticks);
    let LiveEvent::ChannelVoice(cv) = e.event() else {
        panic!();
    };

    assert_eq!(cv.channel(), channel);
    match cv.event() {
        VoiceEvent::NoteOn {
            note: key,
            velocity,
        } => {
            assert_eq!(velocity.byte(), 0);
            assert_eq!(key.key(), note);
            assert_eq!(key.octave(), Octave::new(octave));
        }
        VoiceEvent::NoteOff {
            note: key,
            velocity: _,
        } => {
            assert_eq!(key.key(), note);
            assert_eq!(key.octave(), Octave::new(octave));
        }
        _ => panic!(),
    }
}
