## Why

Running the same command in several shells, such as an update on a set of hosts, needs one keystroke stream sent to many windows. Today every key goes to the focused window only, so the user types the same text once per window.

## What Changes

- Add **multi mode**, a per-client state. While it is on, every key and paste that would go to the focused window goes to every window of the band the client views at that moment, tiled and floating, minimized ones included. Mouse input still goes only to the window under the pointer. Plugin windows still take keys while focused.
- Add the client action `toggle_multi`, "toggle multi mode", which turns multi mode on or off.
- Modal key style: `m` in navigation mode toggles multi mode and returns to interactive mode. The prefix key still enters navigation mode, and Escape and Enter still leave it. Leaving navigation mode while multi mode is on returns to multi mode. Pressing `m` in navigation mode again turns it off. `m` is unbound today; `f` keeps toggling full width.
- Direct key style: Ctrl+Space then `m` toggles multi mode, keeping parity with the modal preset.
- Floating key style: the window list gains a third entry after `Settings`, `Multi mode` with the shortcut `m`, which reads `Multi mode (on)` while multi mode is on. Picking it cancels the list and toggles multi mode. Every window entry moves down one row.
- The prefix key sent to the windows, by Ctrl+Space twice, also goes to every window of the band while multi mode is on.
- The sidebar shows `M` instead of `I` while multi mode is on and `root` is active. `N` still shows in navigation mode, and the floating style keeps its apps character.
- `gband.view()` holds `multi = true` while multi mode is on.
- A reload of the configuration turns multi mode off, as it makes `root` active.

## Capabilities

### New Capabilities
- `multi-mode`: the multi mode state, which windows receive the input while it is on, the `toggle_multi` action and when the state resets.

### Modified Capabilities
- `client-attach`: "Key bindings" gains the `m` row in the modal and direct tables; "Input to the server" sends keys and pastes to every window of the viewed band while multi mode is on.
- `actions`: "Lua names of actions" lists `toggle_multi` and says what `send_prefix` does while multi mode is on.
- `sidebar`: "Mode letter" shows `M` while multi mode is on in interactive mode.
- `lua-control`: "Read the view" holds `multi`.
- `lua-api`: "Core primitives" runs `on_state` functions when multi mode changes, and `state()` holds `multi`.
- `floating-key-style`: "Window list" gains the `Multi mode` entry and its shortcut, with its scenarios renumbered; "Desktop menus", "Right press" and "Leader" scenarios follow the extra row.

## Impact

- Client: a multi flag in the client's display state, the fan-out of keys, pastes and the sent prefix key in `crates/client/src/lib.rs`, the new `ClientAction::ToggleMulti` in `crates/core/src/action.rs`, and the reset on reload.
- Lua: the `toggle_multi` built-in in `crates/lua/src/actions.rs`, `multi` in the view state in `crates/lua/src/ui.rs` and in `gband.view()` in `crates/lua/src/control.rs`.
- Bundled Lua: the modal and direct presets, the sidebar and the desktop plugin.
- No wire protocol change: the client sends one key or paste message per window, as it already names the window in each.
- Docs: README, `docs/plugins.md`, the keys tutorial chapter, and the key style and sidebar internals chapters, which quote the bundled files.
- Tests: client unit tests, Lua specs for the sidebar, key list and desktop, and end-to-end tests of typing in multi mode.

## Coordination

### Author
- gmteixeira

### Depends On
- input-before-output

### Expected Files
- openspec/changes/multi-mode/
- crates/core/src/action.rs
- crates/client/src/lib.rs
- crates/client/tests/actions.rs
- crates/lua/src/actions.rs
- crates/lua/src/ui.rs
- crates/lua/src/control.rs
- crates/lua/src/runtime/gband/keystyle/modal.lua
- crates/lua/src/runtime/gband/keystyle/direct.lua
- crates/lua/src/runtime/gband/sidebar.lua
- crates/lua/src/runtime/gband/desktop.lua
- crates/lua/tests/config.rs
- crates/lua/tests/control.rs
- crates/lua/tests/keystyle.rs
- crates/lua/tests/sidebar.rs
- crates/lua/tests/desktop.rs
- tests/keystyle.rs
- tests/multi_mode.rs
- tests/lua/keylist_spec.lua
- tests/lua/screenshots/keylist_spec/
- tests/lua/sidebar_spec.lua
- tests/lua/screenshots/sidebar_spec/
- tests/lua/floating_spec.lua
- tests/lua/screenshots/floating_spec/
- README.md
- docs/plugins.md
- docs/tutorial/01-keys.md
- docs/internals/08-key-styles.md
- docs/internals/09-sidebar-and-errors.md
