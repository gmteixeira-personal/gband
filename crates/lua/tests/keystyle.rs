mod common;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::Arc;

use common::*;
use gband_core::geometry::Size;
use gband_core::layout::{Layout, LayoutOptions};
use gband_lua::plugin_windows::{FloatingFrame, Frame, Run};
use gband_lua::{Binding, Chord, Config, Event, LoadOptions, Locations, Side, ViewState};

const TITLE: &str = "key style  enter choose  esc later";
const MODAL_LINE: &str = "modal   C-space enters a mode, keys repeat until Escape";
const DIRECT_LINE: &str = "direct  C-space then one key per action, back to typing";

fn state(ribbon: Size) -> ViewState {
    let mut layout = Layout::new();
    let window = layout.allocate_window();
    let band = layout.bands()[0].id;
    layout.open(window, band, None, None, &LayoutOptions::default());
    ViewState {
        table: "root".to_owned(),
        window: Some(1),
        width: ribbon.cols,
        height: ribbon.rows,
        layout: Arc::new(layout),
        area: ribbon,
        ribbon,
        ..ViewState::default()
    }
}

fn save_style(scratch: &Scratch, source: &str) {
    scratch.user_file("keystyle.lua", source);
}

fn saved_file(scratch: &Scratch) -> Option<String> {
    fs::read_to_string(scratch.user().join("keystyle.lua")).ok()
}

fn prefix_entries(config: &Config) -> Vec<(String, Option<String>, Option<String>)> {
    let entries: Vec<mlua::Table> = eval(config, "return gband.keymap.list('prefix')");
    entries
        .into_iter()
        .map(|entry| {
            (
                entry.get("key").unwrap(),
                entry.get("action").unwrap(),
                entry.get("desc").unwrap(),
            )
        })
        .collect()
}

fn action_names(config: &Config) -> Vec<String> {
    eval(
        config,
        "local names = {} for _, a in ipairs(gband.action.list()) do names[#names + 1] = a.name end return names",
    )
}

fn label(config: &Config) -> Option<String> {
    eval(config, "return gband.keymap.label('prefix')")
}

fn press_root(config: &Config, name: &str) -> gband_lua::Outcome {
    let chord = Chord::Key(key(name));
    let Some((_, Binding::Callback(callback))) = config.keymap["root"]
        .iter()
        .find(|(bound, _)| *bound == chord)
    else {
        panic!("{name} is not bound to a function");
    };
    config.runtime.call(*callback)
}

fn text(runs: &[Run]) -> String {
    runs.iter().map(|run| run.text.as_str()).collect()
}

struct Chooser {
    scratch: Scratch,
    config: Config,
}

impl Chooser {
    fn new(name: &str) -> Self {
        Self::with(name, "", Size::new(60, 24))
    }

    fn with(name: &str, saved: &str, ribbon: Size) -> Self {
        let scratch = Scratch::new(name);
        if !saved.is_empty() {
            save_style(&scratch, saved);
        }
        scratch.write(JOB);
        let config = scratch.loaded();
        assert!(config.errors.is_empty(), "{:?}", config.errors);
        clean(&config.runtime.set_state(state(ribbon)));
        config.runtime.take_frames();
        Self { scratch, config }
    }

    fn choose(&self) -> gband_lua::Outcome {
        run_job(&self.config, "gband.keystyle.choose()")
    }

    fn windows(&self) -> Vec<u32> {
        eval(&self.config, "return gband.win.list()")
    }

    fn win(&self) -> u32 {
        self.windows()[0]
    }

    fn cursor(&self) -> i64 {
        eval(
            &self.config,
            &format!("return gband.win.info({}).cursor", self.win()),
        )
    }

    fn press(&self, name: &str) -> gband_lua::Outcome {
        self.config.runtime.plugin_window_key(self.win(), key(name))
    }

    fn frame(&self) -> FloatingFrame {
        match self.config.runtime.take_frames().pop() {
            Some((_, Some(Frame::Floating(frame)))) => frame,
            other => panic!("expected a floating frame, got {other:?}"),
        }
    }
}

