## Why

With the modal key style, Ctrl+Space then `n` opens a window and leaves navigation mode, so opening several windows in a row, or opening one and moving on to another layout key, needs Ctrl+Space again after every `n`. Navigation mode exists so that a run of layout keys needs one prefix; `n` should keep it active unless the user asks otherwise in the settings window.

## What Changes

- **I on new is off by default**: with no `user/interactive_on_new.lua`, or one that holds anything but `return true`, the modal preset's `n` opens a window and leaves navigation mode active. Only a saved `return true` makes `n` return to interactive mode.
- **Settings window**: the `I on new` line shows `on` only when `gband.settings.interactive_on_new()` returns `true`, and `off` otherwise. Enter, `h`, `l`, Left and Right still save the other value, so on a fresh configuration they save `return true`.
- **`gband.settings.interactive_on_new()` is unchanged**: it still returns `true`, `false` or nil, and nil still means nothing was saved. Only the meaning of nil changes, from on to off. A saved `return false` keeps its meaning.
- **BREAKING** for users who relied on the default: after the change, Ctrl+Space then `n` no longer returns to interactive mode until they turn `I on new` on, or press Escape or Enter after `n`.
- **Docs**: `README.md` and `docs/plugins.md` describe the new default wherever they say `n` returns to interactive mode unless the setting is off, and say how a binding of one's own follows the setting.
- **Tutorials**: the internals tutorial's chapters on the settings window and the key styles quote the changed lines and describe the new default. The scripting tutorial's options chapter says that `I on new` is off until saved on, so a binding of one's own compares the value with `true`.
- **Tests**: every test that presses Ctrl+Space then `n` under the default modal preset and then expects interactive mode is updated to the new default.

Out of scope:
- Renaming the setting, its file or its function.
- Any change to the direct key style, which still ignores the setting.
- Migrating existing configurations: a saved `return false` or `return true` keeps its meaning, and nothing writes the file on the user's behalf.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `settings`: "Saved I on new" makes the setting off unless `gband.settings.interactive_on_new()` returns `true`, and the settings window's `I on new` line shows `on` only then.
- `client-attach`: the modal key style's `n` returns to interactive mode only when `I on new` is on, and its scenarios follow the new default.
- `key-style`: "Preset parity" names `n` as a binding that returns to interactive mode only while `I on new` is on.

## Impact

- `crates/lua/src/runtime/gband/keystyle/modal.lua`: `n` reads the setting as on only when it is `true`.
- `crates/lua/src/runtime/gband/settings.lua`: the `I on new` line's value and its toggle treat nil as off.
- `crates/lua/tests/settings.rs`, `crates/lua/tests/config.rs`, `crates/client/tests/actions.rs`: the setting's reading, the window's line and the default bindings' dispatch.
- The end-to-end tests under `tests/`, the Lua specs under `tests/lua/` and `examples/plugins/agent-status/tests/` that press Ctrl+Space then `n` under the default modal preset and then need interactive mode, and the settings window's first-start screenshot.
- `README.md`, `docs/plugins.md`: the default of `I on new`.
- `docs/internals/07-settings.md`, `docs/internals/08-key-styles.md`: quotes of the changed lines and the prose around them.
- `docs/tutorial/03-options.md`: the default of `I on new` for a binding of one's own.
- No protocol, server, client or harness change, no new dependency, and `gband.api_version` stays `1`.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/lua/src/runtime/gband/keystyle/modal.lua
- crates/lua/src/runtime/gband/settings.lua
- crates/lua/tests/config.rs
- crates/lua/tests/settings.rs
- crates/client/tests/actions.rs
- tests/attach.rs
- tests/config.rs
- tests/floating.rs
- tests/mouse.rs
- tests/navigation.rs
- tests/plugin_runtime.rs
- tests/plugin_testing.rs
- tests/plugin_windows.rs
- tests/prompt.rs
- tests/settings.rs
- tests/window_names.rs
- tests/lua/keylist_spec.lua
- tests/lua/loop_bands_spec.lua
- tests/lua/settings_spec.lua
- tests/lua/window_names_spec.lua
- tests/lua/screenshots/settings_spec/the-settings-window-beside-the-default-sidebar-on-the-first-start--first-start.txt
- examples/plugins/agent-status/tests/agent_status_spec.lua
- README.md
- docs/plugins.md
- docs/internals/07-settings.md
- docs/internals/08-key-styles.md
- docs/tutorial/03-options.md
- openspec/changes/interactive-on-new-default-off/
