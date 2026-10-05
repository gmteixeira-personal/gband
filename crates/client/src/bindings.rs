use gband_core::input::{Key, KeyCode};
use gband_lua::{Binding, Chord, KeyTables};

pub const ROOT: &str = "root";
pub const PREFIX: &str = "prefix";

pub struct Keymap {
    pub prefix: Key,
    pub tables: KeyTables,
}

impl Keymap {
    pub fn new(prefix: Key, tables: KeyTables) -> Self {
        Self { prefix, tables }
    }

    pub fn table(&self, name: &str) -> &[(Chord, Binding)] {
        self.tables.get(name).map_or(&[], Vec::as_slice)
    }

    fn chord_key(&self, chord: Chord) -> Key {
        match chord {
            Chord::Key(key) => key,
            Chord::Prefix => self.prefix,
        }
    }

    fn find(&self, table: &str, key: Key) -> Option<Binding> {
        self.table(table)
            .iter()
            .find(|&&(chord, _)| matches(self.chord_key(chord), key))
            .map(|&(_, binding)| binding)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Send(Key),
    Run(Binding),
    Discard,
}

#[derive(Debug)]
pub struct Leader {
    active: String,
}

impl Default for Leader {
    fn default() -> Self {
        Self {
            active: ROOT.to_owned(),
        }
    }
}

impl Leader {
    pub fn active(&self) -> &str {
        &self.active
    }

    pub fn handle(&mut self, keymap: &Keymap, key: Key) -> Command {
        if self.active != ROOT {
            let table = std::mem::replace(&mut self.active, ROOT.to_owned());
            return keymap
                .find(&table, key)
                .map_or(Command::Discard, Command::Run);
        }
        if let Some(binding) = keymap.find(ROOT, key) {
            return Command::Run(binding);
        }
        if !keymap.table(PREFIX).is_empty() && matches(keymap.prefix, key) {
            self.active = PREFIX.to_owned();
            return Command::Discard;
        }
        Command::Send(key)
    }

    pub fn enter(&mut self, table: String) {
        self.active = table;
    }

    pub fn reset(&mut self) {
        self.active = ROOT.to_owned();
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
        let config = gband_lua::defaults(gband_lua::Side::Client);
        Keymap::new(config.options.prefix, config.keymap)
    }

    fn configured(source: &str) -> Keymap {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "gband-client-bindings-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let path = gband_lua::user_file(&dir, gband_lua::Side::Client);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, source).unwrap();
        let locations = gband_lua::Locations {
            config: dir.clone(),
            plugins: None,
        };
        let config = gband_lua::load(
            &locations,
            gband_lua::Side::Client,
            &gband_lua::LoadOptions::default(),
        )
        .unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        Keymap::new(config.options.prefix, config.keymap)
    }

    fn key(name: &str) -> Key {
        parse_key(name).unwrap()
    }

    fn char_key(c: char) -> Key {
        Key::plain(KeyCode::Char(c))
    }

    fn ran(command: Command) -> Option<Action> {
        match command {
            Command::Run(Binding::Action(action)) => Some(action),
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
            ('u', Action::View(ViewAction::BandDown)),
            ('i', Action::View(ViewAction::BandUp)),
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
        assert!(keymap.table(ROOT).is_empty());
        assert_eq!(keymap.table(PREFIX).len(), expected.len() + 2);
    }

    #[test]
    fn every_default_binding_names_an_action() {
        let keymap = defaults();
        for (chord, binding) in keymap.table(PREFIX) {
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
        assert!(keymap.table(PREFIX).is_empty());
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

    fn move_table() -> Keymap {
        configured(
            "gband.keymap.set('move', 'h', gband.action.focus_column_left)
gband.keymap.set('move', 'l', gband.action.focus_column_right)
gband.keymap.set('prefix', 'm', function() gband.keymap.enter('move') end)
gband.keymap.set('root', 'alt+m', function() gband.keymap.enter('move') end)",
        )
    }

    #[test]
    fn named_key_table() {
        let keymap = move_table();
        let mut leader = Leader::default();
        assert_eq!(leader.handle(&keymap, keymap.prefix), Command::Discard);
        assert_eq!(leader.active(), PREFIX);
        assert!(matches!(
            leader.handle(&keymap, char_key('m')),
            Command::Run(Binding::Callback(_))
        ));
        assert_eq!(leader.active(), ROOT);
        leader.enter("move".to_owned());
        assert_eq!(
            ran(leader.handle(&keymap, char_key('l'))),
            Some(Action::View(ViewAction::FocusRight))
        );
        assert_eq!(leader.active(), ROOT);
        assert_eq!(
            leader.handle(&keymap, char_key('l')),
            Command::Send(char_key('l'))
        );
    }

    #[test]
    fn unbound_key_in_a_named_table() {
        let keymap = move_table();
        let mut leader = Leader::default();
        leader.enter("move".to_owned());
        assert_eq!(leader.handle(&keymap, char_key('x')), Command::Discard);
        assert_eq!(leader.active(), ROOT);
    }

    #[test]
    fn root_binding_enters_a_table() {
        let keymap = move_table();
        let mut leader = Leader::default();
        assert!(matches!(
            leader.handle(&keymap, key("alt+m")),
            Command::Run(Binding::Callback(_))
        ));
        leader.enter("move".to_owned());
        assert_eq!(
            ran(leader.handle(&keymap, char_key('h'))),
            Some(Action::View(ViewAction::FocusLeft))
        );
    }
}
