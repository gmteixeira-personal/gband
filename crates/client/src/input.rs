use crossterm::event::{
    KeyCode as TermKeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton as TermButton,
    MouseEvent as TermMouseEvent, MouseEventKind,
};
use gband_core::input::{
    Key, KeyCode, Modifiers, MouseButton, MouseEvent, MouseKind, WheelDirection,
};

fn modifiers(held: KeyModifiers) -> Modifiers {
    Modifiers {
        shift: held.contains(KeyModifiers::SHIFT),
        alt: held.contains(KeyModifiers::ALT),
        ctrl: held.contains(KeyModifiers::CONTROL),
    }
}

pub fn mouse_from_event(event: &TermMouseEvent) -> Option<MouseEvent> {
    let button = |button: TermButton| match button {
        TermButton::Left => MouseButton::Left,
        TermButton::Middle => MouseButton::Middle,
        TermButton::Right => MouseButton::Right,
    };
    let kind = match event.kind {
        MouseEventKind::Down(pressed) => MouseKind::Press(button(pressed)),
        MouseEventKind::Up(released) => MouseKind::Release(button(released)),
        MouseEventKind::Drag(held) => MouseKind::Motion(Some(button(held))),
        MouseEventKind::Moved => MouseKind::Motion(None),
        MouseEventKind::ScrollUp => MouseKind::Wheel(WheelDirection::Up),
        MouseEventKind::ScrollDown => MouseKind::Wheel(WheelDirection::Down),
        MouseEventKind::ScrollLeft => MouseKind::Wheel(WheelDirection::Left),
        MouseEventKind::ScrollRight => MouseKind::Wheel(WheelDirection::Right),
    };
    Some(MouseEvent::new(
        kind,
        event.column,
        event.row,
        modifiers(event.modifiers),
    ))
}

pub fn key_from_event(event: &KeyEvent) -> Option<Key> {
    if event.kind == KeyEventKind::Release {
        return None;
    }
    let code = match event.code {
        TermKeyCode::Char(c) => KeyCode::Char(c),
        TermKeyCode::Enter => KeyCode::Enter,
        TermKeyCode::Tab => KeyCode::Tab,
        TermKeyCode::BackTab => KeyCode::BackTab,
        TermKeyCode::Backspace => KeyCode::Backspace,
        TermKeyCode::Esc => KeyCode::Escape,
        TermKeyCode::Up => KeyCode::Up,
        TermKeyCode::Down => KeyCode::Down,
        TermKeyCode::Right => KeyCode::Right,
        TermKeyCode::Left => KeyCode::Left,
        TermKeyCode::Home => KeyCode::Home,
        TermKeyCode::End => KeyCode::End,
        TermKeyCode::Insert => KeyCode::Insert,
        TermKeyCode::Delete => KeyCode::Delete,
        TermKeyCode::PageUp => KeyCode::PageUp,
        TermKeyCode::PageDown => KeyCode::PageDown,
        TermKeyCode::F(n) => KeyCode::F(n),
        _ => return None,
    };
    Some(Key::new(code, modifiers(event.modifiers)))
}
