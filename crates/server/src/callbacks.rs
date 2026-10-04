use vt100::{MouseProtocolEncoding, MouseProtocolMode, Screen};

#[derive(Default)]
pub struct PaneCallbacks {
    pub replies: Vec<u8>,
}

impl vt100::Callbacks for PaneCallbacks {
    fn audible_bell(&mut self, _: &mut Screen) {
        tracing::debug!("bell");
    }

    fn unhandled_control(&mut self, _: &mut Screen, b: u8) {
        tracing::debug!("unhandled control {b:#04x}");
    }

    fn unhandled_escape(&mut self, _: &mut Screen, i1: Option<u8>, i2: Option<u8>, b: u8) {
        let sequence = [i1, i2, Some(b)].into_iter().flatten().map(char::from);
        tracing::debug!("unhandled escape \\e{}", sequence.collect::<String>());
    }

    fn unhandled_csi(
        &mut self,
        screen: &mut Screen,
        i1: Option<u8>,
        i2: Option<u8>,
        params: &[&[u16]],
        c: char,
    ) {
        if let Some(reply) = query_reply(screen, i1, i2, params, c) {
            self.replies.extend_from_slice(reply.as_bytes());
            return;
        }
        let params = params
            .iter()
            .map(|param| {
                param
                    .iter()
                    .map(u16::to_string)
                    .collect::<Vec<_>>()
                    .join(":")
            })
            .collect::<Vec<_>>()
            .join(";");
        let (private, intermediates): (Vec<u8>, Vec<u8>) = [i1, i2]
            .into_iter()
            .flatten()
            .partition(|byte| (b'<'..=b'?').contains(byte));
        let private = String::from_utf8_lossy(&private);
        let intermediates = String::from_utf8_lossy(&intermediates);
        tracing::debug!("unhandled CSI \\e[{private}{params}{intermediates}{c}");
    }

    fn unhandled_osc(&mut self, _: &mut Screen, params: &[&[u8]]) {
        let params = params
            .iter()
            .map(|param| String::from_utf8_lossy(param))
            .collect::<Vec<_>>()
            .join(";");
        tracing::debug!("unhandled OSC \\e]{params}");
    }
}

fn query_reply(
    screen: &Screen,
    i1: Option<u8>,
    i2: Option<u8>,
    params: &[&[u16]],
    c: char,
) -> Option<String> {
    let first = params.first().and_then(|param| param.first()).copied();
    match (i1, i2, c) {
        (None, None, 'c') if matches!(first, None | Some(0)) => Some("\x1b[?62;22c".to_owned()),
        (None, None, 'n') => match first? {
            5 => Some("\x1b[0n".to_owned()),
            6 => {
                let (row, col) = screen.cursor_position();
                Some(format!("\x1b[{};{}R", row + 1, col + 1))
            }
            _ => None,
        },
        (Some(b'>'), None, 'q') if matches!(first, None | Some(0)) => {
            Some(format!("\x1bP>|gband {}\x1b\\", env!("CARGO_PKG_VERSION")))
        }
        (Some(b'?'), Some(b'$'), 'p') => {
            let mode = first?;
            Some(format!(
                "\x1b[?{mode};{}$y",
                private_mode_state(screen, mode)
            ))
        }
        (Some(b'$'), None, 'p') => Some(format!("\x1b[{};0$y", first?)),
        _ => None,
    }
}

fn private_mode_state(screen: &Screen, mode: u16) -> u8 {
    let set = match mode {
        1 => screen.application_cursor(),
        9 => screen.mouse_protocol_mode() == MouseProtocolMode::Press,
        25 => !screen.hide_cursor(),
        47 | 1049 => screen.alternate_screen(),
        1000 => screen.mouse_protocol_mode() == MouseProtocolMode::PressRelease,
        1002 => screen.mouse_protocol_mode() == MouseProtocolMode::ButtonMotion,
        1003 => screen.mouse_protocol_mode() == MouseProtocolMode::AnyMotion,
        1005 => screen.mouse_protocol_encoding() == MouseProtocolEncoding::Utf8,
        1006 => screen.mouse_protocol_encoding() == MouseProtocolEncoding::Sgr,
        2004 => screen.bracketed_paste(),
        _ => return 0,
    };
    if set { 1 } else { 2 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn replies(output: &str) -> String {
        let mut parser = vt100::Parser::new_with_callbacks(24, 80, 0, PaneCallbacks::default());
        parser.process(output.as_bytes());
        String::from_utf8(std::mem::take(&mut parser.callbacks_mut().replies)).unwrap()
    }

    #[test]
    fn device_attributes_claim_a_vt220_with_colour() {
        assert_eq!(replies("\x1b[c"), "\x1b[?62;22c");
        assert_eq!(replies("\x1b[0c"), "\x1b[?62;22c");
    }

    #[test]
    fn operating_status_is_ok() {
        assert_eq!(replies("\x1b[5n"), "\x1b[0n");
    }

    #[test]
    fn cursor_position_is_read_at_the_query() {
        assert_eq!(replies("\x1b[6n"), "\x1b[1;1R");
        assert_eq!(replies("\x1b[5;10H\x1b[6n\x1b[1;1H"), "\x1b[5;10R");
    }

    #[test]
    fn version_names_gband() {
        let expected = format!("\x1bP>|gband {}\x1b\\", env!("CARGO_PKG_VERSION"));
        assert_eq!(replies("\x1b[>q"), expected);
        assert_eq!(replies("\x1b[>0q"), expected);
    }

    #[test]
    fn private_modes_report_their_state() {
        assert_eq!(replies("\x1b[?1$p"), "\x1b[?1;2$y");
        assert_eq!(replies("\x1b[?1h\x1b[?1$p"), "\x1b[?1;1$y");
        assert_eq!(replies("\x1b[?25$p"), "\x1b[?25;1$y");
        assert_eq!(replies("\x1b[?25l\x1b[?25$p"), "\x1b[?25;2$y");
        assert_eq!(
            replies("\x1b[?1049h\x1b[?47$p\x1b[?1049$p"),
            "\x1b[?47;1$y\x1b[?1049;1$y"
        );
        assert_eq!(
            replies("\x1b[?1002h\x1b[?9$p\x1b[?1000$p\x1b[?1002$p\x1b[?1003$p"),
            "\x1b[?9;2$y\x1b[?1000;2$y\x1b[?1002;1$y\x1b[?1003;2$y"
        );
        assert_eq!(
            replies("\x1b[?1006h\x1b[?1005$p\x1b[?1006$p"),
            "\x1b[?1005;2$y\x1b[?1006;1$y"
        );
        assert_eq!(replies("\x1b[?2004$p"), "\x1b[?2004;2$y");
        assert_eq!(replies("\x1b[?2004h\x1b[?2004$p"), "\x1b[?2004;1$y");
    }

    #[test]
    fn unrecognised_modes_report_zero() {
        assert_eq!(replies("\x1b[?7727$p"), "\x1b[?7727;0$y");
        assert_eq!(replies("\x1b[?6$p"), "\x1b[?6;0$y");
        assert_eq!(replies("\x1b[4$p"), "\x1b[4;0$y");
    }

    #[test]
    fn unsupported_queries_stay_unanswered() {
        assert_eq!(replies("\x1b[?u"), "");
        assert_eq!(replies("\x1b]11;?\x07"), "");
        assert_eq!(replies("\x1b[999z"), "");
    }
}
