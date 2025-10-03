use midix::{file::builder::event::FileEvent, prelude::*};

/// Helper function to create a minimal MIDI file with SMPTE offset
/// Returns the complete MIDI file as a byte vector
fn create_midi_with_smpte_offset(
    fps: SmpteFps,
    hour: u8,
    minute: u8,
    second: u8,
    frame: u8,
    subframe: u8,
) -> Vec<u8> {
    let mut bytes = Vec::new();

    // MIDI Header
    bytes.extend_from_slice(b"MThd"); // Header chunk type
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x06]); // Header length (6 bytes)
    bytes.extend_from_slice(&[0x00, 0x00]); // Format 0 (single track)
    bytes.extend_from_slice(&[0x00, 0x01]); // Number of tracks (1)

    // Use SMPTE timing instead of ticks per quarter note
    // High bit set indicates SMPTE timing
    let fps_byte = match fps {
        SmpteFps::TwentyFour => 0xE8, // -24 in two's complement
        SmpteFps::TwentyFive => 0xE7, // -25 in two's complement
        SmpteFps::TwentyNine => 0xE3, // -29 in two's complement
        SmpteFps::Thirty => 0xE2,     // -30 in two's complement
    };
    bytes.push(fps_byte);
    bytes.push(40); // 40 ticks per frame

    // Track Header
    bytes.extend_from_slice(b"MTrk"); // Track chunk type

    // Calculate track length (we'll update this later)
    let track_length_pos = bytes.len();
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]); // Placeholder for length

    let track_start = bytes.len();

    // SMPTE Offset Meta Event
    bytes.push(0x00); // Delta time
    bytes.push(0xFF); // Meta event
    bytes.push(0x54); // SMPTE Offset type
    bytes.push(0x05); // Length (5 bytes)

    // SMPTE data
    let frame_type_bits = match fps {
        SmpteFps::TwentyFour => 0b00,
        SmpteFps::TwentyFive => 0b01,
        SmpteFps::TwentyNine => 0b10,
        SmpteFps::Thirty => 0b11,
    };
    bytes.push((frame_type_bits << 5) | (hour & 0x1F)); // Frame type + hours
    bytes.push(minute);
    bytes.push(second);
    bytes.push(frame);
    bytes.push(subframe);

    // Add a simple note to make it a valid track
    bytes.push(0x00); // Delta time
    bytes.push(0x90); // Note On, channel 0
    bytes.push(0x3C); // Middle C (60)
    bytes.push(0x64); // Velocity 100

    bytes.push(0x60); // Delta time (96 ticks)
    bytes.push(0x80); // Note Off, channel 0
    bytes.push(0x3C); // Middle C
    bytes.push(0x40); // Release velocity 64

    // End of Track
    bytes.push(0x00); // Delta time
    bytes.push(0xFF); // Meta event
    bytes.push(0x2F); // End of track
    bytes.push(0x00); // Length 0

    // Update track length
    let track_length = bytes.len() - track_start;
    bytes[track_length_pos..track_length_pos + 4]
        .copy_from_slice(&(track_length as u32).to_be_bytes());

    bytes
}

