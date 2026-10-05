use gband_core::action::{Action, ClientAction, SessionCommand};
use gband_core::input::{Key, KeyCode, Modifiers};
use gband_core::layout::{Direction, Step};
use gband_core::view::ViewAction;

pub const PREFIX: Key = Key {
    code: KeyCode::Char('a'),
    modifiers: Modifiers::CTRL,
};

const fn char_key(c: char) -> Key {
    Key {
        code: KeyCode::Char(c),
        modifiers: Modifiers::NONE,
    }
}

pub const BINDINGS: &[(Key, Action)] = &[
    (char_key('h'), Action::View(ViewAction::FocusLeft)),
    (char_key('l'), Action::View(ViewAction::FocusRight)),
    (char_key('j'), Action::View(ViewAction::FocusDown)),
    (char_key('k'), Action::View(ViewAction::FocusUp)),
    (char_key('u'), Action::View(ViewAction::WorkspaceDown)),
    (char_key('i'), Action::View(ViewAction::WorkspaceUp)),
    (
        Key {
            code: KeyCode::Enter,
            modifiers: Modifiers::NONE,
        },
        Action::Session(SessionCommand::OpenPane),
    ),
    (char_key('q'), Action::Session(SessionCommand::ClosePane)),
    (
        char_key('['),
        Action::Session(SessionCommand::ConsumeOrExpel(Direction::Left)),
    ),
    (
        char_key(']'),
        Action::Session(SessionCommand::ConsumeOrExpel(Direction::Right)),
    ),
    (char_key('r'), Action::Session(SessionCommand::CycleWidth)),
    (
        char_key('f'),
        Action::Session(SessionCommand::ToggleFullWidth),
    ),
    (
        char_key('-'),
        Action::Session(SessionCommand::StepWidth(Step::Shrink)),
    ),
    (
        char_key('='),
        Action::Session(SessionCommand::StepWidth(Step::Grow)),
    ),
    (
        char_key('_'),
        Action::Session(SessionCommand::StepHeight(Step::Shrink)),
    ),
    (
        char_key('+'),
        Action::Session(SessionCommand::StepHeight(Step::Grow)),
    ),
    (char_key('R'), Action::Session(SessionCommand::ResetHeight)),
    (char_key('D'), Action::Client(ClientAction::Detach)),
    (PREFIX, Action::Client(ClientAction::SendKey(PREFIX))),
];

#[derive(Debug, PartialEq, Eq)]
pub enum Command {
    Send(Key),
    Run(Action),
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
            .map_or(Command::Discard, |&(_, action)| Command::Run(action))
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
        for &(key, action) in BINDINGS {
            assert_eq!(after_prefix(key), Command::Run(action), "{key:?}");
        }
    }

    #[test]
    fn table_keys_follow_the_spec() {
        let expected = [
            ('h', Action::View(ViewAction::FocusLeft)),
            ('l', Action::View(ViewAction::FocusRight)),
            ('j', Action::View(ViewAction::FocusDown)),
            ('k', Action::View(ViewAction::FocusUp)),
            ('u', Action::View(ViewAction::WorkspaceDown)),
            ('i', Action::View(ViewAction::WorkspaceUp)),
            ('q', Action::Session(SessionCommand::ClosePane)),
            (
                '[',
                Action::Session(SessionCommand::ConsumeOrExpel(Direction::Left)),
            ),
            (
                ']',
                Action::Session(SessionCommand::ConsumeOrExpel(Direction::Right)),
            ),
            ('r', Action::Session(SessionCommand::CycleWidth)),
            ('f', Action::Session(SessionCommand::ToggleFullWidth)),
            (
                '-',
                Action::Session(SessionCommand::StepWidth(Step::Shrink)),
            ),
            ('=', Action::Session(SessionCommand::StepWidth(Step::Grow))),
            (
                '_',
                Action::Session(SessionCommand::StepHeight(Step::Shrink)),
            ),
            ('+', Action::Session(SessionCommand::StepHeight(Step::Grow))),
            ('R', Action::Session(SessionCommand::ResetHeight)),
        ];
        for (c, action) in expected {
            assert_eq!(after_prefix(char_key(c)), Command::Run(action), "{c}");
        }
        assert_eq!(
            after_prefix(Key::plain(KeyCode::Enter)),
            Command::Run(Action::Session(SessionCommand::OpenPane))
        );
    }

    #[test]
    fn every_table_entry_names_an_action_of_its_kind() {
        #[derive(Debug, PartialEq)]
        enum Kind {
            View,
            Session,
            Client,
        }
        let kind = |action: &Action| match action {
            Action::View(_) => Kind::View,
            Action::Session(_) => Kind::Session,
            Action::Client(_) => Kind::Client,
        };
        let expected = [
            (char_key('h'), Kind::View),
            (char_key('l'), Kind::View),
            (char_key('j'), Kind::View),
            (char_key('k'), Kind::View),
            (char_key('u'), Kind::View),
            (char_key('i'), Kind::View),
            (Key::plain(KeyCode::Enter), Kind::Session),
            (char_key('q'), Kind::Session),
            (char_key('['), Kind::Session),
            (char_key(']'), Kind::Session),
            (char_key('r'), Kind::Session),
            (char_key('f'), Kind::Session),
            (char_key('-'), Kind::Session),
            (char_key('='), Kind::Session),
            (char_key('_'), Kind::Session),
            (char_key('+'), Kind::Session),
            (char_key('R'), Kind::Session),
            (char_key('D'), Kind::Client),
            (PREFIX, Kind::Client),
        ];
        assert_eq!(BINDINGS.len(), expected.len());
        for (key, expected) in expected {
            let (_, action) = BINDINGS
                .iter()
                .find(|(bound, _)| *bound == key)
                .unwrap_or_else(|| panic!("{key:?} is not bound"));
            assert_eq!(kind(action), expected, "{key:?}");
        }
    }

    #[test]
    fn shift_d_detaches_with_or_without_the_shift_flag() {
        assert_eq!(
            after_prefix(Key::new(KeyCode::Char('D'), Modifiers::SHIFT)),
            Command::Run(Action::Client(ClientAction::Detach))
        );
        assert_eq!(
            after_prefix(char_key('D')),
            Command::Run(Action::Client(ClientAction::Detach))
        );
    }

    #[test]
    fn plus_grows_the_height_with_or_without_the_shift_flag() {
        let grow = Command::Run(Action::Session(SessionCommand::StepHeight(Step::Grow)));
        assert_eq!(
            after_prefix(Key::new(KeyCode::Char('+'), Modifiers::SHIFT)),
            grow
        );
        assert_eq!(after_prefix(char_key('+')), grow);
    }

    #[test]
    fn lowercase_d_is_discarded() {
        assert_eq!(after_prefix(char_key('d')), Command::Discard);
    }

    #[test]
    fn prefix_twice_sends_one_prefix() {
        let mut leader = Leader::default();
        assert_eq!(leader.handle(PREFIX), Command::Discard);
        assert_eq!(
            leader.handle(PREFIX),
            Command::Run(Action::Client(ClientAction::SendKey(PREFIX)))
        );
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
