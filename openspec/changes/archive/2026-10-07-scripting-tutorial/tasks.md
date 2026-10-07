## 1. Checks

- [x] 1.1 Add to `crates/lua/tests/common/docs.rs` a public way to test whether a text names a path under the existing identifier-boundary rule, without reading a file. Verify `cargo test -p gband-lua --test api_docs` still passes
- [x] 1.2 Create `crates/lua/tests/tutorial.rs` with a chapter list read from `docs/tutorial/` (files matching `NN-<slug>.md`) and a test that every chapter has `examples/tutorial/NN-<slug>/` and every directory there has a chapter. Verify it fails on a scratch `docs/tutorial/12-more.md` with no directory and names the file, then remove the scratch file
- [x] 1.3 Add the `lua` block check to `tutorial.rs`: parse fences, take blocks whose info string's first word is `lua`, and require each block's text to be a substring of some file under the chapter's directory. The failure names the chapter and the block's first line. Verify with a temporary chapter whose block is missing from its directory
- [x] 1.4 Add the `screen` block check: the info string `screen <path>` names a reference under the chapter directory, and the block equals the reference's rows with the `N|` prefixes stripped. The failure names the chapter and the reference, both for a missing reference and for different rows. Verify with a temporary block that differs from its reference by one character
- [x] 1.5 Add the `api` check: find `api = <integer>` in every `.lua` file under `examples/tutorial/`, require at least one, and require each to equal `gband_lua::API_VERSION`, naming the file otherwise. Verify by temporarily setting one file to `api = 2`
- [x] 1.6 Add the namespace check: load an empty client and server configuration with `common::Scratch`, collect each `gband` table's top-level string keys, drop `core`, and require `gband.<field>` to appear in the joined chapters under the boundary rule from 1.1. The failure lists every missing field. Verify that a temporary chapter set missing `gband.sessions` fails and names it
- [x] 1.7 Add a `tutorial` test to `tests/lua_specs.rs` that discovers every directory under `examples/tutorial/` and every `plugins/<name>/` in them that holds `tests/`. It runs `gband test` in each concurrently through the existing `run` helper's command and fails naming each directory whose run failed. Verify with `cargo test --test lua_specs tutorial` once chapter 00 exists

## 2. Index and chapters 00 to 05

Each chapter task: write `docs/tutorial/NN-<slug>.md` and `examples/tutorial/NN-<slug>/` with the complete `user/` state at the end of the chapter, and `tests/<slug>_spec.lua` whose cases read the chapter's files with `io.open` and start gband with them. Commit the screenshot references the chapter's `screen` blocks show. Link each API part to its `docs/plugins.md` or `docs/testing.md` section. Verify each with `gband test` in the chapter directory and `cargo test -p gband-lua --test tutorial`.

- [x] 2.1 Write `docs/tutorial/README.md`, the index: what the tutorial builds, how to run a chapter's directory with `gband test`, and links to all twelve chapters in order. Verify every link resolves to a file, once the chapters exist
- [x] 2.2 Write `00-setup`: the configuration directory and `gband.config_dir`; an empty `user/init.lua` that grows to `gband.keystyle.use()`, `gband.plugin("gband.errors")` and the sidebar; reloading on save; the Lua prompt; `print` and the client log; an error, the red `!`, `gband.errors` and `gband.clear_errors`; and copying `defaults/init.lua` as the alternative, linked to the README. The spec covers the reload of a broken file and a prompt line
- [x] 2.3 Write `01-keys`: `gband.keymap.set`, `gband.bind`, `gband.unbind`, key tables and modes, `gband.keystyle`, and mouse names, ending with a binding that marks the focused window. The spec presses the bindings and checks their effect
- [x] 2.4 Write `02-actions`: `gband.action` and action targets, `gband.cmd`, `gband.spawn`, `gband.notify`, `gband.bell`, `gband.clipboard` and `gband.open`, with marking registered as an action and a command. The spec checks the notification, the clipboard write and the opened URL through `g.notifications`, `g.clipboard` and `g.opened`
- [x] 2.5 Write `03-options`: `gband.opt`, `gband.set` and `gband.settings`, with an option for the mark letters. The spec checks the default and a changed value
- [x] 2.6 Write `04-events`: `gband.on`, `gband.augroup` and `gband.emit` with a `User` event raised when a mark changes. The spec checks the handler ran and that clearing the group stops it
- [x] 2.7 Write `05-layout`: `gband.layout`, `gband.view`, `gband.window` and `gband.band`, with jumping to a marked window. The spec opens windows, marks one, moves away and jumps back

## 3. Chapters 06 to 08

- [x] 3.1 Write `06-colors`: `gband.hl` with a `Mark` group, `gband.colorscheme` and a theme, and `gband.palette`. The spec checks the resolved group with `gband.hl.get`. Its `screen` block shows a window carrying a mark
- [x] 3.2 Write `07-plugin-windows`: `gband.win` with a list of marks that Enter jumps from, and `gband.ui.width` and `gband.ui.truncate` for fitting labels. The spec opens the list, moves its cursor and jumps
- [x] 3.3 Write `08-bars`: `gband.bar` with a segment showing the focused window's mark beside the sidebar. The spec and its `screen` block show the bar with two windows

## 4. Chapters 09 to 11

- [x] 4.1 Write `09-plugin`: move the marks code to `plugins/marks/` with `plugin.lua`, `client.lua` and `lua/marks/init.lua` declaring `api = 1`; `setup` options; `gband.plugin`, `gband.plugins`, `gband.runtimepath`, `gband.side` and the side guard, and `gband.api_version`; ownership of what the plugin registers. `user/init.lua` sets it up. The spec passes the plugin with `plugins = { "../plugins/marks" }`
- [x] 4.2 Write `10-server`: `plugins/marks/server.lua` keeps marks in `gband.window_state`, registers a server command listing them, and emits an event when a marked window's program exits; `gband.sessions` and `gband.session`; the client half reads window state, calls the command with `gband.rpc` and notifies on the event. The spec checks a mark set in one client through `g.server`, the command's result and the notification
- [x] 4.3 Write `11-testing`: the reader writes `plugins/marks/tests/marks_spec.lua` with `gband.test`, `g.start`, driving, `g.client` and `g.server`, `g.settle`, and `g.expect_screenshot`, and runs `gband test` in `plugins/marks/`. Verify both `gband test` in the chapter directory and in `plugins/marks/` pass

## 5. Links and the whole suite

- [x] 5.1 Link `docs/tutorial/README.md` from the Scripting section of `README.md` and from the introduction of `docs/plugins.md`. Verify both links resolve
- [x] 5.2 Run `cargo test -p gband-lua --test tutorial` and `cargo test --test lua_specs tutorial`, then the full `cargo test`, and confirm that every check passes and that each chapter directory and `plugins/marks/` ran
