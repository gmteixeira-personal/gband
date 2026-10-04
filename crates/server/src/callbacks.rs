pub struct LoggingCallbacks;

impl vt100::Callbacks for LoggingCallbacks {
    fn audible_bell(&mut self, _: &mut vt100::Screen) {
        tracing::debug!("bell");
    }

    fn unhandled_control(&mut self, _: &mut vt100::Screen, b: u8) {
        tracing::debug!("unhandled control {b:#04x}");
    }

    fn unhandled_escape(&mut self, _: &mut vt100::Screen, i1: Option<u8>, i2: Option<u8>, b: u8) {
        let sequence = [i1, i2, Some(b)].into_iter().flatten().map(char::from);
        tracing::debug!("unhandled escape \\e{}", sequence.collect::<String>());
    }

    fn unhandled_csi(
        &mut self,
        _: &mut vt100::Screen,
        i1: Option<u8>,
        i2: Option<u8>,
        params: &[&[u16]],
        c: char,
    ) {
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

    fn unhandled_osc(&mut self, _: &mut vt100::Screen, params: &[&[u8]]) {
        let params = params
            .iter()
            .map(|param| String::from_utf8_lossy(param))
            .collect::<Vec<_>>()
            .join(";");
        tracing::debug!("unhandled OSC \\e]{params}");
    }
}
