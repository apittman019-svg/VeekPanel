use veek_hardware::protocol::*;

#[test]
fn usb_ids_are_exact_not_vendor_wildcards() {
    assert_eq!(Model::from_usb(0x0483, 0xa3c5), Some(Model::Pro));
    assert_eq!(Model::from_usb(0x0483, 0xa3c4), Some(Model::Mini));
    assert_eq!(Model::from_usb(0x04d8, 0xeb52), Some(Model::Rgb));
    assert_eq!(Model::from_usb(0x0483, 0xffff), None);
    assert_eq!(Model::from_usb(0x1a86, 0x7523), None); // Generic CH340 is not a PCPanel identity.
}

#[test]
fn all_hid_analog_controls_and_extrema() {
    for model in [Model::Rgb, Model::Mini, Model::Pro] {
        for index in 0..model.analog_count() {
            for raw in 0..=model.maximum() {
                assert_eq!(
                    parse_hid(model, &[1, index, raw]),
                    Ok(ControlEvent::Analog {
                        index,
                        raw,
                        maximum: model.maximum()
                    })
                );
            }
        }
        assert_eq!(
            parse_hid(model, &[1, model.analog_count(), 0]),
            Err(ParseError::Index(model.analog_count()))
        );
    }
}

#[test]
fn buttons_are_independent_and_sliders_are_not_buttons() {
    for model in [Model::Rgb, Model::Mini, Model::Pro] {
        for index in 0..model.knobs() {
            for value in 0..=1 {
                assert_eq!(
                    parse_hid(model, &[2, index, value]),
                    Ok(ControlEvent::Button {
                        index,
                        pressed: value == 1
                    })
                );
            }
        }
        assert!(parse_hid(model, &[2, model.knobs(), 1]).is_err());
    }
}

#[test]
fn hid_rejects_truncated_oversized_invalid_and_wrong_transport() {
    for length in [0, 1, 2, 65, 256] {
        assert_eq!(
            parse_hid(Model::Mini, &vec![1; length]),
            Err(ParseError::Length(length))
        );
    }
    assert_eq!(
        parse_hid(Model::Rgb, &[1, 0, 101]),
        Err(ParseError::Value(101))
    );
    assert_eq!(parse_hid(Model::Pro, &[2, 0, 2]), Err(ParseError::Value(2)));
    assert_eq!(
        parse_hid(Model::Pro, &[0, 1, 0, 73]),
        Err(ParseError::Opcode(0))
    );
    assert_eq!(
        parse_hid(Model::Original, &[1, 0, 73]),
        Err(ParseError::Transport)
    );
}

#[test]
fn hid_padding_and_initialization() {
    let mut report = [0; 64];
    report[..3].copy_from_slice(&[1, 8, 255]);
    assert_eq!(
        parse_hid(Model::Pro, &report).unwrap().display(Model::Pro),
        "SLIDER_4 = 100 (raw=255/255)"
    );
    let init = initialization_report();
    assert_eq!(&init[..2], &[0, 1]);
    assert!(init[2..].iter().all(|&b| b == 0));
}

#[test]
fn original_all_knobs_buttons_and_values() {
    for index in 0..4 {
        for raw in 0..=100 {
            assert_eq!(
                parse_original_line(format!("v{index}x{raw}\r").as_bytes()),
                Ok(SerialMessage::Control(ControlEvent::Analog {
                    index,
                    raw,
                    maximum: 100
                }))
            );
        }
        for value in 0..=1 {
            assert_eq!(
                parse_original_line(format!("b{index} {value}").as_bytes()),
                Ok(SerialMessage::Control(ControlEvent::Button {
                    index,
                    pressed: value == 0
                }))
            );
        }
    }
}

#[test]
fn original_invalid_input_never_becomes_a_control() {
    for line in [
        "",
        "v0x",
        "v4x73",
        "v0x101",
        "v0x-1",
        "v0x+1",
        "v0x9999999",
        "b0 2",
        "b0 10",
        "V0x2",
        "pong!",
        "v0x50junk",
    ] {
        assert!(parse_original_line(line.as_bytes()).is_err(), "{line:?}");
    }
    assert!(parse_original_line(&[0xff; 20]).is_err());
}

#[test]
fn fragmented_and_coalesced_serial_match_at_every_split() {
    let input = b"v0x73\r\nb2 0\r\nb2 1\npong\r\n";
    for split in 0..=input.len() {
        let mut decoder = OriginalDecoder::default();
        let mut messages = decoder.feed(&input[..split]);
        messages.extend(decoder.feed(&input[split..]));
        assert_eq!(messages.len(), 4);
        assert_eq!(
            messages[0].as_ref().unwrap(),
            &SerialMessage::Control(ControlEvent::Analog {
                index: 0,
                raw: 73,
                maximum: 100
            })
        );
        assert_eq!(messages[3], Ok(SerialMessage::Heartbeat));
        assert!(decoder.finish().is_ok());
    }
}

#[test]
fn oversized_serial_discards_whole_line_and_recovers() {
    let mut decoder = OriginalDecoder::default();
    assert_eq!(
        decoder.feed(&[b'a'; 10000]),
        vec![Err(ParseError::LineTooLong)]
    );
    assert!(decoder.feed(b"v0x73").is_empty()); // Suffix of invalid line is not an event.
    assert_eq!(
        decoder.feed(b"\npong\n"),
        vec![Ok(SerialMessage::Heartbeat)]
    );
    assert!(decoder.finish().is_ok());
    decoder.feed(b"v0x7");
    assert_eq!(decoder.finish(), Err(ParseError::TruncatedLine));
}

#[test]
fn arbitrary_reports_never_panic() {
    // Deterministic malformed-byte coverage without a device or fuzz runtime.
    let mut seed = 0x12345678u32;
    for size in 0..260 {
        let bytes: Vec<_> = (0..size)
            .map(|_| {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                seed as u8
            })
            .collect();
        for model in [Model::Rgb, Model::Mini, Model::Pro] {
            let _ = parse_hid(model, &bytes);
        }
        let mut decoder = OriginalDecoder::default();
        for chunk in bytes.chunks(7) {
            let _ = decoder.feed(chunk);
        }
    }
}
