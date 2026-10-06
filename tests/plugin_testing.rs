mod common;

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use common::*;
use gband_harness::channel::Listener;
use rustix::process::{Pid, Signal};

struct Project {
    root: PathBuf,
    dir: PathBuf,
}

impl Project {
    fn new(name: &str) -> Self {
        Self::named(name, name)
    }

    fn named(name: &str, directory: &str) -> Self {
        let root = scratch_root("gband-pt", name);
        if root.exists() {
            fs::remove_dir_all(&root).unwrap();
        }
        let dir = root.join(directory);
        fs::create_dir_all(&dir).unwrap();
        Self { root, dir }
    }

    fn file(&self, relative: &str, contents: &str) -> PathBuf {
        let path = self.dir.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, contents).unwrap();
        path
    }

    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new(GBAND);
        command
            .arg("test")
            .args(args)
            .current_dir(&self.dir)
            .env_remove("GBAND")
            .env_remove("GBAND_SESSION")
            .env_remove("GBAND_WINDOW")
            .env_remove("GBAND_TEST_SOCKET")
            .env("XDG_STATE_HOME", self.root.join("outer-state"))
            .env("XDG_CONFIG_HOME", self.root.join("outer-config"))
            .env("XDG_DATA_HOME", self.root.join("outer-data"))
            .env("XDG_RUNTIME_DIR", self.root.join("outer-run"))
            .stdin(Stdio::null());
        command
    }

    fn run(&self, args: &[&str]) -> Output {
        self.command(args).output().unwrap()
    }

    fn run_stdin(&self, args: &[&str], input: &str) -> Output {
        let mut child = self
            .command(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn report(output: &Output) -> String {
    format!(
        "status {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        stdout(output),
        stderr(output)
    )
}

fn assert_passed(output: &Output) {
    assert_eq!(output.status.code(), Some(0), "{}", report(output));
}

fn line_with<'a>(text: &'a str, needle: &str) -> &'a str {
    text.lines()
        .find(|line| line.contains(needle))
        .unwrap_or_else(|| panic!("no line holds {needle:?}:\n{text}"))
}

fn position(text: &str, needle: &str) -> usize {
    text.find(needle)
        .unwrap_or_else(|| panic!("{needle:?} is not in:\n{text}"))
}

const MANIFEST: &str = "return { name = 'window', version = '0.1.0' }";

#[test]
fn plugin_tests_found() {
    let project = Project::new("found");
    project.file("plugin.lua", MANIFEST);
    project.file(
        "tests/a_spec.lua",
        "local t = require('gband.test')\nlocal helpers = require('helpers')\n\
         t.case('a', function() print(helpers.name .. ' a') end)\n",
    );
    project.file(
        "tests/ui/b_spec.lua",
        "local t = require('gband.test')\nt.case('b', function() print('ran b') end)\n",
    );
    project.file("tests/helpers.lua", "return { name = 'helped' }\n");
    let output = project.run(&[]);
    assert_passed(&output);
    let text = stdout(&output);
    assert!(
        position(&text, "helped a") < position(&text, "ran b"),
        "{text}"
    );
    line_with(&text, "PASS tests/a_spec.lua > a");
    line_with(&text, "PASS tests/ui/b_spec.lua > b");
    assert!(!text.contains("helpers.lua"), "{text}");
    assert!(text.ends_with("2 passed, 0 failed\n"), "{text}");
}

#[test]
fn no_tests() {
    let project = Project::new("none");
    let output = project.run(&[]);
    assert_eq!(output.status.code(), Some(2), "{}", report(&output));
    assert_eq!(stderr(&output).lines().count(), 1, "{}", report(&output));
    assert!(
        stderr(&output).contains("no test file"),
        "{}",
        report(&output)
    );
    assert_eq!(stdout(&output), "");
}

