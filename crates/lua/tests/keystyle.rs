mod common;

use common::*;
use gband_lua::{Binding, Chord, Config, LoadOptions, Locations, Side};

fn save_style(scratch: &Scratch, source: &str) {
    scratch.user_file("keystyle.lua", source);
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
    }
}

#[test]
fn only_mouse_names_in_root() {
    for style in ["modal", "direct"] {
        let scratch = Scratch::new(&format!("keystyle-root-{style}"));
        scratch.write(&format!("gband.keystyle.use('{style}')"));
        let config = scratch.loaded();
        let keys: Vec<String> = eval(
            &config,
            "local keys = {} for _, entry in ipairs(gband.keymap.list('root')) do keys[#keys + 1] = entry.key end return keys",
        );
        assert_eq!(
            keys,
            [
                "mod+leftmouse",
                "mod+rightmouse",
                "mod+middlemouse",
                "mod+wheeldown",
                "mod+wheelup",
            ],
            "{style}"
        );
        assert!(
            config.keymap["root"]
                .iter()
                .all(|(chord, _)| matches!(chord, Chord::Mouse { uses_mod: true, .. })),
            "{style}"
        );
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
        ("s", "settings"),
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
        assert!(!names.contains(&"desktop.list".to_owned()), "{style}");
        let bars: i64 = eval(&config, "return #gband.bar.list()");
        assert_eq!(bars, 0, "{style}");
    }
}

fn floating_preset(name: &str) -> Config {
    let scratch = Scratch::new(name);
    scratch.write("gband.keystyle.use('floating')");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    config
}

fn root_entries(config: &Config) -> Vec<(String, Option<String>)> {
    let entries: Vec<mlua::Table> = eval(config, "return gband.keymap.list('root')");
    entries
        .into_iter()
        .map(|entry| (entry.get("key").unwrap(), entry.get("action").unwrap()))
        .collect()
}

const FLOATING_ROOT: [&str; 7] = [
    "mod+leftmouse",
    "mod+rightmouse",
    "mod+middlemouse",
    "mod+wheeldown",
    "mod+wheelup",
    "leftmouse",
    "rightmouse",
];

#[test]
fn floating_prefix_bindings_in_order() {
    let entries = prefix_entries(&floating_preset("keystyle-floating-prefix"));
    let keys: Vec<&str> = entries.iter().map(|(key, _, _)| key.as_str()).collect();
    assert_eq!(keys, ["n", "?", ":", "N", "s", "!", "D", "prefix"]);
    assert_eq!(
        entries[0],
        (
            "n".to_owned(),
            None,
            Some("open a floating window".to_owned())
        )
    );
}

#[test]
fn floating_root_bindings_in_order() {
    let entries = root_entries(&floating_preset("keystyle-floating-root"));
    let keys: Vec<&str> = entries.iter().map(|(key, _)| key.as_str()).collect();
    assert_eq!(keys, FLOATING_ROOT);
    assert_eq!(entries[5].1.as_deref(), Some("desktop.press"));
    assert_eq!(entries[6].1.as_deref(), Some("desktop.menu"));
}

#[test]
fn plugins_set_up_by_the_floating_preset() {
    let config = floating_preset("keystyle-floating-plugins");
    let names = action_names(&config);
    for name in ["keylist.open", "prompt.open", "desktop.list"] {
        assert!(names.contains(&name.to_owned()), "{name}");
    }
    assert!(!names.contains(&"errors.open".to_owned()));
    let bars: i64 = eval(&config, "return #gband.bar.list()");
    assert_eq!(bars, 0);
}

#[test]
fn floating_preset_required_by_name() {
    let scratch = Scratch::new("keystyle-floating-require");
    scratch.write("require('gband.keystyle.floating')");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    let keys: Vec<String> = root_entries(&config)
        .into_iter()
        .map(|(key, _)| key)
        .collect();
    assert_eq!(keys[keys.len() - 2..], ["leftmouse", "rightmouse"]);
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
    let cases: [(Option<&str>, Option<&str>); 7] = [
        (None, None),
        (Some("return 'modal'"), Some("modal")),
        (Some("return \"direct\"\n"), Some("direct")),
        (Some("return \"floating\""), Some("floating")),
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
fn floating_saved() {
    let scratch = Scratch::new("keystyle-saved-floating");
    save_style(&scratch, "return \"floating\"\n");
    scratch.write(JOB);
    let config = scratch.loaded();
    clean(&run_job(&config, "found = gband.keystyle.saved()"));
    assert_eq!(global::<String>(&config, "found"), "floating");
}

#[test]
fn current_style_without_use() {
    let scratch = Scratch::new("keystyle-current-without-use");
    scratch.write(&format!("require('gband.keystyle.floating')\n{JOB}"));
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    clean(&run_job(&config, "found = gband.keystyle.current()"));
    let found: Option<String> = global(&config, "found");
    assert_eq!(found, None);
}

#[test]
fn current_style_after_the_saved_style() {
    let scratch = Scratch::new("keystyle-current-saved");
    save_style(&scratch, "return \"direct\"\n");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    let found: Option<String> = eval(&config, "return gband.keystyle.current()");
    assert_eq!(found.as_deref(), Some("direct"));
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
fn floating_style() {
    let scratch = Scratch::new("keystyle-use-floating");
    scratch.write("style = gband.keystyle.use('floating')");
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    assert_eq!(global::<String>(&config, "style"), "floating");
    assert_eq!(label(&config).as_deref(), Some("prefix"));
    let current: Option<String> = eval(&config, "return gband.keystyle.current()");
    assert_eq!(current.as_deref(), Some("floating"));
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
    assert!(
        error.message.contains(r#""modal", "direct" or "floating""#),
        "{error}"
    );
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
