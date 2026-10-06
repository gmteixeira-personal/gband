## 0. Specs

- [ ] 0.1 For each MODIFIED requirement in this change's delta specs, compare the block with the current `openspec/specs/<capability>/spec.md` block of the same requirement, as rename-panes-to-windows and the changes archived before this one left it, and fold in every requirement text and scenario this change did not write. Verify that the only differences left are this change's own and that `openspec validate navigation-mode --strict` passes

## 1. Modes in the Lua keymap

- [ ] 1.1 Add a `modes` map (table name to optional label) to `Keymaps` in `crates/lua/src/keymap.rs`, and add `gband.keymap.mode(table, opts)` with the load-only rule and the errors for `root`, a bad table name, a bad `opts` and a bad `label`; verify with new cases in `crates/lua/tests/keymap.rs` for each scenario of the configuration "Modes" requirement
- [ ] 1.2 Add `gband.keymap.label(table)`, callable while loading and in callbacks; verify `crates/lua/tests/keymap.rs` covers a labelled mode, a mode without a label, and a plain table
- [ ] 1.3 Let `gband.keymap.enter("root")` pass the "table has no binding" check; verify a test where `root` is empty and a callback enters it without error
- [ ] 1.4 Return the mode set from `finish` and carry it on `Config` in `crates/lua/src/lib.rs`; verify `gband_lua::defaults(Side::Client)` exposes it once 4.1 lands, and a loaded user file that declares `resize` a mode exposes `resize`

## 2. Running a binding from Lua

- [ ] 2.1 Add `gband.keymap.run(table, key)` in `crates/lua/src/keymap.rs`. It parses the key as `set` does, `prefix` included, and looks the binding up by `(table, chord)`. An action binding queues `Dispatch::Action`. A registered action or function binding calls its Lua function under its owner inside the current callback. It returns `true` or `false`, and it is an error outside a callback. Verify with `crates/lua/tests/keymap.rs` cases for the four "Run a binding" scenarios
- [ ] 2.2 Check that a function run through `run` that calls `gband.keymap.enter` queues the `Enter` in the caller's queue, in order after earlier dispatches; verify with a test that runs a function dispatching an action and then entering `root`, and asserts the queued order

## 3. Client leader keeps a mode active

- [ ] 3.1 Give the client `Keymap` in `crates/client/src/bindings.rs` the mode set, and change `Leader::handle` so a mode is not reset to `root`: a bound key returns `Run`, an unbound key `Discard`, and the table stays active. Verify with unit tests in `bindings.rs` for repeated keys, an unbound key in a mode, and a table that is not a mode still resetting
- [ ] 3.2 Pass the modes from `Config` into `Keymap::new` at start and on reload in `crates/client/src/lib.rs`; verify a reload still makes `root` active and emits `KeyTableChanged` once
- [ ] 3.3 Confirm that `KeyTableChanged` is not emitted while a mode stays active; verify with a `crates/client/tests/events.rs` case for Ctrl+Space, `h`, `l`, Escape recording exactly two events

## 4. Default configuration

- [ ] 4.1 In `crates/lua/src/defaults.lua`, declare `gband.keymap.mode("prefix", { label = "navigation" })`. Rebind `enter` and `escape` to a function entering `root` with description `interactive mode`, add `n` as a function that calls `gband.action.open_window()` and then enters `root` with description `open a window`, and rebind `prefix prefix` to a function that calls `gband.action.send_prefix()` and then enters `root` with description `send the prefix key`. Add `left`, `right`, `down` and `up` for the focus actions, and keep every binding in the order of the client-attach default table, with `n` after `c`. Verify the `default_keys_follow_the_spec` and `every_default_binding_names_an_action` tests in `bindings.rs`, updated to the new table and its function bindings, pass
- [ ] 4.2 Update `crates/lua/tests/config.rs` so the defaults test checks the descriptions of action and function bindings, the `navigation` label, and `?` bound to `keylist.open`; verify `cargo test -p gband-lua`

## 5. Bundled Lua modules

- [ ] 5.1 Make `crates/lua/src/runtime/gband/statusline/mode.lua` render `gband.keymap.label(ctx.table)`; verify `crates/lua/tests/statusline.rs` shows `navigation` after Ctrl+Space with the defaults, and `RESIZE` for a labelled user mode
- [ ] 5.2 Make `crates/lua/src/runtime/gband/statusline/hints.lua` label the root prefix hint with `gband.keymap.label("prefix")`; verify `crates/lua/tests/key_hints.rs` shows `C-space navigation` with the defaults, `C-space prefix` without a mode, and the navigation hints' start and end strings of the key-hints scenarios at 520 columns
- [ ] 5.3 In `crates/lua/src/runtime/gband/keylist.lua`, enter `root` when the list opens and when `keylist.open` focuses an open list, title it `gband.keymap.label("prefix") .. " keys"`, run Enter through `gband.keymap.run("prefix", key)`, and mute only the `keylist.open` line; update `tests/lua/keylist_spec.lua` for the title, the muting, running `n` from the list, opening it again from navigation mode, and the list keeping focus, open windows with `n` where it uses the defaults, and regenerate its screenshots under `tests/lua/screenshots/keylist_spec/`; verify `cargo test --test lua_specs bundled_key_list`

## 6. Existing tests and end-to-end coverage

- [ ] 6.1 Update the client tests in `crates/client/tests/` (`actions.rs`, `events.rs`, `statusline.rs`, `plugin_windows.rs`) and the `statusline__default_status_line`, `statusline__prefix_hints_cut` and `statusline__top_placement` snapshots for navigation mode: leave the mode before typing, and open windows with `n`; verify `cargo test -p gband-client`
- [ ] 6.2 Update the end-to-end tests in `tests/attach.rs`, `tests/config.rs`, `tests/statusline.rs`, `tests/plugin_windows.rs`, `tests/floating.rs` and `tests/plugin_runtime.rs`: replace `\x00\r` with `\x00n`, leave navigation mode with `\r` before typing into a window, and drop the repeated `\x00` between keys of one navigation sequence; verify `cargo test --test attach --test config --test statusline --test plugin_windows --test floating --test plugin_runtime`
- [ ] 6.3 Update the Lua tests that use the defaults: the default status line in `tests/lua/statusline_spec.lua` shows `C-space navigation`, and `examples/plugins/agent-status/tests/agent_status_spec.lua` opens its second window with `ctrl+space n`; regenerate `tests/lua/screenshots/statusline_spec/default-status-line.txt` and `examples/plugins/agent-status/tests/screenshots/agent_status_spec/a-prompt-marks-the-window-waiting.txt`; verify `cargo test --test lua_specs`
- [ ] 6.4 Add `tests/navigation.rs` with end-to-end cases for repeated focus moves, repeated resize followed by Escape and `tput cols` printing `54`, Enter not opening a window, `n` opening one and returning to interactive mode, and `?` opening the key list with `j` moving its cursor line; verify `cargo test --test navigation`

## 7. Documentation and final checks

- [ ] 7.1 Update `README.md`: the default keys table for navigation mode, the breaking change from Enter to `n`, and the snippet that keeps one-key prefix bindings or declares the mode in a user file; verify by reading the rendered table against the client-attach default table
- [ ] 7.2 Update `docs/plugins.md`: `gband.keymap.mode`, `gband.keymap.label`, `gband.keymap.run`, `enter("root")`, the hints and mode segment labels, and the key list example; verify every new API in the configuration spec appears there
- [ ] 7.3 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass
