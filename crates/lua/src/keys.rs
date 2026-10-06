use gband_core::input::{
    Key, KeyCode, Modifiers, MouseButton, MouseInput, MouseKey, WheelDirection,
};

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid key name `{0}`")]
pub struct KeyError(pub String);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pressed {
    Key(Key),
    Mouse { key: MouseKey, uses_mod: bool },
}

pub fn parse_key(name: &str) -> Result<Key, KeyError> {
    match parse_pressed(name)? {
        Pressed::Key(key) => Ok(key),
        Pressed::Mouse { .. } => Err(KeyError(name.to_owned())),
    }
}

pub fn parse_pressed(name: &str) -> Result<Pressed, KeyError> {
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
    let mut uses_mod = false;
    for modifier in modifiers
        .into_iter()
        .flat_map(|modifiers| modifiers.split('+'))
    {
        match modifier.to_ascii_lowercase().as_str() {
            "ctrl" => held.ctrl = true,
            "alt" => held.alt = true,
            "shift" => held.shift = true,
            "mod" => uses_mod = true,
            _ => return Err(invalid()),
        }
    }
    if let Some(input) = mouse_input(&key.to_ascii_lowercase()) {
        return Ok(Pressed::Mouse {
            key: MouseKey::new(input, held),
            uses_mod,
        });
    }
    if uses_mod {
        return Err(invalid());
    }
    let mut chars = key.chars();
    let code = match (chars.next(), chars.next()) {
        (Some(c), None) => KeyCode::Char(c),
        _ => named(&key.to_ascii_lowercase()).ok_or_else(invalid)?,
    };
    match code {
        KeyCode::Char(c) if held.shift && c.is_ascii_lowercase() => Ok(Pressed::Key(Key::new(
            KeyCode::Char(c.to_ascii_uppercase()),
            Modifiers {
                shift: false,
                ..held
            },
        ))),
        KeyCode::Char(_) if held.shift && key.chars().count() == 1 => Err(invalid()),
        code => Ok(Pressed::Key(Key::new(code, held))),
    }
}

fn mouse_input(name: &str) -> Option<MouseInput> {
    Some(match name {
        "leftmouse" => MouseInput::Button(MouseButton::Left),
        "middlemouse" => MouseInput::Button(MouseButton::Middle),
        "rightmouse" => MouseInput::Button(MouseButton::Right),
        "wheelup" => MouseInput::Wheel(WheelDirection::Up),
        "wheeldown" => MouseInput::Wheel(WheelDirection::Down),
        "wheelleft" => MouseInput::Wheel(WheelDirection::Left),
        "wheelright" => MouseInput::Wheel(WheelDirection::Right),
        _ => return None,
    })
}

pub fn mouse_name(key: MouseKey) -> String {
    let mut name = String::new();
    for (held, modifier) in [
        (key.modifiers.ctrl, "ctrl+"),
        (key.modifiers.alt, "alt+"),
        (key.modifiers.shift, "shift+"),
    ] {
        if held {
            name.push_str(modifier);
        }
    }
    name.push_str(match key.input {
        MouseInput::Button(MouseButton::Left) => "leftmouse",
        MouseInput::Button(MouseButton::Middle) => "middlemouse",
        MouseInput::Button(MouseButton::Right) => "rightmouse",
        MouseInput::Wheel(WheelDirection::Up) => "wheelup",
        MouseInput::Wheel(WheelDirection::Down) => "wheeldown",
        MouseInput::Wheel(WheelDirection::Left) => "wheelleft",
        MouseInput::Wheel(WheelDirection::Right) => "wheelright",
    });
    name
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

    fn mouse(input: impl Into<MouseInput>, modifiers: Modifiers) -> Result<Pressed, KeyError> {
        Ok(Pressed::Mouse {
            key: MouseKey::new(input, modifiers),
            uses_mod: false,
        })
    }

    #[test]
    fn mouse_names_with_modifiers() {
        assert_eq!(
            parse_pressed("Shift+RightMouse"),
            mouse(MouseButton::Right, Modifiers::SHIFT)
        );
        assert_eq!(
            parse_pressed("leftmouse"),
            mouse(MouseButton::Left, Modifiers::NONE)
        );
        let key = MouseKey::new(
            MouseButton::Middle,
            Modifiers {
                ctrl: true,
                alt: true,
                shift: true,
            },
        );
        assert_eq!(mouse_name(key), "ctrl+alt+shift+middlemouse");
        assert_eq!(
            parse_pressed(&mouse_name(key)),
            mouse(MouseButton::Middle, key.modifiers)
        );
    }

    #[test]
    fn wheel_names_read_back() {
        assert_eq!(
            parse_pressed("Alt+WheelDown"),
            mouse(WheelDirection::Down, Modifiers::ALT)
        );
        for (name, direction) in [
            ("wheelup", WheelDirection::Up),
            ("wheeldown", WheelDirection::Down),
            ("wheelleft", WheelDirection::Left),
            ("ctrl+wheelright", WheelDirection::Right),
        ] {
            let Ok(Pressed::Mouse { key, .. }) = parse_pressed(name) else {
                panic!("{name} is not a mouse name");
            };
            assert_eq!(key.input, MouseInput::Wheel(direction), "{name}");
            assert_eq!(mouse_name(key), name);
        }
    }

    #[test]
    fn mod_before_a_mouse_name() {
        assert_eq!(
            parse_pressed("Mod+LeftMouse"),
            Ok(Pressed::Mouse {
                key: MouseKey::new(MouseButton::Left, Modifiers::NONE),
                uses_mod: true,
            })
        );
        assert_eq!(
            parse_pressed("mod+shift+wheelup"),
            Ok(Pressed::Mouse {
                key: MouseKey::new(WheelDirection::Up, Modifiers::SHIFT),
                uses_mod: true,
            })
        );
    }

    #[test]
    fn mouse_names_are_not_keys() {
        for name in ["leftmouse", "alt+rightmouse", "wheelup", "mod+leftmouse"] {
            assert_eq!(parse_key(name), Err(KeyError(name.to_owned())));
        }
        for name in [
            "scrollup",
            "scrollwheelup",
            "mouse",
            "mod+h",
            "mod+enter",
            "mod",
        ] {
            assert_eq!(
                parse_pressed(name),
                Err(KeyError(name.to_owned())),
                "{name}"
            );
        }
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
