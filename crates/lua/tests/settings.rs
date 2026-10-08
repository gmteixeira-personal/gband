mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

use common::*;
use gband_core::geometry::Size;
use gband_core::layout::{Layout, LayoutOptions};
use gband_lua::plugin_windows::{FloatingFrame, Frame, Run};
use gband_lua::{Binding, Chord, Color, Config, Dispatch, Event, Outcome, Side, ViewState};

const BUNDLED: [&str; 15] = [
    "default",
    "terminal",
    "catppuccin-latte",
    "catppuccin-frappe",
    "catppuccin-macchiato",
    "catppuccin-mocha",
    "tokyo-night",
    "dracula",
    "nord",
    "gruvbox",
    "one-dark",
    "solarized",
    "kanagawa",
    "rose-pine",
    "vesper",
];

fn state(ribbon: Size) -> ViewState {
    let mut layout = Layout::new();
    let window = layout.allocate_window();
    let band = layout.bands()[0].id;
    layout.open(window, band, None, None, &LayoutOptions::default());
    ViewState {
        table: "root".to_owned(),
        window: Some(1),
        width: ribbon.cols + 1,
        height: ribbon.rows,
        layout: Arc::new(layout),
        area: ribbon,
        ribbon,
        ..ViewState::default()
    }
}

fn text(runs: &[Run]) -> String {
    let line: String = runs.iter().map(|run| run.text.as_str()).collect();
    line.trim_end().to_owned()
}

struct Settings {
    scratch: Scratch,
    config: Config,
}

impl Settings {
    fn new(name: &str, files: &[(&str, &str)]) -> Self {
        let scratch = Scratch::new(name);
        for (path, source) in files {
            scratch.user_file(path, source);
        }
        let config = scratch.loaded();
        assert!(config.errors.is_empty(), "{:?}", config.errors);
        clean(&config.runtime.set_state(state(Size::new(79, 24))));
        config.runtime.take_frames();
        Self { scratch, config }
    }

    fn run(&self, code: &str) -> Outcome {
        let (answer, outcome) = self.config.runtime.eval(code, &[]);
        answer.unwrap_or_else(|error| panic!("{error}"));
        outcome
    }

    fn get<T: mlua::FromLua>(&self, code: &str) -> T {
        eval(&self.config, code)
    }

    fn open(&self) -> Outcome {
        self.run("gband.settings.open()")
    }

    fn windows(&self) -> Vec<u32> {
        self.get("return gband.win.list()")
    }

    fn focused(&self) -> u32 {
        self.windows()
            .into_iter()
            .find(|win| self.get(&format!("return gband.win.info({win}).focused")))
            .expect("a plugin window has focus")
    }

    fn cursor(&self) -> i64 {
        self.get(&format!("return gband.win.info({}).cursor", self.focused()))
    }

    fn press(&self, name: &str) -> Outcome {
        self.config
            .runtime
            .plugin_window_key(self.focused(), key(name))
    }

    fn frames(&self) -> Vec<(u32, FloatingFrame)> {
        self.config
            .runtime
            .take_frames()
            .into_iter()
            .filter_map(|(id, frame)| match frame {
                Some(Frame::Floating(frame)) => Some((id, frame)),
                _ => None,
            })
            .collect()
    }

    fn frame(&self) -> FloatingFrame {
        let mut frames = self.frames();
        frames.sort_by_key(|(_, frame)| frame.z);
        frames.pop().expect("a floating frame is presented").1
    }

    fn colorscheme(&self) -> String {
        self.get("return gband.colorscheme()")
    }

    fn file(&self, name: &str) -> Option<String> {
        fs::read_to_string(self.scratch.user().join(name)).ok()
    }

    fn prefix_n(&self) -> Outcome {
        let chord = Chord::Key(key("n"));
        let Some((_, Binding::Callback(callback))) = self.config.keymap["prefix"]
            .iter()
            .find(|(bound, _)| *bound == chord)
        else {
            panic!("prefix n is not bound to a function");
        };
        self.config.runtime.call(*callback)
    }

