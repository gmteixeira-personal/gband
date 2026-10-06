use gband_core::action::Steps;
use gband_core::input::{Key, KeyCode, MouseKey};
use gband_lua::{Binding, Chord, KeyTables, Modes};

pub const ROOT: &str = "root";
pub const PREFIX: &str = "prefix";

pub struct Keymap {
    pub prefix: Key,
    pub steps: Steps,
    pub tables: KeyTables,
    pub modes: Modes,
}

impl Keymap {
    pub fn new(prefix: Key, steps: Steps, tables: KeyTables, modes: Modes) -> Self {
        Self {
            prefix,
            steps,
            tables,
            modes,
        }
    }

    pub fn table(&self, name: &str) -> &[(Chord, Binding)] {
        self.tables.get(name).map_or(&[], Vec::as_slice)
    }

    fn chord_key(&self, chord: Chord) -> Option<Key> {
        match chord {
            Chord::Key(key) => Some(key),
            Chord::Prefix => Some(self.prefix),
            Chord::Mouse(_) => None,
        }
    }

    fn find(&self, table: &str, key: Key) -> Option<Binding> {
        self.find_by(table, |chord| {
            self.chord_key(chord)
                .is_some_and(|bound| matches(bound, key))
        })
    }

    fn find_mouse(&self, table: &str, key: MouseKey) -> Option<Binding> {
        self.find_by(table, |chord| chord == Chord::Mouse(key))
    }

