## Why

With the modal key style, `n` in navigation mode opens a window, focuses it and returns to interactive mode. A user who opens a window and then wants to arrange it, resize it or open another has to press the prefix again. Whether `n` lands in interactive mode is a personal preference, and settings-themes gives preferences one window, so it belongs there as a fourth row.

## What Changes

- **Saved setting**: `user/interactive_on_new.lua` holds `return true` or `return false`, read by `gband.settings.interactive_on_new()`, with the same rules as `user/sidebar.lua`. Nothing saved reads as nil, which means on.
- **Modal `n`**: opens a window and focuses it as today. It returns to interactive mode unless the setting is `false` when the configuration loads. When it is `false`, navigation mode stays active with the new window focused.
- **Settings window**: a fourth line, `I on new`, shows `on` or `off`. Enter, `h`, `l`, Left and Right on it save the other value, and the configuration reloads and the settings window reopens on that line, as for every other row.
  - The line is shown only when the `keys` line shows `modal`. The direct style has no navigation mode, so its `n` always lands in interactive mode and the setting has nothing to change.
  - The window is 6 rows high with the line and 5 without it. Its width stays 31: `I on new` fits the 9-cell label column.
- The direct style, the first-start offer and its condition are unchanged.

Out of scope:
- Applying the setting without a reload.
- Any other way of opening a window, such as an action called from a user's own binding, the Lua prompt or the key list. Only the modal preset's `n` reads the setting.

## Capabilities

### New Capabilities

### Modified Capabilities
- `settings`: the saved `I on new` setting and `gband.settings.interactive_on_new()`; the settings window's fourth line, shown with the modal style only, its height, and its keys.
- `client-attach`: the modal style's `n` returns to interactive mode unless the saved setting is `false`.

## Impact

- Lua runtime: `crates/lua/src/runtime/gband/settings.lua` gains `interactive_on_new()` and the fourth line; `crates/lua/src/runtime/gband/keystyle/modal.lua` reads the setting in its `n` binding.
- Tests: `crates/lua/tests/settings.rs`, `tests/settings.rs`, `crates/client/src/bindings.rs`, and the settings window screen references in `tests/lua/settings_spec.lua`.
- `README.md` and `docs/plugins.md`.
- No Rust host change, no protocol change and no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- settings-themes

### Expected Files
- openspec/changes/settings-interactive-on-new/
- crates/lua/src/runtime/gband/settings.lua
- crates/lua/src/runtime/gband/keystyle/modal.lua
- crates/lua/tests/settings.rs
- crates/client/src/bindings.rs
- tests/settings.rs
- tests/lua/settings_spec.lua
- tests/lua/screenshots/settings_spec/
- README.md
- docs/plugins.md