#[test]
fn duplicate_plugin_names() {
    let project = Project::named("duplicate", "window");
    project.file("plugin.lua", MANIFEST);
    project.file("tests/a_spec.lua", "");
    let other = project.root.join("elsewhere").join("window");
    fs::create_dir_all(&other).unwrap();
    let output = project.run(&["--plugin", other.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(2), "{}", report(&output));
    assert!(stderr(&output).contains("`window`"), "{}", report(&output));
}

#[test]
fn cases_and_assertions() {
    let project = Project::new("cases");
    project.file(
        "tests/cases_spec.lua",
        "local t = require('gband.test')\n\
         t.case('one', function() print('one') end)\n\
         t.case('two', function() print('two') end)\n\
         t.case('fails', function() error('boom') end)\n\
         t.case('deep', function() t.eq({ a = { 1, 2 } }, { a = { 1, 2 } }) end)\n\
         t.case('names', function()\n\
           t.eq(3, 4)\n\
         end)\n\
         t.case('message', function() t.ok(false, 'it was false') end)\n\
         t.case('pattern', function() t.match('hello', '^w') end)\n\
         t.case('tables', function() t.eq({ 1, x = 'y' }, { 1, x = 'z' }) end)\n\
         t.case('after', function() print('test side', gband.side) end)\n",
    );
    let output = project.run(&[]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let text = stdout(&output);
    assert!(
        position(&text, "one\n") < position(&text, "two\n"),
        "{text}"
    );
    line_with(&text, "FAIL tests/cases_spec.lua > fails");
    line_with(&text, "tests/cases_spec.lua:4: boom");
    line_with(&text, "PASS tests/cases_spec.lua > deep");
    line_with(&text, "FAIL tests/cases_spec.lua > names");
    line_with(&text, "tests/cases_spec.lua:7: expected 4, got 3");
    line_with(
        &text,
        "tests/cases_spec.lua:9: it was false: expected a value other than nil and false, got false",
    );
    line_with(&text, "expected \"hello\" to match \"^w\"");
    line_with(&text, "expected { 1, x = \"z\" }, got { 1, x = \"y\" }");
    line_with(&text, "test side\ttest");
    line_with(&text, "PASS tests/cases_spec.lua > after");
    assert!(text.ends_with("4 passed, 5 failed\n"), "{text}");
}

#[test]
fn invalid_cases_and_files() {
    let project = Project::new("invalid");
    project.file(
        "tests/a_spec.lua",
        "local t = require('gband.test')\nt.case('x', function() end)\nt.case('x', function() end)\n",
    );
    project.file(
        "tests/b_spec.lua",
        "local t = require('gband.test')\nt.case('never', function() end)\nreturn gband.opt\n",
    );
    let output = project.run(&[]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let text = stdout(&output);
    line_with(&text, "FAIL tests/a_spec.lua");
    line_with(
        &text,
        "tests/a_spec.lua:3: the case \"x\" is already registered",
    );
    line_with(
        &text,
        "tests/b_spec.lua:3: `gband.opt` is a client and server API",
    );
    assert!(!text.contains("never"), "{text}");
    assert!(
        text.ends_with("0 passed, 0 failed, 2 files failed\n"),
        "{text}"
    );
}

#[test]
fn endless_case() {
    let project = Project::new("endless");
    project.file(
        "tests/endless_spec.lua",
        "local t = require('gband.test')\n\
         t.case('endless', { timeout = 2 }, function() while true do end end)\n\
         t.case('next', function() end)\n",
    );
    let started = Instant::now();
    let output = project.run(&[]);
    let elapsed = started.elapsed();
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let text = stdout(&output);
    line_with(&text, "FAIL tests/endless_spec.lua > endless");
    line_with(&text, "time limit of 2 seconds");
    line_with(&text, "PASS tests/endless_spec.lua > next");
    assert!(
        elapsed >= Duration::from_millis(1900) && elapsed < Duration::from_secs(8),
        "{elapsed:?}"
    );
}

#[test]
fn filter() {
    let project = Project::new("filter");
    project.file(
        "tests/filter_spec.lua",
        "local t = require('gband.test')\n\
         t.case('colour', function() end)\n\
         t.case('colour two', function() end)\n\
         t.case('width', function() end)\n",
    );
    let output = project.run(&["--filter", "colour"]);
    assert_passed(&output);
    let text = stdout(&output);
    line_with(&text, "> colour (");
    line_with(&text, "> colour two (");
    assert!(!text.contains("width"), "{text}");
    assert!(text.ends_with("2 passed, 0 failed\n"), "{text}");
}

#[test]
fn case_environment() {
    let project = Project::named("environment", "window");
    project.file("plugin.lua", MANIFEST);
    project.file(
        "lua/window/init.lua",
        "local M = { name = 'window' }\n\
         function M.setup()\n\
           gband.ui.statusline.add({ align = 'bottom', render = function() return 'WINDOW-SEGMENT' end })\n\
         end\n\
         return M\n",
    );
    project.file(
        "tests/environment_spec.lua",
        r##"local t = require("gband.test")

t.case("configuration of the case", function(g)
  g.start({
    size = "40x6",
    config = [[
      gband.plugin("gband.statusline", { side = "right" })
      gband.plugin("gband.statusline.band")
    ]],
  })
  t.match(g.screen().row(0), "┐band 1 *$")
  t.eq(g.screen().cols, 40)
  t.eq(g.screen().rows, 6)
end)

t.case("plugin under test is installed", function(g)
  g.start({ config = [[gband.plugin("gband.statusline")
gband.plugin("window")]] })
  t.match(g.screen().text(), "WINDOW%-SEGMENT")
end)

t.case("files and env", function(g)
  g.start({
    files = { ["user/lua/extra.lua"] = "return 'from files'" },
    config = [[gband.plugin("gband.statusline.band")]],
    env = { CASE_VALUE = "set by env", LANG = false },
  })
  t.eq(g.client("return require('extra')"), "from files")
  g.run('echo "[$CASE_VALUE] [$LANG] [$GBAND_TEST_SOCKET] [$TZ] [$COLORTERM]"')
  g.wait_text("[set by env] [] [] [UTC] [truecolor]")
end)

t.case("call order", function(g)
  local ok, err = pcall(function()
    g.keys("enter")
  end)
  t.eq(ok, false)
  t.match(err, "environment_spec.lua:35: call g.start before g.keys")
  g.start()
  ok, err = pcall(g.start)
  t.match(err, "already called")
end)
"##,
    );
    project.file(
        "../outer-config/gband/user/init.lua",
        "error('the outer configuration was read')",
    );
    let output = project.run(&[]);
    assert_passed(&output);
}

#[test]
fn key_style_of_a_case() {
    let project = Project::new("keystyle");
    project.file(
        "tests/keystyle_spec.lua",
        r#"local t = require("gband.test")

t.case("default configuration without the chooser", function(g)
  g.start()
  g.settle()
  t.eq(g.client("return #gband.win.list()"), 0)
  t.eq(g.client("return gband.keystyle.saved()"), "modal")
end)

t.case("first start in a case", function(g)
  g.start({ keystyle = false })
  g.settle()
  t.eq(g.client("return #gband.win.list()"), 1)
  t.eq(g.client("return gband.win.info(gband.win.list()[1]).focused"), true)
end)

t.case("direct style in a case", function(g)
  g.start({ keystyle = "direct" })
  t.eq(g.client('return gband.keymap.label("prefix")'), "prefix")
end)

t.case("files replace the saved style", function(g)
  g.start({ keystyle = "direct", files = { ["user/keystyle.lua"] = 'return "modal"' } })
  t.eq(g.client("return gband.keystyle.saved()"), "modal")
end)

t.case("unknown key style", function(g)
  local ok, err = pcall(function()
    g.start({ keystyle = "vi" })
  end)
  t.eq(ok, false)
  t.match(err, "keystyle_spec.lua:29: `keystyle` of g.start")
  ok, err = pcall(function()
    g.start({ keystyle = true })
  end)
  t.match(err, "keystyle_spec.lua:34: `keystyle` of g.start")
end)
"#,
    );
    let output = project.run(&[]);
    assert_passed(&output);
}

fn processes_of(root_prefix: &str) -> Vec<i32> {
    let Ok(entries) = fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .filter_map(|entry| entry.ok()?.file_name().to_str()?.parse::<i32>().ok())
        .filter(|&pid| {
            fs::read(format!("/proc/{pid}/environ"))
                .is_ok_and(|environ| String::from_utf8_lossy(&environ).contains(root_prefix))
        })
        .filter(|&pid| is_running(pid))
        .collect()
}

#[test]
fn nothing_left_behind() {
    let project = Project::new("left-behind");
    project.file(
        "tests/left_spec.lua",
        "local t = require('gband.test')\n\
         t.case('three windows', function(g)\n\
           g.start()\n\
           g.keys('ctrl+space n ctrl+space n')\n\
           g.settle()\n\
           t.eq(g.client('return #gband.layout().bands[1].columns'), 3)\n\
           error('fails on purpose')\n\
         end)\n",
    );
    let child = project
        .command(&[])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let prefix = format!("gband-test-{}-", child.id());
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    line_with(&stdout(&output), "fails on purpose");
    let leftovers: Vec<_> = fs::read_dir(std::env::temp_dir())
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().starts_with(&prefix))
        .collect();
    assert!(leftovers.is_empty(), "{leftovers:?}");
    let running = processes_of(&prefix);
    assert!(running.is_empty(), "{running:?}");
}

#[test]
fn run_from_inside_a_window() {
    let project = Project::new("inside");
    project.file(
        "tests/inside_spec.lua",
        "local t = require('gband.test')\n\
         t.case('nested', function(g)\n\
           g.start()\n\
           t.eq(g.client('return gband.side'), 'client')\n\
         end)\n",
    );
    let output = project
        .command(&[])
        .env("GBAND", "/tmp/some/default.sock")
        .env("GBAND_SESSION", "work")
        .env("GBAND_WINDOW", "3")
        .output()
        .unwrap();
    assert_passed(&output);
}

#[test]
fn users_server_untouched() {
    let env = TestEnv::new("pt-user-server");
    let mut server = env.start_server("/bin/sh");
    let project = Project::new("untouched");
    project.file(
        "tests/untouched_spec.lua",
        "local t = require('gband.test')\n\
         t.case('opens windows', function(g)\n\
           g.start()\n\
           g.keys('ctrl+space n ctrl+space n')\n\
           g.settle()\n\
         end)\n",
    );
    let output = project
        .command(&[])
        .env("XDG_STATE_HOME", env.state_home())
        .env("XDG_RUNTIME_DIR", env.runtime_home())
        .env("XDG_CONFIG_HOME", env.config_home())
        .env("XDG_DATA_HOME", env.data_home())
        .output()
        .unwrap();
    assert_passed(&output);
    let listed = env.command(GBAND, &["list-sessions"]).output().unwrap();
    assert_eq!(String::from_utf8_lossy(&listed.stdout), "default\t1\t0\n");
    let _ = server.kill();
    let _ = server.wait();
}

#[test]
fn driving_a_case() {
    let project = Project::new("driving");
    project.file(
        "tests/driving_spec.lua",
        r##"local t = require("gband.test")

t.case("wait for program output", function(g)
  g.start({ size = "60x12" })
  g.run("echo hi-$((1 + 1))")
  g.wait_text("hi-2")
  t.eq(g.wait(function(screen)
    return screen.text():find("hi%-2") and "found"
  end), "found")
end)

t.case("open a window by key", function(g)
  g.start({ size = "60x12" })
  g.keys("ctrl+space n")
  g.settle()
  t.eq(g.client("return #gband.layout().bands[1].columns"), 2)
  local _, corners = g.screen().row(0):gsub("┐", "")
  t.eq(corners, 2)
end)

t.case("paste and type", function(g)
  g.start({ size = "60x12" })
  g.type("echo typed")
  g.keys("enter")
  g.wait_text("typed")
  g.paste("echo pasted\r")
  g.wait_text("pasted")
end)

t.case("resize", function(g)
  g.start({ size = "60x12" })
  g.resize("50x10")
  g.settle()
  t.eq(g.client("return gband.layout().cols"), 50)
  t.eq(g.screen().cols, 50)
end)

t.case("wait gives up", function(g)
  g.start({ size = "20x4" })
  g.wait_text("never", { timeout = 1 })
end)
"##,
    );
    let output = project.run(&[]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let text = stdout(&output);
    for case in [
        "wait for program output",
        "open a window by key",
        "paste and type",
        "resize",
    ] {
        line_with(&text, &format!("PASS tests/driving_spec.lua > {case} ("));
    }
    let failed = line_with(&text, "FAIL tests/driving_spec.lua > wait gives up");
    let seconds: f64 = failed
        .rsplit_once('(')
        .unwrap()
        .1
        .trim_end_matches("s)")
        .parse()
        .unwrap();
    assert!((0.9..3.0).contains(&seconds), "{failed}");
    line_with(
        &text,
        "driving_spec.lua:40: g.wait_text gave up waiting for \"never\" after 1 second",
    );
    line_with(&text, "size 20x4 cursor");
}

#[test]
fn observing_a_case() {
    let project = Project::new("observing");
    project.file(
        "tests/observing_spec.lua",
        r##"local t = require("gband.test")

t.case("status line colour", function(g)
  g.start()
  t.eq(g.screen().cell(0, 0).fg, "#c0caf5")
  t.eq(g.screen().cell(0, 0).char, "b")
  t.eq(g.screen().cursor.visible, true)
end)

t.case("notification, clipboard, bell, opener and print", function(g)
  g.start({
    config = [[
      gband.opt.notify_style = "osc777"
      print("ready")
      gband.bind("prefix x", function()
        gband.notify("build done", { title = "ci" })
        gband.clipboard("copied")
        gband.bell()
        gband.open("https://example.com")
      end)
    ]],
  })
  g.keys("ctrl+space x")
  g.settle()
  t.eq(g.notifications(), { { title = "ci", body = "build done" } })
  t.eq(g.clipboard(), { "copied" })
  t.ok(g.bells() >= 1)
  g.wait(function()
    return #g.opened() > 0
  end)
  t.eq(g.opened(), { "https://example.com" })
  local printed = false
  for _, line in ipairs(g.log("client")) do
    printed = printed or line:find("ready") ~= nil
  end
  t.ok(printed, "the client log holds the print")
end)
"##,
    );
    let output = project.run(&[]);
    assert_passed(&output);
}

#[test]
fn chunk_from_standard_input() {
    let project = Project::new("stdin");
    let output = project.run_stdin(
        &["-"],
        "local t = require('gband.test')\n\
         t.case('reads stdin', function(g)\n\
           g.start({ size = '30x5' })\n\
           g.expect_screenshot()\n\
         end)\n",
    );
    assert_passed(&output);
    let text = stdout(&output);
    line_with(&text, "PASS stdin > reads stdin (");
    line_with(&text, "screenshot stdin > reads stdin");
    line_with(&text, "size 30x5 cursor");
    assert!(!project.dir.join("screenshots").exists());
}

#[test]
fn print_in_standard_input() {
    let project = Project::new("stdin-print");
    let output = project.run_stdin(&["-"], "print(gband.side)\n");
    assert_passed(&output);
    assert!(stdout(&output).starts_with("test\n"), "{}", report(&output));
}

#[test]
fn standard_input_takes_no_file() {
    let project = Project::new("stdin-file");
    project.file("tests/a_spec.lua", "");
    let output = project.run(&["-", "tests/a_spec.lua"]);
    assert_eq!(output.status.code(), Some(2), "{}", report(&output));
}

#[test]
fn screenshot_references() {
    let project = Project::new("references");
    project.file(
        "tests/window_spec.lua",
        "local t = require('gband.test')\n\
         t.case('Shows the focused window', function(g)\n\
           g.start({ size = '30x5', config = [[gband.plugin('gband.statusline') gband.plugin('gband.statusline.band')]] })\n\
           g.expect_screenshot('two windows')\n\
         end)\n",
    );
    let reference = project
        .dir
        .join("tests/screenshots/window_spec/shows-the-focused-window--two-windows.txt");
    let pending = reference.with_extension("txt.new");

    let first = project.run(&[]);
    assert_eq!(first.status.code(), Some(1), "{}", report(&first));
    line_with(&stdout(&first), "no screenshot reference");
    line_with(&stdout(&first), "size 30x5 cursor");
    assert!(pending.exists());
    assert!(!reference.exists());

    let accepted = project.run(&["--update"]);
    assert_passed(&accepted);
    assert!(reference.exists());
    assert!(!pending.exists());
    let shot = fs::read_to_string(&reference).unwrap();
    assert!(shot.starts_with("size 30x5 cursor"), "{shot}");
    assert!(shot.contains("\n0|band 1"), "{shot}");

    assert_passed(&project.run(&[]));
}

#[test]
fn failure_shows_the_diff_and_the_logs() {
    let project = Project::new("diff");
    project.file(
        "tests/diff_spec.lua",
        "local t = require('gband.test')\n\
         t.case('drawn', function(g)\n\
           g.start({ size = '30x5', config = [[\n\
             gband.plugin('gband.statusline')\n\
             gband.plugin('gband.statusline.band')\n\
             print('drawn')\n\
           ]] })\n\
           g.expect_screenshot()\n\
         end)\n",
    );
    assert_passed(&project.run(&["--update"]));
    let reference = project.dir.join("tests/screenshots/diff_spec/drawn.txt");
    let shot = fs::read_to_string(&reference).unwrap();
    fs::write(&reference, shot.replace("0|band 1", "0|band 9")).unwrap();
    let output = project.run(&[]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let text = stdout(&output);
    line_with(&text, "the screenshot differs from");
    line_with(&text, "-0|band 9");
    line_with(&text, "+0|band 1");
    assert!(!text.contains("-1|"), "{text}");
    line_with(&text, "client log:");
    line_with(&text, "drawn");
}

#[test]
fn show() {
    let project = Project::new("show");
    project.file(
        "tests/show_spec.lua",
        "local t = require('gband.test')\n\
         t.case('two shots', function(g)\n\
           g.start({ size = '30x5' })\n\
           g.screenshot()\n\
           g.screenshot({ styles = false })\n\
         end)\n",
    );
    let output = project.run(&["--show"]);
    assert_passed(&output);
    let text = stdout(&output);
    assert_eq!(
        text.matches("screenshot tests/show_spec.lua > two shots\n")
            .count(),
        2,
        "{text}"
    );
    assert_eq!(text.matches("size 30x5 cursor").count(), 2, "{text}");
}

#[test]
fn evaluating_chunks() {
    let project = Project::new("chunks");
    project.file(
        "tests/chunks_spec.lua",
        r##"local t = require("gband.test")

t.case("read client state", function(g)
  g.start()
  local side, count = g.client("return gband.side, select('#', ...)", 1, 2)
  t.eq(side, "client")
  t.eq(count, 2)
  t.eq(g.client("return ...", { a = { 1, true } }), { a = { 1, true } })
  t.eq(g.server("return gband.side"), "server")
end)

t.case("act in the server", function(g)
  g.start()
  g.server([[gband.window_state("default", 1).agent = "waiting"]])
  g.settle()
  t.eq(g.client("return gband.window_state(1).agent"), "waiting")
end)

t.case("errors are answered", function(g)
  g.start({ config = [[gband.plugin("gband.statusline.band")]] })
  local ok, err = pcall(g.client, "error('boom')")
  t.eq(ok, false)
  t.match(err, "boom")
  ok, err = pcall(g.client, "return function() end")
  t.match(err, "not plain data")
  ok, err = pcall(g.server, "return (")
  t.eq(ok, false)
  g.settle()
  t.eq(g.client("return 1 + 1"), 2)
  t.ok(not g.screen().text():find("boom"), "the client shows no error")
end)

t.case("frozen time", function(g)
  g.start()
  t.eq(g.client("return os.date('!%Y-%m-%d %H:%M:%S')"), "2025-01-01 12:00:00")
  t.eq(g.server("return os.time()"), 1735732800)
  t.eq(g.client("return os.date('%Y', 0)"), "1970")
end)

t.case("real time", function(g)
  g.start({ time = false })
  t.ok(g.client("return os.time()") > 1735732800)
end)

t.case("test channel kept out of windows", function(g)
  g.start()
  g.run('echo "[$GBAND_TEST_SOCKET]"')
  g.wait_text("[]")
end)
"##,
    );
    let output = project.run(&[]);
    assert_passed(&output);
}

#[test]
fn reloading() {
    let project = Project::new("reloading");
    project.file(
        "fixtures/drawer/plugin.lua",
        "return { name = 'drawer', version = '0.1.0' }",
    );
    project.file(
        "fixtures/drawer/client.lua",
        "gband.ui.statusline.add({ id = 'drawer', render = function() return 'old' end })",
    );
    project.file(
        "tests/reloading_spec.lua",
        r##"local t = require("gband.test")

t.case("reload after editing a plugin", function(g)
  g.start({
    files = { ["user/lua/segment.lua"] = "return { setup = function() gband.ui.statusline.add({ id = 'seg', render = function() return 'first' end }) end }" },
    config = [[gband.plugin("gband.statusline")
gband.plugin("segment")]],
  })
  t.match(g.screen().text(), "first")
  g.write("user/lua/segment.lua", "return { setup = function() gband.ui.statusline.add({ id = 'seg', render = function() return 'second' end }) end }")
  t.eq(g.reload(), nil)
  g.settle()
  t.match(g.screen().text(), "second")
end)

t.case("plugin file changed", function(g)
  g.start({ plugins = { "../fixtures/drawer" } })
  t.match(g.screen().text(), "old")
  local file = assert(io.open("fixtures/drawer/client.lua", "w"))
  file:write("gband.ui.statusline.add({ id = 'drawer', render = function() return 'new' end })")
  file:close()
  t.eq(g.reload(), nil)
  g.settle()
  t.match(g.screen().text(), "new")
end)

t.case("broken file", function(g)
  g.start({ config = [[gband.plugin("gband.statusline")
gband.plugin("gband.statusline.band")]] })
  g.write("user/init.lua", "error('bad')")
  t.match(g.reload(), "bad")
  g.settle()
  t.match(g.screen().text(), "band 1")
end)

t.case("clock segment frozen and moved", function(g)
  g.start({ config = [[gband.plugin("gband.statusline")
gband.plugin("gband.statusline.clock")]] })
  t.match(g.screen().text(), "12:00")
  g.set_time("2025-01-01 12:05:00")
  g.wait_text("12:05", { timeout = 3 })
end)
"##,
    );
    let output = project.run(&[]);
    assert_passed(&output);
}

#[test]
fn settling() {
    let project = Project::new("settling");
    project.file(
        "tests/settling_spec.lua",
        r##"local t = require("gband.test")

t.case("key effect is drawn", function(g)
  g.start({ size = "60x12" })
  g.keys("ctrl+space n")
  g.settle()
  local _, corners = g.screen().row(0):gsub("┐", "")
  t.eq(corners, 2)
end)

t.case("server handler effect is drawn", function(g)
  g.start({
    server_config = [[
      gband.on("WindowOpened", function(ev)
        gband.window_state(ev.session, ev.window).agent = "busy-" .. ev.window
      end)
    ]],
    config = [[
      gband.plugin("gband.statusline")
      gband.keymap.set("prefix", "enter", gband.action.open_window)
      gband.ui.statusline.add({
        id = "agent",
        redraw_on = { "WindowStateChanged" },
        render = function(ctx)
          local found = {}
          for _, entry in ipairs(ctx.windows) do
            found[#found + 1] = entry.state.agent
          end
          return table.concat(found, " ")
        end,
      })
    ]],
  })
  g.keys("ctrl+space enter")
  g.settle()
  t.match(g.screen().text(), "busy%-2")
end)

t.case("handlers that keep triggering each other", function(g)
  g.start({
    server_config = [[
      gband.cmd.register("bounce", function()
        gband.emit("ping", {})
      end)
    ]],
    config = [[
      gband.on("ServerEvent", function()
        gband.rpc("bounce")
      end, { pattern = "ping" })
    ]],
  })
  g.client([[gband.rpc("bounce")]])
  g.settle()
end)
"##,
    );
    let output = project.run(&[]);
    assert_eq!(output.status.code(), Some(1), "{}", report(&output));
    let text = stdout(&output);
    line_with(
        &text,
        "PASS tests/settling_spec.lua > key effect is drawn (",
    );
    line_with(
        &text,
        "PASS tests/settling_spec.lua > server handler effect is drawn (",
    );
    line_with(
        &text,
        "FAIL tests/settling_spec.lua > handlers that keep triggering each other (",
    );
    line_with(&text, "after 10 rounds");
}

fn attach_with_socket(env: &TestEnv, socket: &str) -> Attached {
    let socket = socket.to_owned();
    Attached::start_with(env, GBAND, &["attach"], 80, 24, move |command| {
        command.env("GBAND_TEST_SOCKET", &socket);
    })
}

#[test]
fn both_sides_join() {
    let env = TestEnv::new("pt-join");
    let socket = env.root.join("channel.sock");
    let listener = Listener::bind(&socket).unwrap();
    let _attached = attach_with_socket(&env, socket.to_str().unwrap());
    let deadline = Instant::now() + TIMEOUT;
    let (client, server) = listener.accept(None, deadline, || Ok(())).unwrap();
    assert_eq!(client.role().name(), "client");
    assert_eq!(server.role().name(), "server");
}

#[test]
fn missing_socket() {
    let env = TestEnv::new("pt-missing");
    let mut attached = attach_with_socket(&env, "/nonexistent/sock");
    assert_eq!(attached.wait_exit(), 1);
    let screen = attached.contents();
    assert!(screen.contains("/nonexistent/sock"), "{screen}");
    assert_eq!(
        screen
            .lines()
            .filter(|line| line.contains("gband:"))
            .count(),
        1,
        "{screen}"
    );
    assert!(!env.socket().exists());
}

#[test]
fn unset() {
    let env = TestEnv::new("pt-unset");
    let attached = attach_with_socket(&env, "");
    attached.wait_for_prompt();
}

#[test]
fn server_sends_no_chunk() {
    let env = TestEnv::new("pt-server-only");
    let socket = env.root.join("channel.sock");
    let listener = Listener::bind(&socket).unwrap();
    let mut server = env
        .command(GBAND, &["server"])
        .env("GBAND_TEST_SOCKET", &socket)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + TIMEOUT;
    let mut endpoint = listener.accept_one(None, deadline).unwrap();
    assert_eq!(endpoint.role().name(), "server");
    wait_until(|| env.socket().exists(), "the server socket");
    let attached = Attached::start(&env, 80, 24);
    attached.wait_for_prompt();
    let answer = endpoint
        .eval("return gband.side", Vec::new(), Instant::now() + TIMEOUT)
        .unwrap();
    assert_eq!(answer, Ok(vec![gband_protocol::Value::string("server")]));
    attached.wait_for_prompt();
    drop(endpoint);
    let status = wait_process_exit(&mut server);
    assert!(status.success(), "{status:?}");
}

fn read_until(reader: &mut BufReader<std::process::ChildStdout>, needle: &str) {
    let deadline = Instant::now() + TIMEOUT;
    let mut line = String::new();
    while Instant::now() < deadline {
        line.clear();
        if reader.read_line(&mut line).unwrap() == 0 {
            break;
        }
        if line.contains(needle) {
            return;
        }
    }
    panic!("the runner never printed {needle:?}");
}

#[test]
fn runner_killed() {
    let project = Project::new("killed");
    project.file(
        "tests/killed_spec.lua",
        "local t = require('gband.test')\n\
         t.case('waits', { timeout = 120 }, function(g)\n\
           g.start()\n\
           print('started')\n\
           g.wait(function() return false end, { timeout = 100 })\n\
         end)\n",
    );
    let mut child: Child = project
        .command(&[])
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let prefix = format!("gband-test-{}-", child.id());
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    read_until(&mut stdout, "started");
    let processes = processes_of(&prefix);
    assert!(processes.len() >= 3, "{processes:?}");
    rustix::process::kill_process(Pid::from_raw(child.id() as i32).unwrap(), Signal::KILL).unwrap();
    child.wait().unwrap();
    let deadline = Instant::now() + TIMEOUT;
    while processes.iter().any(|&pid| is_running(pid)) {
        assert!(Instant::now() < deadline, "{:?}", processes_of(&prefix));
        thread::sleep(Duration::from_millis(50));
    }
    for entry in fs::read_dir(std::env::temp_dir()).unwrap().flatten() {
        if entry.file_name().to_string_lossy().starts_with(&prefix) {
            let _ = fs::remove_dir_all(entry.path());
        }
    }
}
