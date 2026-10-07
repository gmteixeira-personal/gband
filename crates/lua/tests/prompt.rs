mod common;

use std::sync::Arc;

use common::*;
use gband_core::action::Action;
use gband_core::geometry::Size;
use gband_core::input::{Key, KeyCode};
use gband_core::layout::{Layout, LayoutOptions, WindowId};
use gband_core::view::ViewAction;
use gband_lua::plugin_windows::{FloatingFrame, Frame, Run};
use gband_lua::{Color, Config, Dispatch, Outcome, ViewState, WindowName};

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

struct Prompt {
    _scratch: Scratch,
    config: Config,
}

impl Prompt {
    fn new(name: &str) -> Self {
        Self::sized(name, "", Size::new(60, 24))
    }

    fn sized(name: &str, source: &str, ribbon: Size) -> Self {
        let scratch = Scratch::new(name);
        scratch.write(&format!("{JOB}gband.plugin('gband.prompt')\n{source}"));
        Self::loaded(scratch, scratch_config, ribbon)
    }

    fn loaded(scratch: Scratch, load: fn(&Scratch) -> Config, ribbon: Size) -> Self {
        let config = load(&scratch);
        assert!(config.errors.is_empty(), "{:?}", config.errors);
        clean(&config.runtime.set_state(state(ribbon)));
        config.runtime.take_frames();
        Self {
            _scratch: scratch,
            config,
        }
    }

    fn open(&self) -> Outcome {
        let outcome = run_job(&self.config, "gband.action['prompt.open']()");
        clean(&outcome);
        outcome
    }

    fn rename(&self) -> Outcome {
        let outcome = run_job(&self.config, "gband.action['prompt.rename']()");
        clean(&outcome);
        outcome
    }

    fn named(&self, manual: Option<&str>) {
        let mut view = state(Size::new(60, 24));
        view.names = Arc::new(
            [(
                WindowId(1),
                WindowName {
                    shown: manual.unwrap_or("bash").to_owned(),
                    manual: manual.map(str::to_owned),
                },
            )]
            .into(),
        );
        clean(&self.config.runtime.set_state(view));
        self.config.runtime.take_frames();
    }

    fn frame_of(&self, win: u32) -> FloatingFrame {
        let frames = self.config.runtime.take_frames();
        match frames.into_iter().rfind(|(id, _)| *id == win) {
            Some((_, Some(Frame::Floating(frame)))) => frame,
            other => panic!("expected a floating frame of {win}, got {other:?}"),
        }
    }

    fn win(&self) -> u32 {
        eval(&self.config, "return gband.win.list()[1]")
    }

    fn open_windows(&self) -> Vec<u32> {
        eval(&self.config, "return gband.win.list()")
    }

    fn press(&self, name: &str) -> Outcome {
        self.config.runtime.plugin_window_key(self.win(), key(name))
    }

    fn type_text(&self, text: &str) {
        let win = self.win();
        for character in text.chars() {
            let outcome = self
                .config
                .runtime
                .plugin_window_key(win, Key::plain(KeyCode::Char(character)));
            clean(&outcome);
            assert!(outcome.dispatched.is_empty());
        }
    }

    fn paste(&self, text: &str) {
        clean(&self.config.runtime.plugin_window_paste(self.win(), text));
    }

    fn run(&self, line: &str) -> Outcome {
        self.open();
        self.type_text(line);
        self.press("enter")
    }

    fn frame(&self) -> FloatingFrame {
        match self.config.runtime.take_frames().pop() {
            Some((_, Some(Frame::Floating(frame)))) => frame,
            other => panic!("expected a floating frame, got {other:?}"),
        }
    }

    fn shown(&self) -> String {
        text(&self.frame().lines[0])
    }
}

fn scratch_config(scratch: &Scratch) -> Config {
    scratch.loaded()
}

fn text(runs: &[Run]) -> String {
    runs.iter().map(|run| run.text.as_str()).collect()
}

