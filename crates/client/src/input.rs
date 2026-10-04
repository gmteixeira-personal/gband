use crossterm::event::{KeyCode as TermKeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use gband_core::input::{Key, KeyCode, Modifiers};

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
    let modifiers = Modifiers {
        shift: event.modifiers.contains(KeyModifiers::SHIFT),
        alt: event.modifiers.contains(KeyModifiers::ALT),
        ctrl: event.modifiers.contains(KeyModifiers::CONTROL),
    };
    Some(Key::new(code, modifiers))
}
