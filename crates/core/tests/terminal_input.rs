use gband_core::input::{
    Key, KeyCode, Modes, Modifiers, MouseButton, MouseEncoding, MouseEvent, MouseKind,
    MouseTracking, WheelDirection, encode_key, encode_mouse, encode_paste,
};
use gband_core::terminal_input::{Decoder, TerminalInput};

const ALT_SHIFT: Modifiers = Modifiers {
    shift: true,
    alt: true,
    ctrl: false,
};

fn every_modifiers() -> Vec<Modifiers> {
    (0..8)
        .map(|bits| Modifiers {
            shift: bits & 1 != 0,
            alt: bits & 2 != 0,
            ctrl: bits & 4 != 0,
        })
        .collect()
}

fn decoded(chunks: &[&[u8]]) -> Vec<TerminalInput> {
    let mut decoder = Decoder::new();
    let mut inputs: Vec<_> = chunks
        .iter()
        .flat_map(|chunk| decoder.push(chunk))
        .collect();
    inputs.extend(decoder.flush());
    assert!(!decoder.holds(), "{chunks:?} left bytes held");
    inputs
}

fn pushed(chunks: &[&[u8]]) -> (Vec<TerminalInput>, bool) {
    let mut decoder = Decoder::new();
    let inputs = chunks
        .iter()
        .flat_map(|chunk| decoder.push(chunk))
        .collect();
    (inputs, decoder.holds())
}

fn key(code: KeyCode, modifiers: Modifiers) -> TerminalInput {
    TerminalInput::Key(Key::new(code, modifiers))
}

fn plain(code: KeyCode) -> TerminalInput {
    key(code, Modifiers::NONE)
}

fn char(c: char) -> TerminalInput {
    plain(KeyCode::Char(c))
}

fn ctrl(c: char) -> TerminalInput {
    key(KeyCode::Char(c), Modifiers::CTRL)
}

fn escape() -> TerminalInput {
    plain(KeyCode::Escape)
}

fn mouse(kind: MouseKind, col: u16, row: u16, modifiers: Modifiers) -> TerminalInput {
    TerminalInput::Mouse(MouseEvent::new(kind, col, row, modifiers))
}

fn paste(text: &str) -> TerminalInput {
    TerminalInput::Paste(text.to_owned())
}

fn check(cases: &[(&[u8], Vec<TerminalInput>)]) {
    for (bytes, expected) in cases {
        assert_eq!(
            &decoded(&[bytes]),
            expected,
            "{:?}",
            String::from_utf8_lossy(bytes)
        );
    }
}

#[test]
fn control_byte_becomes_a_key() {
    check(&[(b"\x03", vec![ctrl('c')])]);
}

#[test]
fn backspace_byte_is_ctrl_h() {
    check(&[(b"\x08", vec![ctrl('h')])]);
}

#[test]
fn modified_cursor_key() {
    check(&[(b"\x1b[1;5A", vec![key(KeyCode::Up, Modifiers::CTRL)])]);
}

#[test]
fn rxvt_home() {
    check(&[(b"\x1b[7~", vec![plain(KeyCode::Home)])]);
}

#[test]
fn linux_console_f1() {
    check(&[(b"\x1b[[A", vec![plain(KeyCode::F(1))])]);
}

#[test]
fn unknown_sequence_is_dropped() {
    check(&[
        (b"\x1b[99~a", vec![char('a')]),
        (b"\x1b[?1;2ca", vec![char('a')]),
        (b"\x1b[1 qa", vec![char('a')]),
        (b"\x1b[1xa", vec![char('a')]),
        (b"\x1b[201~a", vec![char('a')]),
    ]);
}

#[test]
fn alt_bracket_before_a_byte_that_starts_no_sequence() {
    let alt_bracket = || key(KeyCode::Char('['), Modifiers::ALT);
    check(&[
        (b"\x1b[a", vec![alt_bracket(), char('a')]),
        (b"\x1b[x", vec![alt_bracket(), char('x')]),
        (
            b"\x1b[Pa",
            vec![
                alt_bracket(),
                key(KeyCode::Char('P'), Modifiers::SHIFT),
                char('a'),
            ],
        ),
        (
            b"\x1b[[Za",
            vec![
                alt_bracket(),
                char('['),
                key(KeyCode::Char('Z'), Modifiers::SHIFT),
                char('a'),
            ],
        ),
        (b"\x1b[;", vec![alt_bracket(), char(';')]),
        (b"\x1b[\x01", vec![alt_bracket(), ctrl('a')]),
    ]);
}

#[test]
fn alt_bracket_then_a_key_within_the_flush_time() {
    let mut decoder = Decoder::new();
    assert_eq!(decoder.push(b"\x1b["), vec![]);
    assert_eq!(
        decoder.push(b"a"),
        vec![key(KeyCode::Char('['), Modifiers::ALT), char('a')]
    );
    assert!(!decoder.holds());
}

