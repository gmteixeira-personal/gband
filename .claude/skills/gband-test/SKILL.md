---
name: gband-test
description: See what gband draws and keep it as a regression test. Use when changing a plugin, the sidebar or a bar, highlight groups, colorschemes, key bindings or anything else that changes gband's screen, and when writing or fixing Lua tests under tests/ or examples/plugins/*/tests/.
---

# Seeing and testing what gband draws

You cannot attach to a terminal, but `gband test` can: it starts a real `gband attach` in a terminal of its own, drives it, and prints the screen as text with every colour and attribute.
Use it instead of asking the user to look, and turn what you checked into a test.

[docs/testing.md](../../../docs/testing.md) is the full API. Read it before writing more than a quick chunk.

## 1. Look

Build first, so the runner is the code you just changed:

```sh
cargo build
```

Write a chunk to standard input with `--show`.
Run it in the plugin's directory, so the plugin under test is installed; anywhere else, give `--plugin PATH`.

```sh
cd examples/plugins/window
../../../target/debug/gband test - --show <<'EOF'
local t = require("gband.test")
t.case("look", function(g)
  g.start({ size = "60x4", config = [[gband.plugin("gband.sidebar") gband.plugin("window")]] })
  g.expect_screenshot()
end)
EOF
```

The output holds the screenshot: a header with the size and cursor, one line per row, then after `--` one line per run of styled cells, such as `0:48-55 fg=#7aa2f7 bold`.
Read the rows for text and the style lines for colours.
From standard input, `g.expect_screenshot` prints and writes nothing, so experiments leave no files.

Things that trip a first chunk:

- `config` replaces the default configuration. Set up the sidebar, the plugins and the bindings the case needs, or leave `config` out for the defaults.
- After keys or a chunk, call `g.settle()` before reading the screen. It waits for gband itself, not for programs in windows: wait for those with `g.wait_text`.
- Use `g.client(chunk)` and `g.server(chunk)` to read state, such as `return #gband.layout().bands[1].columns`, or to act, such as setting window state in the server.
- The time is frozen at `2025-01-01 12:00:00` UTC.

## 2. Change and look again

Edit the code, `cargo build`, and rerun the same chunk until the screenshot shows what you meant.

## 3. Keep it as a test

Move the case into a `_spec.lua` file under the plugin's `tests/`, or under `tests/lua/` for gband's bundled modules, and replace looking with checking:

- `t.eq`, `t.ok` and `t.match` for what matters, such as `t.match(g.screen().row(0), "window 2$")` or `t.eq(g.screen().cell(0, 55).fg, "#f6c177")`
- `g.expect_screenshot("name")` for the whole screen

Write the references and read them:

```sh
cd examples/plugins/window
../../../target/debug/gband test --update
git diff --stat
cat tests/screenshots/window_spec/*.txt
```

Then run without `--update` to confirm the run passes from its references.
A failing run prints the diff of the rows that changed and the client and server log lines from `print` and plugin errors, and writes the new screenshot beside the reference with `.new` added.
When a change to defaults or colours is intended, rerun with `--update` and commit the references in the same commit as the change.

Keep references deterministic: wait for shell prompts before a screenshot that shows them, and use the default `80x24` size when a screenshot shows output the first window's program printed.

`cargo test --test lua_specs` runs gband's own Lua tests and those of every example plugin.
