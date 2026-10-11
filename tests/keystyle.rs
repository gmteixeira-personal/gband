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

#[test]
fn leader_after_a_preview() {
    let (_env, mut client) = floating_saved("keystyle-floating-leader-preview");
    client.run("cat -v");
    client.wait_for_text("$ cat -v");
    client.send(b"\x00");
    client.wait_for_text("┌windows");
    client.send(b"\r");
    client.wait_for("the second window over the first", |screen| {
        buttons(screen) == 2 && !screen.contents().contains("$ cat -v")
    });
    client.send(b"\x00");
    client.wait_for_text("┌windows");
    client.send(b"jjjj");
    client.wait_for_text("$ cat -v");
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

const NAMED: &str = "gband.keystyle.use('floating')
gband.bind('f5', function() gband.action.open_window({ floating = true }) end)
gband.bind('f6', function()
  local floating = gband.layout().bands[1].floating
  for index, name in ipairs({ 'notes', 'logs', 'mail' }) do
    gband.window.rename(floating[index].id, name)
  end
  gband.window.focus(floating[3].id)
end)
";

fn list_entries(screen: &Grid) -> Vec<String> {
    screen
        .contents()
        .lines()
        .filter_map(|line| {
            line.split('│').find_map(|cell| {
                let entry = cell.strip_prefix(' ')?.trim_end();
                let mut chars = entry.chars();
                let shortcut = chars.next()?;
                (shortcut.is_ascii_digit() && chars.next() == Some(' ')).then(|| entry.to_owned())
            })
        })
        .collect()
}

#[test]
fn windows_never_focused_come_last() {
    let env = TestEnv::new("keystyle-floating-never-focused");
    env.write_config(NAMED);
    let mut first = Attached::start(&env, 80, 24);
    first.wait_for("the first window floating", |screen| buttons(screen) == 1);
    first.send(b"\x1b[15~");
    first.wait_for("two windows", |screen| buttons(screen) == 2);
    first.send(b"\x1b[15~");
    first.wait_for("three windows", |screen| buttons(screen) == 3);
    first.send(b"\x1b[17~\x00");
    first.wait_for_text("┌windows");
    first.send(b"jjjj");
    first.wait_for("the first client's window list", |screen| {
        list_entries(screen) == ["1 mail", "2 logs", "3 notes"]
    });
    first.send(b"\x1b");
    first.wait_for("the window list closed", |screen| {
        !screen.contents().contains("┌windows")
    });
    let mut second = Attached::start(&env, 80, 24);
    second.wait_for("three windows", |screen| buttons(screen) == 3);
    second.send(b"\x00");
    second.wait_for("the window list", |screen| {
        list_entries(screen) == ["1 mail", "2 notes", "3 logs"]
    });
}

fn label_plugin(env: &TestEnv, label: &str) {
    env.write_client_plugin(
        "label",
        &format!("gband.bar.add({{ side = 'right', size = 6, lines = {{ '{label}' }} }})\n"),
    );
}

fn client_load(env: &TestEnv) -> u64 {
    let output = env
        .command(GBAND, &["errors", "--json"])
        .env_remove("GBAND_SESSION")
        .output()
        .unwrap();
    let entries: Vec<serde_json::Value> = serde_json::from_slice(&output.stdout).unwrap();
    entries[1]["load"].as_u64().unwrap()
}

fn reload_redraws(name: &str, style: &str) -> (TestEnv, Attached) {
    let env = TestEnv::new(name);
    env.save_key_style(style);
    label_plugin(&env, "old");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_text("old");
    client.wait_for_text("$");
    label_plugin(&env, "new");
    client.send(b"\x00!");
    client.wait_for_text("new");
    (env, client)
}

#[test]
fn reload_the_configuration() {
    let (env, mut client) = reload_redraws("keystyle-reload", "modal");
    assert_eq!(client_load(&env), 2);
    client.run("echo back-to-typing");
    client.wait_for_line("back-to-typing");
}

#[test]
fn reload_with_the_direct_key_style() {
    let (env, _client) = reload_redraws("keystyle-reload-direct", "direct");
    assert_eq!(client_load(&env), 2);
}

#[test]
fn reload_from_the_window_list() {
    let env = TestEnv::new("keystyle-reload-list");
    env.save_key_style("floating");
    label_plugin(&env, "old");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_text("old");
    client.wait_for_text("$");
    label_plugin(&env, "new");
    client.send(b"\x00");
    client.wait_for_text("┌windows");
    client.send(b"!");
    client.wait_for("the window list closed and the new label drawn", |screen| {
        let contents = screen.contents();
        !contents.contains("┌windows") && contents.contains("new")
    });
    assert_eq!(client_load(&env), 2);
}

#[test]
fn reset_height_keeps_its_key() {
    let env = TestEnv::new("keystyle-reset-height");
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    client.send(b"\x00n\r");
    client.wait_for("two tiles", |screen| tiles(screen).len() == 2);
    client.wait_for_prompt();
    client.send(b"\x00[");
    client.wait_for("one column of two windows", |screen| {
        let tiles = tiles(screen);
        tiles.len() == 2 && tiles[0].left == tiles[1].left
    });
    let height = |client: &Attached| {
        let tile = focused_tile(client);
        tile.bottom - tile.top
    };
    let automatic = height(&client);
    client.send(b"_");
    client.wait_for("a fixed, smaller height", |screen| {
        tiles(screen)
            .into_iter()
            .find(|tile| tile.focused)
            .is_some_and(|tile| tile.bottom - tile.top < automatic)
    });
    client.send(b"R");
    client.wait_for("the automatic height again", |screen| {
        tiles(screen)
            .into_iter()
            .find(|tile| tile.focused)
            .is_some_and(|tile| tile.bottom - tile.top == automatic)
    });
    assert_eq!(client_load(&env), 1);
}

fn modal_with_two_windows(name: &str) -> (TestEnv, Attached) {
    let env = TestEnv::new(name);
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    client.shell_pid(&env);
    labelled(&mut client, "W1");
    client.send(b"\x00n\r");
    client.wait_for("the second window", second_window);
    labelled(&mut client, "W2");
    (env, client)
}

fn printed_by(client: &Attached, line: &str, count: usize) {
    client.wait_for(&format!("{count} windows printing {line}"), |screen| {
        tiles_with_line(screen, line) == count
    });
}

#[test]
fn multi_mode_from_navigation_mode() {
    let (_env, mut client) = modal_with_two_windows("keystyle-multi");
    client.send(b"\x00m");
    client.wait_for("the multi mode letter", |screen| {
        sidebar_mode(screen) == "M"
    });
    client.run("echo hi");
    printed_by(&client, "hi", 2);
    client.send(b"\x00m");
    client.wait_for("the interactive mode letter", |screen| {
        sidebar_mode(screen) == "I"
    });
    client.run("echo solo");
    client.wait_for_line("solo");
    assert_eq!(tiles_with_line(&client.screen(), "solo"), 1);
}

#[test]
fn multi_mode_with_the_direct_key_style() {
    let (_env, mut client) = direct_with_two_windows("keystyle-multi-direct");
    client.send(b"\x00m");
    client.wait_for("the multi mode letter", |screen| {
        sidebar_mode(screen) == "M"
    });
    client.run("echo hi");
    printed_by(&client, "hi", 2);
}

#[test]
fn navigation_mode_keeps_multi_mode() {
    let (_env, mut client) = modal_with_two_windows("keystyle-multi-navigation");
    client.send(b"\x00h\r");
    client.wait_for("the first window focused", |screen| {
        focused_lines(screen).iter().any(|line| line == "W1")
    });
    client.send(b"\x00m");
    client.wait_for("the multi mode letter", |screen| {
        sidebar_mode(screen) == "M"
    });
    client.send(b"\x00l");
    client.wait_for("the second window focused in navigation mode", |screen| {
        sidebar_mode(screen) == "N" && focused_lines(screen).iter().any(|line| line == "W2")
    });
    client.send(b"\x1b");
    client.wait_for("the multi mode letter again", |screen| {
        sidebar_mode(screen) == "M"
    });
    client.run("echo hi");
    printed_by(&client, "hi", 2);
}

fn tile_heights(screen: &Grid) -> Vec<u16> {
    tiles(screen)
        .into_iter()
        .map(|tile| tile.bottom - tile.top + 1)
        .collect()
}

fn one_tab_shown(screen: &Grid, line: &str) -> bool {
    tile_heights(screen) == [24] && focused_lines(screen).iter().any(|shown| shown == line)
}

#[test]
fn tabs_from_navigation_mode() {
    let (_env, mut client) = modal_with_two_windows("keystyle-tabs");
    client.send(b"\x00[");
    client.wait_for("one column of two windows", |screen| {
        tile_heights(screen) == [12, 12]
    });
    client.send(b"k");
    client.wait_for("the top window focused", |screen| {
        focused_lines(screen).iter().any(|line| line == "W1")
    });
    client.send(b"w");
    client.wait_for("the top window as the only tab drawn", |screen| {
        one_tab_shown(screen, "W1") && sidebar_mode(screen) == "N"
    });
    client.send(b"w");
    client.wait_for("the stack again", |screen| {
        tile_heights(screen) == [12, 12] && sidebar_mode(screen) == "N"
    });
}

#[test]
fn tabs_with_the_direct_key_style() {
    let (_env, mut client) = direct_with_two_windows("keystyle-tabs-direct");
    client.send(b"\x00[");
    client.wait_for("one column of two windows", |screen| {
        tile_heights(screen) == [12, 12]
    });
    client.send(b"\x00w");
    client.wait_for("the focused window as the only tab drawn", |screen| {
        one_tab_shown(screen, "W2")
    });
    client.run("echo typed");
    client.wait_for_line("typed");
}

const STAND_IN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/crates/server/tests/fixtures/claude/stand-in.sh"
);
const UP: &[u8] = b"\x1b[A";
const DOWN: &[u8] = b"\x1b[B";