#[test]
fn alt_shift_o_before_a_byte_that_ends_no_sequence() {
    check(&[
        (
            b"\x1bOxa",
            vec![key(KeyCode::Char('O'), ALT_SHIFT), char('x'), char('a')],
        ),
        (
            b"\x1bO1",
            vec![key(KeyCode::Char('O'), ALT_SHIFT), char('1')],
        ),
    ]);
}

#[test]
fn single_bytes() {
    check(&[
        (b"\r", vec![plain(KeyCode::Enter)]),
        (b"\t", vec![plain(KeyCode::Tab)]),
        (b"\x7f", vec![plain(KeyCode::Backspace)]),
        (b"\x00", vec![ctrl(' ')]),
        (b"\x01", vec![ctrl('a')]),
        (b"\n", vec![ctrl('j')]),
        (b"\x1a", vec![ctrl('z')]),
        (b"\x1c", vec![ctrl('4')]),
        (b"\x1d", vec![ctrl('5')]),
        (b"\x1e", vec![ctrl('6')]),
        (b"\x1f", vec![ctrl('7')]),
        (b"a", vec![char('a')]),
        (b" ", vec![char(' ')]),
        (b"~", vec![char('~')]),
        (b"A", vec![key(KeyCode::Char('A'), Modifiers::SHIFT)]),
    ]);
}

#[test]
fn printable_characters_in_utf8() {
    check(&[
        ("é".as_bytes(), vec![char('é')]),
        (
            "Ž".as_bytes(),
            vec![key(KeyCode::Char('Ž'), Modifiers::SHIFT)],
        ),
        ("中".as_bytes(), vec![char('中')]),
        ("🦀".as_bytes(), vec![char('🦀')]),
        (b"hello", "hello".chars().map(char).collect()),
    ]);
}

#[test]
fn invalid_utf8_is_dropped() {
    check(&[
        (b"\x80a", vec![char('a')]),
        (b"\xffa", vec![char('a')]),
        (b"\xc3a", vec![char('a')]),
    ]);
}

#[test]
fn cursor_keys_in_both_forms() {
    let letters = [
        (b'A', KeyCode::Up),
        (b'B', KeyCode::Down),
        (b'C', KeyCode::Right),
        (b'D', KeyCode::Left),
        (b'H', KeyCode::Home),
        (b'F', KeyCode::End),
    ];
    for (letter, code) in letters {
        check(&[
            (&[0x1b, b'[', letter], vec![plain(code)]),
            (&[0x1b, b'O', letter], vec![plain(code)]),
        ]);
    }
}

#[test]
fn function_keys_in_every_form() {
    check(&[
        (b"\x1bOP", vec![plain(KeyCode::F(1))]),
        (b"\x1bOQ", vec![plain(KeyCode::F(2))]),
        (b"\x1bOR", vec![plain(KeyCode::F(3))]),
        (b"\x1bOS", vec![plain(KeyCode::F(4))]),
        (b"\x1b[[B", vec![plain(KeyCode::F(2))]),
        (b"\x1b[[C", vec![plain(KeyCode::F(3))]),
        (b"\x1b[[D", vec![plain(KeyCode::F(4))]),
        (b"\x1b[[E", vec![plain(KeyCode::F(5))]),
        (b"\x1b[1;2P", vec![key(KeyCode::F(1), Modifiers::SHIFT)]),
        (b"\x1b[1;5S", vec![key(KeyCode::F(4), Modifiers::CTRL)]),
    ]);
}

#[test]
fn tilde_keys() {
    let numbers = [
        (1, KeyCode::Home),
        (2, KeyCode::Insert),
        (3, KeyCode::Delete),
        (4, KeyCode::End),
        (5, KeyCode::PageUp),
        (6, KeyCode::PageDown),
        (7, KeyCode::Home),
        (8, KeyCode::End),
        (11, KeyCode::F(1)),
        (12, KeyCode::F(2)),
        (13, KeyCode::F(3)),
        (14, KeyCode::F(4)),
        (15, KeyCode::F(5)),
        (17, KeyCode::F(6)),
        (18, KeyCode::F(7)),
        (19, KeyCode::F(8)),
        (20, KeyCode::F(9)),
        (21, KeyCode::F(10)),
        (23, KeyCode::F(11)),
        (24, KeyCode::F(12)),
    ];
    for (number, code) in numbers {
        check(&[
            (format!("\x1b[{number}~").as_bytes(), vec![plain(code)]),
            (
                format!("\x1b[{number};3~").as_bytes(),
                vec![key(code, Modifiers::ALT)],
            ),
        ]);
    }
    check(&[
        (b"\x1b[16~", vec![]),
        (b"\x1b[22~", vec![]),
        (b"\x1b[25~", vec![]),
    ]);
}

#[test]
fn modifier_parameter() {
    for modifiers in every_modifiers() {
        let parameter = 1
            + u8::from(modifiers.shift)
            + 2 * u8::from(modifiers.alt)
            + 4 * u8::from(modifiers.ctrl);
        check(&[
            (
                format!("\x1b[1;{parameter}D").as_bytes(),
                vec![key(KeyCode::Left, modifiers)],
            ),
            (
                format!("\x1b[6;{parameter}~").as_bytes(),
                vec![key(KeyCode::PageDown, modifiers)],
            ),
        ]);
    }
}

