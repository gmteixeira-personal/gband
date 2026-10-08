mod common;

use common::*;

fn second_window(screen: &Grid) -> bool {
    let lines = focused_lines(screen);
    !lines.iter().any(|line| line == "W1")
        && lines
            .iter()
            .rfind(|line| !line.is_empty())
            .is_some_and(|line| line.ends_with('$'))
}

fn labelled(client: &mut Attached, label: &str) {
    client.run(&format!("clear; echo {label}"));
    client.wait_for_line(label);
}

fn sgr(client: &mut Attached, code: u16, (col, row): (u16, u16), end: char) {
    client.send(format!("\x1b[<{code};{};{}{end}", col + 1, row + 1).as_bytes());
}

fn focused_tile(client: &Attached) -> Tile {
    client
        .tiles()
        .into_iter()
        .find(|tile| tile.focused)
        .unwrap_or_else(|| panic!("no focused tile:\n{}", client.contents()))
}

fn direct_with_two_windows(name: &str) -> (TestEnv, Attached) {
    let env = TestEnv::new(name);
    env.save_key_style("direct");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    labelled(&mut client, "W1");
    client.send(b"\x00n");
    client.wait_for("the second window", second_window);
    labelled(&mut client, "W2");
    (env, client)
}

#[test]
fn drag_after_the_prefix_with_the_direct_key_style() {
    let (_env, mut client) = direct_with_two_windows("keystyle-direct-drag");
    client.send(b"\x00v");
    client.wait_for("the floating box", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && tile.top > 0)
    });
    let window = focused_tile(&client);
    let (left, top) = (window.left, window.top);
    client.send(b"\x00");
    sgr(&mut client, 0, (left + 10, top + 3), 'M');
    sgr(&mut client, 32, (left + 16, top + 3), 'M');
    sgr(&mut client, 0, (left + 16, top + 3), 'm');
    client.wait_for("the box moved 6 cells right", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| tile.focused && (tile.left, tile.top) == (left + 6, top))
    });
    client.run("echo after");
    client.wait_for("keys reach the window", |screen| {
        focused_lines(screen).iter().any(|line| line == "after")
    });
}

#[test]
fn other_press_after_the_prefix_with_the_direct_key_style() {
    let (_env, mut client) = direct_with_two_windows("keystyle-direct-press");
    let second = focused_tile(&client);
    client.send(b"\x00");
    let cell = (second.left - 20, second.top + 3);
    sgr(&mut client, 16, cell, 'M');
    sgr(&mut client, 16, cell, 'm');
    client.send(b"x");
    client.wait_for("x in the focused window", |screen| {
        focused_lines(screen)
            .iter()
            .any(|line| line.ends_with("$ x"))
    });
    assert_eq!(focused_tile(&client).left, second.left);
}

fn rename_line(screen: &Grid) -> Option<String> {
    let contents = screen.contents();
    let lines: Vec<&str> = contents.lines().collect();
    let top = lines.iter().position(|line| line.contains("┌rename"))?;
    let row = lines.get(top + 1)?;
    let start = row.find('│')? + '│'.len_utf8();
    let end = row.rfind('│')?;
    Some(row.get(start..end)?.trim_end().to_owned())
}

fn renaming(name: &str, style: &str) {
    let env = TestEnv::new(name);
    env.save_key_style(style);
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x00N");
    client.wait_for("the rename prompt", |screen| {
        rename_line(screen).as_deref() == Some("")
    });
    client.send(b"j");
    client.wait_for("j typed into the rename prompt", |screen| {
        rename_line(screen).as_deref() == Some("j")
    });
}

#[test]
fn rename_the_focused_window() {
    renaming("keystyle-rename-modal", "modal");
}

#[test]
fn rename_with_the_direct_key_style() {
    renaming("keystyle-rename-direct", "direct");
}

const STYLED: &str = "gband.keystyle.use(os.getenv('GBAND_TEST_STYLE'))
gband.bind('f5', function() gband.spawn({ cmd = 'sh' }) end)
gband.bind('f6', function() gband.win.open({ kind = 'tiled', lines = { 'hi' } }) end)
gband.bind('f7', function()
  gband.window.set_width(1, 0.2)
  gband.window.set_height(1, { rows = 6 })
  gband.window.set_position(1, { col = 0, row = 16 })
end)
";
const F7: &[u8] = b"\x1b[18~";

fn aside(client: &mut Attached) {
    client.send(F7);
    client.wait_for("the first window aside", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| (tile.left, tile.top) == (0, 16))
    });
}
const BUTTONS: &str = "[_][□][X]";

fn styled(env: &TestEnv, style: &str, cols: u16, rows: u16) -> Attached {
    Attached::start_with(env, &env.executable, &["attach"], cols, rows, |command| {
        command.env("GBAND_TEST_STYLE", style);
    })
}

