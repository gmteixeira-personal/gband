## Why

The floating key style's window list works like a plain menu: moving through it shows nothing, the windows sit in layout order, and every pick needs the arrow keys or a click. Users who come from MS Windows expect it to behave more like Alt+Tab: the most recent windows on top, each one shown as it is selected, a one-key shortcut per line, and Escape to go back. They also find the red close button loud, and the apps character `⊞` reads as a cross. The prerequisite window-peek gives Lua a focus that records nothing, the recency of each focus and pointer motion over a plugin window, which this change needs.

## What Changes

- **Close button.** `DesktopClose` defaults to `{ bold = true }`, as `DesktopButton` does, so `[X]` takes the border's colour. The group stays, so a theme can still colour it. No bundled colorscheme or theme sets it.
- **Apps character.** The sidebar's apps character becomes `∷`, U+2237, in place of `⊞`.
- **Most recent first.** The window list keeps `New window` and `Settings` first. The window entries follow: the windows this client focused, most recent first, by `last_focus`, then the windows it never focused, in layout order. window-peek records every focus, so the focused window comes first. The order is fixed while the list is open.
- **Shortcuts.** Each entry starts with a shortcut and a space: `n` for `New window`, `s` for `Settings`, `1` to `9` for the first nine windows and `0` for the tenth. Later windows have none. Pressing a shortcut acts as a click on its entry. The list's own keys, the shortcuts and the digits win over any `prefix` binding of the same key.
- **Band labels move to the end.** Once windows sit in two or more bands, each window entry ends with `band <label>`, so a band label never looks like a shortcut. The new group `DesktopShortcut`, which links to `KeyListKey`, draws the shortcuts.
- **Live preview.** Whenever the selection lands on a window entry, by a key, a press or the pointer, the list peeks that window: it views its band and draws it on top, a minimized one included, and records nothing. On `New window` or `Settings` it shows the state at opening. Moving the pointer over an entry selects it. The wheel scrolls the list and leaves the selection.
- **Keep or cancel.** Enter, a click and a shortcut keep the selected window, focused and recorded as `gband.window.focus` does. A `prefix` shortcut, the leader and a right press that opens another desktop menu also keep what the list shows. Escape, `q`, `[X]`, closing by other code and a press on empty ribbon or a bar cancel: the client views the band it viewed at opening again, as `gband.band.view` does, which ends a peek and returns focus to the window focused at opening. Peeks record nothing, so the band, the focused window, the stacking order of every band and the minimized windows are as they were. A press on a window focuses that window.
- The window menu does not change.
- **Hover only where it is used.** The window list opens with `hover = true`, so it receives pointer moves and stays focused through the previews its `on_mouse` causes. The window menu opens without it and keeps its behaviour.
- **Hold passes to an opened window.** A floating plugin window that keeps focus after its own `keys` or hover `on_mouse` passes that hold to a floating plugin window it opens or focuses, so the settings window that `s` opens after cancelling a preview stays focused when the cancel's band view arrives.
- This change does not raise `gband.api_version`, which stays 2: it adds no API, only a bundled plugin's behaviour and a group.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `floating-key-style`: "Title bar buttons" (the close button style), "Desktop menus" (`hover` on the window list only, entry cells, first shown row, wheel and closing for the window list), "Window list" (order, shortcuts, band labels, preview, keep and cancel, keys, pointer, wheel, rebuild), "Leader" (keeps what the list shows) and "Desktop groups" (`DesktopClose`, `DesktopShortcut`).
- `sidebar`: "Apps character" shows `∷`, U+2237.
- `plugin-windows`: "Focused plugin window" passes the focus that a floating plugin window holds after its own `keys` or hover `on_mouse` to a floating plugin window it opens or focuses, so the window list's `s` leaves the settings window focused after cancelling a preview.

## Impact

- Lua: `crates/lua/src/runtime/gband/desktop.lua`, `crates/lua/src/runtime/gband/sidebar.lua` and `crates/lua/src/runtime/gband/win.lua`, which passes a held focus to a floating plugin window opened or focused while it lasts. No colorscheme or theme changes: none sets `DesktopClose`.
- Rust tests: `crates/lua/tests/desktop.rs`, `crates/lua/tests/sidebar.rs`, `crates/lua/tests/plugin_windows.rs` and `tests/keystyle.rs`.
- Lua tests: `tests/lua/floating_spec.lua`, `tests/lua/sidebar_spec.lua`, and the screenshots under `tests/lua/screenshots/floating_spec/`, `tests/lua/screenshots/sidebar_spec/` and `tests/lua/screenshots/settings_spec/` that show `⊞` or a red `[X]`.
- Docs: `README.md`, `docs/plugins.md`, `docs/internals/05-plugin-windows.md`, `docs/internals/08-key-styles.md`, `docs/internals/09-sidebar-and-errors.md`, `docs/tutorial/00-setup.md` and `docs/tutorial/01-keys.md`.
- Depends on window-peek for `gband.window.focus(window, { peek = true })`, `last_focus` in `gband.layout()`, `peek` in `gband.view()`, the end of a peek when the client views the band it already views, a focus of the peeked window that emits no `FocusChanged`, and the `hover` option of `gband.win.open`, which brings the `on_mouse` kind `"move"` and keeps the window focused after its `on_mouse`.
- No Rust code, protocol, server or dependency change.

## Coordination

### Author
- gmteixeira

### Depends On
- window-peek

### Expected Files
- openspec/changes/desktop-list-tweaks/
- README.md
- crates/lua/src/runtime/gband/desktop.lua
- crates/lua/src/runtime/gband/sidebar.lua
- crates/lua/src/runtime/gband/win.lua
- crates/lua/tests/desktop.rs
- crates/lua/tests/plugin_windows.rs
- crates/lua/tests/sidebar.rs
- docs/internals/05-plugin-windows.md
- docs/internals/08-key-styles.md
- docs/internals/09-sidebar-and-errors.md
- docs/plugins.md
- docs/tutorial/00-setup.md
- docs/tutorial/01-keys.md
- tests/keystyle.rs
- tests/lua/floating_spec.lua
- tests/lua/screenshots/floating_spec/
- tests/lua/screenshots/settings_spec/the-settings-window-with-the-floating-key-style--floating.txt
- tests/lua/screenshots/sidebar_spec/the-floating-style-shows-the-apps-character--apps.txt
- tests/lua/sidebar_spec.lua