#[test]
fn higher_modifier_bits_are_dropped() {
    check(&[
        (b"\x1b[1;9A", vec![plain(KeyCode::Up)]),
        (b"\x1b[1;13A", vec![key(KeyCode::Up, Modifiers::CTRL)]),
        (b"\x1b[3;17~", vec![plain(KeyCode::Delete)]),
    ]);
}

#[test]
fn shift_tab() {
    check(&[(b"\x1b[Z", vec![plain(KeyCode::BackTab)])]);
}

#[test]
fn sgr_mouse_reports() {
    check(&[
        (
            b"\x1b[<16;13;4M",
            vec![mouse(
                MouseKind::Press(MouseButton::Left),
                12,
                3,
                Modifiers::CTRL,
            )],
        ),
        (
            b"\x1b[<65;1;1M",
            vec![mouse(
                MouseKind::Wheel(WheelDirection::Down),
                0,
                0,
                Modifiers::NONE,
            )],
        ),
        (
            b"\x1b[<2;1;1m",
            vec![mouse(
                MouseKind::Release(MouseButton::Right),
                0,
                0,
                Modifiers::NONE,
            )],
        ),
        (
            b"\x1b[<35;10;5M",
            vec![mouse(MouseKind::Motion(None), 9, 4, Modifiers::NONE)],
        ),
        (
            b"\x1b[<33;3;2M",
            vec![mouse(
                MouseKind::Motion(Some(MouseButton::Middle)),
                2,
                1,
                Modifiers::NONE,
            )],
        ),
        (
            b"\x1b[<66;42;12M\x1b[<67;42;12M",
            vec![
                mouse(
                    MouseKind::Wheel(WheelDirection::Left),
                    41,
                    11,
                    Modifiers::NONE,
                ),
                mouse(
                    MouseKind::Wheel(WheelDirection::Right),
                    41,
                    11,
                    Modifiers::NONE,
                ),
            ],
        ),
        (
            b"\x1b[<0;20;10;M",
            vec![mouse(
                MouseKind::Press(MouseButton::Left),
                19,
                9,
                Modifiers::NONE,
            )],
        ),
        (
            b"\x1b[<0;20;10;m",
            vec![mouse(
                MouseKind::Release(MouseButton::Left),
                19,
                9,
                Modifiers::NONE,
            )],
        ),
    ]);
}

#[test]
fn other_mouse_buttons_are_no_event() {
    check(&[
        (b"\x1b[<128;1;1Ma", vec![char('a')]),
        (b"\x1b[<3;1;1Ma", vec![char('a')]),
    ]);
}

#[test]
fn malformed_mouse_report_reads_as_alt_bracket_and_keys() {
    let alt_bracket = || key(KeyCode::Char('['), Modifiers::ALT);
    check(&[
        (
            b"\x1b[<0;1Ma",
            vec![
                alt_bracket(),
                char('<'),
                char('0'),
                char(';'),
                char('1'),
                key(KeyCode::Char('M'), Modifiers::SHIFT),
                char('a'),
            ],
        ),
        (b"\x1b[<x", vec![alt_bracket(), char('<'), char('x')]),
        (
            b"\x1b[<1;;1M",
            vec![
                alt_bracket(),
                char('<'),
                char('1'),
                char(';'),
                char(';'),
                char('1'),
                key(KeyCode::Char('M'), Modifiers::SHIFT),
            ],
        ),
    ]);
}

#[test]
fn default_mouse_reports() {
    check(&[
        (
            b"\x1b[M *%",
            vec![mouse(
                MouseKind::Press(MouseButton::Left),
                9,
                4,
                Modifiers::NONE,
            )],
        ),
        (
            b"\x1b[M#*%",
            vec![mouse(
                MouseKind::Release(MouseButton::Left),
                9,
                4,
                Modifiers::NONE,
            )],
        ),
        (
            b"\x1b[M0\x60\x70",
            vec![mouse(
                MouseKind::Press(MouseButton::Left),
                63,
                79,
                Modifiers::CTRL,
            )],
        ),
        (
            b"\x1b[M\x60\xff\xff",
            vec![mouse(
                MouseKind::Wheel(WheelDirection::Up),
                222,
                222,
                Modifiers::NONE,
            )],
        ),
    ]);
}

#[test]
fn focus_gained() {
    check(&[(b"\x1b[I", vec![TerminalInput::Focus(true)])]);
}

#[test]
fn focus_lost() {
    check(&[(b"\x1b[O", vec![TerminalInput::Focus(false)])]);
}

#[test]
fn paste_holding_an_escape() {
    check(&[(b"\x1b[200~a\x1b[Ab\x1b[201~", vec![paste("a\x1b[Ab")])]);
}