fn cursor(runs: &[Run]) -> (usize, &Run) {
    let mut at = 0;
    for run in runs {
        if run.style.reverse || run.style.bg.is_some() {
            return (at, run);
        }
        at += run.text.chars().count();
    }
    panic!("no cursor in {runs:?}");
}

#[test]
fn action_registered() {
    let prompt = Prompt::new("prompt-registered");
    let desc: String = eval(
        &prompt.config,
        "for _, a in ipairs(gband.action.list()) do if a.name == 'prompt.open' then return a.desc end end",
    );
    assert_eq!(desc, "run Lua");
}

#[test]
fn require_gives_the_plugin_and_its_path() {
    let scratch = Scratch::new("prompt-require");
    scratch.write(
        "local module, path = require('gband.prompt')\n\
         name, where = module.name, path\n\
         setup = type(module.setup)",
    );
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    assert_eq!(global::<String>(&config, "name"), "prompt");
    assert_eq!(global::<String>(&config, "where"), "gband/prompt.lua");
    assert_eq!(global::<String>(&config, "setup"), "function");
}

#[test]
fn bundled_plugins_still_set_up_once() {
    let scratch = Scratch::new("prompt-bundled");
    scratch.write(
        "local before = gband.bar\n\
         first = gband.plugin('gband.sidebar')\n\
         second = gband.plugin('gband.errors')\n\
         same = before == gband.bar and require('gband.sidebar') == package.loaded['gband.sidebar']",
    );
    let config = scratch.loaded();
    assert!(config.errors.is_empty(), "{:?}", config.errors);
    for name in ["first", "second", "same"] {
        assert!(global::<bool>(&config, name), "{name}");
    }
}

#[test]
fn unknown_option() {
    let scratch = Scratch::new("prompt-unknown-option");
    scratch.write("ok = gband.plugin('gband.prompt', { title = 'eval' })");
    let config = scratch.loaded();
    assert!(!global::<bool>(&config, "ok"));
    let error = config.errors.first().expect("a plugin error");
    assert_eq!(error.plugin.as_deref(), Some("prompt"));
    assert!(error.message.contains("title"), "{error}");
}

#[test]
fn prompt_on_the_bottom_rows() {
    let prompt = Prompt::new("prompt-bottom");
    let outcome = prompt.open();
    assert!(
        outcome
            .dispatched
            .contains(&Dispatch::Enter("root".to_owned()))
    );
    let frame = prompt.frame();
    assert_eq!(
        (frame.col, frame.row, frame.width, frame.height),
        (0, 21, 60, 3)
    );
    assert_eq!(frame.title.as_deref(), Some("lua"));
    assert!(frame.border.is_some());
    assert!(frame.focused);
    let row = &frame.lines[0];
    assert_eq!(text(row).trim_end(), ":");
    let (at, run) = cursor(row);
    assert_eq!((at, run.text.as_str()), (1, " "));
    assert!(run.style.reverse);
}

#[test]
fn open_again_while_open() {
    let prompt = Prompt::new("prompt-again");
    prompt.open();
    prompt.type_text("ab");
    let first = prompt.win();
    clean(&run_job(&prompt.config, "other = gband.win.open({})"));
    let other: u32 = global(&prompt.config, "other");
    assert!(!eval::<bool>(
        &prompt.config,
        &format!("return gband.win.info({first}).focused")
    ));
    prompt.config.runtime.take_frames();
    prompt.open();
    assert_eq!(prompt.open_windows(), [first, other]);
    assert!(eval::<bool>(
        &prompt.config,
        &format!("return gband.win.info({first}).focused")
    ));
    let frames = prompt.config.runtime.take_frames();
    let Some((_, Some(Frame::Floating(frame)))) = frames.iter().find(|(id, _)| *id == first) else {
        panic!("the prompt was not drawn again: {frames:?}");
    };
    assert!(frame.focused);
    assert_eq!(text(&frame.lines[0]).trim_end(), ":ab");
}

