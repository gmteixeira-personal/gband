## Why

gband has two key styles, modal and direct, and both assume a user who drives a scrolling layout from the keyboard. Users who come from a traditional desktop such as MS Windows expect something else: every window floats, the mouse moves and resizes it by its border, each window has minimize, maximize and close buttons, and one menu lists every open window. The five prerequisite changes give Lua all it needs for that. This change builds the third key style on them, in Lua only.

## What Changes

- **A third key style, `floating`.** A new preset, `gband.keystyle.floating`, and a new bundled plugin, `gband.desktop`, that the preset sets up after the key list and the Lua prompt. `gband.keystyle.use` and `gband.keystyle.saved` accept `"floating"`, and the new `gband.keystyle.current()` returns the style that `use` picked in this load, or nil.
- **Every window floats.** A new window from the style opens with `open_window({ floating = true })`. Every tiled window floats as soon as it appears, from `WindowOpened`, with `toggle_window_floating({ window = id, floating = true })`, wherever it came from: `gband.spawn`, a binding, another client. Only this client's own tiled plugin windows are left out, since a window's name arrives after `WindowOpened`, so another client's tiled plugin window floats too, without buttons. At attach and after each reload, the style floats every tiled window that runs a program the same way. The server floats each window once, so several floating-style clients agree. The drawn window of a tiled plugin window this client opened stays tiled, and a window that a user tiles later stays tiled. The layout is shared, so other clients see these windows float. Windows that float onto the same top-left cell cascade, two columns right and one row down each, while they fit.
- **Title bar buttons.** A decorations provider gives every floating window that runs a program `[_][□][X]` on its top border, with `[❐]` in place of `[□]` while the window is maximized. New highlight groups `DesktopButton`, `DesktopClose` and `DesktopMinimized` style them.
- **Left button on a border.** `root` binds `leftmouse` to `desktop.press`. On a floating window's border it acts on a button (on the release over the same button), resizes the two edges of a corner zone (the corner cell and the cell beside it along each side), moves the window from the rest of the top border, and resizes one edge from the rest of the left, right or bottom border. Every other press is declined, so content clicks keep the interactive-mode defaults: focus, forward to the program, select.
- **Right button.** `root` binds `rightmouse` to `desktop.menu`. On a floating window's border it opens the window menu at the pointer: Close, Maximize (Restore while maximized), Minimize, Tile left and Tile right. On empty ribbon it opens the window list at the pointer. Every other press is declined, so a right click in content still pastes.
- **Maximize, restore and tile.** Maximize gives a window the whole screen area, Tile left and Tile right its left or right half at full height. The style remembers the box before, per window in this client, and Restore puts it back. The boxes follow the screen area when `LayoutChanged` reports a new size, whichever client changed it.
- **The window list.** One list, opened by the leader, by a right click on empty ribbon, and by the sidebar's new apps character: `New window`, `Settings`, then every window by its shown name, minimized windows marked, with band labels when windows sit in more than one band. It is scrollable. j, k, Up, Down, PageUp, PageDown, Home and End move, Enter and a click pick, Escape, `q` and a press outside close. Its frame shows `[□][X]` only, and its `[□]` toggles its height between its own and the ribbon's. A key that `prefix` binds runs that binding from the list, so `D` detaches, `:` opens the Lua prompt and `?` opens the key list. The bottom border shows `? keys`.
- **The leader.** The preset binds a few keys in `prefix` (`n`, `?`, `:`, `N`, `s`, `D` and the prefix key) and registers a `KeyTableChanged` handler. When `prefix` becomes active, the handler returns to `root` and dispatches `desktop.leader`, which opens the window list. Pressing the prefix key while the list has focus closes it and sends the prefix key to the focused window, as the other styles' prefix key twice does.
- **Sidebar.** With the floating style, row 0 shows `⊞` in place of the mode letter, and a left press on it opens the window list. Band labels, the wheel and the error marker stay as they are.
- **Settings.** The `keys` line cycles modal, direct, floating: Enter, `l` and Right go forward, `h` and Left go back. The `I on new` line stays modal only.
- **Packaging.** The new preset and module join the bundled file list and `KEY_STYLES` in `crates/lua/src/bundled.rs`, and `KEY_STYLES` in `crates/lua/src/lib.rs` grows to three entries. gband writes the preset to `defaults/keystyle/floating.lua` and the module to `defaults/lua/gband/desktop.lua`.
- **Client fix.** `gband.win.open({ kind = "tiled" })` without `band` or `after` opened nothing while a floating window had focus, because the client named that floating window as `after`. The client now resolves the default as `open_window` does, which the plugin-windows capability already requires.
- **Settle fix.** A settle could complete while the client still held back the window move or resize of a pointer drag for its next frame, so a second drag within one frame of the first read the old box. The client now answers a settle only once that send has gone. No other Rust, protocol or harness change.
- `gband.api_version` does not rise: the change only adds a style name, a function, a plugin and its actions.

