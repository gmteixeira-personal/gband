mod common;

use std::sync::Arc;

use common::*;
use gband_core::geometry::Size;
use gband_core::layout::{Layout, LayoutOptions};
use gband_lua::plugin_windows::{FloatingFrame, Frame};
use gband_lua::{Config, Dispatch, Outcome, PluginWindowRequest, ViewState};

fn state(errors: &[String], ribbon: Size) -> ViewState {
    let mut layout = Layout::new();
    let window = layout.allocate_window();
    let band = layout.bands()[0].id;
    layout.open(window, band, None, None, &LayoutOptions::default());
    ViewState {
        table: "root".to_owned(),
        window: Some(1),
        width: ribbon.cols,
        height: ribbon.rows,
        error: errors.last().cloned(),
        errors: errors.to_vec(),
        layout: Arc::new(layout),
        area: ribbon,
        ribbon,
        ..ViewState::default()
    }
}

fn client(name: &str, setup: &str, errors: &[String]) -> (Scratch, Config) {
    let scratch = Scratch::new(name);
    scratch.write(&format!("{JOB}gband.plugin('gband.errors'{setup})"));
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    clean(&config.runtime.set_state(state(errors, Size::new(60, 24))));
    config.runtime.take_frames();
    (scratch, config)
}

fn open(config: &Config) -> Outcome {
    let outcome = run_job(config, "gband.action['errors.open']()");
    clean(&outcome);
    outcome
}

fn float(config: &Config) -> FloatingFrame {
    match config.runtime.take_frames().pop() {
        Some((_, Some(Frame::Floating(frame)))) => frame,
        other => panic!("expected a floating frame, got {other:?}"),
    }
}

fn texts(frame: &FloatingFrame) -> Vec<String> {
    frame
        .lines
        .iter()
        .map(|runs| {
            runs.iter()
                .map(|run| run.text.as_str())
                .collect::<String>()
                .trim_end()
                .to_owned()
        })
        .collect()
}

fn errors(texts: &[&str]) -> Vec<String> {
    texts.iter().map(|text| (*text).to_owned()).collect()
}

#[test]
fn action_registered() {
    let (_scratch, config) = client("registered", "", &[]);
    let desc: String = eval(
        &config,
        "for _, a in ipairs(gband.action.list()) do if a.name == 'errors.open' then return a.desc end end",
    );
    assert_eq!(desc, "list the errors");
    let args: Vec<String> = eval(
        &config,
        "for _, c in ipairs(gband.cmd.list()) do if c.name == 'errors.open' then return c.args end end",
    );
    assert_eq!(args, ["kind"]);
}

#[test]
fn floating_plugin_window_of_the_defaults() {
    let (_scratch, config) = client(
        "floating",
        "",
        &errors(&["alpha: init.lua:1: first", "beta: init.lua:2: second"]),
    );
    let outcome = open(&config);
    assert!(
        outcome
            .dispatched
            .contains(&Dispatch::Enter("root".to_owned()))
    );
    let frame = float(&config);
    assert_eq!((frame.width, frame.height), (45, 12));
    assert_eq!((frame.col, frame.row), (7, 6));
    assert_eq!(frame.title.as_deref(), Some("errors"));
    assert!(frame.border.is_some());
    assert!(frame.focused);
    assert_eq!(
        texts(&frame)[..3],
        ["alpha: init.lua:1: first", "", "beta: init.lua:2: second"]
    );
}

#[test]
fn long_error_wrapped() {
    let long = "x".repeat(100);
    let (_scratch, config) = client("wrapped", "", &[long]);
    open(&config);
    let frame = float(&config);
    let lines = texts(&frame);
    let widths: Vec<usize> = lines[..3].iter().map(|line| line.len()).collect();
    assert_eq!(widths, [43, 43, 14]);
}

#[test]
fn wrapped_by_display_width() {
    let (_scratch, config) = client("wrapped-wide", "", &["日".repeat(30)]);
    open(&config);
    let lines = texts(&float(&config));
    assert_eq!(gband_lua::ui::width(&lines[0]), 42);
    assert_eq!(gband_lua::ui::width(&lines[1]), 18);
}

#[test]
fn wrapped_again_on_resize() {
    let long = "x".repeat(100);
    let (_scratch, config) = client("rewrapped", "", std::slice::from_ref(&long));
    open(&config);
    float(&config);
    clean(&config.runtime.set_state(state(&[long], Size::new(40, 24))));
    let frame = float(&config);
    assert_eq!(frame.width, 40);
    let widths: Vec<usize> = texts(&frame)[..3].iter().map(|line| line.len()).collect();
    assert_eq!(widths, [38, 38, 24]);
}

#[test]
fn no_errors() {
    let (_scratch, config) = client("none", "", &[]);
    open(&config);
    assert_eq!(texts(&float(&config))[0], "no errors");
}

#[test]
fn opening_again_focuses_the_open_list() {
    let (_scratch, config) = client("again", "", &[]);
    open(&config);
    open(&config);
    let open: Vec<u32> = eval(&config, "return gband.win.list()");
    assert_eq!(open.len(), 1);
}

#[test]
fn close_with_q_and_escape() {
    for key in ["q", "escape"] {
        let (_scratch, config) = client("close", "", &[]);
        open(&config);
        let [id]: [u32; 1] = eval::<Vec<u32>>(&config, "return gband.win.list()")
            .try_into()
            .unwrap();
        clean(
            &config
                .runtime
                .plugin_window_key(id, gband_lua::keys::parse_key(key).unwrap()),
        );
        let open: Vec<u32> = eval(&config, "return gband.win.list()");
        assert!(open.is_empty(), "{key}");
    }
}

fn tiled_request(outcome: &Outcome) -> bool {
    outcome.dispatched.iter().any(|entry| {
        matches!(
            entry,
            Dispatch::PluginWindow(PluginWindowRequest::Open { focus: true, .. })
        )
    })
}

#[test]
fn tiled_by_option() {
    let (_scratch, config) = client("tiled", ", { kind = 'tiled' }", &[]);
    assert!(tiled_request(&open(&config)));
    assert!(config.runtime.take_frames().is_empty());
}

#[test]
fn kind_from_the_command() {
    let (_scratch, config) = client("command", "", &[]);
    let outcome = run_job(&config, "gband.cmd.run('errors.open', { kind = 'tiled' })");
    clean(&outcome);
    assert!(tiled_request(&outcome));
    let (_scratch, config) = client("command-default", "", &[]);
    clean(&run_job(&config, "gband.cmd.run('errors.open')"));
    float(&config);
}

#[test]
fn command_with_another_kind() {
    let (_scratch, config) = client("command-bad", "", &[]);
    let outcome = run_job(&config, "gband.cmd.run('errors.open', { kind = 'popup' })");
    assert!(
        outcome
            .errors
            .iter()
            .any(|error| error.message.contains("kind")),
        "{:?}",
        outcome.errors
    );
}

#[test]
fn unknown_option() {
    for (opts, mentions) in [("{ width = 40 }", "width"), ("{ kind = 'popup' }", "kind")] {
        let scratch = Scratch::new("unknown-option");
        scratch.write(&format!("ok = gband.plugin('gband.errors', {opts})"));
        let config = scratch.loaded();
        assert!(!global::<bool>(&config, "ok"), "{opts}");
        let error = config.errors.first().expect("a plugin error");
        assert_eq!(error.plugin.as_deref(), Some("errors"));
        assert!(error.message.contains(mentions), "{error}");
    }
}