    fn user_names(&self) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(self.scratch.user())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }
}

const INIT: (&str, &str) = (
    "init.lua",
    "gband.keystyle.use()\ngband.plugin('gband.sidebar')",
);
const MODAL: (&str, &str) = ("keystyle.lua", "return \"modal\"\n");

#[test]
fn saved_theme_written_by_hand() {
    let settings = Settings::new("theme-by-hand", &[INIT, ("theme.lua", "return 'dracula'")]);
    assert_eq!(
        settings.get::<String>("return gband.settings.theme()"),
        "dracula"
    );
}

#[test]
fn invalid_saved_theme() {
    for (index, source) in ["return 42", "return 'no spaces'", "return ("]
        .into_iter()
        .enumerate()
    {
        let settings = Settings::new(
            &format!("theme-invalid-{index}"),
            &[INIT, ("theme.lua", source)],
        );
        let found: Option<String> = settings.get("return gband.settings.theme()");
        assert_eq!(found, None, "{source}");
    }
}

#[test]
fn broken_theme_file_is_not_a_configuration_error() {
    let settings = Settings::new("theme-broken", &[INIT, ("theme.lua", "error('boom')")]);
    assert_eq!(settings.colorscheme(), "default");
}

#[test]
fn saved_theme_gets_an_empty_environment() {
    let settings = Settings::new(
        "theme-environment",
        &[
            INIT,
            (
                "theme.lua",
                "gband.bind('alt+y', gband.action.detach) return 'nord'",
            ),
        ],
    );
    let found: Option<String> = settings.get("return gband.settings.theme()");
    assert_eq!(found, None);
}

#[test]
fn saved_settings_without_a_configuration_directory() {
    let config = gband_lua::defaults(Side::Client);
    let none: bool = eval(
        &config,
        "return gband.settings.theme() == nil and gband.settings.sidebar() == nil and gband.settings.interactive_on_new() == nil",
    );
    assert!(none);
}

#[test]
fn sidebar_turned_off() {
    let settings = Settings::new("sidebar-off", &[MODAL, ("sidebar.lua", "return false\n")]);
    let bars: i64 = settings.get("return #gband.bar.list()");
    assert_eq!(bars, 0);
    let saved: Option<bool> = settings.get("return gband.settings.sidebar()");
    assert_eq!(saved, Some(false));
}

#[test]
fn sidebar_with_nothing_saved() {
    let settings = Settings::new("sidebar-nothing", &[MODAL]);
    let side: String = settings.get("return gband.bar.info('sidebar').side");
    assert_eq!(side, "left");
}

#[test]
fn unknown_saved_sidebar_value() {
    let settings = Settings::new(
        "sidebar-unknown",
        &[MODAL, ("sidebar.lua", "return \"off\"")],
    );
    let saved: Option<bool> = settings.get("return gband.settings.sidebar()");
    assert_eq!(saved, None);
    let side: String = settings.get("return gband.bar.info('sidebar').side");
    assert_eq!(side, "left");
}

#[test]
fn sidebar_kept_by_a_configuration_file() {
    let settings = Settings::new("sidebar-own", &[INIT, ("sidebar.lua", "return false\n")]);
    let bars: i64 = settings.get("return #gband.bar.list()");
    assert_eq!(bars, 1);
}

fn enters_root(outcome: &Outcome) -> bool {
    outcome
        .dispatched
        .contains(&Dispatch::Enter("root".to_owned()))
}

#[test]
fn interactive_on_new_with_nothing_saved() {
    let settings = Settings::new("interactive-nothing", &[MODAL]);
    let saved: Option<bool> = settings.get("return gband.settings.interactive_on_new()");
    assert_eq!(saved, None);
    let outcome = settings.prefix_n();
    clean(&outcome);
    assert!(!enters_root(&outcome), "{:?}", outcome.dispatched);
}