Out of scope:
- Keyboard moving and resizing of floating windows, a title bar double click, and snapping by dragging to a screen edge.
- A per-client or shared memory of maximized boxes across a reload. A reload keeps which windows are maximized, by their boxes, but forgets the box to restore.
- Extending `g.start`'s `keystyle` option to accept `"floating"`. Tests save the style through `files`.

## Capabilities

### New Capabilities
- `floating-key-style`: the `gband.desktop` plugin and the floating preset's bindings, the rules that float windows, the title bar buttons, the left and right border presses, maximize, restore and tile, the window list and window menu, the leader, and the desktop highlight groups.

### Modified Capabilities
- `key-style`: three presets, the floating preset's mouse bindings in `root`, `"floating"` in `use` and `saved`, `gband.keystyle.current()`, a new "Floating preset" requirement, and "Preset parity" limited to modal and direct.
- `client-attach`: "Key bindings" gives the floating style's bindings and leader, and "Default mouse bindings" gives its mouse bindings.
- `configuration`: "Configuration directory" writes `defaults/keystyle/floating.lua`, and "Defaults use the public API" covers three styles and the desktop plugin.
- `settings`: the `keys` line cycles three styles, and the window holds three lines with the floating style.
- `sidebar`: row 0 shows the apps character with the floating style, and a left press on it opens the window list.
- `lua-api`: "Bundled Lua consumes the API" lists `gband.desktop`.
- `plugins`: "Module lookup" lists `gband.keystyle.floating` and `gband.desktop`.
- `lua-prompt`: "Default setup" names the bindings of every key style.
- `tutorials`: "Internals tutorial" has chapter 08 teach the floating preset and `gband.desktop`.
- `test-channel`: "Settle" waits for the window move or resize that a pointer drag holds back.

## Impact

- New Lua: `crates/lua/src/runtime/gband/keystyle/floating.lua` and `crates/lua/src/runtime/gband/desktop.lua`.
- Changed Lua: `crates/lua/src/runtime/gband/keystyle.lua`, `sidebar.lua` and `settings.lua`.
- Packaging: `crates/lua/src/bundled.rs`, `crates/lua/src/lib.rs`, and the unit test in `crates/lua/src/directory.rs`.
- Client: the default target of a tiled plugin window in `crates/client/src/lib.rs`, with a regression case in `tests/lua/floating_target_spec.lua`.
- Test channel: the settle answer in `crates/client/src/lib.rs`, covered by the border drags of `tests/lua/floating_spec.lua`.
- Tests: `crates/lua/tests/keystyle.rs`, `settings.rs`, `sidebar.rs`, `config.rs` and a new `desktop.rs`, the key table tests in `crates/client/src/bindings.rs`, `tests/config.rs`, `tests/keystyle.rs`, `tests/settings.rs`, a new `tests/lua/floating_spec.lua` with its screenshots, `tests/lua/settings_spec.lua`, `tests/lua/sidebar_spec.lua`, `tests/lua/bundled_copies_spec.lua`, and `tests/lua_specs.rs`.
- Docs: `README.md`, `docs/plugins.md`, `docs/testing.md`, `docs/internals/` chapters 00, 07, 08, 09, 10, 11 and the index, and `docs/tutorial/00-setup.md`, `01-keys.md` and `03-options.md`.
- No protocol change, no new dependency, no change to the server.

## Coordination

### Author
- gmteixeira

### Depends On
- mouse-binding-fallthrough
- drag-resize-edges
- window-decorations
- minimize-windows
- floating-target

### Expected Files
- openspec/changes/floating-key-style/
- README.md
- crates/client/src/bindings.rs
- crates/lua/src/bundled.rs
- crates/lua/src/directory.rs
- crates/lua/src/lib.rs
- crates/lua/src/runtime/gband/desktop.lua
- crates/lua/src/runtime/gband/keystyle.lua
- crates/lua/src/runtime/gband/keystyle/floating.lua
- crates/lua/src/runtime/gband/settings.lua
- crates/lua/src/runtime/gband/sidebar.lua
- crates/lua/tests/config.rs
- crates/lua/tests/desktop.rs
- crates/lua/tests/keystyle.rs
- crates/lua/tests/settings.rs
- crates/lua/tests/sidebar.rs
- docs/internals/00-boundary.md
- docs/internals/07-settings.md
- docs/internals/08-key-styles.md
- docs/internals/09-sidebar-and-errors.md
- docs/internals/10-keylist-and-prompt.md
- docs/internals/11-default-configs.md
- docs/internals/README.md
- docs/plugins.md
- docs/testing.md
- docs/tutorial/00-setup.md
- docs/tutorial/01-keys.md
- docs/tutorial/03-options.md
- tests/config.rs
- tests/keystyle.rs
- tests/lua/bundled_copies_spec.lua
- tests/lua/floating_spec.lua
- tests/lua/screenshots/floating_spec/
- tests/lua/screenshots/settings_spec/the-settings-window-with-the-floating-key-style--floating.txt
- tests/lua/screenshots/sidebar_spec/the-floating-style-shows-the-apps-character--apps.txt
- tests/lua/settings_spec.lua
- tests/lua/sidebar_spec.lua
- tests/lua_specs.rs
- tests/settings.rs