fn focused_label(client: &Attached, label: &str) {
    client.wait_for_focused(&format!("{label} focused"), |lines| {
        lines.iter().any(|line| line == label)
    });
}

fn mode_letter(client: &Attached, letter: &str) {
    client.wait_for(&format!("the mode letter {letter}"), |screen| {
        sidebar_mode(screen) == letter
    });
}

fn shell_line_untouched(client: &Attached, label: &str) {
    let screen = client.screen();
    let tile = tiles(&screen)
        .into_iter()
        .find(|tile| tile.lines(&screen).iter().any(|line| line == label))
        .unwrap_or_else(|| panic!("no tile shows {label}:\n{}", screen.contents()));
    let lines = tile.lines(&screen);
    let last = lines.iter().rfind(|line| !line.is_empty()).unwrap();
    assert!(last.ends_with('$'), "{last:?}\n{}", screen.contents());
}

fn with_agents(name: &str, style: &str) -> (TestEnv, Attached) {
    let env = TestEnv::new(name);
    env.save_key_style(style);
    let mut client = Attached::start(&env, 120, 30);
    client.wait_for_prompt();
    labelled(&mut client, "W1");
    let open: &[u8] = if style == "modal" {
        b"\x00n\r"
    } else {
        b"\x00n"
    };
    for window in 2..=4 {
        client.send(open);
        client.wait_for_focused("a new shell", |lines| {
            !lines
                .iter()
                .any(|line| line == "W1" || line.starts_with("agent "))
                && lines
                    .iter()
                    .rfind(|line| !line.is_empty())
                    .is_some_and(|line| line.ends_with('$'))
        });
        client.run(&format!("bash -c 'exec -a claude bash {STAND_IN}'"));
        focused_label(&client, &format!("agent {window}"));
    }
    client.wait_for("three agent labels", |screen| {
        sidebar_cell(screen, 18) == "3"
    });
    let back: &[u8] = if style == "modal" {
        b"\x00hhh\r"
    } else {
        b"\x00h\x00h\x00h"
    };
    client.send(back);
    focused_label(&client, "W1");
    (env, client)
}