#[test]
fn interactive_on_new_turned_off() {
    let settings = Settings::new(
        "interactive-off",
        &[MODAL, ("interactive_on_new.lua", "return false\n")],
    );
    let saved: Option<bool> = settings.get("return gband.settings.interactive_on_new()");
    assert_eq!(saved, Some(false));
    let outcome = settings.prefix_n();
    clean(&outcome);
    assert!(!enters_root(&outcome), "{:?}", outcome.dispatched);
}

#[test]
fn unknown_saved_interactive_on_new_value() {
    let settings = Settings::new(
        "interactive-unknown",
        &[MODAL, ("interactive_on_new.lua", "return \"off\"")],
    );
    let saved: Option<bool> = settings.get("return gband.settings.interactive_on_new()");
    assert_eq!(saved, None);
}

#[test]
fn broken_interactive_on_new_file_is_not_a_configuration_error() {
    let settings = Settings::new(
        "interactive-broken",
        &[MODAL, ("interactive_on_new.lua", "error('boom')")],
    );
    let outcome = settings.prefix_n();
    clean(&outcome);
    assert!(!enters_root(&outcome), "{:?}", outcome.dispatched);
}

#[test]
fn own_configuration_with_the_modal_preset() {
    let settings = Settings::new(
        "interactive-own",
        &[
            ("init.lua", "gband.keystyle.use(\"modal\")"),
            ("interactive_on_new.lua", "return true\n"),
        ],
    );
    let outcome = settings.prefix_n();
    clean(&outcome);
    assert!(enters_root(&outcome), "{:?}", outcome.dispatched);
}

#[test]
fn bundled_themes_only() {
    let settings = Settings::new("themes-bundled", &[INIT]);
    let names: Vec<String> = settings.get("return gband.settings.themes()");
    assert_eq!(names, BUNDLED);
}

#[test]
fn user_colorschemes_after_the_bundled_themes() {
    let settings = Settings::new(
        "themes-user",
        &[
            INIT,
            ("colors/zen.lua", ""),
            ("colors/dusk.lua", ""),
            ("colors/nord.lua", ""),
            ("colors/not a name.lua", ""),
            ("colors/notes.txt", ""),
        ],
    );
    let names: Vec<String> = settings.get("return gband.settings.themes()");
    assert_eq!(names.len(), BUNDLED.len() + 2);
    assert_eq!(&names[..BUNDLED.len()], BUNDLED);
    assert_eq!(&names[BUNDLED.len()..], ["dusk", "zen"]);
}

#[test]
fn window_opens() {
    let settings = Settings::new("window-opens", &[MODAL]);
    let outcome = settings.open();
    clean(&outcome);
    assert!(
        outcome
            .dispatched
            .contains(&Dispatch::Enter("root".to_owned()))
    );
    let frame = settings.frame();
    assert_eq!(frame.title.as_deref(), Some("settings"));
    assert!(frame.border.is_some());
    assert!(frame.focused);
    let lines: Vec<String> = frame.lines.iter().map(|line| text(line)).collect();
    assert_eq!(
        lines,
        [
            "theme    default",
            "sidebar  on",
            "keys     modal",
            "I on new off"
        ]
    );
    assert_eq!(settings.cursor(), 1);
}

#[test]
fn window_beside_the_default_sidebar() {
    let settings = Settings::new("window-size", &[MODAL]);
    settings.open();
    let frame = settings.frame();
    assert_eq!(
        (frame.col, frame.row, frame.width, frame.height),
        (24, 9, 31, 6)
    );
}

#[test]
fn window_with_the_direct_style() {
    let settings = Settings::new("window-direct", &[("keystyle.lua", "return \"direct\"\n")]);
    settings.open();
    let frame = settings.frame();
    assert_eq!(frame.lines.len(), 3);
    assert_eq!(text(&frame.lines[2]), "keys     direct");
    assert_eq!(
        (frame.col, frame.row, frame.width, frame.height),
        (24, 9, 31, 5)
    );
}

