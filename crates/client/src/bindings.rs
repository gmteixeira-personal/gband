use gband_core::input::{Key, KeyCode};
use gband_lua::{Binding, Chord, Keys};

pub struct Keymap {
    pub prefix: Key,
    pub direct: Vec<(Key, Binding)>,
    pub prefixed: Vec<(Chord, Binding)>,
}

impl Keymap {
    pub fn new(prefix: Key, bindings: Vec<(Keys, Binding)>) -> Self {
        let mut keymap = Self {
            prefix,
            direct: Vec::new(),
            prefixed: Vec::new(),
        };
        for (keys, binding) in bindings {
            match keys {
                Keys::Direct(key) => keymap.direct.push((key, binding)),
                Keys::Prefixed(chord) => keymap.prefixed.push((chord, binding)),
            }
        }
        keymap
    }

    fn chord_key(&self, chord: Chord) -> Key {
        match chord {
            Chord::Key(key) => key,
            Chord::Prefix => self.prefix,
        }
    }
}

#[derive(Debug)]
pub enum Command<'a> {
    Send(Key),
    Run(&'a Binding),
    Discard,
}

impl PartialEq for Command<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Command::Send(a), Command::Send(b)) => a == b,
            (Command::Run(a), Command::Run(b)) => std::ptr::eq(*a, *b),
            (Command::Discard, Command::Discard) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Default)]
pub struct Leader {
    after_prefix: bool,
}