#[test]
fn test_smpte_offset_24fps() {
    let midi_data = create_midi_with_smpte_offset(
        SmpteFps::TwentyFour,
        12, // hour (noon)
        30, // minute
        15, // second
        18, // frame
        50, // subframe
    );

    let mut reader = Reader::from_byte_slice(&midi_data);

    // Read header
    let Ok(FileEvent::Header(header)) = reader.read_event() else {
        panic!("Failed to read header");
    };

    // Verify SMPTE timing
    match header.timing() {
        Timing::Smpte(smpte) => {
            assert_eq!(smpte.fps(), SmpteFps::TwentyFour);
            assert_eq!(smpte.ticks_per_frame(), 40);
        }
        _ => panic!("Expected SMPTE timing"),
    }

    // Read track header
    let Ok(FileEvent::Track(track)) = reader.read_event() else {
        panic!("Failed to read track header");
    };
    assert!(track.len() > 0);

    // Read SMPTE offset event
    let Ok(FileEvent::TrackEvent(event)) = reader.read_event() else {
        panic!("Failed to read track event");
    };

    match event.event() {
        TrackMessage::Meta(MetaMessage::SmpteOffset(offset)) => {
            assert_eq!(offset.fps, SmpteFps::TwentyFour);
            assert_eq!(offset.hour, 12);
            assert_eq!(offset.minute, 30);
            assert_eq!(offset.second, 15);
            assert_eq!(offset.frame, 18);
            assert_eq!(offset.subframe, 50);

            // Verify microsecond calculation
            let expected_micros = (12 * 3600 + 30 * 60 + 15) as f64 * 1_000_000.0
                + (18.0 / 24.0) * 1_000_000.0
                + (50.0 / 100.0 / 24.0) * 1_000_000.0;
            assert!((offset.as_micros() - expected_micros).abs() < 0.01);
        }
        _ => panic!("Expected SMPTE offset meta event"),
    }
}

#[test]
fn test_smpte_offset_25fps_pal() {
    let midi_data = create_midi_with_smpte_offset(
        SmpteFps::TwentyFive,
        0,  // midnight
        0,  // minute
        1,  // second
        12, // frame (middle of second)
        75, // subframe
    );

    let mut reader = Reader::from_byte_slice(&midi_data);

    // Skip to SMPTE offset event
    reader.read_event().unwrap(); // Header
    reader.read_event().unwrap(); // Track

    let Ok(FileEvent::TrackEvent(event)) = reader.read_event() else {
        panic!("Failed to read track event");
    };

    match event.event() {
        TrackMessage::Meta(MetaMessage::SmpteOffset(offset)) => {
            assert_eq!(offset.fps, SmpteFps::TwentyFive);
            assert_eq!(offset.hour, 0);
            assert_eq!(offset.minute, 0);
            assert_eq!(offset.second, 1);
            assert_eq!(offset.frame, 12);
            assert_eq!(offset.subframe, 75);
        }
        _ => panic!("Expected SMPTE offset meta event"),
    }
}

#[test]
fn test_smpte_offset_29_97_drop_frame() {
    let midi_data = create_midi_with_smpte_offset(
        SmpteFps::TwentyNine,
        23, // 11 PM
        59, // 59 minutes
        59, // 59 seconds
        28, // frame 28 (out of 29)
        99, // maximum subframe
    );

    let mut reader = Reader::from_byte_slice(&midi_data);

    // Skip to SMPTE offset event
    reader.read_event().unwrap(); // Header
    reader.read_event().unwrap(); // Track

    let Ok(FileEvent::TrackEvent(event)) = reader.read_event() else {
        panic!("Failed to read track event");
    };

    match event.event() {
        TrackMessage::Meta(MetaMessage::SmpteOffset(offset)) => {
            assert_eq!(offset.fps, SmpteFps::TwentyNine);
            assert_eq!(offset.hour, 23);
            assert_eq!(offset.minute, 59);
            assert_eq!(offset.second, 59);
            assert_eq!(offset.frame, 28);
            assert_eq!(offset.subframe, 99);

            // This should be just before midnight
            let micros = offset.as_micros();
            let expected = 86_399_000_000.0 + // 23:59:59 in microseconds
                          (28.0 * 1_000_000.0 / 29.97) + // frames
                          (99.0 * 10_000.0 / 29.97); // subframes
            assert!((micros - expected).abs() < 1.0);
        }
        _ => panic!("Expected SMPTE offset meta event"),
    }
}