#[test]
fn window_with_the_floating_style() {
    let settings = Settings::new(
        "window-floating",
        &[("keystyle.lua", "return \"floating\"\n")],
    );
    settings.open();
    let frame = settings.frame();
    assert_eq!(frame.lines.len(), 3);
    assert_eq!(text(&frame.lines[2]), "keys     floating");
    assert_eq!(
        (frame.col, frame.row, frame.width, frame.height),
        (24, 9, 31, 5)
    );
}

#[test]
fn saved_values_shown() {
    let settings = Settings::new(
        "window-saved",
        &[
            MODAL,
            ("sidebar.lua", "return false\n"),
            ("interactive_on_new.lua", "return true\n"),
        ],
    );
    settings.open();
    let frame = settings.frame();
    assert_eq!(text(&frame.lines[1]), "sidebar  off");
    assert_eq!(text(&frame.lines[3]), "I on new on");
}

#[test]
fn open_again_while_open() {
    let settings = Settings::new("window-again", &[MODAL]);
    settings.open();
    let first = settings.focused();
    clean(&settings.run("gband.win.open({})"));
    clean(&settings.open());
    assert_eq!(settings.windows().len(), 2);
    assert_eq!(settings.focused(), first);
}

#[test]
fn open_while_loading() {
    let scratch = Scratch::new("window-loading");
    let path =
        scratch.write("local a = 1\nlocal b = 2\nlocal c = 3\nlocal d = 4\ngband.settings.open()");
    let error = scratch.load().err().expect("loading fails");
    assert_error_at(&error, &path, 5, "while the configuration loads");
}

#[test]
fn labels_in_their_group() {
    let settings = Settings::new(
        "window-labels",
        &[(
            "init.lua",
            "gband.keystyle.use()\ngband.hl.set('SettingsLabel', { fg = 3 })",
        )],
    );
    settings.open();
    let frame = settings.frame();
    for line in &frame.lines {
        assert_eq!(line[0].style.fg, Some(Color::Index(3)), "{line:?}");
        assert_eq!(line[0].text.chars().count(), 9, "{line:?}");
        assert_eq!(line[1].style.fg, None, "{line:?}");
    }
}

#[test]
fn default_label_style() {
    let settings = Settings::new(
        "window-label-default",
        &[
            ("colors/blank.lua", ""),
            ("init.lua", "gband.colorscheme('blank')"),
        ],
    );
    let dim: bool = settings.get("return gband.hl.get('SettingsLabel', { resolve = true }).dim");
    let fields: i64 = settings.get(
        "local n = 0 for _ in pairs(gband.hl.get('SettingsLabel', { resolve = true })) do n = n + 1 end return n",
    );
    assert!(dim);
    assert_eq!(fields, 1);
}

#[test]
fn turn_the_sidebar_off() {
    let settings = Settings::new("keys-sidebar", &[MODAL]);
    settings.open();
    clean(&settings.press("j"));
    clean(&settings.press("enter"));
    assert_eq!(
        settings.file("sidebar.lua").as_deref(),
        Some("return false\n")
    );
    assert_eq!(settings.config.runtime.take_settings_reopen(), Some(2));
    clean(&settings.press("l"));
    assert_eq!(
        settings.file("sidebar.lua").as_deref(),
        Some("return true\n")
    );
}

#[test]
fn switch_the_key_style() {
    let settings = Settings::new("keys-style", &[MODAL]);
    settings.open();
    clean(&settings.press("down"));
    clean(&settings.press("down"));
    clean(&settings.press("l"));
    assert_eq!(
        settings.file("keystyle.lua").as_deref(),
        Some("return \"direct\"\n")
    );
    assert_eq!(settings.config.runtime.take_settings_reopen(), Some(3));
    assert_eq!(settings.user_names(), ["keystyle.lua"]);
}