#[test]
fn agents_mode_letter() {
    let (_env, mut client) = with_agents("agents-letter", "modal");
    client.send(b"\x00a");
    mode_letter(&client, "A");
    client.send(b"\x1b");
    mode_letter(&client, "I");
}

#[test]
fn unbound_key_discarded_in_agents_mode() {
    let (_env, mut client) = with_agents("agents-unbound", "modal");
    client.send(b"\x00ax");
    mode_letter(&client, "A");
    client.send(b"\x1b");
    mode_letter(&client, "I");
    focused_label(&client, "W1");
    shell_line_untouched(&client, "W1");
}

#[test]
fn jump_to_an_agent() {
    let (_env, mut client) = with_agents("agents-jump", "modal");
    client.send(b"\x00a3");
    focused_label(&client, "agent 4");
    mode_letter(&client, "I");
    shell_line_untouched(&client, "W1");
}

#[test]
fn jump_with_the_direct_style() {
    let (_env, mut client) = with_agents("agents-jump-direct", "direct");
    client.send(b"\x00a1");
    focused_label(&client, "agent 2");
    mode_letter(&client, "I");
}

#[test]
fn position_past_the_list() {
    let (_env, mut client) = with_agents("agents-past", "modal");
    client.send(b"\x00a5");
    mode_letter(&client, "A");
    client.send(b"\x1b");
    mode_letter(&client, "I");
    focused_label(&client, "W1");
}