#[test]
fn letters_that_scroll_elsewhere() {
    let prompt = Prompt::new("prompt-jkq");
    prompt.open();
    prompt.type_text("jkq");
    assert_eq!(prompt.open_windows().len(), 1);
    assert_eq!(prompt.shown().trim_end(), ":jkq");
}

#[test]
fn typed_text_and_the_cursor() {
    let prompt = Prompt::new("prompt-typed");
    prompt.open();
    prompt.type_text("abc");
    let frame = prompt.frame();
    let row = &frame.lines[0];
    assert_eq!(text(row).trim_end(), ":abc");
    let (at, run) = cursor(row);
    assert_eq!((at, run.text.as_str()), (4, " "));
    assert!(run.style.reverse);
}

#[test]
fn backspace() {
    let prompt = Prompt::new("prompt-backspace");
    prompt.open();
    prompt.type_text("aé");
    clean(&prompt.press("backspace"));
    assert_eq!(prompt.shown().trim_end(), ":a");
}

#[test]
fn backspace_on_an_empty_line() {
    let prompt = Prompt::new("prompt-backspace-empty");
    prompt.open();
    let outcome = prompt.press("backspace");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty());
    assert!(prompt.open_windows().is_empty());
}

#[test]
fn clear_the_line() {
    let prompt = Prompt::new("prompt-clear");
    prompt.open();
    prompt.type_text("abc");
    clean(&prompt.press("ctrl+u"));
    assert_eq!(prompt.shown().trim_end(), ":");
    assert_eq!(prompt.open_windows().len(), 1);
}

#[test]
fn escape() {
    let prompt = Prompt::new("prompt-escape");
    prompt.open();
    prompt.type_text("gband.action.detach()");
    let outcome = prompt.press("escape");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty());
    assert!(prompt.open_windows().is_empty());
}

#[test]
fn paste_with_line_breaks() {
    let prompt = Prompt::new("prompt-paste");
    prompt.open();
    prompt.paste("a = 1\nb = 2\r\n");
    let row = &prompt.frame().lines[0];
    assert_eq!(cursor(row).0, 13);
    assert_eq!(&text(row)[..13], ":a = 1 b = 2 ");
}

#[test]
fn long_line_shows_its_end() {
    let prompt = Prompt::sized("prompt-long", "", Size::new(12, 24));
    prompt.open();
    prompt.type_text("abcdefghijkl");
    let row = &prompt.frame().lines[0];
    assert_eq!(text(row), "defghijkl ");
    assert_eq!(cursor(row).0, 9);
}

#[test]
fn theme_sets_the_cursor() {
    let prompt = Prompt::sized(
        "prompt-theme",
        "gband.hl.set('PromptCursor', { bg = '#ff0000' })",
        Size::new(60, 24),
    );
    prompt.open();
    let row = &prompt.frame().lines[0];
    assert_eq!(cursor(row).1.style.bg, Some(Color::Rgb(0xff, 0, 0)));
}

#[test]
fn run_an_action() {
    let prompt = Prompt::new("prompt-action");
    let outcome = prompt.run("gband.action.focus_column_left()");
    clean(&outcome);
    assert_eq!(
        outcome.dispatched,
        [Dispatch::Action(Action::View(ViewAction::FocusLeft))]
    );
    assert!(prompt.open_windows().is_empty());
}

#[test]
fn open_a_plugin_window() {
    let prompt = Prompt::new("prompt-window");
    let outcome = prompt.run("gband.win.open({ lines = { 'hi' } })");
    clean(&outcome);
    let windows = prompt.open_windows();
    assert_eq!(windows.len(), 1);
    assert!(eval::<bool>(
        &prompt.config,
        "return gband.win.info(gband.win.list()[1]).focused"
    ));
    let frames = prompt.config.runtime.take_frames();
    let Some((_, Some(Frame::Floating(frame)))) = frames.iter().find(|(id, _)| *id == windows[0])
    else {
        panic!("the plugin window was not drawn: {frames:?}");
    };
    assert_eq!(text(&frame.lines[0]).trim_end(), "hi");
}