fn step_key_style(name: &str, saved: &str, key: &str, expected: &str) {
    let saved = format!("return \"{saved}\"\n");
    let settings = Settings::new(name, &[("keystyle.lua", &saved)]);
    settings.open();
    clean(&settings.press("down"));
    clean(&settings.press("down"));
    clean(&settings.press(key));
    assert_eq!(
        settings.file("keystyle.lua"),
        Some(format!("return \"{expected}\"\n")),
        "{key}"
    );
    assert_eq!(settings.config.runtime.take_settings_reopen(), Some(3));
}

#[test]
fn switch_to_the_floating_style() {
    step_key_style("keys-to-floating", "direct", "enter", "floating");
}

#[test]
fn back_from_the_modal_style() {
    step_key_style("keys-back-from-modal", "modal", "h", "floating");
    step_key_style("keys-back-from-modal-left", "modal", "left", "floating");
}

#[test]
fn forward_from_the_floating_style() {
    step_key_style("keys-forward-from-floating", "floating", "right", "modal");
    step_key_style("keys-forward-from-floating-l", "floating", "l", "modal");
}

#[test]
fn back_from_the_floating_style() {
    step_key_style("keys-back-from-floating", "floating", "h", "direct");
}

#[test]
fn turn_interactive_on_new_on() {
    let settings = Settings::new("keys-interactive-fresh", &[MODAL]);
    settings.open();
    for _ in 0..3 {
        clean(&settings.press("j"));
    }
    clean(&settings.press("enter"));
    assert_eq!(
        settings.file("interactive_on_new.lua").as_deref(),
        Some("return true\n")
    );
    assert_eq!(settings.config.runtime.take_settings_reopen(), Some(4));
    for name in ["l", "right", "h", "left"] {
        let before = settings.file("interactive_on_new.lua");
        clean(&settings.press(name));
        assert_ne!(settings.file("interactive_on_new.lua"), before, "{name}");
    }
}

#[test]
fn turn_interactive_on_new_off() {
    let settings = Settings::new(
        "keys-interactive-off",
        &[MODAL, ("interactive_on_new.lua", "return true\n")],
    );
    settings.open();
    for _ in 0..3 {
        clean(&settings.press("j"));
    }
    clean(&settings.press("enter"));
    assert_eq!(
        settings.file("interactive_on_new.lua").as_deref(),
        Some("return false\n")
    );
    assert_eq!(settings.config.runtime.take_settings_reopen(), Some(4));
}

#[test]
fn turn_interactive_on_new_back_on() {
    let settings = Settings::new(
        "keys-interactive-on",
        &[MODAL, ("interactive_on_new.lua", "return false\n")],
    );
    clean(&settings.config.runtime.open_settings(4));
    clean(&settings.press("h"));
    assert_eq!(
        settings.file("interactive_on_new.lua").as_deref(),
        Some("return true\n")
    );
    assert_eq!(settings.config.runtime.take_settings_reopen(), Some(4));
}

#[test]
fn next_theme() {
    let settings = Settings::new(
        "keys-next-theme",
        &[MODAL, ("theme.lua", "return \"gruvbox\"\n")],
    );
    settings.open();
    clean(&settings.press("l"));
    assert_eq!(settings.colorscheme(), "one-dark");
    assert_eq!(
        settings.file("theme.lua").as_deref(),
        Some("return \"one-dark\"\n")
    );
    assert_eq!(settings.config.runtime.take_settings_reopen(), Some(1));
    assert_eq!(text(&settings.frame().lines[0]), "theme    one-dark");
    clean(&settings.press("left"));
    assert_eq!(settings.colorscheme(), "gruvbox");
}

#[test]
fn theme_wraps() {
    let settings = Settings::new("keys-wraps", &[MODAL]);
    settings.open();
    clean(&settings.press("h"));
    assert_eq!(settings.colorscheme(), "vesper");
    assert_eq!(
        settings.file("theme.lua").as_deref(),
        Some("return \"vesper\"\n")
    );
    clean(&settings.press("right"));
    assert_eq!(settings.colorscheme(), "default");
}