#[test]
fn step_down_twice() {
    let (_env, mut client) = with_agents("agents-down", "modal");
    client.send(b"\x00a1");
    focused_label(&client, "agent 2");
    client.send(b"\x00a");
    client.send(DOWN);
    client.send(DOWN);
    focused_label(&client, "agent 4");
    mode_letter(&client, "A");
    client.send(b"\r");
    mode_letter(&client, "I");
    focused_label(&client, "agent 4");
}

#[test]
fn step_agents_with_j_and_k() {
    let (_env, mut client) = with_agents("agents-jk", "modal");
    client.send(b"\x00a1");
    focused_label(&client, "agent 2");
    client.send(b"\x00aj");
    focused_label(&client, "agent 3");
    client.send(b"j");
    focused_label(&client, "agent 4");
    client.send(b"k");
    focused_label(&client, "agent 3");
    mode_letter(&client, "A");
}

#[test]
fn up_with_no_agent_focused() {
    let (_env, mut client) = with_agents("agents-up", "modal");
    client.send(b"\x00a");
    client.send(UP);
    focused_label(&client, "agent 4");
}

#[test]
fn escape_keeps_the_new_focus() {
    let (_env, mut client) = with_agents("agents-escape", "modal");
    client.send(b"\x00a1");
    focused_label(&client, "agent 2");
    client.send(b"\x00a");
    client.send(DOWN);
    focused_label(&client, "agent 3");
    client.send(b"\x1b");
    mode_letter(&client, "I");
    focused_label(&client, "agent 3");
}

fn with_bands(name: &str) -> (TestEnv, Attached) {
    let env = TestEnv::new(name);
    let mut client = Attached::start(&env, 80, 24);
    client.wait_for_prompt();
    labelled(&mut client, "B1");
    client.send(b"\x00un\r");
    client.wait_for_focused("a shell in band 2", |lines| {
        !lines.iter().any(|line| line == "B1")
            && lines
                .iter()
                .rfind(|line| !line.is_empty())
                .is_some_and(|line| line.ends_with('$'))
    });
    labelled(&mut client, "B2");
    client.wait_for("band 3", |screen| sidebar_cell(screen, 4) == "3");
    client.send(b"\x00i\r");
    focused_label(&client, "B1");
    (env, client)
}

fn empty_band(client: &Attached) {
    client.wait_for("an empty band", |screen| tiles(screen).is_empty());
}

#[test]
fn bands_mode_letter() {
    let (_env, mut client) = with_bands("bands-letter");
    client.send(b"\x00b");
    mode_letter(&client, "B");
    client.send(b"\x1b");
    mode_letter(&client, "I");
}

#[test]
fn jump_to_a_band() {
    let (_env, mut client) = with_bands("bands-jump");
    client.send(b"\x00b3");
    empty_band(&client);
    mode_letter(&client, "B");
    client.send(b"2");
    focused_label(&client, "B2");
    mode_letter(&client, "B");
    client.send(b"\x1b");
    mode_letter(&client, "I");
    focused_label(&client, "B2");
}

#[test]
fn band_past_the_layout() {
    let (_env, mut client) = with_bands("bands-past");
    client.send(b"\x00b7");
    mode_letter(&client, "B");
    focused_label(&client, "B1");
}

#[test]
fn step_through_bands() {
    let (_env, mut client) = with_bands("bands-step");
    client.send(b"\x00b");
    client.send(DOWN);
    client.send(DOWN);
    empty_band(&client);
    mode_letter(&client, "B");
    client.send(UP);
    focused_label(&client, "B2");
    mode_letter(&client, "B");
}

#[test]
fn switch_between_the_modes() {
    let (_env, mut client) = with_bands("modes-switch");
    client.send(b"\x00a");
    mode_letter(&client, "A");
    client.send(b"b");
    mode_letter(&client, "B");
    client.send(b"j");
    focused_label(&client, "B2");
    client.send(b"a");
    mode_letter(&client, "A");
    client.send(b"\x1b");
    mode_letter(&client, "I");
    focused_label(&client, "B2");
}

#[test]
fn step_bands_with_j_and_k() {
    let (_env, mut client) = with_bands("bands-jk");
    client.send(b"\x00bj");
    focused_label(&client, "B2");
    client.send(b"j");
    empty_band(&client);
    client.send(b"k");
    focused_label(&client, "B2");
    mode_letter(&client, "B");
}
