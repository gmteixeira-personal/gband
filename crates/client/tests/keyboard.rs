use gband_client::{release_keyboard_protocols, request_keyboard_protocols};

#[test]
fn attach_requests_both_protocols() {
    let mut bytes = Vec::new();
    request_keyboard_protocols(&mut bytes).unwrap();
    assert_eq!(bytes, b"\x1b[>4;1m\x1b[>5u");
}

#[test]
fn detach_releases_both_protocols() {
    let mut bytes = Vec::new();
    release_keyboard_protocols(&mut bytes).unwrap();
    assert_eq!(bytes, b"\x1b[<1u\x1b[>4m");
}