#[test]
fn failing_theme_is_not_saved() {
    let settings = Settings::new(
        "keys-failing-theme",
        &[
            MODAL,
            ("theme.lua", "return \"vesper\"\n"),
            ("colors/aaa.lua", "error('boom')"),
        ],
    );
    settings.open();
    let outcome = settings.press("l");
    assert_eq!(outcome.errors.len(), 1, "{:?}", outcome.errors);
    assert!(outcome.errors[0].to_string().contains("colors/aaa"));
    assert_eq!(settings.colorscheme(), "vesper");
    assert_eq!(
        settings.file("theme.lua").as_deref(),
        Some("return \"vesper\"\n")
    );
    assert_eq!(settings.config.runtime.take_settings_reopen(), None);
}

#[test]
fn dismiss() {
    let settings = Settings::new("keys-dismiss", &[MODAL]);
    settings.open();
    clean(&settings.press("escape"));
    assert!(settings.windows().is_empty());
    assert_eq!(settings.user_names(), ["keystyle.lua"]);
}

#[test]
fn cannot_save() {
    let settings = Settings::new("keys-read-only", &[MODAL]);
    let user = settings.scratch.user();
    fs::set_permissions(&user, fs::Permissions::from_mode(0o555)).unwrap();
    if fs::write(user.join("probe"), "").is_ok() {
        fs::set_permissions(&user, fs::Permissions::from_mode(0o755)).unwrap();
        return;
    }
    settings.open();
    clean(&settings.press("j"));
    let outcome = settings.press("enter");
    fs::set_permissions(&user, fs::Permissions::from_mode(0o755)).unwrap();
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert!(error.message.contains("user/sidebar.lua"), "{error}");
    assert_eq!(settings.windows().len(), 1);
    assert_eq!(settings.file("sidebar.lua"), None);
    assert_eq!(settings.user_names(), ["keystyle.lua"]);
    assert_eq!(settings.config.runtime.take_settings_reopen(), None);
    let bars: i64 = settings.get("return #gband.bar.list()");
    assert_eq!(bars, 1);
}

#[test]
fn list_opens_on_the_active_theme() {
    let settings = Settings::new(
        "list-opens",
        &[MODAL, ("theme.lua", "return \"gruvbox\"\n")],
    );
    settings.open();
    clean(&settings.press("enter"));
    let frame = settings.frame();
    assert_eq!(frame.title.as_deref(), Some("theme"));
    assert!(frame.focused);
    let lines: Vec<String> = frame.lines.iter().map(|line| text(line)).collect();
    assert_eq!(lines, BUNDLED);
    assert_eq!(settings.cursor(), 10);
}

#[test]
fn list_size_beside_the_default_sidebar() {
    let settings = Settings::new("list-size", &[MODAL]);
    settings.open();
    clean(&settings.press("enter"));
    let frame = settings.frame();
    assert_eq!(
        (frame.col, frame.row, frame.width, frame.height),
        (28, 3, 22, 17)
    );
}

#[test]
fn preview_while_moving() {
    let settings = Settings::new(
        "list-preview",
        &[MODAL, ("theme.lua", "return \"gruvbox\"\n")],
    );
    settings.open();
    clean(&settings.press("enter"));
    clean(&settings.press("j"));
    assert_eq!(settings.colorscheme(), "one-dark");
    assert_eq!(
        settings.file("theme.lua").as_deref(),
        Some("return \"gruvbox\"\n")
    );
    clean(&settings.press("home"));
    assert_eq!(settings.colorscheme(), "default");
    clean(&settings.press("end"));
    assert_eq!(settings.colorscheme(), "vesper");
    assert_eq!(settings.config.runtime.take_settings_reopen(), None);
}

