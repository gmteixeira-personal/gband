## 1. Deltas on the current specs

- [ ] 1.1 Re-copy each modified requirement from the main spec on `dev`: client-attach's "Key bindings", lua-control's "Read the layout" and "Dispatch order", wire-protocol's "Client messages", "Server messages" and "Handshake", configuration's "Options", and plugin-testing's "Case environment". Re-apply this change's additions to each: the `N` row and its two scenarios, `name` and `manual_name`, `gband.window.rename`, the `rename` and `window name` messages and the attach order, the `window_titles` option and its scenarios, and `g.start`'s `window_titles` field, `GBAND_WINDOW_TITLES=off` and their scenarios. Set the protocol version to one above `PROTOCOL_VERSION` on `dev`, and fix the "Version 9 client meets a version 10 server" scenario to match. Verify with `openspec validate window-names --strict` and by diffing each requirement against the main spec, which must show only these additions

## 2. Emulator title

- [ ] 2.1 In `crates/emulator/src/callbacks.rs`, keep the title, a stack of at most 10 entries that can hold no title, and a changed flag. Implement `set_window_title`, join the parameters of longer OSC 0 and 2 in `unhandled_osc`, ignore OSC 1, and handle `CSI 22 t` and `CSI 23 t` with the parameters 0 and 2 or none in `unhandled_csi`. Remove control characters, trim, and clear on empty. Expose `title()` and `take_title_changed()` on `Grid` in `crates/emulator/src/lib.rs`. Verify every "Window title" scenario as unit tests in `crates/emulator/tests/contract.rs`

## 3. Shown names in core

- [ ] 3.1 Add `crates/core/src/names.rs`, exported from `crates/core/src/lib.rs`: from a `Band` and a map from window id to name, return each named window's shown name, numbered in layout order. Verify every "Shown name" scenario in `crates/core/tests/names.rs`

## 4. Protocol

- [ ] 4.1 In `crates/protocol/src/message.rs`, add the `Rename` client message and the `WindowName` server message, and bump `PROTOCOL_VERSION` to the version task 1.1 settled. Verify the "Rename round trip", "Window name round trip" and new handshake scenarios in `crates/protocol/tests/messages.rs`

## 5. Server

- [ ] 5.1 Add `crates/server/src/process.rs`, declared in `crates/server/src/lib.rs`: the foreground command from `/proc/<pgid>/cmdline`, then `comm`, then the program's own command, with the base name taken and a leading `-` stripped. Verify the base name, the stripping and each fallback with unit tests in the module, and the "Login shell" scenario
- [ ] 5.2 In `crates/server/src/window.rs` and `session.rs`, keep each window's automatic name, manual name and last process group. Recompute the automatic name after each `process` and on a one-second interval in the session loop. Send `WindowName` to every attached client when either name changes, and after the snapshots of an attach, before the window state messages. Handle `Rename` by trimming the name and clearing it on empty, ignoring a drawn window or a window no longer in the layout. Verify the "Automatic name", "Foreground command", "Manual name" and "Names shared by every client" scenarios and "Attach order with window names" end to end in `tests/window_names.rs`

## 6. Client

- [ ] 6.1 In `crates/client/src/lib.rs`, keep each window's names from `WindowName`, recompute shown names per band with `names.rs` on each layout and name message, drop them for windows the layout no longer holds, and redraw. Verify "Number follows the layout" end to end in `tests/window_names.rs`
- [ ] 6.2 In `crates/client/src/render.rs`, draw the shown name on the top border of each tiled and floating window that runs a program, after `draw_border`, only when the top side is drawn, in the border's resolved style, cut by the same helper `draw_float` uses for plugin titles. Verify every "Border title" scenario in the client's render tests and in `tests/lua/window_names_spec.lua`
- [ ] 6.3 Declare the client option `window_titles` in `crates/lua/src/options.rs`, a boolean defaulting to `true`. Read `GBAND_WINDOW_TITLES` once at client start in `crates/client/src/lib.rs`, following `crates/client/src/animation.rs`. Skip the border title in `render.rs` when the variable is `off` or the option is `false`. Verify "Window titles off" and "Window titles of the wrong type" in `crates/lua/tests/options.rs`, and every "Titles turned off" scenario in `tests/window_names.rs`

## 7. Lua

- [ ] 7.1 In `crates/lua/src/control.rs`, add `name` and `manual_name` to the window and floating tables of `gband.layout()`, and add `gband.window.rename`, validated and dispatched in order as `set_position` is. Verify "Window names", "Drawn window has no name", "Rename during loading", and every error scenario of "Manual name" in `crates/lua/tests/control.rs`
- [ ] 7.2 In `crates/lua/src/runtime/gband/prompt.lua`, turn the editor state into one table per prompt kind, register `prompt.rename`, open the rename prompt titled `rename` with the manual name and no `:`, close the other kind on open, and rename the target on Enter when it is still in the layout. Verify every "Rename prompt" scenario in `crates/lua/tests/prompt.rs` and `tests/prompt.rs`, and that every existing Lua prompt test still passes
- [ ] 7.3 Bind `prefix N` to `gband.action["prompt.rename"]` after `:` in `crates/lua/src/runtime/gband/keystyle/modal.lua` and `direct.lua`. With the modal style it returns to interactive mode through the action's own `keymap.enter("root")`. Verify `default_keys_follow_the_spec`, `direct_keys_follow_the_spec` and `every_default_binding_names_an_action` in `crates/client/src/bindings.rs`, and the "Rename the focused window" and "Rename with the direct key style" scenarios in `tests/keystyle.rs`

## 8. Titles off in tests

- [ ] 8.1 Add the `window_titles` field to `g.start` in `crates/harness/src/case.rs`, default `false`, and set `GBAND_WINDOW_TITLES=off` in the case environment unless it is `true`. Set `GBAND_WINDOW_TITLES=off` in the base environment of `crates/harness/src/env.rs`, and make the tests in `tests/window_names.rs` that read titles remove it through the terminal's `adjust` hook. Verify "No window titles by default" and "Window titles asked for" in `tests/lua/window_names_spec.lua`, and `cargo test --test window_names`

## 9. Screens

- [ ] 9.1 Add `tests/lua/window_names_spec.lua`, every case started with `window_titles = true`, and register it in `tests/lua_specs.rs`. Cover a titled tile, two numbered shells, a cut title, the unfocused style and the rename prompt. Regenerate the key list references under `tests/lua/screenshots/keylist_spec/` with `GBAND_UPDATE_SCREENSHOTS`, and check that each diff holds only the `N` line and lines moved by it. Verify with `cargo test --test lua_specs`, which also runs the example plugins' cases, and with `git status --porcelain -- tests/lua/screenshots/ examples/` showing no other reference changed

## 10. Documentation and gates

- [ ] 10.1 Update `README.md` (the `N` row of the navigation key table, a window names section on the name order, numbering, manual names, the prompt-title trade-off with the fish default and a zsh and bash preexec example, the `window_titles` option, and `GBAND_WINDOW_TITLES` beside `GBAND_ANIMATIONS`), `docs/plugins.md` (`gband.window.rename`, `name` and `manual_name` in `gband.layout()`, `prompt.rename`, and `window_titles` in the options list) and `docs/testing.md` (`g.start`'s `window_titles` and the default without titles). Verify with `rg 'prompt.rename|manual_name|window.rename|window_titles|GBAND_WINDOW_TITLES'` that every mention names the function, field, action or default correctly
- [ ] 10.2 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `openspec validate window-names --strict`, and verify all pass
