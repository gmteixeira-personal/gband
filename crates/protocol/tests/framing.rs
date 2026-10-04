use gband_protocol::{ClientMessage, Decoder, FrameError, MAX_FRAME_LEN, ServerMessage, encode};

fn sample() -> ServerMessage {
    ServerMessage::Snapshot {
        cols: 80,
        rows: 24,
        contents: b"\x1b[1;31mhello\x1b[m".to_vec(),
    }
}

#[test]
fn frame_split_at_every_boundary_decodes_once() {
    let frame = encode(&sample()).unwrap();
    for split in 0..=frame.len() {
        let mut decoder = Decoder::new();
        decoder.feed(&frame[..split]).unwrap();
        if split < frame.len() {
            assert_eq!(
                decoder.next_message::<ServerMessage>().unwrap(),
                None,
                "{split}"
            );
        }
        decoder.feed(&frame[split..]).unwrap();
        assert_eq!(
            decoder.next_message::<ServerMessage>().unwrap(),
            Some(sample())
        );
        assert_eq!(decoder.next_message::<ServerMessage>().unwrap(), None);
    }
}

#[test]
fn frame_fed_byte_by_byte_decodes_once() {
    let frame = encode(&sample()).unwrap();
    let mut decoder = Decoder::new();
    let mut decoded = Vec::new();
    for byte in &frame {
        decoder.feed(std::slice::from_ref(byte)).unwrap();
        decoded.extend(decoder.next_message::<ServerMessage>().unwrap());
    }
    assert_eq!(decoded, [sample()]);
}

#[test]
fn two_frames_in_one_chunk_decode_in_order() {
    let first = ServerMessage::Update(b"one".to_vec());
    let second = ServerMessage::Exited;
    let chunk = [encode(&first).unwrap(), encode(&second).unwrap()].concat();
    let mut decoder = Decoder::new();
    decoder.feed(&chunk).unwrap();
    assert_eq!(
        decoder.next_message::<ServerMessage>().unwrap(),
        Some(first)
    );
    assert_eq!(
        decoder.next_message::<ServerMessage>().unwrap(),
        Some(second)
    );
    assert_eq!(decoder.next_message::<ServerMessage>().unwrap(), None);
}

#[test]
fn oversized_header_is_rejected_before_payload() {
    let announced = 17 * 1024 * 1024;
    assert!(announced > MAX_FRAME_LEN);
    let header = (announced as u32).to_be_bytes();
    let mut decoder = Decoder::new();
    match decoder.feed(&header) {
        Err(FrameError::Oversized(len)) => assert_eq!(len, announced),
        other => panic!("expected an oversized frame, got {other:?}"),
    }
}

#[test]
fn largest_allowed_header_is_accepted() {
    let header = (MAX_FRAME_LEN as u32).to_be_bytes();
    Decoder::new().feed(&header).unwrap();
}

#[test]
fn trailing_bytes_are_an_error() {
    let mut decoder = Decoder::new();
    decoder.feed(&[0, 0, 0, 3, 0x03, 0x00, 0x00]).unwrap();
    assert!(matches!(
        decoder.next_message::<ClientMessage>(),
        Err(FrameError::TrailingBytes(2))
    ));
}

#[test]
fn undecodable_payload_is_an_error() {
    let mut decoder = Decoder::new();
    decoder.feed(&[0, 0, 0, 2, 0x7f, 0x00]).unwrap();
    assert!(matches!(
        decoder.next_message::<ClientMessage>(),
        Err(FrameError::Decode(_))
    ));
}