#[test]
fn test_smpte_offset_30fps() {
    let midi_data = create_midi_with_smpte_offset(
        SmpteFps::Thirty,
        1,  // 1 AM
        23, // 23 minutes
        45, // 45 seconds
        15, // frame 15 (middle frame)
        0,  // no subframe
    );

    let mut reader = Reader::from_byte_slice(&midi_data);

    // Skip to SMPTE offset event
    reader.read_event().unwrap(); // Header
    reader.read_event().unwrap(); // Track

    let Ok(FileEvent::TrackEvent(event)) = reader.read_event() else {
        panic!("Failed to read track event");
    };

    match event.event() {
        TrackMessage::Meta(MetaMessage::SmpteOffset(offset)) => {
            assert_eq!(offset.fps, SmpteFps::Thirty);
            assert_eq!(offset.hour, 1);
            assert_eq!(offset.minute, 23);
            assert_eq!(offset.second, 45);
            assert_eq!(offset.frame, 15);
            assert_eq!(offset.subframe, 0);
        }
        _ => panic!("Expected SMPTE offset meta event"),
    }
}

#[test]
fn test_smpte_offset_with_override_fps() {
    // Create a file with 24fps SMPTE timing
    let midi_data = create_midi_with_smpte_offset(SmpteFps::TwentyFour, 10, 20, 30, 12, 50);

    let mut reader = Reader::from_byte_slice(&midi_data);

    // Read header to get file timing
    let Ok(FileEvent::Header(header)) = reader.read_event() else {
        panic!("Failed to read header");
    };

    let file_fps = match header.timing() {
        Timing::Smpte(smpte) => smpte.fps(),
        _ => panic!("Expected SMPTE timing"),
    };

    // Skip track header
    reader.read_event().unwrap();

    // Read SMPTE offset
    let Ok(FileEvent::TrackEvent(event)) = reader.read_event() else {
        panic!("Failed to read track event");
    };

    match event.event() {
        TrackMessage::Meta(MetaMessage::SmpteOffset(offset)) => {
            // Calculate with original fps
            let micros_original = offset.as_micros();

            // Calculate with override fps (using file's fps)
            let micros_override = offset.as_micros_with_override(file_fps);

            // They should be equal when the fps matches
            assert!((micros_original - micros_override).abs() < 0.01);

            // But different with a different fps
            let micros_different = offset.as_micros_with_override(SmpteFps::Thirty);
            assert!((micros_original - micros_different).abs() > 1.0);
        }
        _ => panic!("Expected SMPTE offset meta event"),
    }
}

#[test]
fn test_multiple_tracks_with_different_offsets() {
    let mut bytes = Vec::new();

    // MIDI Header (Format 1 - multiple simultaneous tracks)
    bytes.extend_from_slice(b"MThd");
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x06]);
    bytes.extend_from_slice(&[0x00, 0x01]); // Format 1
    bytes.extend_from_slice(&[0x00, 0x02]); // 2 tracks
    bytes.push(0xE7); // 25 fps SMPTE
    bytes.push(40); // 40 ticks per frame

    // Track 1 with offset at 00:00:10:00
    bytes.extend_from_slice(b"MTrk");
    let track1_length_pos = bytes.len();
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    let track1_start = bytes.len();

    // SMPTE Offset for track 1
    bytes.extend_from_slice(&[
        0x00, 0xFF, 0x54, 0x05, // Delta time, Meta, SMPTE Offset, length
        0x20, // 25fps (01) + 0 hours
        0x00, // 0 minutes
        0x0A, // 10 seconds
        0x00, // 0 frames
        0x00, // 0 subframes
    ]);

    // End of track 1
    bytes.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    let track1_length = bytes.len() - track1_start;
    bytes[track1_length_pos..track1_length_pos + 4]
        .copy_from_slice(&(track1_length as u32).to_be_bytes());

    // Track 2 with offset at 00:01:00:00
    bytes.extend_from_slice(b"MTrk");
    let track2_length_pos = bytes.len();
    bytes.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
    let track2_start = bytes.len();

    // SMPTE Offset for track 2
    bytes.extend_from_slice(&[
        0x00, 0xFF, 0x54, 0x05, // Delta time, Meta, SMPTE Offset, length
        0x20, // 25fps (01) + 0 hours
        0x01, // 1 minute
        0x00, // 0 seconds
        0x00, // 0 frames
        0x00, // 0 subframes
    ]);

    // End of track 2
    bytes.extend_from_slice(&[0x00, 0xFF, 0x2F, 0x00]);

    let track2_length = bytes.len() - track2_start;
    bytes[track2_length_pos..track2_length_pos + 4]
        .copy_from_slice(&(track2_length as u32).to_be_bytes());

    // Parse the file
    let mut reader = Reader::from_byte_slice(&bytes);
    let mut offsets = Vec::new();

    while let Ok(event) = reader.read_event() {
        if let FileEvent::TrackEvent(track_event) = event
            && let TrackMessage::Meta(MetaMessage::SmpteOffset(offset)) = track_event.event()
        {
            offsets.push(offset.clone());
        }
    }

    assert_eq!(offsets.len(), 2);

    // Track 1 offset: 10 seconds
    assert_eq!(offsets[0].hour, 0);
    assert_eq!(offsets[0].minute, 0);
    assert_eq!(offsets[0].second, 10);

    // Track 2 offset: 1 minute
    assert_eq!(offsets[1].hour, 0);
    assert_eq!(offsets[1].minute, 1);
    assert_eq!(offsets[1].second, 0);

    // Verify the time difference is 50 seconds
    let diff = offsets[1].as_micros() - offsets[0].as_micros();
    assert!((diff - 50_000_000.0).abs() < 1.0);
}

