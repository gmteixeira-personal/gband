use crossterm::event::{
    KeyCode as TermKeyCode, KeyEvent, KeyEventKind, KeyModifiers, MediaKeyCode, ModifierKeyCode,
};
use gband_client::input::key_from_event;
use gband_core::input::{Key, KeyCode, Modes, Modifiers, encode_key};

fn written(code: TermKeyCode, modifiers: KeyModifiers, kind: KeyEventKind) -> Vec<u8> {
    key_from_event(&KeyEvent::new_with_kind(code, modifiers, kind))
        .map(|key| encode_key(key, Modes::default()))
        .unwrap_or_default()
}

#[test]
fn press_becomes_a_key() {
    let event = KeyEvent::new(TermKeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(
        key_from_event(&event),
        Some(Key::new(KeyCode::Char('c'), Modifiers::CTRL))
    );
    assert_eq!(
        written(
            TermKeyCode::Char('c'),
            KeyModifiers::CONTROL,
            KeyEventKind::Press
        ),
        b"\x03"
    );
}

#[test]
fn repeat_becomes_a_key() {
    assert_eq!(
        written(
            TermKeyCode::Char('a'),
            KeyModifiers::NONE,
            KeyEventKind::Repeat
        ),
        b"a"
    );
}

#[test]
fn release_is_dropped() {
    let event = KeyEvent::new_with_kind(
        TermKeyCode::Char('a'),
        KeyModifiers::NONE,
        KeyEventKind::Release,
    );
    assert_eq!(key_from_event(&event), None);
    assert_eq!(
        written(
            TermKeyCode::Char('a'),
            KeyModifiers::NONE,
            KeyEventKind::Release
        ),
        b""
    );
}

#[test]
fn super_hyper_and_meta_are_dropped() {
    for modifier in [KeyModifiers::SUPER, KeyModifiers::HYPER, KeyModifiers::META] {
        assert_eq!(
            written(TermKeyCode::Char('a'), modifier, KeyEventKind::Press),
            b"a",
            "{modifier:?}"
        );
    }
}

#[test]
fn keys_outside_the_spec_are_dropped() {
    for code in [
        TermKeyCode::CapsLock,
        TermKeyCode::Media(MediaKeyCode::Play),
        TermKeyCode::Modifier(ModifierKeyCode::LeftShift),
        TermKeyCode::Null,
    ] {
        assert_eq!(
            key_from_event(&KeyEvent::new(code, KeyModifiers::NONE)),
            None,
            "{code:?}"
        );
    }
}

#[test]
fn every_encoded_code_translates() {
    let cases = [
        (TermKeyCode::Enter, KeyCode::Enter),
        (TermKeyCode::Tab, KeyCode::Tab),
        (TermKeyCode::BackTab, KeyCode::BackTab),
        (TermKeyCode::Backspace, KeyCode::Backspace),
        (TermKeyCode::Esc, KeyCode::Escape),
        (TermKeyCode::Up, KeyCode::Up),
        (TermKeyCode::Down, KeyCode::Down),
        (TermKeyCode::Right, KeyCode::Right),
        (TermKeyCode::Left, KeyCode::Left),
        (TermKeyCode::Home, KeyCode::Home),
        (TermKeyCode::End, KeyCode::End),
        (TermKeyCode::Insert, KeyCode::Insert),
        (TermKeyCode::Delete, KeyCode::Delete),
        (TermKeyCode::PageUp, KeyCode::PageUp),
        (TermKeyCode::PageDown, KeyCode::PageDown),
        (TermKeyCode::F(5), KeyCode::F(5)),
    ];
    for (term, code) in cases {
        let modifiers = KeyModifiers::SHIFT | KeyModifiers::ALT;
        assert_eq!(
            key_from_event(&KeyEvent::new(term, modifiers)),
            Some(Key::new(
                code,
                Modifiers {
                    shift: true,
                    alt: true,
                    ctrl: false
                }
            )),
            "{term:?}"
        );
    }
}