#[test]
fn pick_a_theme() {
    let settings = Settings::new("list-pick", &[MODAL, ("theme.lua", "return \"gruvbox\"\n")]);
    settings.open();
    clean(&settings.press("enter"));
    clean(&settings.press("k"));
    clean(&settings.press("enter"));
    assert_eq!(settings.windows().len(), 1);
    assert_eq!(
        settings.file("theme.lua").as_deref(),
        Some("return \"nord\"\n")
    );
    assert_eq!(settings.config.runtime.take_settings_reopen(), Some(1));
    assert_eq!(settings.cursor(), 1);
}

#[test]
fn cancel_the_preview() {
    let settings = Settings::new(
        "list-cancel",
        &[MODAL, ("theme.lua", "return \"gruvbox\"\n")],
    );
    settings.open();
    let window = settings.focused();
    clean(&settings.press("enter"));
    clean(&settings.press("j"));
    clean(&settings.press("j"));
    clean(&settings.press("escape"));
    assert_eq!(settings.windows(), [window]);
    assert_eq!(settings.focused(), window);
    assert_eq!(settings.colorscheme(), "gruvbox");
    assert_eq!(
        settings.file("theme.lua").as_deref(),
        Some("return \"gruvbox\"\n")
    );
}

#[test]
fn open_while_the_list_is_open() {
    let settings = Settings::new(
        "list-reopen",
        &[MODAL, ("theme.lua", "return \"gruvbox\"\n")],
    );
    settings.open();
    let window = settings.focused();
    clean(&settings.press("enter"));
    clean(&settings.press("j"));
    clean(&settings.open());
    assert_eq!(settings.windows(), [window]);
    assert_eq!(settings.focused(), window);
    assert_eq!(settings.colorscheme(), "gruvbox");
}

#[test]
fn opened_on_a_line_after_a_reload() {
    let settings = Settings::new("reopen-line", &[MODAL]);
    clean(&settings.config.runtime.open_settings(3));
    let frame = settings.frame();
    assert_eq!(frame.title.as_deref(), Some("settings"));
    assert_eq!(settings.cursor(), 3);
}

fn attached(config: &Config) -> Vec<u32> {
    clean(&config.runtime.set_state(state(Size::new(79, 24))));
    clean(&config.runtime.emit(&Event::Attached {
        session: "main".to_owned(),
    }));
    eval(config, "return gband.win.list()")
}

#[test]
fn offer_on_the_first_start() {
    let scratch = Scratch::new("offer-first");
    let config = scratch.loaded();
    let windows = attached(&config);
    assert_eq!(windows.len(), 1);
    let frames = config.runtime.take_frames();
    let Some((_, Some(Frame::Floating(frame)))) = frames.last() else {
        panic!("{frames:?}");
    };
    assert_eq!(frame.title.as_deref(), Some("settings"));
    assert!(frame.focused);
    assert_eq!(
        eval::<String>(&config, "return gband.colorscheme()"),
        "default"
    );
    assert_eq!(
        eval::<String>(&config, "return gband.keymap.label('prefix')"),
        "navigation"
    );
}

#[test]
fn no_offer_with_a_setting_saved() {
    for (index, (file, source)) in [
        ("theme.lua", "return \"nord\"\n"),
        ("sidebar.lua", "return true\n"),
        ("keystyle.lua", "return \"modal\"\n"),
    ]
    .into_iter()
    .enumerate()
    {
        let scratch = Scratch::new(&format!("offer-saved-{index}"));
        scratch.user_file(file, source);
        let config = scratch.loaded();
        assert!(attached(&config).is_empty(), "{file}");
    }
}

#[test]
fn no_offer_without_a_configuration_directory() {
    let config = gband_lua::defaults(Side::Client);
    assert!(attached(&config).is_empty());
}

#[test]
fn no_offer_with_an_own_configuration() {
    let scratch = Scratch::new("offer-own");
    scratch.write("gband.bind('alt+h', gband.action.focus_column_left)");
    let config = scratch.loaded();
    assert!(attached(&config).is_empty());
}
