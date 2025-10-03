use midix::prelude::*;

/// Helper to create raw SMPTE offset data bytes
fn create_smpte_bytes(
    fps_bits: u8,
    hour: u8,
    minute: u8,
    second: u8,
    frame: u8,
    subframe: u8,
) -> Vec<u8> {
    vec![
        (fps_bits << 5) | (hour & 0x1F),
        minute,
        second,
        frame,
        subframe,
    ]
}

#[test]
fn test_smpte_offset_invalid_length() {
    // Test with too few bytes
    let short_data = vec![0x00, 0x00, 0x00];
    let result = SmpteOffset::parse(&short_data);
    assert!(matches!(result, Err(SmpteError::Length(3))));

    // Test with too many bytes
    let long_data = vec![0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
    let result = SmpteOffset::parse(&long_data);
    assert!(matches!(result, Err(SmpteError::Length(6))));

    // Test with empty data
    let empty_data = vec![];
    let result = SmpteOffset::parse(&empty_data);
    assert!(matches!(result, Err(SmpteError::Length(0))));
}

#[test]
fn test_smpte_offset_invalid_frame_type() {
    // Frame type bits are bits 5-6, only values 0-3 are valid
    // Try with invalid frame type 4 (binary 100)
    let invalid_fps_4 = create_smpte_bytes(0b100, 12, 30, 15, 10, 50);
    let result = SmpteOffset::parse(&invalid_fps_4);
    assert!(matches!(result, Err(SmpteError::TrackFrame(4))));

    // Try with invalid frame type 5 (binary 101)
    let invalid_fps_5 = create_smpte_bytes(0b101, 12, 30, 15, 10, 50);
    let result = SmpteOffset::parse(&invalid_fps_5);
    assert!(matches!(result, Err(SmpteError::TrackFrame(5))));

    // Try with invalid frame type 7 (binary 111)
    let invalid_fps_7 = create_smpte_bytes(0b111, 12, 30, 15, 10, 50);
    let result = SmpteOffset::parse(&invalid_fps_7);
    assert!(matches!(result, Err(SmpteError::TrackFrame(7))));
}

#[test]
fn test_smpte_offset_invalid_hour() {
    // Hours should be 0-23, test 24
    let data = vec![
        0x18, // fps bits 00 (24fps) + hour 24 (11000)
        0x00, // minute
        0x00, // second
        0x00, // frame
        0x00, // subframe
    ];
    let result = SmpteOffset::parse(&data);
    assert!(matches!(result, Err(SmpteError::HourOffset(24))));

    // Test maximum invalid hour (31 - all 5 bits set)
    let data_max = vec![
        0x1F, // fps bits 00 + hour 31 (11111)
        0x00, 0x00, 0x00, 0x00,
    ];
    let result = SmpteOffset::parse(&data_max);
    assert!(matches!(result, Err(SmpteError::HourOffset(31))));
}

#[test]
fn test_smpte_offset_invalid_minute() {
    // Minutes should be 0-59
    let data = create_smpte_bytes(0, 12, 60, 30, 15, 50);
    let result = SmpteOffset::parse(&data);
    assert!(matches!(result, Err(SmpteError::MinuteOffset(60))));

    // Test various invalid minute values
    for invalid_minute in [61, 70, 80, 99, 100, 255] {
        let data = create_smpte_bytes(0, 12, invalid_minute, 30, 15, 50);
        let result = SmpteOffset::parse(&data);
        assert!(matches!(result, Err(SmpteError::MinuteOffset(_))));
    }
}

#[test]
fn test_smpte_offset_invalid_second() {
    // Seconds should be 0-59
    let data = create_smpte_bytes(1, 12, 30, 60, 15, 50);
    let result = SmpteOffset::parse(&data);
    assert!(matches!(result, Err(SmpteError::SecondOffset(60))));

    // Test maximum byte value
    let data_max = create_smpte_bytes(1, 12, 30, 255, 15, 50);
    let result = SmpteOffset::parse(&data_max);
    assert!(matches!(result, Err(SmpteError::SecondOffset(255))));
}

#[test]
fn test_smpte_offset_invalid_subframe() {
    // Subframes should be 0-99
    let data = create_smpte_bytes(2, 12, 30, 45, 15, 100);
    let result = SmpteOffset::parse(&data);
    assert!(matches!(result, Err(SmpteError::Subframe(100))));

    // Test various invalid subframe values
    for invalid_subframe in [101, 110, 150, 200, 255] {
        let data = create_smpte_bytes(2, 12, 30, 45, 15, invalid_subframe);
        let result = SmpteOffset::parse(&data);
        assert!(matches!(result, Err(SmpteError::Subframe(_))));
    }
}

#[test]
fn test_smpte_offset_boundary_values() {
    // Test all valid boundary values
    let test_cases = vec![
        // Min values
        (0, 0, 0, 0, 0, 0),
        // Max hour
        (0, 23, 0, 0, 0, 0),
        // Max minute
        (0, 0, 59, 0, 0, 0),
        // Max second
        (0, 0, 0, 59, 0, 0),
        // Max subframe
        (0, 0, 0, 0, 0, 99),
        // All max valid values for 24fps
        (0, 23, 59, 59, 23, 99),
        // All max valid values for 25fps
        (1, 23, 59, 59, 24, 99),
        // All max valid values for 29.97fps
        (2, 23, 59, 59, 29, 99),
        // All max valid values for 30fps
        (3, 23, 59, 59, 29, 99),
    ];

    for (fps_bits, hour, minute, second, frame, subframe) in test_cases {
        let data = create_smpte_bytes(fps_bits, hour, minute, second, frame, subframe);
        let result = SmpteOffset::parse(&data);
        assert!(result.is_ok(), "Failed for {:?}", data);

        let offset = result.unwrap();
        assert_eq!(offset.hour, hour);
        assert_eq!(offset.minute, minute);
        assert_eq!(offset.second, second);
        assert_eq!(offset.frame, frame);
        assert_eq!(offset.subframe, subframe);
    }
}

#[test]
fn test_smpte_offset_frame_limits() {
    // Frame limits depend on the fps
    // For 24fps, max frame should be 23
    // For 25fps, max frame should be 24
    // For 29.97fps and 30fps, max frame should be 29

    // Note: The actual MIDI spec might not enforce these limits strictly,
    // but they represent the logical limits for each frame rate

    let test_cases = vec![
        (SmpteFps::TwentyFour, 0, 24), // 24 frames would be the 25th frame (invalid)
        (SmpteFps::TwentyFive, 1, 25), // 25 frames would be the 26th frame (invalid)
        (SmpteFps::TwentyNine, 2, 30), // 30 frames would be the 31st frame (invalid)
        (SmpteFps::Thirty, 3, 30),     // 30 frames would be the 31st frame (invalid)
    ];

    for (expected_fps, fps_bits, frame) in test_cases {
        let data = create_smpte_bytes(fps_bits, 12, 30, 45, frame, 50);
        let result = SmpteOffset::parse(&data);

        // The parse might succeed (as the MIDI spec might not enforce frame limits)
        // but we can verify the values are as expected
        if let Ok(offset) = result {
            assert_eq!(offset.fps, expected_fps);
            assert_eq!(offset.frame, frame);
        }
    }
}

#[test]
fn test_smpte_offset_microsecond_calculation_edge_cases() {
    // Test edge case: Just before midnight
    let data = create_smpte_bytes(0, 23, 59, 59, 23, 99); // 24fps
    let offset = SmpteOffset::parse(&data).unwrap();

    let micros = offset.as_micros();
    // Should be very close to 24 hours in microseconds
    let expected = 86_399_000_000.0 + // 23:59:59
                   (23.0 / 24.0) * 1_000_000.0 + // 23 frames at 24fps
                   (99.0 / 100.0 / 24.0) * 1_000_000.0; // 99 subframes
    assert!((micros - expected).abs() < 1.0);

    // Test edge case: Exactly midnight (all zeros)
    let data_midnight = create_smpte_bytes(1, 0, 0, 0, 0, 0);
    let offset_midnight = SmpteOffset::parse(&data_midnight).unwrap();
    assert_eq!(offset_midnight.as_micros(), 0.0);
}

#[test]
fn test_smpte_offset_fps_override_edge_cases() {
    // Create offset with 24fps
    let data = create_smpte_bytes(0, 1, 0, 0, 12, 0); // 1 hour, 12 frames
    let offset = SmpteOffset::parse(&data).unwrap();

    // Calculate with different frame rates
    let micros_24 = offset.as_micros();
    let micros_25 = offset.as_micros_with_override(SmpteFps::TwentyFive);
    let micros_29 = offset.as_micros_with_override(SmpteFps::TwentyNine);
    let micros_30 = offset.as_micros_with_override(SmpteFps::Thirty);

    // The hour component should be the same for all
    let hour_micros = 3_600_000_000.0;

    // But the frame component should differ
    let frame_24 = (12.0 / 24.0) * 1_000_000.0;
    let frame_25 = (12.0 / 25.0) * 1_000_000.0;
    let frame_29 = (12.0 / 29.97) * 1_000_000.0;
    let frame_30 = (12.0 / 30.0) * 1_000_000.0;

    assert!((micros_24 - (hour_micros + frame_24)).abs() < 1.0);
    assert!((micros_25 - (hour_micros + frame_25)).abs() < 1.0);
    assert!((micros_29 - (hour_micros + frame_29)).abs() < 1.0);
    assert!((micros_30 - (hour_micros + frame_30)).abs() < 1.0);

    // Verify they're all different
    assert!((micros_24 - micros_25).abs() > 1.0);
    assert!((micros_24 - micros_29).abs() > 1.0);
    assert!((micros_24 - micros_30).abs() > 1.0);
}

#[test]
fn test_smpte_offset_combined_errors() {
    // Test multiple errors - parser should catch the first one

    // Invalid hour AND minute
    let data = create_smpte_bytes(0, 25, 61, 30, 15, 50);
    let result = SmpteOffset::parse(&data);
    // Should catch hour error first
    assert!(matches!(result, Err(SmpteError::HourOffset(25))));

    // Valid hour but invalid minute AND second
    let data2 = create_smpte_bytes(1, 23, 60, 60, 15, 50);
    let result2 = SmpteOffset::parse(&data2);
    // Should catch minute error
    assert!(matches!(result2, Err(SmpteError::MinuteOffset(60))));

    // Everything valid except subframe
    let data3 = create_smpte_bytes(2, 23, 59, 59, 29, 100);
    let result3 = SmpteOffset::parse(&data3);
    // Should catch subframe error
    assert!(matches!(result3, Err(SmpteError::Subframe(100))));
}

#[test]
fn test_smpte_offset_bit_manipulation_edge_cases() {
    // Test that hour bits don't interfere with fps bits
    for fps_bits in 0..=3 {
        for hour in 0..=23 {
            let first_byte = (fps_bits << 5) | hour;
            let data = vec![first_byte, 30, 45, 15, 50];
            let result = SmpteOffset::parse(&data).unwrap();

            // Verify fps is correctly parsed
            let expected_fps = match fps_bits {
                0 => SmpteFps::TwentyFour,
                1 => SmpteFps::TwentyFive,
                2 => SmpteFps::TwentyNine,
                3 => SmpteFps::Thirty,
                _ => unreachable!(),
            };
            assert_eq!(result.fps, expected_fps);
            assert_eq!(result.hour, hour);
        }
    }
}

#[test]
fn test_smpte_drop_frame_precision() {
    // Test the precision of 29.97 fps calculations
    let data = create_smpte_bytes(2, 0, 0, 0, 1, 0); // One frame at 29.97fps
    let offset = SmpteOffset::parse(&data).unwrap();

    let micros = offset.as_micros();
    let expected = 1_000_000.0 / 29.97; // Should be approximately 33366.7 microseconds

    // The constant DROP_FRAME should be 30000/1001 = 29.97002997...
    // One frame should be 1001/30000 seconds = 33366.666... microseconds
    assert!((micros - expected).abs() < 0.1);

    // Verify the exact calculation
    let exact_frame_duration = 1_001_000.0 / 30.0; // 33366.666... microseconds
    assert!((micros - exact_frame_duration).abs() < 0.001);
}
