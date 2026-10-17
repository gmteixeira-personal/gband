## Why

Switching the key style to `floating` in the settings window floats every tiled window, and switching back to `modal` or `direct` leaves them all floating. The pre-floating arrangement is lost, so the user must retile every window by hand after a round trip through floating.

## What Changes

- The settings window records the current layout when its `keys` line switches a non-floating style to `floating`, before the reload floats the windows.
- The settings window restores that recorded layout when its `keys` line switches `floating` to `modal` or `direct`: previously tiled windows tile again in their old bands, columns, order, stacking, widths, full-width and tabbed state; previously floating windows keep their box; windows opened while floating tile after the restored windows.
- When no recorded layout exists, because `floating` was already in use when the settings window opened, switching away from `floating` still tiles every floating window.
- The recorded layout is deleted once restored, and overwritten when `floating` is entered again.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `settings`: the `keys` line's key-style switch records the layout entering `floating` and restores it leaving `floating`.

## Impact

- `defaults/lua/gband/settings.lua`: the `keys` line's step function, plus a layout snapshot written under `user/` and a restore routine driven by `gband.layout`, `gband.action.toggle_window_floating`, `gband.window.set_width`, `gband.window.set_height`, `gband.window.set_position`, `gband.action.toggle_full_width` and `gband.action.toggle_column_tabbed`.
- A new saved file under `user/` holding the recorded layout, read and written through the module's existing `read` and `write` helpers.
- `openspec/specs/settings/spec.md`: the key-style switch requirement.
- `docs/internals/07-settings.md`: the chapter that reads the module in full.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- crates/lua/src/runtime/gband/settings.lua
- tests/lua/settings_spec.lua
- docs/internals/07-settings.md
- openspec/changes/leave-floating-restores-layout/
