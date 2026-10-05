mod common;

use std::io::Write;
use std::sync::{Arc, Mutex};

use common::*;
use gband_lua::{Color, Config};
use mlua::Table;

fn loaded(name: &str, source: &str) -> (Scratch, Config) {
    let scratch = Scratch::new(name);
    scratch.write(&format!("{JOB}{source}"));
    let config = scratch.loaded();
    (scratch, config)
}

fn fields(config: &Config, source: &str) -> Vec<String> {
    let table: Option<Table> = eval(config, source);
    let Some(table) = table else {
        return vec!["nil".to_owned()];
    };
    let mut fields: Vec<String> = table
        .pairs::<String, mlua::Value>()
        .map(|pair| {
            let (key, value) = pair.unwrap();
            let value = match value {
                mlua::Value::String(text) => text.to_string_lossy(),
                mlua::Value::Integer(number) => number.to_string(),
                mlua::Value::Boolean(flag) => flag.to_string(),
                other => format!("{other:?}"),
            };
            format!("{key}={value}")
        })
        .collect();
    fields.sort();
    fields
}

fn get(config: &Config, name: &str) -> Vec<String> {
    fields(config, &format!("return gband.hl.get('{name}')"))
}

fn resolved(config: &Config, name: &str) -> Vec<String> {
    fields(
        config,
        &format!("return gband.hl.get('{name}', {{ resolve = true }})"),
    )
}

#[test]
fn valid_spec() {
    let (_scratch, config) = loaded(
        "valid",
        "gband.hl.set('Title', { fg = '#FFAA00', bg = 236, bold = true })",
    );
    assert_eq!(get(&config, "Title"), ["bg=236", "bold=true", "fg=#ffaa00"]);
}

#[test]
fn named_color() {
    let (_scratch, config) = loaded(
        "named",
        "gband.hl.set('Warn', { fg = 'bright_red' })\ngband.hl.set('StatusLine', { fg = 'bright_red' })",
    );
    assert_eq!(get(&config, "Warn"), ["fg=bright_red"]);
    let line = presented(&config, drawn(20));
    assert_eq!(line.base.fg, Some(Color::Index(9)));
}

#[test]
fn invalid_color() {
    let scratch = Scratch::new("invalid-color");
    let path = scratch.write("\n\n\ngband.hl.set('Title', { fg = '#ffaa0' })");
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &path, 4, "fg");
}

#[test]
fn invalid_names_and_values() {
    for (source, mentions) in [
        ("gband.hl.set('1Title', {})", "1Title"),
        ("gband.hl.set('Ti tle', {})", "Ti tle"),
        ("gband.hl.set('Title', { bold = 1 })", "bold"),
        ("gband.hl.set('Title', { bg = 256 })", "bg"),
        ("gband.hl.set('Title', { fg = 'orange' })", "fg"),
        ("gband.hl.set('Title', { link = 'Title' })", "Title"),
        ("gband.hl.set('Title', 'red')", "Title"),
    ] {
        let scratch = Scratch::new("invalid-values");
        let path = scratch.write(source);
        let error = scratch.load().err().unwrap();
        assert_error_at(&error, &path, 1, mentions);
    }
}

#[test]
fn unknown_field() {
    let scratch = Scratch::new("unknown-field");
    let module = scratch.plugin_file(
        "hello",
        "lua/hello/init.lua",
        "return { setup = function()\n  gband.hl.set('Title', { colour = 1 })\nend }",
    );
    scratch.write("gband.plugin('hello')");
    let config = scratch.loaded();
    let [error] = config.errors.as_slice() else {
        panic!("{:?}", config.errors);
    };
    assert_eq!(error.plugin.as_deref(), Some("hello"));
    assert_error_at(error, &module, 2, "colour");
}

#[test]
fn invalid_set_leaves_the_group_unchanged() {
    let (_scratch, config) = loaded(
        "unchanged",
        "gband.hl.set('Title', { fg = 1 })\nok = pcall(gband.hl.set, 'Title', { fg = 'nope' })",
    );
    assert!(!global::<bool>(&config, "ok"));
    assert_eq!(get(&config, "Title"), ["fg=1"]);
}

#[test]
fn default_under_an_explicit_setting() {
    let scratch = Scratch::new("default-under");
    scratch.client_plugin(
        "hello",
        "gband.hl.default('HelloSegment', { fg = 4, bold = true })",
    );
    scratch.write("gband.hl.set('HelloSegment', { fg = 2 })");
    let config = scratch.loaded();
    assert_eq!(get(&config, "HelloSegment"), ["fg=2"]);
}

#[test]
fn default_alone() {
    let (_scratch, config) = loaded(
        "default-alone",
        "gband.hl.default('HelloSegment', { fg = 4 })",
    );
    assert_eq!(get(&config, "HelloSegment"), ["fg=4"]);
}

#[test]
fn removing_the_explicit_setting() {
    let (_scratch, config) = loaded(
        "remove",
        "gband.hl.default('G', { fg = 4 })\ngband.hl.set('G', { fg = 2 })\ngband.hl.set('G', nil)",
    );
    assert_eq!(get(&config, "G"), ["fg=4"]);
}

#[test]
fn empty_explicit_setting_hides_the_default() {
    let (_scratch, config) = loaded(
        "empty",
        "gband.hl.default('G', { fg = 4 })\ngband.hl.set('G', {})",
    );
    assert_eq!(get(&config, "G"), Vec::<String>::new());
    assert_eq!(resolved(&config, "G"), Vec::<String>::new());
}

