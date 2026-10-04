use gband_core::input::{Key, KeyCode, Modes, Modifiers, encode_key, encode_paste};

const NORMAL: Modes = Modes {
    application_cursor: false,
    bracketed_paste: false,
};
const APPLICATION: Modes = Modes {
    application_cursor: true,
    bracketed_paste: false,
};
const BRACKETED: Modes = Modes {
    application_cursor: false,
    bracketed_paste: true,
};

const SHIFT_ALT_CTRL: Modifiers = Modifiers {
    shift: true,
    alt: true,
    ctrl: true,
};
const ALT_CTRL: Modifiers = Modifiers {
    shift: false,
    alt: true,
    ctrl: true,
};
const CTRL_SHIFT: Modifiers = Modifiers {
    shift: true,
    alt: false,
    ctrl: true,
};

fn check(cases: &[(KeyCode, Modifiers, Modes, &[u8])]) {
    for &(code, modifiers, modes, expected) in cases {
        let key = Key::new(code, modifiers);
        assert_eq!(encode_key(key, modes), expected, "{key:?} under {modes:?}");
    }
}

#[test]
fn same_input_gives_same_bytes() {
    let key = Key::new(KeyCode::Up, Modifiers::CTRL);
    assert_eq!(encode_key(key, APPLICATION), encode_key(key, APPLICATION));
    assert_eq!(
        encode_paste("a\nb", BRACKETED),
        encode_paste("a\nb", BRACKETED)
    );
}

#[test]
fn printable_characters() {
    check(&[
        (KeyCode::Char('a'), Modifiers::NONE, NORMAL, b"a"),
        (KeyCode::Char('A'), Modifiers::SHIFT, NORMAL, b"A"),
        (KeyCode::Char('é'), Modifiers::NONE, NORMAL, b"\xc3\xa9"),
    ]);
}

#[test]
fn control_characters() {
    check(&[
        (KeyCode::Char('a'), Modifiers::CTRL, NORMAL, b"\x01"),
        (KeyCode::Char('A'), CTRL_SHIFT, NORMAL, b"\x01"),
        (KeyCode::Char('z'), Modifiers::CTRL, NORMAL, b"\x1a"),
        (KeyCode::Char('h'), Modifiers::CTRL, NORMAL, b"\x08"),
        (KeyCode::Char('i'), Modifiers::CTRL, NORMAL, b"\x09"),
        (KeyCode::Char(' '), Modifiers::CTRL, NORMAL, b"\x00"),
        (KeyCode::Char('@'), Modifiers::CTRL, NORMAL, b"\x00"),
        (KeyCode::Char('2'), Modifiers::CTRL, NORMAL, b"\x00"),
        (KeyCode::Char('['), Modifiers::CTRL, NORMAL, b"\x1b"),
        (KeyCode::Char('3'), Modifiers::CTRL, NORMAL, b"\x1b"),
        (KeyCode::Char('\\'), Modifiers::CTRL, NORMAL, b"\x1c"),
        (KeyCode::Char('4'), Modifiers::CTRL, NORMAL, b"\x1c"),
        (KeyCode::Char(']'), Modifiers::CTRL, NORMAL, b"\x1d"),
        (KeyCode::Char('5'), Modifiers::CTRL, NORMAL, b"\x1d"),
        (KeyCode::Char('^'), Modifiers::CTRL, NORMAL, b"\x1e"),
        (KeyCode::Char('6'), Modifiers::CTRL, NORMAL, b"\x1e"),
        (KeyCode::Char('_'), Modifiers::CTRL, NORMAL, b"\x1f"),
        (KeyCode::Char('/'), Modifiers::CTRL, NORMAL, b"\x1f"),
        (KeyCode::Char('7'), Modifiers::CTRL, NORMAL, b"\x1f"),
        (KeyCode::Char('?'), Modifiers::CTRL, NORMAL, b"\x7f"),
        (KeyCode::Char('8'), Modifiers::CTRL, NORMAL, b"\x7f"),
        (KeyCode::Char('1'), Modifiers::CTRL, NORMAL, b"1"),
    ]);
}

#[test]
fn editing_keys() {
    check(&[
        (KeyCode::Enter, Modifiers::NONE, NORMAL, b"\r"),
        (KeyCode::Enter, Modifiers::SHIFT, NORMAL, b"\r"),
        (KeyCode::Enter, Modifiers::CTRL, NORMAL, b"\r"),
        (KeyCode::Tab, Modifiers::NONE, NORMAL, b"\t"),
        (KeyCode::Tab, Modifiers::CTRL, NORMAL, b"\t"),
        (KeyCode::BackTab, Modifiers::SHIFT, NORMAL, b"\x1b[Z"),
        (KeyCode::Backspace, Modifiers::NONE, NORMAL, b"\x7f"),
        (KeyCode::Backspace, Modifiers::SHIFT, NORMAL, b"\x7f"),
        (KeyCode::Backspace, Modifiers::CTRL, NORMAL, b"\x08"),
        (KeyCode::Escape, Modifiers::NONE, NORMAL, b"\x1b"),
        (KeyCode::Escape, Modifiers::SHIFT, NORMAL, b"\x1b"),
    ]);
}

#[test]
fn alt_prefixes_escape() {
    check(&[
        (KeyCode::Char('x'), Modifiers::ALT, NORMAL, b"\x1bx"),
        (KeyCode::Char('a'), ALT_CTRL, NORMAL, b"\x1b\x01"),
        (KeyCode::Backspace, Modifiers::ALT, NORMAL, b"\x1b\x7f"),
        (KeyCode::Escape, Modifiers::ALT, NORMAL, b"\x1b\x1b"),
        (KeyCode::Enter, Modifiers::ALT, NORMAL, b"\x1b\r"),
    ]);
}