#[test]
fn config_dir_from_xdg_config_home() {
    let scratch = Scratch::new("keystyle-config-dir-xdg");
    scratch.write("dir = gband.config_dir");
    let config = gband_lua::config_dir_from(Some(scratch.0.join("config").into()), None).unwrap();
    assert_eq!(config, scratch.dir());
    let loaded = gband_lua::load(
        &Locations {
            config,
            plugins: None,
        },
        Side::Client,
        &LoadOptions::default(),
    )
    .unwrap();
    assert_eq!(
        global::<String>(&loaded, "dir"),
        scratch.dir().to_string_lossy()
    );
}

#[test]
fn config_dir_from_the_home_directory_in_the_server() {
    let scratch = Scratch::new("keystyle-config-dir-home");
    let home = scratch.0.join("home");
    let config = gband_lua::config_dir_from(None, Some(home.clone().into())).unwrap();
    assert_eq!(config, home.join(".config").join("gband"));
    let path = config.join("user").join("server.lua");
    write(&path, "dir = gband.config_dir");
    let loaded = gband_lua::load(
        &Locations {
            config: config.clone(),
            plugins: None,
        },
        Side::Server,
        &LoadOptions::default(),
    )
    .unwrap();
    assert_eq!(global::<String>(&loaded, "dir"), config.to_string_lossy());
}

#[test]
fn config_dir_of_the_defaults_alone() {
    let config = gband_lua::defaults(Side::Client);
    let dir: Option<String> = eval(&config, "return gband.config_dir");
    assert_eq!(dir, None);
}

#[test]
fn assigning_config_dir_changes_nothing_else() {
    let scratch = Scratch::new("keystyle-config-dir-assign");
    scratch.write("gband.config_dir = '/nowhere'\nrequire('extra')");
    scratch.user_file("lua/extra.lua", "found = true");
    let config = scratch.loaded();
    assert!(global::<bool>(&config, "found"));
}

#[test]
fn preset_required_by_name() {
    for style in ["modal", "direct"] {
        let scratch = Scratch::new(&format!("keystyle-require-{style}"));
        scratch.write(&format!("require('gband.keystyle.{style}')"));
        let config = scratch.loaded();
        assert!(config.errors.is_empty(), "{:?}", config.errors);
        let entries = prefix_entries(&config);
        assert_eq!(entries[0].0, "h");
        assert_eq!(entries[0].1.as_deref(), Some("focus_column_left"));
        assert!(config.keymap.get("root").is_none_or(Vec::is_empty));
    }
}

#[test]
fn arrow_keys_beside_hjkl() {
    let twins = [("h", "left"), ("j", "down"), ("k", "up"), ("l", "right")];
    for style in ["modal", "direct"] {
        let scratch = Scratch::new(&format!("keystyle-arrows-{style}"));
        scratch.write(&format!("gband.keystyle.use('{style}')"));
        let entries = prefix_entries(&scratch.loaded());
        let action_of = |wanted: &str| {
            entries
                .iter()
                .find(|(key, _, _)| key == wanted)
                .and_then(|(_, action, _)| action.clone())
        };
        let mut checked = 0;
        for (key, action, _) in &entries {
            let (modifier, letter) = key.rsplit_once('+').unwrap_or(("", key));
            let Some((_, arrow)) = twins.iter().find(|(name, _)| *name == letter) else {
                continue;
            };
            let Some(action) = action else {
                continue;
            };
            let twin = match modifier {
                "" => arrow.to_string(),
                modifier => format!("{modifier}+{arrow}"),
            };
            assert_eq!(action_of(&twin).as_ref(), Some(action), "{style} {key}");
            checked += 1;
        }
        assert_eq!(checked, 8, "{style}");
    }
}

#[test]
fn every_preset_binding_is_described() {
    let functions = [
        ("n", "open a window"),
        ("escape", "interactive mode"),
        ("enter", "interactive mode"),
        ("prefix", "send the prefix key"),
    ];
    for style in ["modal", "direct"] {
        let scratch = Scratch::new(&format!("keystyle-described-{style}"));
        scratch.write(&format!("gband.keystyle.use('{style}')"));
        let config = scratch.loaded();
        let descs: Vec<(String, String)> = eval::<Vec<Vec<String>>>(
            &config,
            "local list = {} for _, a in ipairs(gband.action.list()) do list[#list + 1] = { a.name, a.desc } end return list",
        )
        .into_iter()
        .map(|pair| (pair[0].clone(), pair[1].clone()))
        .collect();
        for (key, action, desc) in prefix_entries(&config) {
            let expected = match &action {
                Some(action) => descs
                    .iter()
                    .find(|(name, _)| name == action)
                    .map(|(_, desc)| desc.as_str()),
                None => functions
                    .iter()
                    .find(|(name, _)| *name == key)
                    .map(|(_, desc)| *desc),
            };
            assert_eq!(desc.as_deref(), expected, "{style} {key}");
        }
    }
}

