## Why

Nobody can see a plugin's result without attaching a client in a real terminal and looking at it. A plugin author has no way to check that a status line segment draws in the right colours, that a server plugin notices a waiting agent, or that a binding opens a pane, short of trying it by hand. Claude, which writes much of gband and its plugins, cannot attach to a terminal at all, so every change to drawing or plugin behaviour waits for the user to look. gband's own end-to-end harness already runs the real binary in a PTY and reads its screen, but only Rust tests can use it, and it shows the screen only when an assertion fails.

## What Changes

- **`gband test`**: a new subcommand that runs Lua test files. `gband test` runs `tests/**/*_spec.lua` under the plugin in the working directory, `gband test FILE...` runs the named files, and `gband test -` runs one chunk read from standard input. Options: `--update` rewrites screenshot references, `--show` prints every screenshot taken, `--filter PATTERN` runs only the matching cases, and `--plugin PATH`, repeatable, installs another plugin beside the one under test. It refuses `-s`, `-S` and `-p`. Results go to standard output, and the exit status is 0 only when every case passed.
- **The test side**: test files run in a Lua state of their own with `gband.side` `"test"`, under the side guard. `require("gband.test")` returns the test API: `case`, assertions, and a handle per case. `print` in the test side writes to standard output.
- **Isolated cases**: each case runs in a new directory tree of its own. The tree holds its own XDG directories, the plugin under test linked into the plugins directory, `user/init.lua` and `user/server.lua` as the case writes them, `/bin/sh` as the shell, animations off and truecolor. The case starts the real `gband attach` in a PTY of the size it asks for, attach starts the server, and gband's emulator reads the output. Nothing the user has configured or running is touched.
- **Driving and observing**: a case types keys and text, pastes, resizes, rewrites its configuration to reload it, and waits for a condition. It reads the screen and cursor, the desktop notifications, clipboard writes and bells the client sent to its terminal, and the lines `print` and plugin errors wrote to the case's logs.
- **Screenshots**: a screenshot is plain text, a header naming the size and the cursor, then the rows, then one line per run of cells with the same non-default style, with colours in hex. `expect_screenshot` compares one with a reference file under `tests/screenshots/`, writes the reference when none exists, and fails with a diff otherwise. `styles = false` compares the text only.
- **The test channel**: when `GBAND_TEST_SOCKET` names a socket, the client and the server connect to it at start. Through it a case evaluates a Lua chunk in the client's or the server's Lua state and gets back a plain data result, waits until both have settled, and sets the time those states read. The server removes the variable from every pane's environment. The wire protocol does not change, and no Lua crosses the connection between client and server.
- **Frozen time**: each case freezes `os.time` and `os.date` in the client and the server at one instant, unless it opts out, so a clock segment draws the same text on every run.
- **Trust rule**: the plugin-bridge rule that a client evaluates only Lua files from its own machine is widened to also allow chunks from a test runner its launcher enabled through `GBAND_TEST_SOCKET`.
- **gband's own tests**: the Rust end-to-end harness (`TestEnv`, `Attached`, tile parsing) moves out of `tests/common` into a crate shared with `gband test`. The Rust tests stay. The sample plugins under `examples/plugins/` and the bundled status line modules gain Lua tests, and one Rust test runs `gband test` over them.
- A project skill tells Claude to use `gband test -` and `--show` when it changes plugins, drawing or the status line.

Out of scope:
- Line-by-line coverage, benchmarks or profiling.
- Running cases against a server on another machine.
- Pixel images of screenshots.
- Highlight group names in screenshots. Cells carry their colours only.
- Mouse input.
- A package manager or test discovery outside a plugin's `tests/` directory.

## Capabilities

### New Capabilities

- `plugin-testing`: the `gband test` subcommand, test discovery, the per-case environment, the test side and its API, driving and observing a case, screenshots and their references, and the report.
- `test-channel`: `GBAND_TEST_SOCKET`, how the client and the server connect to it, evaluating chunks in either Lua state, settling, frozen time, and keeping the variable out of panes.

### Modified Capabilities

- `plugin-bridge`: the rule on which Lua a client evaluates also allows test channel chunks.
- `plugins`: `gband.side` is `"test"` in the test side, the side guard covers it, and `print` there writes to standard output.
- `command-line`: `-s`, `-S` and `-p` are refused by `gband test`.
- `session-server`: a pane's environment never holds `GBAND_TEST_SOCKET`.

## Impact

- New crate `crates/harness`: the per-case environment, the PTY host and the screen reading moved from `tests/common/mod.rs`, plus screenshots and the test channel's runner end.
- `crates/lua`: the `"test"` side, the bundled `gband.test` module, the test channel's Lua end (evaluate a chunk, frozen time).
- `crates/client` and `crates/server`: connect to the test channel, answer evaluate and settle requests, and, in the server, strip the variable from pane environments.
- `crates/emulator`: record notifications, clipboard writes and bells instead of only logging them.
- `src/main.rs`: the `test` subcommand.
- `tests/`: `tests/common` becomes a thin re-export of the shared crate. A new test runs the Lua tests of the examples and the bundled modules.
- `examples/plugins/*/tests/` for the sample plugins, `tests/lua/` for the bundled status line modules (`band`, `mode`, `hints`, `position`, `clock`), `docs/plugins.md`, `docs/testing.md`, `.claude/skills/`.
- Depends on `split-plugin-runtime` for the server Lua state and the plain data value, and on `plugin-windows`, which also changes `tests/common`.
- No new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- split-plugin-runtime
- plugin-windows

### Expected Files
- .claude/skills/gband-test/
- Cargo.toml
- Cargo.lock
- README.md
- crates/client/src/
- crates/emulator/src/
- crates/harness/
- crates/lua/src/
- crates/lua/tests/plugins.rs
- crates/protocol/src/
- crates/protocol/tests/io.rs
- crates/protocol/tests/messages.rs
- crates/server/src/
- crates/server/tests/scripting.rs
- crates/test-support/src/lib.rs
- docs/plugins.md
- docs/testing.md
- examples/plugins/
- src/channel.rs
- src/lib.rs
- src/main.rs
- tests/common/mod.rs
- tests/lua/
- tests/lua_specs.rs
- tests/plugin_testing.rs
- tests/subcommands.rs
- openspec/changes/plugin-testing/