#[test]
fn modifier_parameter() {
    check(&[
        (KeyCode::Up, SHIFT_ALT_CTRL, NORMAL, b"\x1b[1;8A"),
        (KeyCode::Up, Modifiers::SHIFT, NORMAL, b"\x1b[1;2A"),
        (KeyCode::Up, Modifiers::ALT, NORMAL, b"\x1b[1;3A"),
        (KeyCode::Up, Modifiers::CTRL, NORMAL, b"\x1b[1;5A"),
        (KeyCode::Delete, ALT_CTRL, NORMAL, b"\x1b[3;7~"),
    ]);
}

#[test]
fn cursor_keys_follow_application_cursor_mode() {
    check(&[
        (KeyCode::Up, Modifiers::NONE, NORMAL, b"\x1b[A"),
        (KeyCode::Down, Modifiers::NONE, NORMAL, b"\x1b[B"),
        (KeyCode::Right, Modifiers::NONE, NORMAL, b"\x1b[C"),
        (KeyCode::Left, Modifiers::NONE, NORMAL, b"\x1b[D"),
        (KeyCode::Home, Modifiers::NONE, NORMAL, b"\x1b[H"),
        (KeyCode::End, Modifiers::NONE, NORMAL, b"\x1b[F"),
        (KeyCode::Up, Modifiers::NONE, APPLICATION, b"\x1bOA"),
        (KeyCode::Down, Modifiers::NONE, APPLICATION, b"\x1bOB"),
        (KeyCode::Right, Modifiers::NONE, APPLICATION, b"\x1bOC"),
        (KeyCode::Left, Modifiers::NONE, APPLICATION, b"\x1bOD"),
        (KeyCode::Home, Modifiers::NONE, APPLICATION, b"\x1bOH"),
        (KeyCode::End, Modifiers::NONE, APPLICATION, b"\x1bOF"),
        (KeyCode::Up, Modifiers::SHIFT, APPLICATION, b"\x1b[1;2A"),
        (KeyCode::Left, Modifiers::CTRL, NORMAL, b"\x1b[1;5D"),
        (KeyCode::Left, Modifiers::CTRL, APPLICATION, b"\x1b[1;5D"),
        (KeyCode::Up, Modifiers::ALT, NORMAL, b"\x1b[1;3A"),
        (KeyCode::End, Modifiers::SHIFT, APPLICATION, b"\x1b[1;2F"),
    ]);
}

#[test]
fn tilde_and_function_keys() {
    check(&[
        (KeyCode::Insert, Modifiers::NONE, NORMAL, b"\x1b[2~"),
        (KeyCode::Delete, Modifiers::NONE, NORMAL, b"\x1b[3~"),
        (KeyCode::PageUp, Modifiers::NONE, NORMAL, b"\x1b[5~"),
        (KeyCode::PageDown, Modifiers::NONE, NORMAL, b"\x1b[6~"),
        (KeyCode::Delete, Modifiers::CTRL, NORMAL, b"\x1b[3;5~"),
        (KeyCode::PageUp, Modifiers::SHIFT, APPLICATION, b"\x1b[5;2~"),
        (KeyCode::F(1), Modifiers::NONE, NORMAL, b"\x1bOP"),
        (KeyCode::F(2), Modifiers::NONE, NORMAL, b"\x1bOQ"),
        (KeyCode::F(3), Modifiers::NONE, NORMAL, b"\x1bOR"),
        (KeyCode::F(4), Modifiers::NONE, NORMAL, b"\x1bOS"),
        (KeyCode::F(1), Modifiers::SHIFT, NORMAL, b"\x1b[1;2P"),
        (KeyCode::F(4), Modifiers::CTRL, NORMAL, b"\x1b[1;5S"),
        (KeyCode::F(5), Modifiers::NONE, NORMAL, b"\x1b[15~"),
        (KeyCode::F(6), Modifiers::NONE, NORMAL, b"\x1b[17~"),
        (KeyCode::F(7), Modifiers::NONE, NORMAL, b"\x1b[18~"),
        (KeyCode::F(8), Modifiers::NONE, NORMAL, b"\x1b[19~"),
        (KeyCode::F(9), Modifiers::NONE, NORMAL, b"\x1b[20~"),
        (KeyCode::F(10), Modifiers::NONE, NORMAL, b"\x1b[21~"),
        (KeyCode::F(11), Modifiers::NONE, NORMAL, b"\x1b[23~"),
        (KeyCode::F(12), Modifiers::NONE, NORMAL, b"\x1b[24~"),
        (KeyCode::F(5), Modifiers::ALT, NORMAL, b"\x1b[15;3~"),
        (KeyCode::F(13), Modifiers::NONE, NORMAL, b""),
        (KeyCode::F(0), Modifiers::NONE, NORMAL, b""),
    ]);
}

#[test]
fn paste() {
    let cases: &[(&str, Modes, &[u8])] = &[
        ("ls\n", BRACKETED, b"\x1b[200~ls\n\x1b[201~"),
        ("a\nb\r\nc", NORMAL, b"a\rb\rc"),
        ("x\x1b[201~rm", BRACKETED, b"\x1b[200~x[201~rm\x1b[201~"),
        ("a\tb", NORMAL, b"a\tb"),
        ("a\u{7}b\u{9b}c\u{0}d", NORMAL, b"abcd"),
        ("a\r\nb", BRACKETED, b"\x1b[200~a\r\nb\x1b[201~"),
        ("é", NORMAL, b"\xc3\xa9"),
    ];
    for &(text, modes, expected) in cases {
        assert_eq!(
            encode_paste(text, modes),
            expected,
            "{text:?} under {modes:?}"
        );
    }
}