#[test]
fn bundled_plugins_set_up_by_the_preset() {
    for style in ["modal", "direct"] {
        let scratch = Scratch::new(&format!("keystyle-plugins-{style}"));
        scratch.write(&format!("gband.keystyle.use('{style}')"));
        let config = scratch.loaded();
        assert!(config.errors.is_empty(), "{:?}", config.errors);
        let names = action_names(&config);
        assert!(names.contains(&"keylist.open".to_owned()), "{style}");
        assert!(names.contains(&"prompt.open".to_owned()), "{style}");
        assert!(!names.contains(&"errors.open".to_owned()), "{style}");
        let components: i64 = eval(&config, "return #gband.ui.statusline.list()");
        let bars: i64 = eval(&config, "return #gband.bar.list()");
        assert_eq!((components, bars), (0, 0), "{style}");
    }
}

#[test]
fn preset_shadowed_by_a_user_module() {
    let scratch = Scratch::new("keystyle-shadowed");
    scratch.user_file(
        "lua/gband/keystyle/direct.lua",
        "gband.keymap.set('prefix', 'x', gband.action.detach)",
    );
    scratch.write("gband.keystyle.use('direct')");
    let config = scratch.loaded();
    let keys: Vec<String> = prefix_entries(&config)
        .into_iter()
        .map(|(key, _, _)| key)
        .collect();
    assert_eq!(keys, ["x"]);
}

#[test]
fn saved_style_reading() {
    let cases: [(Option<&str>, Option<&str>); 6] = [
        (None, None),
        (Some("return 'modal'"), Some("modal")),
        (Some("return \"direct\"\n"), Some("direct")),
        (Some("return \"vi\""), None),
        (Some("error(\"boom\")"), None),
        (Some("return ("), None),
    ];
    for (index, (source, expected)) in cases.into_iter().enumerate() {
        let scratch = Scratch::new(&format!("keystyle-saved-{index}"));
        if let Some(source) = source {
            save_style(&scratch, source);
        }
        scratch.write(JOB);
        let config = scratch.loaded();
        assert!(config.errors.is_empty(), "{source:?}: {:?}", config.errors);
        let outcome = run_job(&config, "found = gband.keystyle.saved()");
        clean(&outcome);
        let found: Option<String> = global(&config, "found");
        assert_eq!(found.as_deref(), expected, "{source:?}");
    }
}

#[test]
fn saved_style_without_a_configuration_directory() {
    let config = gband_lua::defaults(Side::Client);
    let found: Option<String> = eval(&config, "return gband.keystyle.saved()");
    assert_eq!(found, None);
}

#[test]
fn saved_file_gets_an_empty_environment() {
    let scratch = Scratch::new("keystyle-saved-environment");
    save_style(
        &scratch,
        "gband.bind('alt+y', gband.action.detach) return 'direct'",
    );
    scratch.write("style = gband.keystyle.use()");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    assert_eq!(global::<String>(&config, "style"), "modal");
}

#[test]
fn explicit_style() {
    let scratch = Scratch::new("keystyle-use-explicit");
    scratch.write("style = gband.keystyle.use('direct')");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "style"), "direct");
    assert_eq!(label(&config).as_deref(), Some("prefix"));
    assert!(config.modes.is_empty());
}

#[test]
fn saved_style_is_used() {
    let scratch = Scratch::new("keystyle-use-saved");
    save_style(&scratch, "return \"direct\"\n");
    scratch.write("style = gband.keystyle.use()");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "style"), "direct");
    let keys: Vec<String> = prefix_entries(&config)
        .into_iter()
        .map(|(key, _, _)| key)
        .collect();
    assert!(!keys.contains(&"escape".to_owned()));
}

#[test]
fn nothing_saved_uses_modal() {
    let scratch = Scratch::new("keystyle-use-nothing");
    scratch.write("style = gband.keystyle.use()");
    let config = scratch.loaded();
    assert_eq!(global::<String>(&config, "style"), "modal");
    assert_eq!(label(&config).as_deref(), Some("navigation"));
    assert!(config.modes.contains("prefix"));
}