#[test]
fn inheritance_with_an_override() {
    let (_scratch, config) = loaded(
        "inherit",
        "gband.hl.set('Base', { fg = 1, bg = 2, bold = true })\ngband.hl.set('Child', { link = 'Base', fg = 3 })",
    );
    assert_eq!(resolved(&config, "Child"), ["bg=2", "bold=true", "fg=3"]);
}

#[test]
fn chain() {
    let (_scratch, config) = loaded(
        "chain",
        "gband.hl.set('A', { link = 'B' })\ngband.hl.set('B', { link = 'C' })\ngband.hl.set('C', { fg = 5 })",
    );
    assert_eq!(resolved(&config, "A"), ["fg=5"]);
}

#[test]
fn link_to_an_undefined_group() {
    let (_scratch, config) = loaded(
        "undefined-link",
        "gband.hl.set('A', { link = 'Nowhere', bold = true })",
    );
    assert_eq!(resolved(&config, "A"), ["bold=true"]);
}

#[test]
fn later_change_of_the_target() {
    let (_scratch, config) = loaded(
        "later",
        "gband.hl.set('Base', { fg = 1 })\ngband.hl.set('Child', { link = 'Base' })",
    );
    clean(&run_job(&config, "gband.hl.set('Base', { fg = 6 })"));
    assert_eq!(resolved(&config, "Child"), ["fg=6"]);
}

#[test]
fn cycle_refused() {
    let scratch = Scratch::new("cycle");
    let path = scratch.write(
        "gband.hl.set('A', { link = 'B' })\ngband.hl.set('B', { link = 'C' })\n\n\n\n\ngband.hl.set('C', { link = 'A' })",
    );
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &path, 7, "A");
    assert!(
        error.message.contains('B') && error.message.contains('C'),
        "{error}"
    );
}

#[test]
fn default_cycle_refused() {
    let scratch = Scratch::new("default-cycle");
    let path = scratch
        .write("gband.hl.default('A', { link = 'B' })\ngband.hl.default('B', { link = 'A' })");
    let error = scratch.load().err().unwrap();
    assert_error_at(&error, &path, 2, "A");
}

#[derive(Clone, Default)]
struct Captured(Arc<Mutex<Vec<u8>>>);

impl Write for Captured {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn cycle_made_across_layers() {
    let (scratch, config) = loaded(
        "cross-layer",
        "gband.hl.default('A', { link = 'B' })\ngband.hl.set('B', { fg = 1 })\ngband.hl.default('B', { link = 'A' })",
    );
    scratch.user_file("colors/plain.lua", "");
    clean(&run_job(&config, "gband.colorscheme('plain')"));
    let captured = Captured::default();
    let writer = captured.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_ansi(false)
        .finish();
    let style = tracing::subscriber::with_default(subscriber, || {
        let first = resolved(&config, "A");
        resolved(&config, "A");
        first
    });
    assert_eq!(style, Vec::<String>::new());
    let log = String::from_utf8(captured.0.lock().unwrap().clone()).unwrap();
    assert!(log.contains("WARN"), "{log}");
    assert!(log.contains("A, B"), "{log}");
    assert_eq!(log.matches("cycle").count(), 1, "{log}");
}

#[test]
fn definition_and_resolved_style() {
    let (_scratch, config) = loaded(
        "definition",
        "gband.hl.set('Base', { fg = 1 })\ngband.hl.set('Child', { link = 'Base', bold = true })",
    );
    assert_eq!(get(&config, "Child"), ["bold=true", "link=Base"]);
    assert_eq!(resolved(&config, "Child"), ["bold=true", "fg=1"]);
}

#[test]
fn undefined_group() {
    let (_scratch, config) = loaded("undefined", "");
    assert_eq!(get(&config, "Absent"), ["nil"]);
    assert_eq!(resolved(&config, "Absent"), Vec::<String>::new());
}

#[test]
fn returned_tables_are_copies() {
    let (_scratch, config) = loaded(
        "copies",
        "gband.hl.set('Base', { fg = 1 })\ngband.hl.get('Base').fg = 2\ngband.hl.get('Base', { resolve = true }).fg = 3",
    );
    assert_eq!(get(&config, "Base"), ["fg=1"]);
}

const RECORD: &str =
    "log = {}\ngband.on('HighlightChanged', function(e) log[#log + 1] = e.group end)\n";

#[test]
fn change_in_a_callback() {
    let (_scratch, config) = loaded("change", RECORD);
    clean(&run_job(
        &config,
        "gband.hl.set('StatusLineAccent', { fg = 3 })",
    ));
    assert_eq!(global::<Vec<String>>(&config, "log"), ["StatusLineAccent"]);
    clean(&run_job(
        &config,
        "gband.hl.set('StatusLineAccent', { fg = 3 })",
    ));
    assert_eq!(global::<Vec<String>>(&config, "log"), ["StatusLineAccent"]);
    clean(&run_job(&config, "gband.hl.default('Other', { fg = 3 })"));
    assert_eq!(
        global::<Vec<String>>(&config, "log"),
        ["StatusLineAccent", "Other"]
    );
}

#[test]
fn no_event_during_the_load() {
    let (_scratch, config) = loaded(
        "no-event",
        &format!("{RECORD}gband.hl.set('Title', {{ fg = 1 }})"),
    );
    assert_eq!(global::<Vec<String>>(&config, "log"), Vec::<String>::new());
}