#[test]
fn paste_then_keys() {
    check(&[(
        b"x\x1b[200~on and on\x1b[201~y",
        vec![char('x'), paste("on and on"), char('y')],
    )]);
}

#[test]
fn lone_escape_waits_for_the_flush() {
    let (inputs, holds) = pushed(&[b"\x1b"]);
    assert_eq!(inputs, vec![]);
    assert!(holds);
    assert_eq!(decoded(&[b"\x1b"]), vec![escape()]);
}

#[test]
fn arrow_split_after_its_escape() {
    let (inputs, holds) = pushed(&[b"\x1b", b"[A"]);
    assert_eq!(inputs, vec![plain(KeyCode::Up)]);
    assert!(!holds);
}

#[test]
fn mouse_report_split_mid_way() {
    let (inputs, holds) = pushed(&[b"\x1b[<35;10", b";5M"]);
    assert_eq!(
        inputs,
        vec![mouse(MouseKind::Motion(None), 9, 4, Modifiers::NONE)]
    );
    assert!(!holds);
}

#[test]
fn escape_then_a_later_key() {
    let mut decoder = Decoder::new();
    assert_eq!(decoder.push(b"\x1b"), vec![]);
    assert_eq!(decoder.flush(), vec![escape()]);
    assert_eq!(decoder.push(b"["), vec![char('[')]);
    assert!(!decoder.holds());
}

#[test]
fn flush_resolves_an_unfinished_sequence_as_alt_and_keys() {
    let alt_bracket = key(KeyCode::Char('['), Modifiers::ALT);
    check(&[
        (
            b"\x1b[<35;1",
            vec![
                alt_bracket.clone(),
                char('<'),
                char('3'),
                char('5'),
                char(';'),
                char('1'),
            ],
        ),
        (b"\x1b[11", vec![alt_bracket.clone(), char('1'), char('1')]),
        (
            b"\x1b[M ",
            vec![
                alt_bracket,
                key(KeyCode::Char('M'), Modifiers::SHIFT),
                char(' '),
            ],
        ),
        (b"\x1bO", vec![key(KeyCode::Char('O'), ALT_SHIFT)]),
    ]);
}

#[test]
fn flush_reads_escape_and_one_key_as_alt() {
    check(&[
        (b"\x1b[", vec![key(KeyCode::Char('['), Modifiers::ALT)]),
        (b"\x1bO", vec![key(KeyCode::Char('O'), ALT_SHIFT)]),
    ]);
}

#[test]
fn malformed_sequence_reads_as_alt_and_keys() {
    check(&[
        (
            b"\x1b[1\x01",
            vec![
                key(KeyCode::Char('['), Modifiers::ALT),
                char('1'),
                ctrl('a'),
            ],
        ),
        (
            b"\x1bO1",
            vec![key(KeyCode::Char('O'), ALT_SHIFT), char('1')],
        ),
    ]);
}

#[test]
fn flush_keeps_an_unfinished_paste() {
    let mut decoder = Decoder::new();
    assert_eq!(decoder.push(b"\x1b[200~12"), vec![]);
    assert_eq!(decoder.flush(), vec![]);
    assert!(decoder.holds());
    assert_eq!(decoder.push(b"34\x1b[201~"), vec![paste("1234")]);
    assert!(!decoder.holds());
}

#[test]
fn alt_with_a_letter() {
    check(&[(b"\x1bx", vec![key(KeyCode::Char('x'), Modifiers::ALT)])]);
}

#[test]
fn alt_with_other_keys() {
    check(&[
        (b"\x1bH", vec![key(KeyCode::Char('H'), ALT_SHIFT)]),
        (
            b"\x1b\x14",
            vec![key(
                KeyCode::Char('t'),
                Modifiers {
                    alt: true,
                    ..Modifiers::CTRL
                },
            )],
        ),
        (b"\x1b\r", vec![key(KeyCode::Enter, Modifiers::ALT)]),
        (b"\x1b\x7f", vec![key(KeyCode::Backspace, Modifiers::ALT)]),
        (
            "\x1bé".as_bytes(),
            vec![key(KeyCode::Char('é'), Modifiers::ALT)],
        ),
    ]);
}

#[test]
fn doubled_escape_before_an_arrow() {
    check(&[(b"\x1b\x1b[A", vec![key(KeyCode::Up, Modifiers::ALT)])]);
}

#[test]
fn alt_with_escape() {
    let (inputs, holds) = pushed(&[b"\x1b\x1b"]);
    assert_eq!(inputs, vec![]);
    assert!(holds);
    check(&[(b"\x1b\x1b", vec![key(KeyCode::Escape, Modifiers::ALT)])]);
}

#[test]
fn escape_before_a_key_that_holds_alt() {
    check(&[
        (
            b"\x1b\x1bx",
            vec![escape(), key(KeyCode::Char('x'), Modifiers::ALT)],
        ),
        (
            b"\x1b\x1b[1;3A",
            vec![escape(), key(KeyCode::Up, Modifiers::ALT)],
        ),
        (
            b"\x1b\x1b\x1b",
            vec![escape(), key(KeyCode::Escape, Modifiers::ALT)],
        ),
    ]);
}