fn floating_boxes(screen: &Grid) -> usize {
    tiles(screen).iter().filter(|tile| tile.top > 0).count()
}

fn buttons(screen: &Grid) -> usize {
    screen.contents().matches(BUTTONS).count()
}

fn first_window_floats(client: &Attached) {
    client.wait_for("the first window floating", |screen| {
        buttons(screen) == 1 && floating_boxes(screen) == 1
    });
}

#[test]
fn window_of_a_client_with_another_style() {
    let env = TestEnv::new("keystyle-floating-other-style");
    env.write_config(STYLED);
    let mut floating = styled(&env, "floating", 80, 24);
    first_window_floats(&floating);
    aside(&mut floating);
    let mut modal = styled(&env, "modal", 80, 24);
    modal.wait_for("the floating window", |screen| floating_boxes(screen) == 1);
    modal.send(b"\x00n");
    modal.wait_for("both windows floating", |screen| {
        floating_boxes(screen) == 2 && tiles(screen).iter().all(|tile| tile.top > 0)
    });
    floating.wait_for("both windows with buttons", |screen| buttons(screen) == 2);
}

#[test]
fn another_clients_tiled_plugin_window() {
    let env = TestEnv::new("keystyle-floating-other-plugin-window");
    env.write_config(STYLED);
    let mut floating = styled(&env, "floating", 80, 24);
    first_window_floats(&floating);
    aside(&mut floating);
    let mut modal = styled(&env, "modal", 80, 24);
    modal.wait_for("the floating window", |screen| floating_boxes(screen) == 1);
    modal.send(b"\x1b[17~");
    modal.wait_for("the drawn window floating", |screen| {
        screen.contents().contains("│hi")
            && floating_boxes(screen) == 2
            && tiles(screen).iter().all(|tile| tile.top > 0)
    });
    floating.wait_for("the drawn window floating without buttons", |screen| {
        screen.contents().contains("│hi") && floating_boxes(screen) == 2 && buttons(screen) == 1
    });
}

#[test]
fn two_floating_clients_float_a_window_once() {
    let env = TestEnv::new("keystyle-floating-two-clients");
    env.write_config(STYLED);
    let mut first = styled(&env, "floating", 80, 24);
    first_window_floats(&first);
    let second = styled(&env, "floating", 80, 24);
    first_window_floats(&second);
    aside(&mut first);
    first.send(b"\x1b[15~");
    for client in [&first, &second] {
        client.wait_for("the spawned window floating", |screen| {
            buttons(screen) == 2 && floating_boxes(screen) == 2
        });
    }
    let log = env.log_text("server");
    assert!(!log.contains("ERROR"), "{log}");
}

#[test]
fn another_client_grows_the_screen_area() {
    let env = TestEnv::new("keystyle-floating-grow");
    env.write_config(STYLED);
    let mut first = styled(&env, "floating", 80, 24);
    first_window_floats(&first);
    sgr(&mut first, 2, (30, 2), 'M');
    sgr(&mut first, 2, (30, 2), 'm');
    first.wait_for_text("Tile right");
    first.send(b"jjjj\r");
    first.wait_for("the window tiled right", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| (tile.left, tile.top, tile.right, tile.bottom) == (40, 0, 79, 23))
    });
    let second = styled(&env, "floating", 100, 30);
    second.wait_for("the window refitted to the right half", |screen| {
        tiles(screen)
            .iter()
            .any(|tile| (tile.left, tile.top, tile.right, tile.bottom) == (50, 0, 99, 29))
    });
}

fn floating_saved(name: &str) -> (TestEnv, Attached) {
    let env = TestEnv::new(name);
    env.save_key_style("floating");
    let client = Attached::start(&env, 80, 24);
    client.wait_for("the first window floating", |screen| buttons(screen) == 1);
    client.wait_for_text("$");
    (env, client)
}

#[test]
fn leader_twice_sends_the_prefix_key() {
    let (_env, mut client) = floating_saved("keystyle-floating-leader-twice");
    client.run("cat -v");
    client.wait_for_text("$ cat -v");
    client.send(b"\x00");
    client.wait_for_text("┌windows");
    client.send(b"\x00");
    client.wait_for("the window list closed", |screen| {
        !screen.contents().contains("┌windows")
    });
    client.send(b"\r");
    client.wait_for("cat -v printing ^@", |screen| {
        screen
            .contents()
            .lines()
            .filter(|line| line.contains("│^@ "))
            .count()
            >= 2
    });
}

#[test]
fn detach_from_the_list() {
    let (_env, mut client) = floating_saved("keystyle-floating-detach");
    client.send(b"\x00");
    client.wait_for_text("┌windows");
    client.send(b"D");
    client.wait_exit();
}
