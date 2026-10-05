## 1. Starting point

- [ ] 1.1 Confirm `split-plugin-runtime` and `plugin-windows` have archive records on `origin/<base>`, and that `Side`, `gband_protocol::Value`, the server's `gband-lua` thread `Input` enum and `gband.layout()` exist. Verify with `git ls-tree --name-only origin/<base> openspec/changes/archive/` and `rg -n "enum Side|pub enum Value|enum Input|fn layout" crates/`.
- [ ] 1.2 Confirm the main specs now hold `plugin-bridge`'s "No code crosses the connection" and `plugins`' "Side guard", so this change's deltas apply. Verify with `rg -n "No code crosses the connection|Requirement: Side guard" openspec/specs/`.

## 2. Harness crate (black box)

- [ ] 2.1 Create `crates/harness` (`gband-harness`) and move `TestEnv` into `env.rs` and `Attached`, `Tile`, `tiles` and `focused_lines` into `terminal.rs` from `tests/common/mod.rs`. Leave `tests/common/mod.rs` re-exporting them beside the process helpers only it uses. Verify with `cargo test --workspace` passing with no test file changed beyond imports.
- [ ] 2.2 Write the emulator's `write_back` bytes to the PTY in `terminal.rs`, so the runner answers terminal queries. Verify with a harness test where a program in the PTY sends `ESC [ c` and reads the device attributes reply.
- [ ] 2.3 Add an opt-in recorder to `gband-emulator`'s `Callbacks` for OSC 9, OSC 777 `notify`, OSC 52 `c` (base64 decoded locally), bells and `OSC 7777 ; settle ; <n>`, off by default. Verify with emulator unit tests per sequence, and one showing the client's pane emulators record nothing.

## 3. Screenshots

- [ ] 3.1 Add `screenshot.rs`: render a `Grid` in the format of plugin-testing's "Screenshots", with maximal runs, palette and `#rrggbb` colours, the attribute order, wide characters once, and `styles = false`. Verify with unit tests for "Plain screen", "Text only", "Run of one colour", a single-cell run `r:c`, and a wide character.
- [ ] 3.2 Add references: the path rule with the slug and `--<name>`, `.new` on a missing or differing reference, `--update` writing and removing `.new`, a duplicate reference in one run as an error, and the row-keyed diff. Verify with unit tests for "First run", "Accept with update", "Named screenshot" and a diff that names only the changed row.

## 4. Test side in `crates/lua`

- [ ] 4.1 Add `Side::Test`: a state holding only `side` and `api_version` under the side guard, with the guard naming both sides for a shared field, and `print` replaced by a writer the caller supplies. Verify with `crates/lua/tests/plugins.rs` cases for "Test side" and "Client API in a test file", and "Print in a test" against a captured writer.

## 5. The runner (black box)

- [ ] 5.1 Add the `test` subcommand to `src/main.rs` with `FILE...`, `-`, `--update`, `--show`, `--filter` and `--plugin`, no log setup, and the `-s`, `-S` and `-p` refusals. Verify with `tests/subcommands.rs` cases for "Server option given to test", "Session option given to test", `-` combined with a file, and a missing file, each exiting with status 2 and creating no log file.
- [ ] 5.2 Add `runner.rs`: discovery under `tests/`, the plugin under test from `plugin.lua`, duplicate plugin names refused, one test-side state per file with the file's directory on `package.path`, and `gband.test` with `case`, `eq`, `ok`, `match`, case order, file and case time limits, `--filter` and exit statuses. Verify with a new `tests/plugin_testing.rs` covering "Plugin tests found", "No tests", "Cases run in order", "Failing case does not stop the file", "Endless case", "Filter", "Deep equality" and "Failure names both values".
- [ ] 5.3 Add `g.start`: the case tree, the environment table of "Case environment", `config`, `server_config`, `files`, `plugins`, `env`, plugin links, the `xdg-open` and `open` stubs, `gband attach` from the current executable in a PTY of `size`, and cleanup of every process and the tree at the end of the case. Verify with `tests/plugin_testing.rs` cases for "Configuration of the case", "Plugin under test is installed", "Nothing left behind", "Run from inside a pane" and "User's server untouched".
- [ ] 5.4 Add driving: `keys` through `parse_key` and `gband_core::input::encode_key` with the emulator's modes, `type`, `paste` through `encode_paste`, `run`, `resize`, `write`, `wait` and `wait_text` with their timeouts and the screenshot in the error. Verify with cases for "Wait for program output" and "Wait gives up", and a `g.keys("ctrl+space enter")` case waiting for a second tile.
- [ ] 5.5 Add observing: `screen` with `row`, `text`, `cell` and `cursor`, `notifications`, `clipboard`, `opened`, `bells` and `log`. Verify with cases for "Status line colour", "Notification observed", "Opener recorded" and "Print observed".
- [ ] 5.6 Add `screenshot` and `expect_screenshot` to the handle, standard input mode printing instead of comparing, and the report: per-case lines, totals, the failure block with error, diff or screenshot and the case's `print` and error log lines, and `--show`. Verify with cases for "Chunk from standard input", "Failure shows the diff and the logs" and "Show".