#[test]
fn escape_merged_with_a_mouse_report() {
    check(&[(
        b"\x1b\x1b[<35;10;5M",
        vec![
            escape(),
            mouse(MouseKind::Motion(None), 9, 4, Modifiers::NONE),
        ],
    )]);
}

#[test]
fn escape_merged_with_a_paste() {
    check(&[(b"\x1b\x1b[200~hi\x1b[201~", vec![escape(), paste("hi")])]);
}

#[test]
fn escape_merged_with_a_focus_report() {
    check(&[(b"\x1b\x1b[I", vec![escape(), TerminalInput::Focus(true)])]);
}

#[test]
fn escape_merged_with_a_dropped_sequence() {
    check(&[(b"\x1b\x1b[99~", vec![escape()])]);
}

const NORMAL: Modes = Modes::DEFAULT;
const APPLICATION: Modes = Modes {
    application_cursor: true,
    ..Modes::DEFAULT
};

fn every_code() -> Vec<KeyCode> {
    let mut codes = vec![
        KeyCode::Enter,
        KeyCode::Tab,
        KeyCode::BackTab,
        KeyCode::Backspace,
        KeyCode::Escape,
        KeyCode::Up,
        KeyCode::Down,
        KeyCode::Right,
        KeyCode::Left,
        KeyCode::Home,
        KeyCode::End,
        KeyCode::Insert,
        KeyCode::Delete,
        KeyCode::PageUp,
        KeyCode::PageDown,
    ];
    codes.extend((1..=12).map(KeyCode::F));
    codes.extend((' '..='~').map(KeyCode::Char));
    codes.extend(['é', 'Ž', '中', '🦀'].map(KeyCode::Char));
    codes
}

fn every_key() -> Vec<Key> {
    every_code()
        .into_iter()
        .flat_map(|code| {
            every_modifiers()
                .into_iter()
                .map(move |modifiers| Key::new(code, modifiers))
        })
        .collect()
}

fn is_control_alias(key: Key) -> bool {
    key.modifiers.ctrl
        && matches!(
            key.code,
            KeyCode::Backspace | KeyCode::Char('i' | 'I' | 'm' | 'M' | '[')
        )
}

#[test]
fn encoded_keys_decode_to_the_same_bytes() {
    for modes in [NORMAL, APPLICATION] {
        for original in every_key() {
            let bytes = encode_key(original, modes);
            let inputs = decoded(&[&bytes]);
            if is_control_alias(original) {
                continue;
            }
            let [TerminalInput::Key(decoded)] = inputs.as_slice() else {
                panic!("{original:?} as {bytes:?} decoded to {inputs:?}");
            };
            assert_eq!(
                encode_key(*decoded, modes),
                bytes,
                "{original:?} decoded to {decoded:?}"
            );
        }
    }
}

#[test]
fn encoded_keys_decode_to_the_same_key() {
    let exact = |key: Key| match key.code {
        KeyCode::Char(c) => {
            !key.modifiers.ctrl && key.modifiers.shift == c.is_uppercase()
                || key.modifiers.ctrl
                    && !key.modifiers.shift
                    && (c.is_ascii_lowercase() || matches!(c, ' ' | '4'..='7'))
        }
        KeyCode::Enter | KeyCode::Tab | KeyCode::Escape | KeyCode::BackTab => {
            !key.modifiers.shift && !key.modifiers.ctrl
        }
        KeyCode::Backspace => !key.modifiers.shift && !key.modifiers.ctrl,
        _ => true,
    };
    for modes in [NORMAL, APPLICATION] {
        for original in every_key().into_iter().filter(|&key| exact(key)) {
            if is_control_alias(original) {
                continue;
            }
            let bytes = encode_key(original, modes);
            assert_eq!(
                decoded(&[&bytes]),
                vec![TerminalInput::Key(original)],
                "{original:?} as {bytes:?}"
            );
        }
    }
}

#[test]
fn control_aliases_decode_to_their_twin() {
    check(&[
        (
            &encode_key(Key::new(KeyCode::Backspace, Modifiers::CTRL), NORMAL),
            vec![ctrl('h')],
        ),
        (
            &encode_key(Key::new(KeyCode::Char('i'), Modifiers::CTRL), NORMAL),
            vec![plain(KeyCode::Tab)],
        ),
        (
            &encode_key(Key::new(KeyCode::Char('m'), Modifiers::CTRL), NORMAL),
            vec![plain(KeyCode::Enter)],
        ),
        (
            &encode_key(Key::new(KeyCode::Char('['), Modifiers::CTRL), NORMAL),
            vec![escape()],
        ),
    ]);
}

