use gband_core::input::{Key, KeyCode, Modifiers};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid key name `{0}`")]
pub struct KeyError(pub String);

pub fn parse_key(name: &str) -> Result<Key, KeyError> {
    let invalid = || KeyError(name.to_owned());
    let (modifiers, key) = if name == "+" {
        (None, "+")
    } else if let Some(rest) = name.strip_suffix("++") {
        (Some(rest), "+")
    } else {
        match name.rsplit_once('+') {
            Some((modifiers, key)) => (Some(modifiers), key),
            None => (None, name),
        }
    };
    let mut held = Modifiers::NONE;
    for modifier in modifiers
        .into_iter()
        .flat_map(|modifiers| modifiers.split('+'))
    {
        match modifier.to_ascii_lowercase().as_str() {
            "ctrl" => held.ctrl = true,
            "alt" => held.alt = true,
            "shift" => held.shift = true,
            _ => return Err(invalid()),
        }
    }
    let mut chars = key.chars();
    let code = match (chars.next(), chars.next()) {
        (Some(c), None) => KeyCode::Char(c),
        _ => named(&key.to_ascii_lowercase()).ok_or_else(invalid)?,
    };
    match code {
        KeyCode::Char(c) if held.shift && c.is_ascii_lowercase() => Ok(Key::new(
            KeyCode::Char(c.to_ascii_uppercase()),
            Modifiers {
                shift: false,
                ..held
            },
        )),
        KeyCode::Char(_) if held.shift && key.chars().count() == 1 => Err(invalid()),
        code => Ok(Key::new(code, held)),
    }
}

pub fn key_name(key: Key) -> String {
    let mut name = String::new();
    if key.modifiers.ctrl {
        name.push_str("ctrl+");
    }
    if key.modifiers.alt {
        name.push_str("alt+");
    }
    if key.modifiers.shift && !matches!(key.code, KeyCode::Char(_)) {
        name.push_str("shift+");
    }
    match key.code {
        KeyCode::Char(' ') => name.push_str("space"),
        KeyCode::Char(c) => name.push(c),
        KeyCode::Enter => name.push_str("enter"),
        KeyCode::Tab => name.push_str("tab"),
        KeyCode::BackTab => name.push_str("backtab"),
        KeyCode::Backspace => name.push_str("backspace"),
        KeyCode::Escape => name.push_str("escape"),
        KeyCode::Up => name.push_str("up"),
        KeyCode::Down => name.push_str("down"),
        KeyCode::Left => name.push_str("left"),
        KeyCode::Right => name.push_str("right"),
        KeyCode::Home => name.push_str("home"),
        KeyCode::End => name.push_str("end"),
        KeyCode::Insert => name.push_str("insert"),
        KeyCode::Delete => name.push_str("delete"),
        KeyCode::PageUp => name.push_str("pageup"),
        KeyCode::PageDown => name.push_str("pagedown"),
        KeyCode::F(number) => name.push_str(&format!("f{number}")),
    }
    name
}

fn named(name: &str) -> Option<KeyCode> {
    Some(match name {
        "enter" => KeyCode::Enter,
        "tab" => KeyCode::Tab,
        "backtab" => KeyCode::BackTab,
        "backspace" => KeyCode::Backspace,
        "escape" | "esc" => KeyCode::Escape,
        "space" => KeyCode::Char(' '),
        "up" => KeyCode::Up,
        "down" => KeyCode::Down,
        "left" => KeyCode::Left,
        "right" => KeyCode::Right,
        "home" => KeyCode::Home,
        "end" => KeyCode::End,
        "insert" => KeyCode::Insert,
        "delete" => KeyCode::Delete,
        "pageup" => KeyCode::PageUp,
        "pagedown" => KeyCode::PageDown,
        _ => {
            let number: u8 = name.strip_prefix('f')?.parse().ok()?;
            if !(1..=12).contains(&number) || name.starts_with("f0") {
                return None;
            }
            KeyCode::F(number)
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode, modifiers: Modifiers) -> Result<Key, KeyError> {
        Ok(Key::new(code, modifiers))
    }

    #[test]
    fn modifier_and_character() {
        assert_eq!(parse_key("alt+h"), key(KeyCode::Char('h'), Modifiers::ALT));
    }

    #[test]
    fn shift_with_a_letter_names_the_uppercase_letter() {
        assert_eq!(parse_key("shift+d"), parse_key("D"));
        assert_eq!(parse_key("D"), key(KeyCode::Char('D'), Modifiers::NONE));
    }

    #[test]
    fn one_character_keys_keep_their_case() {
        assert_ne!(parse_key("d"), parse_key("D"));
    }

    #[test]
    fn plus_as_the_key() {
        assert_eq!(parse_key("alt++"), key(KeyCode::Char('+'), Modifiers::ALT));
        assert_eq!(parse_key("+"), key(KeyCode::Char('+'), Modifiers::NONE));
        assert_eq!(
            parse_key("ctrl+alt++"),
            key(
                KeyCode::Char('+'),
                Modifiers {
                    ctrl: true,
                    alt: true,
                    shift: false
                }
            )
        );
    }

    #[test]
    fn named_key_in_any_case() {
        assert_eq!(
            parse_key("Ctrl+PageUp"),
            key(KeyCode::PageUp, Modifiers::CTRL)
        );
        assert_eq!(parse_key("ENTER"), key(KeyCode::Enter, Modifiers::NONE));
    }

    #[test]
    fn more_named_keys() {
        assert_eq!(parse_key("space"), key(KeyCode::Char(' '), Modifiers::NONE));
        assert_eq!(parse_key("esc"), key(KeyCode::Escape, Modifiers::NONE));
        assert_eq!(parse_key("escape"), key(KeyCode::Escape, Modifiers::NONE));
        assert_eq!(parse_key("f12"), key(KeyCode::F(12), Modifiers::NONE));
        assert_eq!(parse_key("F1"), key(KeyCode::F(1), Modifiers::NONE));
        assert_eq!(parse_key("shift+tab"), key(KeyCode::Tab, Modifiers::SHIFT));
    }

    #[test]
    fn names_read_back_as_the_same_key() {
        for name in [
            "ctrl+space",
            "ctrl+b",
            "alt+h",
            "D",
            "+",
            "ctrl+alt++",
            "shift+tab",
            "Ctrl+PageUp",
            "f12",
            "enter",
        ] {
            let key = parse_key(name).unwrap();
            assert_eq!(parse_key(&key_name(key)), Ok(key), "{name}");
        }
        assert_eq!(key_name(parse_key("ctrl+space").unwrap()), "ctrl+space");
    }

    #[test]
    fn invalid_names_are_rejected() {
        for name in [
            "alt+hyper",
            "shift+=",
            "shift+D",
            "f13",
            "f0",
            "f01",
            "hyper+a",
            "",
            "alt+",
            "++",
            "alt+h+",
            "ctrl++a",
        ] {
            assert_eq!(parse_key(name), Err(KeyError(name.to_owned())), "{name}");
        }
    }
}
