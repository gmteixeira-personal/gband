## Why

A column that stacks several windows splits its height between them, so each window gets a few rows. niri solves this with tabbed columns: every window of the column takes the whole column, one is shown at a time, and an indicator on the column's left edge shows the tabs. gband follows niri where no spec says otherwise, and has no tabbed display today.

## What Changes

- Add a **tabbed** flag to every column, off for a new column. A tabbed column shows one window at a time, its **shown tab**, and every window of the column gets the same tile: the whole column width and the column's tab height, from the top row.
- Add the session action `toggle_column_tabbed`, "toggle tabbed display of the window's column", which flips the flag of the named window's column. It changes nothing for a floating window. Turning tabs off brings back the stacked heights the windows had.
- Add a **tab height** to every column, shared by all its tabs. Growing, shrinking, resetting or setting the height of any window in a tabbed column changes the tab height instead of the window's own height, so a resize applies to the whole column. Automatic, the default, fills the column.
- The shown tab is per client: the focused window when it is in the column, otherwise the window of the column this client focused most recently, otherwise the top window. `focus_window_down` and `focus_window_up` move between tabs, as they move within a stack today. Other tabs are not shown windows.
- Draw a **tab indicator** on the tile's left border column, as niri draws it on the left side. The border column takes no extra width, so terminal sizes do not change. The column is split into one segment per tab, top to bottom, with a blank row between segments when they fit. The shown tab's segment is drawn with `█`, the others with `░`. Two new highlight groups, `TabIndicatorActive` and `TabIndicator`, link to `WindowBorderFocused` and `WindowBorder` by default.
- A press on a tab's segment targets that tab's window, so a left click in interactive mode focuses it and makes it the shown tab, as in niri.
- Modal and direct key styles: `w` after the prefix key toggles tabbed display of the focused window's column. `w` is unbound today.
- `gband.layout()` and the server's `gband.session()` give each column `tabbed`, and `gband.layout()` gives `tab_rows` when the tab height is fixed. `LayoutChanged` runs when a column's tabbed flag or tab height changes.

## Capabilities

### New Capabilities
- `tabbed-columns`: the tabbed flag and its toggle, the tab height, the shown tab of each client, the tab indicator and its highlight groups, and the indicator as a pointer target.

### Modified Capabilities
- `layout`: "Tile geometry" places the tiles of a tabbed column on top of one another; "Window heights" and "Set a window's height" send a tabbed column's height changes to its tab height; "Move a column" keeps the tabbed flag and the tab height.
- `layout-view`: "Shown windows" leaves out the tabs that are not this client's shown tab.
- `actions`: "Lua names of actions" lists `toggle_column_tabbed`.
- `client-attach`: "Key bindings" gains the `w` row in the modal and direct tables.
- `mouse`: "Pointer targets" resolves a tab indicator cell to its tab's window, and "Move a tiled window by dragging" drops above or below a tabbed column's shown tab.
- `lua-control`: "Read the layout" gives each column `tabbed` and `tab_rows`; "Action targets" lets `toggle_column_tabbed` take a `window`.
- `server-runtime`: "Reading structure" gives each column `tabbed`.
- `wire-protocol`: "Server messages" lists each column's tabbed flag and tab height in the layout message.
- `session-server`: "Window resizes" resizes every tab of a tabbed column one of whose tabs is shown, so hidden tabs keep the shown tab's size.
- `lua-events`: "Built-in events" runs `LayoutChanged` when a tabbed flag or tab height changes.
- `highlights`: "Client groups" lists `TabIndicatorActive` and `TabIndicator`.

## Impact

- Core: `Column` gains `tabbed` and `tab_height` in `crates/core/src/layout.rs`, with the toggle, the height routing, a new layout event and the move rules; `crates/core/src/geometry.rs` gives tabbed tiles; `crates/core/src/view.rs` computes the shown tab and filters shown windows; `crates/core/src/action.rs` and `crates/core/src/event.rs` gain the new action, command and event.
- Wire format: the layout message carries the two new column fields. The version handshake already refuses a server from another build, so no compatibility shim is needed.
- Client: `crates/client/src/render.rs` draws only the shown tab and the indicator; `crates/client/src/mouse.rs` resolves segment cells and the drop place; `crates/client/src/lib.rs` passes the new fields to Lua.
- Lua: the `toggle_column_tabbed` built-in in `crates/lua/src/actions.rs`, `tabbed` and `tab_rows` in `crates/lua/src/control.rs` and `crates/lua/src/server.rs`, the two groups in `crates/lua/src/runtime/gband/hl.lua` and the drawn styles in `crates/lua/src/ui.rs`, and `w` in the modal and direct presets.
- Server: PTY sizes follow the new tile geometry, and `crates/server/src/session.rs` resizes every tab of a tabbed column with a shown tab, so every tab of a column has the shown tab's size and switching tabs resizes nothing.
- Docs: README key and action tables, `docs/plugins.md`, and the quoted presets in `docs/internals/08-key-styles.md`.
- Tests: core layout, geometry and view tests, client render and mouse tests, Lua action and keystyle tests, a new screen spec for the indicator, and the key list screenshots that list the `prefix` bindings.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- openspec/changes/tabbed-columns/
- README.md
- crates/client/src/lib.rs
- crates/client/src/mouse.rs
- crates/client/src/render.rs
- crates/client/tests/actions.rs
- crates/client/tests/animation.rs
- crates/client/tests/events.rs
- crates/client/tests/look.rs
- crates/client/tests/render.rs
- crates/core/src/action.rs
- crates/core/src/event.rs
- crates/core/src/geometry.rs
- crates/core/src/layout.rs
- crates/core/src/view.rs
- crates/core/tests/events.rs
- crates/core/tests/geometry.rs
- crates/core/tests/layout.rs
- crates/core/tests/view.rs
- crates/lua/src/actions.rs
- crates/lua/src/control.rs
- crates/lua/src/runtime/gband/hl.lua
- crates/lua/src/runtime/gband/keystyle/direct.lua
- crates/lua/src/runtime/gband/keystyle/modal.lua
- crates/lua/src/server.rs
- crates/lua/src/ui.rs
- crates/lua/tests/config.rs
- crates/lua/tests/control.rs
- crates/lua/tests/highlights.rs
- crates/lua/tests/keystyle.rs
- crates/lua/tests/server.rs
- crates/protocol/tests/messages.rs
- crates/server/tests/events.rs
- docs/internals/02-highlights.md
- docs/internals/08-key-styles.md
- docs/plugins.md
- tests/keystyle.rs
- tests/lua/screenshots/keylist_spec/
- tests/lua/screenshots/tabbed_spec/
- tests/lua/tabbed_spec.lua
- tests/lua_specs.rs
- tests/tabbed.rs