    fn find_by(&self, table: &str, matching: impl Fn(Chord) -> bool) -> Option<Binding> {
        self.table(table)
            .iter()
            .find(|&&(chord, _)| matching(chord))
            .map(|&(_, binding)| match binding {
                Binding::Action(action) => Binding::Action(action.stepped(self.steps)),
                binding => binding,
            })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Send(Key),
    Run(Binding),
    Discard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseCommand {
    Default,
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
        if keymap.modes.contains(&self.active) {
            return keymap
                .find(&self.active, key)
                .map_or(Command::Discard, Command::Run);
        }
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

    pub fn handle_mouse(&mut self, keymap: &Keymap, key: MouseKey) -> MouseCommand {
        let found = |table: &str| {
            keymap
                .find_mouse(table, key)
                .map_or(MouseCommand::Discard, MouseCommand::Run)
        };
        if keymap.modes.contains(&self.active) {
            return found(&self.active);
        }
        if self.active != ROOT {
            let table = std::mem::replace(&mut self.active, ROOT.to_owned());
            return found(&table);
        }
        keymap
            .find_mouse(ROOT, key)
            .map_or(MouseCommand::Default, MouseCommand::Run)
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
    use gband_core::layout::{Direction, Proportion, Step, Vertical};
    use gband_core::view::ViewAction;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use gband_lua::keys::parse_key;

    use super::*;

    fn defaults() -> Keymap {
        let config = gband_lua::defaults(gband_lua::Side::Client);
        Keymap::new(
            config.options.prefix,
            config.options.steps,
            config.keymap,
            config.modes,
        )
    }

    fn default_prefix_entries() -> Vec<(String, Option<String>, String)> {
        prefix_entries(&gband_lua::defaults(gband_lua::Side::Client))
    }

    fn prefix_entries(config: &gband_lua::Config) -> Vec<(String, Option<String>, String)> {
        let names: Vec<String> = config
            .runtime
            .lua()
            .load("local names = {} for _, action in ipairs(gband.action.list()) do names[#names + 1] = action.name end return names")
            .eval()
            .unwrap();
        let entries: Vec<mlua::Table> = config
            .runtime
            .lua()
            .load("return gband.keymap.list('prefix')")
            .eval()
            .unwrap();
        entries
            .into_iter()
            .map(|entry| {
                let action: Option<String> = entry.get("action").unwrap();
                assert!(
                    action.as_ref().is_none_or(|action| names.contains(action)),
                    "{action:?}"
                );
                (
                    entry.get("key").unwrap(),
                    action,
                    entry.get("desc").unwrap(),
                )
            })
            .collect()
    }

    fn with_saved_style(style: &str) -> gband_lua::Config {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = std::env::temp_dir().join(format!(
            "gband-client-bindings-style-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let user = gband_lua::user_dir(&dir);
        std::fs::create_dir_all(&user).unwrap();
        std::fs::write(user.join("keystyle.lua"), format!("return \"{style}\"\n")).unwrap();
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
        config
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
        Keymap::new(
            config.options.prefix,
            config.options.steps,
            config.keymap,
            config.modes,
        )
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
            ('c', Action::View(ViewAction::CenterColumn)),
            ('q', Action::Session(SessionCommand::CloseWindow)),
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
                Action::Session(SessionCommand::StepWidth {
                    step: Step::Shrink,
                    by: Proportion::TENTH,
                }),
            ),
            (
                '=',
                Action::Session(SessionCommand::StepWidth {
                    step: Step::Grow,
                    by: Proportion::TENTH,
                }),
            ),
            (
                '_',
                Action::Session(SessionCommand::StepHeight {
                    step: Step::Shrink,
                    by: Proportion::TENTH,
                }),
            ),
            (
                '+',
                Action::Session(SessionCommand::StepHeight {
                    step: Step::Grow,
                    by: Proportion::TENTH,
                }),
            ),
            ('R', Action::Session(SessionCommand::ResetHeight)),
            ('v', Action::Session(SessionCommand::ToggleFloating)),
            ('V', Action::View(ViewAction::SwitchLayer)),
            ('D', Action::Client(ClientAction::Detach)),
        ];
        for (c, action) in expected {
            assert_eq!(after_prefix(&keymap, char_key(c)), Some(action), "{c}");
        }
        let arrows = [
            (KeyCode::Left, ViewAction::FocusLeft),
            (KeyCode::Right, ViewAction::FocusRight),
            (KeyCode::Down, ViewAction::FocusDown),
            (KeyCode::Up, ViewAction::FocusUp),
        ];
        for (arrow, action) in arrows {
            assert_eq!(
                after_prefix(&keymap, Key::plain(arrow)),
                Some(Action::View(action)),
                "{arrow:?}"
            );
        }
        let moves = [
            (
                'h',
                KeyCode::Left,
                SessionCommand::MoveColumn(Direction::Left),
            ),
            (
                'l',
                KeyCode::Right,
                SessionCommand::MoveColumn(Direction::Right),
            ),
            (
                'j',
                KeyCode::Down,
                SessionCommand::MoveWindow(Vertical::Down),
            ),
            ('k', KeyCode::Up, SessionCommand::MoveWindow(Vertical::Up)),
        ];
        for (c, arrow, command) in moves {
            let action = Some(Action::Session(command));
            let ctrl = |code| Key::new(code, Modifiers::CTRL);
            assert_eq!(after_prefix(&keymap, ctrl(KeyCode::Char(c))), action, "{c}");
            assert_eq!(after_prefix(&keymap, ctrl(arrow)), action, "{arrow:?}");
        }
        for pressed in [
            char_key('?'),
            char_key(':'),
            char_key('n'),
            Key::plain(KeyCode::Escape),
            Key::plain(KeyCode::Enter),
            keymap.prefix,
        ] {
            let mut leader = Leader::default();
            leader.handle(&keymap, keymap.prefix);
            assert!(
                matches!(
                    leader.handle(&keymap, pressed),
                    Command::Run(Binding::Callback(_))
                ),
                "{pressed:?}"
            );
        }
        let keys: Vec<String> = default_prefix_entries()
            .into_iter()
            .map(|(key, _, _)| key)
            .collect();
        let order = "h l j k u i c n q [ ] r f - = _ + R v V ctrl+h ctrl+l ctrl+j ctrl+k \
            ctrl+left ctrl+right ctrl+down ctrl+up ? : D escape enter left right down up prefix \
            leftmouse rightmouse middlemouse";
        assert_eq!(keys, order.split_whitespace().collect::<Vec<_>>());
        assert!(keymap.table(ROOT).is_empty());
        assert_eq!(keymap.table(PREFIX).len(), keys.len());
        assert!(keymap.modes.contains(PREFIX));
    }

    #[test]
    fn direct_keys_follow_the_spec() {
        let modal = defaults();
        let modal_entries = default_prefix_entries();
        let config = with_saved_style("direct");
        let entries = prefix_entries(&config);
        let keymap = Keymap::new(
            config.options.prefix,
            config.options.steps,
            config.keymap,
            config.modes,
        );
        let keys: Vec<&str> = entries.iter().map(|(key, _, _)| key.as_str()).collect();
        let expected: Vec<&str> = modal_entries
            .iter()
            .map(|(key, _, _)| key.as_str())
            .filter(|key| {
                !matches!(
                    *key,
                    "escape" | "enter" | "leftmouse" | "rightmouse" | "middlemouse"
                )
            })
            .collect();
        assert_eq!(keys, expected);
        assert!(keymap.table(ROOT).is_empty());
        assert!(keymap.modes.is_empty());
        let own = [
            (
                "n",
                "open_window",
                "open a window running the user's shell",
                Action::Session(SessionCommand::OpenWindow),
            ),
            (
                "prefix",
                "send_prefix",
                "send the prefix key to the focused window",
                Action::Client(ClientAction::SendPrefix),
            ),
        ];
        for ((key, action, desc), (_, binding)) in entries.iter().zip(keymap.table(PREFIX)) {
            if let Some((_, name, described, bound)) = own.iter().find(|(own, ..)| own == key) {
                assert_eq!(action.as_deref(), Some(*name), "{key}");
                assert_eq!(desc, described, "{key}");
                assert_eq!(*binding, Binding::Action(*bound), "{key}");
                continue;
            }
            let index = modal_entries
                .iter()
                .position(|(modal_key, _, _)| modal_key == key)
                .unwrap();
            let (_, modal_action, modal_desc) = &modal_entries[index];
            assert_eq!((action, desc), (modal_action, modal_desc), "{key}");
            let modal_binding = modal.table(PREFIX)[index].1;
            match (binding, modal_binding) {
                (Binding::Action(direct), Binding::Action(modal)) => {
                    assert_eq!(*direct, modal, "{key}")
                }
                _ => assert!(
                    matches!(action.as_deref(), Some("keylist.open" | "prompt.open")),
                    "{key}"
                ),
            }
        }
        for pressed in [char_key('l'), Key::plain(KeyCode::Escape)] {
            let mut leader = Leader::default();
            assert_eq!(leader.handle(&keymap, keymap.prefix), Command::Discard);
            leader.handle(&keymap, pressed);
            assert_eq!(leader.active(), ROOT, "{pressed:?}");
        }
        let mut leader = Leader::default();
        leader.handle(&keymap, keymap.prefix);
        assert_eq!(
            leader.handle(&keymap, Key::plain(KeyCode::Enter)),
            Command::Discard
        );
    }

    #[test]
    fn every_default_binding_names_an_action() {
        let keymap = defaults();
        let entries = default_prefix_entries();
        assert_eq!(entries.len(), keymap.table(PREFIX).len());
        let functions = [
            ("n", "open a window"),
            ("escape", "interactive mode"),
            ("enter", "interactive mode"),
            ("prefix", "send the prefix key"),
        ];
        for ((chord, binding), (key, action, desc)) in keymap.table(PREFIX).iter().zip(&entries) {
            match functions.iter().find(|(name, _)| name == key) {
                Some((_, described)) => {
                    assert!(action.is_none(), "{key}");
                    assert_eq!(desc, described, "{key}");
                    assert!(matches!(binding, Binding::Callback(_)), "{chord:?} {key}");
                }
                None => {
                    let registered =
                        matches!(action.as_deref(), Some("keylist.open" | "prompt.open"));
                    assert!(action.is_some(), "{key}");
                    assert!(
                        matches!(binding, Binding::Action(_)) != registered,
                        "{chord:?} {key}"
                    );
                }
            }
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
        let grow = Some(Action::Session(SessionCommand::StepHeight {
            step: Step::Grow,
            by: Proportion::TENTH,
        }));
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
    fn prefix_twice_runs_the_send_prefix_function() {
        let keymap = defaults();
        assert_eq!(keymap.prefix, key("ctrl+space"));
        assert_eq!(encode_key(keymap.prefix, Modes::default()), b"\x00");
        let mut leader = Leader::default();
        assert_eq!(leader.handle(&keymap, keymap.prefix), Command::Discard);
        assert!(matches!(
            leader.handle(&keymap, keymap.prefix),
            Command::Run(Binding::Callback(_))
        ));
        assert_eq!(leader.active(), PREFIX);
    }

    #[test]
    fn unbound_key_in_navigation_mode_is_discarded() {
        let keymap = defaults();
        let mut leader = Leader::default();
        assert_eq!(leader.handle(&keymap, keymap.prefix), Command::Discard);
        assert_eq!(leader.handle(&keymap, char_key('x')), Command::Discard);
        assert_eq!(leader.active(), PREFIX);
        assert_eq!(
            ran(leader.handle(&keymap, char_key('l'))),
            Some(Action::View(ViewAction::FocusRight))
        );
    }

    #[test]
    fn navigation_mode_repeats_keys() {
        let keymap = defaults();
        let mut leader = Leader::default();
        assert_eq!(leader.handle(&keymap, keymap.prefix), Command::Discard);
        for _ in 0..3 {
            assert_eq!(
                ran(leader.handle(&keymap, char_key('l'))),
                Some(Action::View(ViewAction::FocusRight))
            );
            assert_eq!(leader.active(), PREFIX);
        }
        leader.enter(ROOT.to_owned());
        assert_eq!(
            leader.handle(&keymap, char_key('l')),
            Command::Send(char_key('l'))
        );
    }

    #[test]
    fn prefix_table_that_is_not_a_mode() {
        let keymap = configured("gband.keymap.set('prefix', 'h', gband.action.focus_column_left)");
        assert!(keymap.modes.is_empty());
        let mut leader = Leader::default();
        assert_eq!(leader.handle(&keymap, keymap.prefix), Command::Discard);
        assert_eq!(
            ran(leader.handle(&keymap, char_key('h'))),
            Some(Action::View(ViewAction::FocusLeft))
        );
        assert_eq!(leader.active(), ROOT);
        assert_eq!(
            leader.handle(&keymap, char_key('h')),
            Command::Send(char_key('h'))
        );
    }

    #[test]
    fn a_user_mode() {
        let keymap = configured(
            "gband.keymap.mode('resize')
gband.keymap.set('resize', '=', gband.action.grow_column_width)
gband.keymap.set('resize', 'escape', function() gband.keymap.enter('root') end)
gband.keymap.set('root', 'alt+r', function() gband.keymap.enter('resize') end)",
        );
        let mut leader = Leader::default();
        leader.enter("resize".to_owned());
        let grow = Some(Action::Session(SessionCommand::StepWidth {
            step: Step::Grow,
            by: Proportion::TENTH,
        }));
        assert_eq!(ran(leader.handle(&keymap, char_key('='))), grow);
        assert_eq!(ran(leader.handle(&keymap, char_key('='))), grow);
        assert_eq!(leader.handle(&keymap, char_key('x')), Command::Discard);
        assert_eq!(leader.active(), "resize");
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
    fn unbound_alt_key_reaches_the_window() {
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
            Some(Action::Session(SessionCommand::CloseWindow))
        );
        assert_eq!(leader.handle(&keymap, key("ctrl+space")), Command::Discard);
        assert!(matches!(
            leader.handle(&keymap, key("ctrl+b")),
            Command::Run(Binding::Callback(_))
        ));
        leader.enter(ROOT.to_owned());
        assert_eq!(
            leader.handle(&keymap, key("ctrl+space")),
            Command::Send(key("ctrl+space"))
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