#[test]
fn names_are_not_namespaced() {
    let prompt = Prompt::new("prompt-names");
    clean(&prompt.run("id = gband.bar.add({ id = 'greet', side = 'left' })"));
    assert_eq!(global::<String>(&prompt.config, "id"), "greet");
    assert!(eval::<bool>(
        &prompt.config,
        "return gband.bar.list()[1].plugin == nil"
    ));
}

#[test]
fn runtime_error() {
    let prompt = Prompt::new("prompt-runtime-error");
    let outcome = prompt.run("error('boom')");
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_eq!(error.to_string(), "prompt:1: boom");
    assert!(prompt.open_windows().is_empty());
}

#[test]
fn syntax_error() {
    let prompt = Prompt::new("prompt-syntax-error");
    let outcome = prompt.run("gband.(");
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert!(error.to_string().starts_with("prompt:1:"), "{error}");
    assert!(outcome.dispatched.is_empty());
}

#[test]
fn empty_line() {
    let prompt = Prompt::new("prompt-empty");
    let outcome = prompt.run("");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty());
    assert!(prompt.open_windows().is_empty());
}

#[test]
fn endless_loop() {
    let scratch = Scratch::new("prompt-endless");
    scratch.write(&format!("{JOB}gband.plugin('gband.prompt')"));
    let prompt = Prompt::loaded(
        scratch,
        |scratch| scratch.load_with_budget(100_000).unwrap(),
        Size::new(60, 24),
    );
    let outcome = prompt.run("while true do end");
    let [error] = outcome.errors.as_slice() else {
        panic!("{:?}", outcome.errors);
    };
    assert_eq!(error.to_string(), "prompt:1: instruction limit exceeded");
    assert!(prompt.open_windows().is_empty());
    prompt.open();
    assert_eq!(prompt.open_windows().len(), 1);
    prompt.config.runtime.take_frames();
    clean(&prompt.config.runtime.refresh_plugins());
    assert_eq!(prompt.open_windows().len(), 1);
    prompt.type_text("ran = true");
    let outcome = prompt.press("enter");
    clean(&outcome);
    assert!(global::<bool>(&prompt.config, "ran"));
}

fn renamed(window: u32, name: Option<&str>) -> Dispatch {
    Dispatch::Rename {
        window: WindowId(window),
        name: name.map(str::to_owned),
    }
}

#[test]
fn rename_action_registered() {
    let prompt = Prompt::new("rename-registered");
    let desc: String = eval(
        &prompt.config,
        "for _, a in ipairs(gband.action.list()) do if a.name == 'prompt.rename' then return a.desc end end",
    );
    assert_eq!(desc, "rename the window");
}

#[test]
fn rename_the_focused_window() {
    let prompt = Prompt::new("rename-focused");
    let outcome = prompt.rename();
    assert!(
        outcome
            .dispatched
            .contains(&Dispatch::Enter("root".to_owned()))
    );
    let frame = prompt.frame();
    assert_eq!(
        (frame.col, frame.row, frame.width, frame.height),
        (0, 21, 60, 3)
    );
    assert_eq!(frame.title.as_deref(), Some("rename"));
    assert!(frame.focused);
    let row = &frame.lines[0];
    assert_eq!(text(row).trim_end(), "");
    assert_eq!(cursor(row).0, 0);
    prompt.type_text("logs");
    let row = &prompt.frame().lines[0];
    assert_eq!(text(row).trim_end(), "logs");
    assert_eq!(cursor(row).0, 4);
    let outcome = prompt.press("enter");
    clean(&outcome);
    assert_eq!(outcome.dispatched, [renamed(1, Some("logs"))]);
    assert!(prompt.open_windows().is_empty());
}

