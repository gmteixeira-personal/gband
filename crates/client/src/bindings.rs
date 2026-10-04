use gband_core::input::{Key, KeyCode, Modifiers};
use gband_core::layout::Direction;
use gband_core::view::ViewAction;

pub const PREFIX: Key = Key {
    code: KeyCode::Char('a'),
    modifiers: Modifiers::CTRL,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionCommand {
    OpenPane,
    ClosePane,
    ConsumeOrExpel(Direction),
    CycleWidth,
    ToggleFullWidth,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Binding {
    View(ViewAction),
    Session(SessionCommand),
    Detach,
    SendPrefix,
}

const fn char_key(c: char) -> Key {
    Key {
        code: KeyCode::Char(c),
        modifiers: Modifiers::NONE,
    }
}

pub const BINDINGS: &[(Key, Binding)] = &[
    (char_key('h'), Binding::View(ViewAction::FocusLeft)),
    (char_key('l'), Binding::View(ViewAction::FocusRight)),
    (char_key('j'), Binding::View(ViewAction::FocusDown)),
    (char_key('k'), Binding::View(ViewAction::FocusUp)),
    (char_key('u'), Binding::View(ViewAction::WorkspaceDown)),
    (char_key('i'), Binding::View(ViewAction::WorkspaceUp)),
    (
        Key {
            code: KeyCode::Enter,
            modifiers: Modifiers::NONE,
        },
        Binding::Session(SessionCommand::OpenPane),
    ),
    (char_key('q'), Binding::Session(SessionCommand::ClosePane)),
    (
        char_key('['),
        Binding::Session(SessionCommand::ConsumeOrExpel(Direction::Left)),
    ),
    (
        char_key(']'),
        Binding::Session(SessionCommand::ConsumeOrExpel(Direction::Right)),
    ),
    (char_key('r'), Binding::Session(SessionCommand::CycleWidth)),
    (
        char_key('f'),
        Binding::Session(SessionCommand::ToggleFullWidth),
    ),
    (char_key('D'), Binding::Detach),
    (PREFIX, Binding::SendPrefix),
];

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Send(Key),
    Run(Binding),
    Discard,
}

#[derive(Debug, Default)]
pub struct Leader {
    after_prefix: bool,
}

impl Leader {
    pub fn handle(&mut self, key: Key) -> Command {
        if !std::mem::take(&mut self.after_prefix) {
            if matches(PREFIX, key) {
                self.after_prefix = true;
                return Command::Discard;
            }
            return Command::Send(key);
        }
        BINDINGS
            .iter()
            .find(|(bound, _)| matches(*bound, key))
            .map_or(Command::Discard, |&(_, binding)| Command::Run(binding))
    }
}

fn matches(bound: Key, key: Key) -> bool {
    match (bound.code, key.code) {
        (KeyCode::Char(expected), KeyCode::Char(pressed)) => {
            expected == pressed
                && bound.modifiers.ctrl == key.modifiers.ctrl
                && bound.modifiers.alt == key.modifiers.alt
        }
        _ => bound == key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn after_prefix(key: Key) -> Command {
        let mut leader = Leader::default();
        assert_eq!(leader.handle(PREFIX), Command::Discard);
        leader.handle(key)
    }

    #[test]
    fn every_table_entry_runs_after_the_prefix() {
        for &(key, binding) in BINDINGS {
            assert_eq!(after_prefix(key), Command::Run(binding), "{key:?}");
        }
    }

    #[test]
    fn table_keys_follow_the_spec() {
        let expected = [
            ('h', Binding::View(ViewAction::FocusLeft)),
            ('l', Binding::View(ViewAction::FocusRight)),
            ('j', Binding::View(ViewAction::FocusDown)),
            ('k', Binding::View(ViewAction::FocusUp)),
            ('u', Binding::View(ViewAction::WorkspaceDown)),
            ('i', Binding::View(ViewAction::WorkspaceUp)),
            ('q', Binding::Session(SessionCommand::ClosePane)),
            (
                '[',
                Binding::Session(SessionCommand::ConsumeOrExpel(Direction::Left)),
            ),
            (
                ']',
                Binding::Session(SessionCommand::ConsumeOrExpel(Direction::Right)),
            ),
            ('r', Binding::Session(SessionCommand::CycleWidth)),
            ('f', Binding::Session(SessionCommand::ToggleFullWidth)),
        ];
        for (c, binding) in expected {
            assert_eq!(after_prefix(char_key(c)), Command::Run(binding), "{c}");
        }
        assert_eq!(
            after_prefix(Key::plain(KeyCode::Enter)),
            Command::Run(Binding::Session(SessionCommand::OpenPane))
        );
    }

    #[test]
    fn shift_d_detaches_with_or_without_the_shift_flag() {
        assert_eq!(
            after_prefix(Key::new(KeyCode::Char('D'), Modifiers::SHIFT)),
            Command::Run(Binding::Detach)
        );
        assert_eq!(after_prefix(char_key('D')), Command::Run(Binding::Detach));
    }

    #[test]
    fn lowercase_d_is_discarded() {
        assert_eq!(after_prefix(char_key('d')), Command::Discard);
    }

    #[test]
    fn prefix_twice_sends_one_prefix() {
        let mut leader = Leader::default();
        assert_eq!(leader.handle(PREFIX), Command::Discard);
        assert_eq!(leader.handle(PREFIX), Command::Run(Binding::SendPrefix));
        assert_eq!(leader.handle(char_key('h')), Command::Send(char_key('h')));
    }

    #[test]
    fn unbound_key_after_the_prefix_is_discarded() {
        let mut leader = Leader::default();
        assert_eq!(leader.handle(PREFIX), Command::Discard);
        assert_eq!(leader.handle(char_key('x')), Command::Discard);
        assert_eq!(leader.handle(char_key('x')), Command::Send(char_key('x')));
    }

    #[test]
    fn modifiers_other_than_shift_must_match() {
        assert_eq!(
            after_prefix(Key::new(KeyCode::Char('h'), Modifiers::ALT)),
            Command::Discard
        );
        assert_eq!(
            after_prefix(Key::new(KeyCode::Char('q'), Modifiers::CTRL)),
            Command::Discard
        );
    }

    #[test]
    fn keys_without_the_prefix_pass_through() {
        let mut leader = Leader::default();
        let up = Key::plain(KeyCode::Up);
        let ctrl_b = Key::new(KeyCode::Char('b'), Modifiers::CTRL);
        assert_eq!(leader.handle(char_key('h')), Command::Send(char_key('h')));
        assert_eq!(leader.handle(up), Command::Send(up));
        assert_eq!(leader.handle(ctrl_b), Command::Send(ctrl_b));
    }
}