## 6. Test channel in the client and the server (white box)

- [ ] 6.1 Add `gband_protocol::test` with the messages of design.md's table. Verify with round-trip tests in `crates/protocol/tests/messages.rs`.
- [ ] 6.2 Join the channel from `gband attach` and `gband server` when `GBAND_TEST_SOCKET` is set: connect before loading, check `SO_PEERCRED`, send `Hello`, wait for `Start`, exit with status 1 and one line on failure, and end on channel close. Verify with `tests/plugin_testing.rs` cases for "Both sides join", "Missing socket", "Unset" and "Runner killed".
- [ ] 6.3 Remove `GBAND_TEST_SOCKET` from pane environments in `crates/server/src/pane.rs`. Verify with an end-to-end case for "Test channel kept out of panes".
- [ ] 6.4 Add frozen time in `crates/lua`: `os.time` and `os.date` wrappers over a shared instant, for every state a process with a frozen clock creates, including after reloads. Verify with unit tests for "Explicit time unchanged", `os.time()` returning the instant, and the instant surviving a reload.
- [ ] 6.5 Handle `Eval` in the client's `select!` loop and as a `gband-lua` thread `Input` in the server, as callbacks that belong to no plugin, under the instruction limit, with results through `Value` and errors answered without being reported. Verify with cases for "Read client state", "Act in the server", "Error answered", "Function result refused", and "Chunk from a local runner" and "Server sends no chunk".
- [ ] 6.6 Handle `Reload` and `SetTime` in both processes. Verify with cases for "Plugin file changed", "Broken file" and "Time moved".
- [ ] 6.7 Implement settle as design.md describes: input markers counted from `FocusGained` on a test channel only, the server barrier through each FIFO, the client draw with the OSC marker, and rounds up to ten with the failing step named after 10 seconds. Verify with cases for "Key effect is drawn" and "Server handler effect is drawn", and a pair of handlers that trigger each other failing after ten rounds.

## 7. Runner (white box)

- [ ] 7.1 Add `g.client`, `g.server`, `g.settle`, `g.reload`, `g.set_time` and the `time` option of `g.start`, and make `g.start` return after the first settle. Verify with cases for "Open a pane by key", "Reload after editing a plugin" and "Clock segment frozen".

## 8. gband's own Lua tests

- [ ] 8.1 Write `tests/lua/statusline_spec.lua` for the bundled `band`, `mode`, `hints`, `position` and `clock` segments, with committed references under `tests/lua/screenshots/`. Verify with `gband test tests/lua/statusline_spec.lua` exiting with status 0.
- [ ] 8.2 Write `tests/*_spec.lua` and references for each plugin under `examples/plugins/`. Verify with `gband test` in each plugin's directory exiting with status 0.
- [ ] 8.3 Add `tests/lua_specs.rs`, which runs both sets with the built binary and prints the report when one fails. Verify with `cargo test --test lua_specs`, and with a deliberately broken reference making it fail with the diff in its output.

## 9. Documentation and the project skill

- [ ] 9.1 Write `docs/testing.md` for plugin authors: layout, CLI, the API, screenshots and references, frozen time, settle versus wait, and a worked example. Link it from `docs/plugins.md`, `README.md` and the example READMEs. Verify that every function and option in the plugin-testing and test-channel specs appears in it.
- [ ] 9.2 Add `.claude/skills/gband-test/SKILL.md` describing the `gband test - --show` loop, promoting a run to a spec file and accepting references with `--update`. Verify by running its example command against `examples/plugins/pane` and reading a screenshot from the output.

## 10. Gate

- [ ] 10.1 Run `.claude/flow-gate`: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings` and `cargo test --workspace --locked`, all passing.
- [ ] 10.2 Manual: the user runs `gband test` in `examples/plugins/pane` from a real terminal inside their own gband session, and confirms the report and that their session is unchanged.