const SGR: Modes = Modes {
    mouse_tracking: MouseTracking::AnyMotion,
    mouse_encoding: MouseEncoding::Sgr,
    ..Modes::DEFAULT
};
const DEFAULT_MOUSE: Modes = Modes {
    mouse_tracking: MouseTracking::AnyMotion,
    mouse_encoding: MouseEncoding::Default,
    ..Modes::DEFAULT
};

fn every_mouse_kind() -> Vec<MouseKind> {
    let buttons = [MouseButton::Left, MouseButton::Middle, MouseButton::Right];
    let mut kinds = Vec::new();
    for button in buttons {
        kinds.push(MouseKind::Press(button));
        kinds.push(MouseKind::Release(button));
        kinds.push(MouseKind::Motion(Some(button)));
    }
    kinds.push(MouseKind::Motion(None));
    kinds.extend(
        [
            WheelDirection::Up,
            WheelDirection::Down,
            WheelDirection::Left,
            WheelDirection::Right,
        ]
        .map(MouseKind::Wheel),
    );
    kinds
}

fn every_mouse_event() -> Vec<MouseEvent> {
    let mut events = Vec::new();
    for kind in every_mouse_kind() {
        for modifiers in every_modifiers() {
            for (col, row) in [(0, 0), (9, 4), (94, 95), (222, 222)] {
                events.push(MouseEvent::new(kind, col, row, modifiers));
            }
        }
    }
    events
}

#[test]
fn encoded_mouse_events_decode_to_the_same_event() {
    for event in every_mouse_event() {
        let sgr = encode_mouse(event, SGR);
        assert_eq!(
            decoded(&[&sgr]),
            vec![TerminalInput::Mouse(event)],
            "{event:?}"
        );
        let default = encode_mouse(event, DEFAULT_MOUSE);
        let expected = match event.kind {
            MouseKind::Release(_) => MouseEvent {
                kind: MouseKind::Release(MouseButton::Left),
                ..event
            },
            _ => event,
        };
        assert_eq!(
            decoded(&[&default]),
            vec![TerminalInput::Mouse(expected)],
            "{event:?}"
        );
    }
}

#[test]
fn sgr_mouse_beyond_the_default_range() {
    let event = MouseEvent::new(
        MouseKind::Press(MouseButton::Left),
        1000,
        300,
        Modifiers::NONE,
    );
    assert_eq!(
        decoded(&[&encode_mouse(event, SGR)]),
        vec![TerminalInput::Mouse(event)]
    );
}

fn samples() -> Vec<Vec<u8>> {
    let mut samples: Vec<Vec<u8>> = every_key()
        .into_iter()
        .map(|key| encode_key(key, NORMAL))
        .chain(
            every_key()
                .into_iter()
                .map(|key| encode_key(key, APPLICATION)),
        )
        .collect();
    for event in every_mouse_event() {
        samples.push(encode_mouse(event, SGR));
        samples.push(encode_mouse(event, DEFAULT_MOUSE));
    }
    samples.push(b"\x1b[I".to_vec());
    samples.push(b"\x1b[O".to_vec());
    let bracketed = Modes {
        bracketed_paste: true,
        ..Modes::DEFAULT
    };
    samples.push(encode_paste("hello\nworld", bracketed));
    samples.push(encode_paste("", bracketed));
    samples.push(b"\x1b[200~a\x1b[Ab\x1b\x1b\x1b[201~".to_vec());
    samples.sort();
    samples.dedup();
    samples
}

fn with_escape_before(inputs: Vec<TerminalInput>) -> Vec<TerminalInput> {
    match inputs.as_slice() {
        [TerminalInput::Key(Key { code, modifiers })] if !modifiers.alt => vec![key(
            *code,
            Modifiers {
                alt: true,
                ..*modifiers
            },
        )],
        _ => std::iter::once(escape()).chain(inputs).collect(),
    }
}

#[test]
fn byte_at_a_time_decodes_the_same() {
    for sample in samples() {
        let whole = decoded(&[&sample]);
        let bytes: Vec<&[u8]> = sample.chunks(1).collect();
        assert_eq!(decoded(&bytes), whole, "{sample:?}");
    }
}

#[test]
fn every_split_decodes_the_same() {
    for sample in samples() {
        let whole = decoded(&[&sample]);
        for at in 1..sample.len() {
            let (head, tail) = sample.split_at(at);
            assert_eq!(decoded(&[head, tail]), whole, "{sample:?} split at {at}");
        }
    }
}

#[test]
fn escape_merged_before_decodes_as_its_prefix() {
    for sample in samples() {
        let whole = decoded(&[&sample]);
        let merged = [b"\x1b".as_slice(), &sample].concat();
        assert_eq!(decoded(&[&merged]), with_escape_before(whole), "{sample:?}");
    }
}

fn alt(code: KeyCode) -> TerminalInput {
    key(code, Modifiers::ALT)
}

fn decoded_with_flushes(chunks: &[Option<&[u8]>]) -> Vec<TerminalInput> {
    let mut decoder = Decoder::new();
    let mut inputs = Vec::new();
    for chunk in chunks {
        match chunk {
            Some(bytes) => inputs.extend(decoder.push(bytes)),
            None => inputs.extend(decoder.flush()),
        }
    }
    inputs
}