#[test]
fn broken_saved_file_is_not_a_configuration_error() {
    let scratch = Scratch::new("keystyle-use-broken");
    save_style(&scratch, "error(\"boom\")");
    scratch.write("style = gband.keystyle.use()");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    assert_eq!(global::<String>(&config, "style"), "modal");
}

#[test]
fn unknown_style() {
    let scratch = Scratch::new("keystyle-use-unknown");
    let path = scratch.write("local a = 1\nlocal b = 2\ngband.keystyle.use('vi')");
    let error = scratch.load().err().expect("loading fails");
    assert_error_at(&error, &path, 3, "vi");
}

#[test]
fn used_twice() {
    let scratch = Scratch::new("keystyle-use-twice");
    let path = scratch
        .write("local a = 1\ngband.keystyle.use()\nlocal b = 2\ngband.keystyle.use('direct')");
    let error = scratch.load().err().expect("loading fails");
    assert_error_at(&error, &path, 4, "once");
}

#[test]
fn used_after_the_load() {
    let scratch = Scratch::new("keystyle-use-late");
    let path = scratch.write(
        "local a = 1\nlocal b = 2\nlocal c = 3\nlocal d = 4\ngband.bind('alt+x', function()\n  gband.keystyle.use('modal')\nend)",
    );
    let config = scratch.loaded();
    let outcome = press_root(&config, "alt+x");
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_error_at(error, &path, 6, "while the configuration loads");
}

#[test]
fn change_a_preset_binding() {
    let scratch = Scratch::new("keystyle-use-rebind");
    scratch.write("gband.keystyle.use()\ngband.keymap.set('prefix', 'q', gband.action.detach)");
    let config = scratch.loaded();
    let entries = prefix_entries(&config);
    let actions: Vec<Option<&str>> = entries
        .iter()
        .filter(|(key, _, _)| key == "q")
        .map(|(_, action, _)| action.as_deref())
        .collect();
    assert_eq!(actions, [Some("detach")]);
    assert!(
        !entries
            .iter()
            .any(|(_, action, _)| action.as_deref() == Some("close_window"))
    );
}

#[test]
fn chooser_opens() {
    let chooser = Chooser::new("keystyle-choose-opens");
    let outcome = chooser.choose();
    clean(&outcome);
    assert!(
        outcome
            .dispatched
            .contains(&gband_lua::Dispatch::Enter("root".to_owned()))
    );
    let frame = chooser.frame();
    assert_eq!(frame.title.as_deref(), Some(TITLE));
    assert!(frame.border.is_some());
    assert!(frame.focused);
    assert_eq!(text(&frame.lines[0]), MODAL_LINE);
    assert_eq!(text(&frame.lines[1]), DIRECT_LINE);
    assert!(frame.lines[0].iter().all(|run| run.style.reverse));
    assert_eq!(chooser.cursor(), 1);
}

#[test]
fn chooser_beside_the_default_status_line() {
    let chooser = Chooser::new("keystyle-choose-size");
    chooser.choose();
    let frame = chooser.frame();
    assert_eq!(
        (frame.col, frame.row, frame.width, frame.height),
        (1, 10, 57, 4)
    );
    assert_eq!(text(&frame.lines[0]).chars().count(), 55);
}

#[test]
fn chooser_in_a_narrow_ribbon_area() {
    let chooser = Chooser::with("keystyle-choose-narrow", "", Size::new(40, 24));
    chooser.choose();
    let frame = chooser.frame();
    assert_eq!(frame.width, 40);
    for (line, start) in frame.lines.iter().zip(["modal", "direct"]) {
        let shown = text(line);
        assert_eq!(shown.chars().count(), 38, "{shown}");
        assert!(shown.starts_with(start), "{shown}");
    }
}

#[test]
fn cursor_on_the_saved_style() {
    let chooser = Chooser::with(
        "keystyle-choose-saved",
        "return \"direct\"\n",
        Size::new(60, 24),
    );
    chooser.choose();
    assert_eq!(chooser.cursor(), 2);
}

#[test]
fn move_with_the_arrow_keys() {
    let chooser = Chooser::new("keystyle-choose-arrows");
    chooser.choose();
    for name in ["down", "up", "down"] {
        clean(&chooser.press(name));
    }
    assert_eq!(chooser.cursor(), 2);
}