#[test]
fn rename_prompt_starts_with_the_manual_name() {
    let prompt = Prompt::new("rename-manual");
    prompt.named(Some("logs"));
    prompt.rename();
    let row = &prompt.frame().lines[0];
    assert_eq!(text(row).trim_end(), "logs");
    assert_eq!(cursor(row).0, 4);
}

#[test]
fn clear_by_renaming_to_nothing() {
    let prompt = Prompt::new("rename-clear");
    prompt.named(Some("logs"));
    prompt.rename();
    clean(&prompt.press("ctrl+u"));
    let outcome = prompt.press("enter");
    clean(&outcome);
    assert_eq!(outcome.dispatched, [renamed(1, None)]);
}

#[test]
fn escape_keeps_the_name() {
    let prompt = Prompt::new("rename-escape");
    prompt.named(Some("logs"));
    prompt.rename();
    prompt.type_text("x");
    let outcome = prompt.press("escape");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty());
    assert!(prompt.open_windows().is_empty());
}

#[test]
fn rename_in_an_empty_band() {
    let prompt = Prompt::new("rename-empty");
    let view = ViewState {
        window: None,
        layout: Arc::new(Layout::new()),
        ..state(Size::new(60, 24))
    };
    clean(&prompt.config.runtime.set_state(view));
    let outcome = prompt.rename();
    assert!(outcome.dispatched.is_empty(), "{:?}", outcome.dispatched);
    assert!(prompt.open_windows().is_empty());
}

#[test]
fn window_closed_while_renaming() {
    let prompt = Prompt::new("rename-closed");
    prompt.rename();
    prompt.type_text("logs");
    let view = ViewState {
        window: None,
        layout: Arc::new(Layout::new()),
        ..state(Size::new(60, 24))
    };
    clean(&prompt.config.runtime.set_state(view));
    let outcome = prompt.press("enter");
    clean(&outcome);
    assert!(outcome.dispatched.is_empty(), "{:?}", outcome.dispatched);
    assert!(prompt.open_windows().is_empty());
}

#[test]
fn rename_again_while_open_keeps_the_line() {
    let prompt = Prompt::new("rename-again");
    prompt.rename();
    prompt.type_text("ab");
    let first = prompt.win();
    prompt.named(Some("other"));
    prompt.rename();
    assert_eq!(prompt.open_windows(), [first]);
    let outcome = prompt.press("enter");
    assert_eq!(outcome.dispatched, [renamed(1, Some("ab"))]);
}

#[test]
fn lua_prompt_replaces_the_rename_prompt() {
    let prompt = Prompt::new("rename-replaced");
    prompt.rename();
    prompt.type_text("ab");
    prompt.config.runtime.take_frames();
    prompt.open();
    let windows = prompt.open_windows();
    assert_eq!(windows.len(), 1);
    let frame = prompt.frame_of(windows[0]);
    assert_eq!(frame.title.as_deref(), Some("lua"));
    assert_eq!(text(&frame.lines[0]).trim_end(), ":");
    assert!(eval::<bool>(
        &prompt.config,
        "return gband.win.info(gband.win.list()[1]).focused"
    ));
}

#[test]
fn rename_prompt_replaces_the_lua_prompt() {
    let prompt = Prompt::new("rename-replaces");
    prompt.open();
    prompt.type_text("ab");
    prompt.config.runtime.take_frames();
    prompt.rename();
    let windows = prompt.open_windows();
    assert_eq!(windows.len(), 1);
    assert_eq!(prompt.frame_of(windows[0]).title.as_deref(), Some("rename"));
    prompt.open();
    let windows = prompt.open_windows();
    assert_eq!(text(&prompt.frame_of(windows[0]).lines[0]).trim_end(), ":");
}