#[test]
fn test_smpte_edge_cases() {
    // Test maximum valid values
    let midi_data = create_midi_with_smpte_offset(
        SmpteFps::TwentyFour,
        23, // max hour
        59, // max minute
        59, // max second
        23, // max frame for 24fps
        99, // max subframe
    );

    let mut reader = Reader::from_byte_slice(&midi_data);
    reader.read_event().unwrap(); // Header
    reader.read_event().unwrap(); // Track

    let Ok(FileEvent::TrackEvent(event)) = reader.read_event() else {
        panic!("Failed to read track event");
    };

    match event.event() {
        TrackMessage::Meta(MetaMessage::SmpteOffset(offset)) => {
            assert_eq!(offset.hour, 23);
            assert_eq!(offset.minute, 59);
            assert_eq!(offset.second, 59);
            assert_eq!(offset.frame, 23);
            assert_eq!(offset.subframe, 99);
        }
        _ => panic!("Expected SMPTE offset meta event"),
    }
}

#[test]
fn test_smpte_offset_precision() {
    // Test that subframe precision is maintained correctly
    let test_cases = vec![
        (SmpteFps::TwentyFour, 0, 0, 0, 0, 1), // 1/100th of 1/24th second
        (SmpteFps::TwentyFive, 0, 0, 0, 0, 50), // Half a subframe
        (SmpteFps::TwentyNine, 0, 0, 0, 1, 0), // Exactly one frame
        (SmpteFps::Thirty, 0, 0, 1, 0, 0),     // Exactly one second
    ];

    for (fps, hour, minute, second, frame, subframe) in test_cases {
        let midi_data = create_midi_with_smpte_offset(fps, hour, minute, second, frame, subframe);
        let mut reader = Reader::from_byte_slice(&midi_data);

        reader.read_event().unwrap(); // Header
        reader.read_event().unwrap(); // Track

        let Ok(FileEvent::TrackEvent(event)) = reader.read_event() else {
            panic!("Failed to read track event");
        };

        match event.event() {
            TrackMessage::Meta(MetaMessage::SmpteOffset(offset)) => {
                let micros = offset.as_micros();

                // Verify the calculation is precise
                match fps {
                    SmpteFps::TwentyFour => {
                        // 1 subframe at 24fps = 1/100 * 1/24 second
                        let expected = 1_000_000.0 / 24.0 / 100.0;
                        assert!((micros - expected).abs() < 0.001);
                    }
                    SmpteFps::Thirty => {
                        // Exactly 1 second
                        assert!((micros - 1_000_000.0).abs() < 0.001);
                    }
                    _ => {}
                }
            }
            _ => panic!("Expected SMPTE offset meta event"),
        }
    }
}
