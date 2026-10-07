use gband_core::action::Steps;
use gband_core::input::{Key, KeyCode, Modifiers, MouseKey};
use gband_lua::{Binding, Chord, KeyTables, Modes, Options};

pub const ROOT: &str = "root";
pub const PREFIX: &str = "prefix";

pub struct Keymap {
    pub prefix: Key,
    pub mouse_mod: Modifiers,
    pub steps: Steps,
    pub tables: KeyTables,
    pub modes: Modes,
}

impl Keymap {
    pub fn new(options: &Options, tables: KeyTables, modes: Modes) -> Self {
        Self {
            prefix: options.prefix,
            mouse_mod: options.mouse_mod,
            steps: options.steps,
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
            Chord::Mouse { .. } => None,
        }
    }

    fn chord_mouse(&self, chord: Chord) -> Option<MouseKey> {
        match chord {
            Chord::Mouse { key, uses_mod } if uses_mod => Some(MouseKey {
                modifiers: Modifiers {
                    ctrl: key.modifiers.ctrl || self.mouse_mod.ctrl,
                    alt: key.modifiers.alt || self.mouse_mod.alt,
                    shift: key.modifiers.shift || self.mouse_mod.shift,
                },
                ..key
            }),
            Chord::Mouse { key, .. } => Some(key),
            Chord::Key(_) | Chord::Prefix => None,
        }
    }

    fn find(&self, table: &str, key: Key) -> Option<Binding> {
        self.table(table)
            .iter()
            .find(|&&(chord, _)| {
                self.chord_key(chord)
                    .is_some_and(|bound| matches(bound, key))
            })
            .map(|&(_, binding)| self.stepped(binding))
    }

    fn find_mouse(&self, table: &str, key: MouseKey) -> Option<Binding> {
        self.table(table)
            .iter()
            .rev()
            .find(|&&(chord, _)| self.chord_mouse(chord) == Some(key))
            .map(|&(_, binding)| self.stepped(binding))
    }