type Chunks<'a> = &'a [Option<&'a [u8]>];

fn check_with_flushes(cases: &[(Chunks, Vec<TerminalInput>)]) {
    for (chunks, expected) in cases {
        assert_eq!(&decoded_with_flushes(chunks), expected, "{chunks:?}");
    }
}

#[test]
fn reference_key_vectors() {
    check(&[
        (b"\x1b[1;3A", vec![alt(KeyCode::Up)]),
        (b"\x1b\x7f", vec![alt(KeyCode::Backspace)]),
        (b"\x02", vec![ctrl('b')]),
        (b"\x1b[14~", vec![plain(KeyCode::F(4))]),
        (b"\x1b[11;2~", vec![key(KeyCode::F(1), Modifiers::SHIFT)]),
        (b"\x1b[14;3~", vec![alt(KeyCode::F(4))]),
        (
            b"\x1bOz",
            vec![key(KeyCode::Char('O'), ALT_SHIFT), char('z')],
        ),
        (
            b"\x03\x1bJ\x7f",
            vec![
                ctrl('c'),
                key(KeyCode::Char('J'), ALT_SHIFT),
                plain(KeyCode::Backspace),
            ],
        ),
        (
            b"\x1bOA\x1bOB\x1bOC\x1bOD",
            vec![
                plain(KeyCode::Up),
                plain(KeyCode::Down),
                plain(KeyCode::Right),
                plain(KeyCode::Left),
            ],
        ),
        (b"\n\r", vec![ctrl('j'), plain(KeyCode::Enter)]),
        (b"\x1b[1;3D", vec![alt(KeyCode::Left)]),
        (b"\x1b[1;3H", vec![alt(KeyCode::Home)]),
        (b"\x1b[1;3F", vec![alt(KeyCode::End)]),
        (b"\x1b\t", vec![alt(KeyCode::Tab)]),
        (b"\x1b[5;3~", vec![alt(KeyCode::PageUp)]),
        (b"\x1b[2;2~", vec![key(KeyCode::Insert, Modifiers::SHIFT)]),
        (b"\x1b[1;2F", vec![key(KeyCode::End, Modifiers::SHIFT)]),
        (b"\x1b[3;2~", vec![key(KeyCode::Delete, Modifiers::SHIFT)]),
        (b"\x1b[x", vec![alt(KeyCode::Char('[')), char('x')]),
        (b"\x1b\x1b[D", vec![alt(KeyCode::Left)]),
        (
            "\x1béx".as_bytes(),
            vec![alt(KeyCode::Char('é')), char('x')],
        ),
        (b"c", vec![char('c')]),
        (b"C", vec![key(KeyCode::Char('C'), Modifiers::SHIFT)]),
        (b"\x1bc", vec![alt(KeyCode::Char('c'))]),
    ]);
}

#[test]
fn reference_escape_timing_vectors() {
    check_with_flushes(&[
        (&[Some(b"\x1b"), Some(b"[B")], vec![plain(KeyCode::Down)]),
        (&[Some(b"\x1b"), Some(b"b")], vec![alt(KeyCode::Char('b'))]),
        (
            &[Some(b"\x1b\xc3"), None, Some(b"\xa9")],
            vec![alt(KeyCode::Char('é'))],
        ),
        (&[Some(b"\x1b[11"), Some(b"~")], vec![plain(KeyCode::F(1))]),
        (
            &[Some(b"\x1b"), Some(b"\x1b[<35;10;20M"), None],
            vec![
                escape(),
                mouse(MouseKind::Motion(None), 9, 19, Modifiers::NONE),
            ],
        ),
        (
            &[Some(b"\x1b"), Some(b"\x1b[MCN1"), None],
            vec![
                escape(),
                mouse(MouseKind::Motion(None), 45, 16, Modifiers::NONE),
            ],
        ),
        (
            &[Some(b"\x1b"), Some(b"[<65;43;26M")],
            vec![mouse(
                MouseKind::Wheel(WheelDirection::Down),
                42,
                25,
                Modifiers::NONE,
            )],
        ),
        (
            &[Some(b"\x1b["), Some(b"I")],
            vec![TerminalInput::Focus(true)],
        ),
    ]);
}

#[test]
fn reference_utf8_vectors() {
    check_with_flushes(&[
        (&[Some(b"\xc3"), Some(b"\xa9")], vec![char('é')]),
        (
            &[Some(b"\xe5"), None, Some(b"\xa5"), None, Some(b"\xbd")],
            vec![char('好')],
        ),
        (
            &[
                Some(b"\xf0"),
                None,
                Some(b"\x9f"),
                None,
                Some(b"\x99"),
                None,
                Some(b"\x82"),
            ],
            vec![char('🙂')],
        ),
        (&[Some(b"\xe4\xbd\xa0\xe5")], vec![char('你')]),
        (
            &[Some(b"\xe4\xbd\xa0\xe5"), None, Some(b"\xa5\xbd")],
            vec![char('你'), char('好')],
        ),
        (&[Some(b"\xc0"), None], vec![]),
    ]);
    let text: String = "漢字かなカナ한글🙂".repeat(200);
    let bytes: Vec<&[u8]> = text.as_bytes().chunks(1).collect();
    let expected: Vec<TerminalInput> = text.chars().map(char).collect();
    assert_eq!(decoded(&[text.as_bytes()]), expected);
    assert_eq!(decoded(&bytes), expected);
}

#[test]
fn reference_paste_vectors() {
    check_with_flushes(&[
        (
            &[Some(b"\x1b[200~hello\x1b[201~rest")],
            vec![paste("hello"), char('r'), char('e'), char('s'), char('t')],
        ),
        (
            &[Some(b"\x1b[200~hello"), Some(b"\x1b[201~")],
            vec![paste("hello")],
        ),
        (
            &[Some(b"\x1b[200~hello\nworld"), None, Some(b"\x1b[201~")],
            vec![paste("hello\nworld")],
        ),
        (&[Some(b"\x1b[200~"), None], vec![]),
        (&[Some(b"\x1b[200~hel"), None], vec![]),
    ]);
    check(&[(
        b"\x1b[O\x1b[I",
        vec![TerminalInput::Focus(false), TerminalInput::Focus(true)],
    )]);
}

#[test]
fn reference_mouse_vectors() {
    let none = Modifiers::NONE;
    let left = MouseKind::Press(MouseButton::Left);
    let alt_ctrl = Modifiers {
        alt: true,
        ..Modifiers::CTRL
    };
    check(&[
        (b"\x1b[<0;20;10M", vec![mouse(left, 19, 9, none)]),
        (b"\x1b[<8;20;10M", vec![mouse(left, 19, 9, Modifiers::ALT)]),
        (
            b"\x1b[<16;20;10M",
            vec![mouse(left, 19, 9, Modifiers::CTRL)],
        ),
        (b"\x1b[<24;1;1M", vec![mouse(left, 0, 0, alt_ctrl)]),
        (
            b"\x1b[MCN1",
            vec![mouse(MouseKind::Motion(None), 45, 16, none)],
        ),
        (b"\x1b[M\x82AAx", vec![char('x')]),
        (
            b"\x1b[<0;42;12m",
            vec![mouse(MouseKind::Release(MouseButton::Left), 41, 11, none)],
        ),
        (
            b"\x1b[<6;10;20M",
            vec![mouse(
                MouseKind::Press(MouseButton::Right),
                9,
                19,
                Modifiers::SHIFT,
            )],
        ),
        (
            b"\x1b[<32;5;5M",
            vec![mouse(
                MouseKind::Motion(Some(MouseButton::Left)),
                4,
                4,
                none,
            )],
        ),
        (
            b"\x1b[<64;1;1M",
            vec![mouse(MouseKind::Wheel(WheelDirection::Up), 0, 0, none)],
        ),
        (b"\x1b[<0;999;999M", vec![mouse(left, 998, 998, none)]),
        (
            b"\x1b[<0;1;1Mhello",
            std::iter::once(mouse(left, 0, 0, none))
                .chain("hello".chars().map(char))
                .collect(),
        ),
        (
            b"\x1b[<0;1;1M\x1b[<0;2;2M",
            vec![mouse(left, 0, 0, none), mouse(left, 1, 1, none)],
        ),
        (
            b"a\x1b[<35;52;16M\x1b[<35;49;16M",
            vec![
                char('a'),
                mouse(MouseKind::Motion(None), 51, 15, none),
                mouse(MouseKind::Motion(None), 48, 15, none),
            ],
        ),
        (
            b"\x1b\x1b[<35;42;12M",
            vec![escape(), mouse(MouseKind::Motion(None), 41, 11, none)],
        ),
        (
            b"\x1b[<0;1Ma",
            vec![
                alt(KeyCode::Char('[')),
                char('<'),
                char('0'),
                char(';'),
                char('1'),
                key(KeyCode::Char('M'), Modifiers::SHIFT),
                char('a'),
            ],
        ),
    ]);
    check_with_flushes(&[
        (
            &[Some(b"\x1b[<3"), Some(b"5;58;30M")],
            vec![mouse(MouseKind::Motion(None), 57, 29, none)],
        ),
        (
            &[Some(b"\x1b"), Some(b"\x1b[<35;62;16M")],
            vec![escape(), mouse(MouseKind::Motion(None), 61, 15, none)],
        ),
    ]);
}

#[test]
fn reference_unfinished_mouse_report_vectors() {
    check_with_flushes(&[(
        &[Some(b"\x1b[<"), Some(b"0;0;0"), None],
        vec![
            alt(KeyCode::Char('[')),
            char('<'),
            char('0'),
            char(';'),
            char('0'),
            char(';'),
            char('0'),
        ],
    )]);
}