#[test]
fn choose_while_loading() {
    let scratch = Scratch::new("keystyle-choose-loading");
    let path = scratch
        .write("local a = 1\nlocal b = 2\nlocal c = 3\nlocal d = 4\ngband.keystyle.choose()");
    let error = scratch.load().err().expect("loading fails");
    assert_error_at(&error, &path, 5, "while the configuration loads");
}

#[test]
fn open_again_while_open() {
    let chooser = Chooser::new("keystyle-choose-again");
    chooser.choose();
    let first = chooser.win();
    clean(&run_job(&chooser.config, "other = gband.win.open({})"));
    chooser.config.runtime.take_frames();
    clean(&chooser.choose());
    let windows = chooser.windows();
    assert_eq!(windows.len(), 2);
    assert!(eval::<bool>(
        &chooser.config,
        &format!("return gband.win.info({first}).focused")
    ));
}

#[test]
fn pick_saves_the_style() {
    let chooser = Chooser::new("keystyle-pick");
    let init = fs::read_to_string(chooser.scratch.user().join("init.lua")).unwrap();
    chooser.choose();
    clean(&chooser.press("j"));
    clean(&chooser.press("enter"));
    assert!(chooser.windows().is_empty());
    assert_eq!(
        saved_file(&chooser.scratch).as_deref(),
        Some("return \"direct\"\n")
    );
    assert_eq!(
        fs::read_to_string(chooser.scratch.user().join("init.lua")).unwrap(),
        init
    );
    let mut names: Vec<String> = fs::read_dir(chooser.scratch.user())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert_eq!(names, ["init.lua", "keystyle.lua"]);
}

#[test]
fn escape_saves_nothing() {
    let chooser = Chooser::new("keystyle-escape");
    chooser.choose();
    clean(&chooser.press("escape"));
    assert!(chooser.windows().is_empty());
    assert_eq!(saved_file(&chooser.scratch), None);
}

#[test]
fn cannot_save() {
    let chooser = Chooser::new("keystyle-read-only");
    let user = chooser.scratch.user();
    fs::set_permissions(&user, fs::Permissions::from_mode(0o555)).unwrap();
    if fs::write(user.join("probe"), "").is_ok() {
        return;
    }
    chooser.choose();
    clean(&chooser.press("j"));
    let outcome = chooser.press("enter");
    fs::set_permissions(&user, fs::Permissions::from_mode(0o755)).unwrap();
    assert!(chooser.windows().is_empty());
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert!(error.message.contains("user/keystyle.lua"), "{error}");
    assert_eq!(saved_file(&chooser.scratch), None);
    let names: Vec<String> = fs::read_dir(&user)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, ["init.lua"]);
}

#[test]
fn no_offer_without_a_configuration_directory() {
    let config = gband_lua::defaults(Side::Client);
    clean(&config.runtime.set_state(state(Size::new(60, 24))));
    clean(&config.runtime.emit(&Event::Attached {
        session: "main".to_owned(),
    }));
    let windows: Vec<u32> = eval(&config, "return gband.win.list()");
    assert!(windows.is_empty());
}

#[test]
fn offer_on_the_first_start() {
    let scratch = Scratch::new("keystyle-offer");
    let config =
        gband_lua::load(&scratch.locations(), Side::Client, &LoadOptions::default()).unwrap();
    clean(&config.runtime.set_state(state(Size::new(60, 24))));
    config.runtime.take_frames();
    clean(&config.runtime.emit(&Event::Attached {
        session: "main".to_owned(),
    }));
    let windows: Vec<u32> = eval(&config, "return gband.win.list()");
    assert_eq!(windows.len(), 1);
    let frames = config.runtime.take_frames();
    let Some((_, Some(Frame::Floating(frame)))) = frames.last() else {
        panic!("{frames:?}");
    };
    assert_eq!(frame.title.as_deref(), Some(TITLE));
    assert!(frame.focused);
}

#[test]
fn no_offer_with_a_saved_style() {
    let scratch = Scratch::new("keystyle-offer-saved");
    save_style(&scratch, "return \"modal\"\n");
    let config =
        gband_lua::load(&scratch.locations(), Side::Client, &LoadOptions::default()).unwrap();
    clean(&config.runtime.set_state(state(Size::new(60, 24))));
    clean(&config.runtime.emit(&Event::Attached {
        session: "main".to_owned(),
    }));
    let windows: Vec<u32> = eval(&config, "return gband.win.list()");
    assert!(windows.is_empty());
}