impl Leader {
    pub fn handle<'a>(&mut self, keymap: &'a Keymap, key: Key) -> Command<'a> {
        if std::mem::take(&mut self.after_prefix) {
            return keymap
                .prefixed
                .iter()
                .find(|&&(chord, _)| matches(keymap.chord_key(chord), key))
                .map_or(Command::Discard, |(_, binding)| Command::Run(binding));
        }
        if let Some((_, binding)) = keymap.direct.iter().find(|(bound, _)| matches(*bound, key)) {
            return Command::Run(binding);
        }
        if !keymap.prefixed.is_empty() && matches(keymap.prefix, key) {
            self.after_prefix = true;
            return Command::Discard;
        }
        Command::Send(key)
    }

    pub fn reset(&mut self) {
        self.after_prefix = false;
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
    use gband_core::action::{Action, ClientAction, SessionCommand};
    use gband_core::input::{Modes, Modifiers, encode_key};
    use gband_core::layout::{Direction, Step};
    use gband_core::view::ViewAction;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use gband_lua::keys::parse_key;

    use super::*;

    fn defaults() -> Keymap {
        let config = gband_lua::defaults();
        Keymap::new(config.options.prefix, config.bindings)
    }

    fn configured(source: &str) -> Keymap {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "gband-client-bindings-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let path = gband_lua::user_file(&dir);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, source).unwrap();
        let config = gband_lua::load(&dir).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        Keymap::new(config.options.prefix, config.bindings)
    }

    fn key(name: &str) -> Key {
        parse_key(name).unwrap()
    }

    fn char_key(c: char) -> Key {
        Key::plain(KeyCode::Char(c))
    }

    fn ran(command: Command<'_>) -> Option<Action> {
        match command {
            Command::Run(Binding::Action(action)) => Some(*action),
            _ => None,
        }
    }

    fn after_prefix(keymap: &Keymap, key: Key) -> Option<Action> {
        let mut leader = Leader::default();
        assert_eq!(leader.handle(keymap, keymap.prefix), Command::Discard);
        ran(leader.handle(keymap, key))
    }

    #[test]
    fn default_keys_follow_the_spec() {
        let keymap = defaults();
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
            ('D', Action::Client(ClientAction::Detach)),
        ];
        for (c, action) in expected {
            assert_eq!(after_prefix(&keymap, char_key(c)), Some(action), "{c}");
        }
        assert_eq!(
            after_prefix(&keymap, Key::plain(KeyCode::Enter)),
            Some(Action::Session(SessionCommand::OpenPane))
        );
        assert!(keymap.direct.is_empty());
        assert_eq!(keymap.prefixed.len(), expected.len() + 2);
    }

    #[test]
    fn every_default_binding_names_an_action() {
        let keymap = defaults();
        for (chord, binding) in &keymap.prefixed {
            assert!(matches!(binding, Binding::Action(_)), "{chord:?}");
        }
    }

    #[test]
    fn shift_d_detaches_with_or_without_the_shift_flag() {
        let keymap = defaults();
        let detach = Some(Action::Client(ClientAction::Detach));
        assert_eq!(
            after_prefix(&keymap, Key::new(KeyCode::Char('D'), Modifiers::SHIFT)),
            detach
        );
        assert_eq!(after_prefix(&keymap, char_key('D')), detach);
    }

    #[test]
    fn plus_grows_the_height_with_or_without_the_shift_flag() {
        let keymap = defaults();
        let grow = Some(Action::Session(SessionCommand::StepHeight(Step::Grow)));
        assert_eq!(
            after_prefix(&keymap, Key::new(KeyCode::Char('+'), Modifiers::SHIFT)),
            grow
        );
        assert_eq!(after_prefix(&keymap, char_key('+')), grow);
    }

    #[test]
    fn lowercase_d_is_discarded() {
        let keymap = defaults();
        let mut leader = Leader::default();
        leader.handle(&keymap, keymap.prefix);
        assert_eq!(leader.handle(&keymap, char_key('d')), Command::Discard);
    }

    #[test]
    fn prefix_twice_sends_one_prefix() {
        let keymap = defaults();
        assert_eq!(keymap.prefix, key("ctrl+space"));
        assert_eq!(encode_key(keymap.prefix, Modes::default()), b"\x00");
        let mut leader = Leader::default();
        assert_eq!(leader.handle(&keymap, keymap.prefix), Command::Discard);
        assert_eq!(
            ran(leader.handle(&keymap, keymap.prefix)),
            Some(Action::Client(ClientAction::SendPrefix))
        );
        assert_eq!(
            leader.handle(&keymap, char_key('h')),
            Command::Send(char_key('h'))
        );
    }

    #[test]
    fn unbound_key_after_the_prefix_is_discarded() {
        let keymap = defaults();
        let mut leader = Leader::default();
        assert_eq!(leader.handle(&keymap, keymap.prefix), Command::Discard);
        assert_eq!(leader.handle(&keymap, char_key('x')), Command::Discard);
        assert_eq!(
            leader.handle(&keymap, char_key('x')),
            Command::Send(char_key('x'))
        );
    }

    #[test]
    fn modifiers_other_than_shift_must_match() {
        let keymap = defaults();
        assert_eq!(after_prefix(&keymap, key("alt+h")), None);
        assert_eq!(after_prefix(&keymap, key("ctrl+q")), None);
    }

    #[test]
    fn keys_without_the_prefix_pass_through() {
        let keymap = defaults();
        let mut leader = Leader::default();
        for pressed in [
            char_key('h'),
            Key::plain(KeyCode::Up),
            key("ctrl+a"),
            key("ctrl+b"),
        ] {
            assert_eq!(leader.handle(&keymap, pressed), Command::Send(pressed));
        }
    }

    #[test]
    fn direct_binding_acts_without_the_prefix() {
        let keymap = configured("gband.bind('alt+h', gband.action.focus_column_left)");
        let mut leader = Leader::default();
        assert_eq!(
            ran(leader.handle(&keymap, key("alt+h"))),
            Some(Action::View(ViewAction::FocusLeft))
        );
    }

    #[test]
    fn unbound_alt_key_reaches_the_pane() {
        let keymap = configured("gband.bind('alt+h', gband.action.focus_column_left)");
        let mut leader = Leader::default();
        assert_eq!(
            leader.handle(&keymap, key("alt+x")),
            Command::Send(key("alt+x"))
        );
    }

    #[test]
    fn another_prefix_key() {
        let keymap = configured(&format!(
            "{}\ngband.set {{ prefix = 'ctrl+b' }}",
            gband_lua::DEFAULTS
        ));
        let mut leader = Leader::default();
        assert_eq!(leader.handle(&keymap, key("ctrl+b")), Command::Discard);
        assert_eq!(
            ran(leader.handle(&keymap, char_key('q'))),
            Some(Action::Session(SessionCommand::ClosePane))
        );
        assert_eq!(
            leader.handle(&keymap, key("ctrl+space")),
            Command::Send(key("ctrl+space"))
        );
        assert_eq!(leader.handle(&keymap, key("ctrl+b")), Command::Discard);
        assert_eq!(
            ran(leader.handle(&keymap, key("ctrl+b"))),
            Some(Action::Client(ClientAction::SendPrefix))
        );
    }

    #[test]
    fn no_prefix_binding_left() {
        let keymap = configured("");
        assert!(keymap.prefixed.is_empty());
        let mut leader = Leader::default();
        assert_eq!(
            leader.handle(&keymap, key("ctrl+space")),
            Command::Send(key("ctrl+space"))
        );
        assert_eq!(
            leader.handle(&keymap, char_key('h')),
            Command::Send(char_key('h'))
        );
    }

    #[test]
    fn reset_ends_a_prefix_sequence() {
        let keymap = defaults();
        let mut leader = Leader::default();
        assert_eq!(leader.handle(&keymap, keymap.prefix), Command::Discard);
        leader.reset();
        assert_eq!(
            leader.handle(&keymap, char_key('q')),
            Command::Send(char_key('q'))
        );
    }
}