    fn stepped(&self, binding: Binding) -> Binding {
        match binding {
            Binding::Action(action) => Binding::Action(action.stepped(self.steps)),
            binding => binding,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Send(Key),
    Run(Binding),
    Discard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unbound {
    Default,
    Discard,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseCommand {
    Default,
    Run { binding: Binding, unbound: Unbound },
    Discard,
}

impl From<Unbound> for MouseCommand {
    fn from(unbound: Unbound) -> Self {
        match unbound {
            Unbound::Default => MouseCommand::Default,
            Unbound::Discard => MouseCommand::Discard,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WheelCommand {
    Run { binding: Binding, ends: bool },
    Pass,
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
        let (table, unbound) = if keymap.modes.contains(&self.active) {
            (self.active.clone(), Unbound::Discard)
        } else if self.active != ROOT {
            (
                std::mem::replace(&mut self.active, ROOT.to_owned()),
                Unbound::Discard,
            )
        } else {
            (ROOT.to_owned(), Unbound::Default)
        };
        keymap
            .find_mouse(&table, key)
            .map_or(unbound.into(), |binding| MouseCommand::Run {
                binding,
                unbound,
            })
    }

    pub fn handle_wheel(&self, keymap: &Keymap, key: MouseKey) -> WheelCommand {
        let Some(binding) = keymap.find_mouse(&self.active, key) else {
            return WheelCommand::Pass;
        };
        WheelCommand::Run {
            binding,
            ends: self.active != ROOT && !keymap.modes.contains(&self.active),
        }
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
    use gband_core::input::{Modes, MouseButton, MouseInput, WheelDirection, encode_key};
    use gband_core::layout::{Direction, Proportion, Step, Vertical};
    use gband_core::view::ViewAction;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use gband_lua::keys::parse_key;

    use super::*;

    fn defaults() -> Keymap {
        let config = gband_lua::defaults(gband_lua::Side::Client);
        Keymap::new(&config.options, config.keymap, config.modes)
    }

    fn default_prefix_entries() -> Vec<(String, Option<String>, String)> {
        prefix_entries(&gband_lua::defaults(gband_lua::Side::Client))
    }

    fn prefix_entries(config: &gband_lua::Config) -> Vec<(String, Option<String>, String)> {
        table_entries(config, PREFIX)
    }

    fn table_entries(
        config: &gband_lua::Config,
        table: &str,
    ) -> Vec<(String, Option<String>, String)> {
        let names: Vec<String> = config
            .runtime
            .lua()
            .load("local names = {} for _, action in ipairs(gband.action.list()) do names[#names + 1] = action.name end return names")
            .eval()
            .unwrap();
        let entries: Vec<mlua::Table> = config
            .runtime
            .lua()
            .load(format!("return gband.keymap.list('{table}')"))
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
        let dir = gband_scratch::Scratch::new(
            "client",
            &format!("bindings-style-{}", NEXT.fetch_add(1, Ordering::Relaxed)),
        );
        let user = gband_lua::user_dir(&dir);
        std::fs::create_dir_all(&user).unwrap();
        std::fs::write(user.join("keystyle.lua"), format!("return \"{style}\"\n")).unwrap();
        let locations = gband_lua::Locations {
            config: dir.to_path_buf(),
            plugins: None,
        };
        gband_lua::load(
            &locations,
            gband_lua::Side::Client,
            &gband_lua::LoadOptions::default(),
        )
        .unwrap()
    }

    fn configured(source: &str) -> Keymap {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let dir = gband_scratch::Scratch::new(
            "client",
            &format!("bindings-{}", NEXT.fetch_add(1, Ordering::Relaxed)),
        );
        let path = gband_lua::user_file(&dir, gband_lua::Side::Client);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, source).unwrap();
        let locations = gband_lua::Locations {
            config: dir.to_path_buf(),
            plugins: None,
        };
        let config = gband_lua::load(
            &locations,
            gband_lua::Side::Client,
            &gband_lua::LoadOptions::default(),
        )
        .unwrap();
        Keymap::new(&config.options, config.keymap, config.modes)
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
            char_key('N'),
            char_key('s'),
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
            ctrl+left ctrl+right ctrl+down ctrl+up ? : N s D escape enter left right down up prefix \
            leftmouse rightmouse middlemouse mod+leftmouse mod+rightmouse mod+middlemouse \
            mod+wheeldown mod+wheelup";
        assert_eq!(keys, order.split_whitespace().collect::<Vec<_>>());
        assert_eq!(keymap.table(PREFIX).len(), keys.len());
        assert!(keymap.modes.contains(PREFIX));
        let root: Vec<String> = table_entries(&gband_lua::defaults(gband_lua::Side::Client), ROOT)
            .into_iter()
            .map(|(key, _, _)| key)
            .collect();
        assert_eq!(root, MOD_ROWS);
        assert_mouse_rows(&keymap);
    }

    const MOD_ROWS: [&str; 5] = [
        "mod+leftmouse",
        "mod+rightmouse",
        "mod+middlemouse",
        "mod+wheeldown",
        "mod+wheelup",
    ];

    fn mouse(input: impl Into<MouseInput>, modifiers: Modifiers) -> MouseKey {
        MouseKey::new(input, modifiers)
    }

    fn assert_mouse_rows(keymap: &Keymap) {
        let drags = [
            (MouseButton::Left, ClientAction::DragWindow),
            (MouseButton::Right, ClientAction::DragResize),
            (MouseButton::Middle, ClientAction::DragBand),
        ];
        let wheels = [
            (WheelDirection::Down, ViewAction::BandDown),
            (WheelDirection::Up, ViewAction::BandUp),
        ];
        for (button, action) in drags {
            let alt = mouse(button, Modifiers::ALT);
            let ran = |command| match command {
                MouseCommand::Run {
                    binding: Binding::Action(action),
                    ..
                } => Some(action),
                _ => None,
            };
            let action = Some(Action::Client(action));
            assert_eq!(ran(Leader::default().handle_mouse(keymap, alt)), action);
            for pressed in [alt, mouse(button, Modifiers::NONE)] {
                let mut leader = Leader::default();
                leader.handle(keymap, keymap.prefix);
                assert_eq!(
                    ran(leader.handle_mouse(keymap, pressed)),
                    action,
                    "{pressed:?}"
                );
            }
        }
        for (direction, action) in wheels {
            let alt = mouse(direction, Modifiers::ALT);
            let run = |ends| WheelCommand::Run {
                binding: Binding::Action(Action::View(action)),
                ends,
            };
            assert_eq!(Leader::default().handle_wheel(keymap, alt), run(false));
            let mut leader = Leader::default();
            leader.handle(keymap, keymap.prefix);
            assert_eq!(
                leader.handle_wheel(keymap, alt),
                run(!keymap.modes.contains(PREFIX))
            );
            assert_eq!(
                Leader::default().handle_wheel(keymap, mouse(direction, Modifiers::NONE)),
                WheelCommand::Pass
            );
        }
    }

    #[test]
    fn direct_keys_follow_the_spec() {
        let modal = defaults();
        let modal_entries = default_prefix_entries();
        let config = with_saved_style("direct");
        let entries = prefix_entries(&config);
        let keymap = Keymap::new(&config.options, config.keymap, config.modes);
        let keys: Vec<&str> = entries.iter().map(|(key, _, _)| key.as_str()).collect();
        let expected: Vec<&str> = modal_entries
            .iter()
            .map(|(key, _, _)| key.as_str())
            .filter(|key| !matches!(*key, "escape" | "enter"))
            .collect();
        assert_eq!(keys, expected);
        assert!(keymap.modes.is_empty());
        assert_mouse_rows(&keymap);
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
                    matches!(
                        action.as_deref(),
                        Some("keylist.open" | "prompt.open" | "prompt.rename")
                    ) || key == "s",
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
    fn presets_keep_parity() {
        let modal = with_saved_style("modal");
        let direct = with_saved_style("direct");
        for table in [ROOT, PREFIX] {
            let modal_entries: Vec<_> = table_entries(&modal, table)
                .into_iter()
                .filter(|(key, _, _)| {
                    table != PREFIX || !matches!(key.as_str(), "escape" | "enter")
                })
                .collect();
            let direct_entries = table_entries(&direct, table);
            let keys = |entries: &[(String, Option<String>, String)]| {
                entries
                    .iter()
                    .map(|(key, _, _)| key.clone())
                    .collect::<Vec<_>>()
            };
            assert_eq!(keys(&modal_entries), keys(&direct_entries), "{table}");
            for ((key, modal_action, _), (_, direct_action, _)) in
                modal_entries.iter().zip(&direct_entries)
            {
                if let (Some(modal_action), Some(direct_action)) = (modal_action, direct_action) {
                    assert_eq!(modal_action, direct_action, "{table} {key}");
                }
            }
        }
    }

    #[test]
    fn default_mouse_modifier() {
        let keymap =
            configured("gband.keymap.set('root', 'mod+leftmouse', gband.action.drag_window)");
        let drag = MouseCommand::Run {
            binding: Binding::Action(Action::Client(ClientAction::DragWindow)),
            unbound: Unbound::Default,
        };
        let mut leader = Leader::default();
        assert_eq!(
            leader.handle_mouse(&keymap, mouse(MouseButton::Left, Modifiers::ALT)),
            drag
        );
        assert_eq!(
            leader.handle_mouse(&keymap, mouse(MouseButton::Left, Modifiers::NONE)),
            MouseCommand::Default
        );
    }

    #[test]
    fn changed_mouse_modifier() {
        let keymap = configured("gband.keystyle.use()\ngband.opt.mouse_mod = 'ctrl+alt'");
        let ctrl_alt = Modifiers {
            ctrl: true,
            alt: true,
            shift: false,
        };
        let drag = MouseCommand::Run {
            binding: Binding::Action(Action::Client(ClientAction::DragWindow)),
            unbound: Unbound::Default,
        };
        let mut leader = Leader::default();
        assert_eq!(
            leader.handle_mouse(&keymap, mouse(MouseButton::Left, ctrl_alt)),
            drag
        );
        assert_eq!(
            leader.handle_mouse(&keymap, mouse(MouseButton::Left, Modifiers::ALT)),
            MouseCommand::Default
        );
    }

    #[test]
    fn last_matching_mouse_binding_wins() {
        let alt_left = mouse(MouseButton::Left, Modifiers::ALT);
        let drag = |action| MouseCommand::Run {
            binding: Binding::Action(Action::Client(action)),
            unbound: Unbound::Default,
        };
        let keymap = configured(
            "gband.keymap.set('root', 'mod+leftmouse', gband.action.drag_window)
gband.keymap.set('root', 'alt+leftmouse', gband.action.drag_band)",
        );
        assert_eq!(
            Leader::default().handle_mouse(&keymap, alt_left),
            drag(ClientAction::DragBand)
        );
        let keymap = configured(
            "gband.keymap.set('root', 'mod+leftmouse', gband.action.drag_window)
gband.keymap.set('root', 'alt+leftmouse', gband.action.drag_band)
gband.keymap.set('root', 'mod+leftmouse', gband.action.drag_resize_window)",
        );
        assert_eq!(
            Leader::default().handle_mouse(&keymap, alt_left),
            drag(ClientAction::DragResize)
        );
    }

    #[test]
    fn own_binding_after_the_preset() {
        let keymap = configured(
            "gband.keystyle.use()
gband.keymap.set('root', 'alt+leftmouse', function() end)
gband.keymap.set('root', 'alt+wheeldown', function() end)
gband.keymap.set('prefix', 'alt+leftmouse', function() end)",
        );
        let alt_left = mouse(MouseButton::Left, Modifiers::ALT);
        let mut leader = Leader::default();
        assert!(matches!(
            leader.handle_mouse(&keymap, alt_left),
            MouseCommand::Run {
                binding: Binding::Callback(_),
                unbound: Unbound::Default
            }
        ));
        assert!(matches!(
            leader.handle_wheel(&keymap, mouse(WheelDirection::Down, Modifiers::ALT)),
            WheelCommand::Run {
                binding: Binding::Callback(_),
                ends: false
            }
        ));
        assert_eq!(leader.handle(&keymap, keymap.prefix), Command::Discard);
        assert!(matches!(
            leader.handle_mouse(&keymap, alt_left),
            MouseCommand::Run {
                binding: Binding::Callback(_),
                unbound: Unbound::Discard
            }
        ));
        assert_eq!(
            leader.handle_mouse(&keymap, mouse(MouseButton::Right, Modifiers::ALT)),
            MouseCommand::Run {
                binding: Binding::Action(Action::Client(ClientAction::DragResize)),
                unbound: Unbound::Discard
            }
        );
    }

    #[test]
    fn navigation_mode_keeps_wheel_bindings() {
        let keymap = defaults();
        let down = mouse(WheelDirection::Down, Modifiers::ALT);
        let band_down = WheelCommand::Run {
            binding: Binding::Action(Action::View(ViewAction::BandDown)),
            ends: false,
        };
        let mut leader = Leader::default();
        assert_eq!(leader.handle(&keymap, keymap.prefix), Command::Discard);
        assert_eq!(leader.handle_wheel(&keymap, down), band_down);
        assert_eq!(leader.active(), PREFIX);
        assert_eq!(
            leader.handle_wheel(&keymap, mouse(WheelDirection::Down, Modifiers::NONE)),
            WheelCommand::Pass
        );
        assert_eq!(leader.active(), PREFIX);
        assert_eq!(leader.handle_wheel(&keymap, down), band_down);
        assert_eq!(leader.active(), PREFIX);
    }

    #[test]
    fn wheel_after_a_prefix_table_that_is_not_a_mode() {
        let keymap = configured(
            "gband.keymap.set('prefix', 'h', gband.action.focus_column_left)
gband.keymap.set('prefix', 'alt+wheeldown', gband.action.focus_band_down)",
        );
        let mut leader = Leader::default();
        assert_eq!(leader.handle(&keymap, keymap.prefix), Command::Discard);
        assert_eq!(
            leader.handle_wheel(&keymap, mouse(WheelDirection::Up, Modifiers::NONE)),
            WheelCommand::Pass
        );
        assert_eq!(leader.active(), PREFIX);
        assert_eq!(
            leader.handle_wheel(&keymap, mouse(WheelDirection::Down, Modifiers::ALT)),
            WheelCommand::Run {
                binding: Binding::Action(Action::View(ViewAction::BandDown)),
                ends: true,
            }
        );
        assert_eq!(leader.active(), PREFIX);
        leader.reset();
        assert_eq!(
            leader.handle_wheel(&keymap, mouse(WheelDirection::Down, Modifiers::ALT)),
            WheelCommand::Pass
        );
    }

    #[test]
    fn mouse_run_names_the_unbound_path() {
        let keymap = configured(
            "gband.keymap.set('root', 'leftmouse', function() end)
gband.keymap.set('seq', 'leftmouse', function() end)
gband.keymap.set('resize', 'leftmouse', function() end)
gband.keymap.mode('resize')",
        );
        let left = mouse(MouseButton::Left, Modifiers::NONE);
        let unbound = |leader: &mut Leader| match leader.handle_mouse(&keymap, left) {
            MouseCommand::Run { unbound, .. } => unbound,
            other => panic!("{other:?}"),
        };
        let mut leader = Leader::default();
        assert_eq!(unbound(&mut leader), Unbound::Default);
        leader.enter("seq".to_owned());
        assert_eq!(unbound(&mut leader), Unbound::Discard);
        assert_eq!(leader.active(), ROOT);
        leader.enter("resize".to_owned());
        assert_eq!(unbound(&mut leader), Unbound::Discard);
        assert_eq!(leader.active(), "resize");
    }

    #[test]
    fn wheel_run_ends_only_a_sequence() {
        let keymap = configured(
            "gband.keymap.set('root', 'wheeldown', function() end)
gband.keymap.set('seq', 'wheeldown', function() end)
gband.keymap.set('resize', 'wheeldown', function() end)
gband.keymap.mode('resize')",
        );
        let down = mouse(WheelDirection::Down, Modifiers::NONE);
        let mut leader = Leader::default();
        for (table, expected) in [(ROOT, false), ("seq", true), ("resize", false)] {
            leader.enter(table.to_owned());
            assert!(
                matches!(
                    leader.handle_wheel(&keymap, down),
                    WheelCommand::Run { ends, .. } if ends == expected
                ),
                "{table}"
            );
            assert_eq!(leader.active(), table);
        }
    }

    #[test]
    fn every_default_binding_names_an_action() {
        let keymap = defaults();
        let config = gband_lua::defaults(gband_lua::Side::Client);
        let functions = [
            ("n", "open a window"),
            ("escape", "interactive mode"),
            ("enter", "interactive mode"),
            ("prefix", "send the prefix key"),
            ("s", "settings"),
        ];
        for table in [PREFIX, ROOT] {
            let entries = table_entries(&config, table);
            assert_eq!(entries.len(), keymap.table(table).len(), "{table}");
            for ((chord, binding), (key, action, desc)) in keymap.table(table).iter().zip(&entries)
            {
                match functions.iter().find(|(name, _)| name == key) {
                    Some((_, described)) => {
                        assert!(action.is_none(), "{key}");
                        assert_eq!(desc, described, "{key}");
                        assert!(matches!(binding, Binding::Callback(_)), "{chord:?} {key}");
                    }
                    None => {
                        let registered = matches!(
                            action.as_deref(),
                            Some("keylist.open" | "prompt.open" | "prompt.rename")
                        );
                        assert!(action.is_some(), "{key}");
                        assert!(
                            matches!(binding, Binding::Action(_)) != registered,
                            "{chord:?} {key}"
                        );
                    }
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
