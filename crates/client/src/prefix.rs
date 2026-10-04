use gband_core::input::{Key, KeyCode, Modifiers};

pub const PREFIX: Key = Key {
    code: KeyCode::Char('a'),
    modifiers: Modifiers::CTRL,
};
const DETACH: Key = Key {
    code: KeyCode::Char('d'),
    modifiers: Modifiers::NONE,
};

#[derive(Debug, PartialEq, Eq)]
pub enum Action {
    Send(Key),
    Detach,
    Discard,
}

#[derive(Debug, Default)]
pub struct PrefixState {
    after_prefix: bool,
}

impl PrefixState {
    pub fn handle(&mut self, key: Key) -> Action {
        if !std::mem::take(&mut self.after_prefix) {
            if key == PREFIX {
                self.after_prefix = true;
                return Action::Discard;
            }
            return Action::Send(key);
        }
        match key {
            PREFIX => Action::Send(PREFIX),
            DETACH => Action::Detach,
            _ => Action::Discard,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn char_key(c: char) -> Key {
        Key::plain(KeyCode::Char(c))
    }

    #[test]
    fn prefix_then_d_detaches() {
        let mut state = PrefixState::default();
        assert_eq!(state.handle(PREFIX), Action::Discard);
        assert_eq!(state.handle(char_key('d')), Action::Detach);
    }

    #[test]
    fn prefix_twice_sends_one_prefix() {
        let mut state = PrefixState::default();
        assert_eq!(state.handle(PREFIX), Action::Discard);
        assert_eq!(state.handle(PREFIX), Action::Send(PREFIX));
        assert_eq!(state.handle(char_key('d')), Action::Send(char_key('d')));
    }

    #[test]
    fn unbound_key_after_prefix_is_discarded() {
        let mut state = PrefixState::default();
        assert_eq!(state.handle(PREFIX), Action::Discard);
        assert_eq!(state.handle(char_key('x')), Action::Discard);
        assert_eq!(state.handle(char_key('x')), Action::Send(char_key('x')));
    }

    #[test]
    fn keys_without_prefix_pass_through() {
        let mut state = PrefixState::default();
        let up = Key::plain(KeyCode::Up);
        let ctrl_b = Key::new(KeyCode::Char('b'), Modifiers::CTRL);
        assert_eq!(state.handle(char_key('d')), Action::Send(char_key('d')));
        assert_eq!(state.handle(up), Action::Send(up));
        assert_eq!(state.handle(ctrl_b), Action::Send(ctrl_b));
    }
}
