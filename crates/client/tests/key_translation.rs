use crossterm::event::{
    KeyCode as TermKeyCode, KeyEvent, KeyEventKind, KeyModifiers, MediaKeyCode, ModifierKeyCode,
    MouseButton as TermButton, MouseEvent as TermMouseEvent, MouseEventKind,
};
use gband_client::input::{key_from_event, mouse_from_event};
use gband_core::input::{
    Key, KeyCode, Modes, Modifiers, MouseButton, MouseEvent, MouseKind, WheelDirection, encode_key,
};

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

fn mouse(
    kind: MouseEventKind,
    column: u16,
    row: u16,
    modifiers: KeyModifiers,
) -> Option<MouseEvent> {
    mouse_from_event(&TermMouseEvent {
        kind,
        column,
        row,
        modifiers,
    })
}

#[test]
fn sgr_press_becomes_an_event() {
    assert_eq!(
        mouse(
            MouseEventKind::Down(TermButton::Left),
            12,
            3,
            KeyModifiers::CONTROL
        ),
        Some(MouseEvent::new(
            MouseKind::Press(MouseButton::Left),
            12,
            3,
            Modifiers::CTRL
        ))
    );
}

#[test]
fn wheel_becomes_a_step() {
    assert_eq!(
        mouse(MouseEventKind::ScrollDown, 0, 0, KeyModifiers::NONE),
        Some(MouseEvent::new(
            MouseKind::Wheel(WheelDirection::Down),
            0,
            0,
            Modifiers::NONE
        ))
    );
}

#[test]
fn every_mouse_kind_converts() {
    let cases = [
        (
            MouseEventKind::Up(TermButton::Right),
            MouseKind::Release(MouseButton::Right),
        ),
        (
            MouseEventKind::Drag(TermButton::Middle),
            MouseKind::Motion(Some(MouseButton::Middle)),
        ),
        (MouseEventKind::Moved, MouseKind::Motion(None)),
        (
            MouseEventKind::ScrollUp,
            MouseKind::Wheel(WheelDirection::Up),
        ),
        (
            MouseEventKind::ScrollLeft,
            MouseKind::Wheel(WheelDirection::Left),
        ),
        (
            MouseEventKind::ScrollRight,
            MouseKind::Wheel(WheelDirection::Right),
        ),
    ];
    for (term, kind) in cases {
        let held = KeyModifiers::SHIFT | KeyModifiers::ALT | KeyModifiers::META;
        assert_eq!(
            mouse(term, 4, 5, held),
            Some(MouseEvent::new(
                kind,
                4,
                5,
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
