## Context

See proposal.md for the motivation. This change starts after the five prerequisites are archived on `dev`. The state it builds on:

- `crates/lua/src/runtime/gband/keystyle.lua` holds `use` and `saved`, and accepts only `"modal"` and `"direct"`. The presets `keystyle/modal.lua` and `keystyle/direct.lua` are files of top-level calls. They set up `gband.keylist` and `gband.prompt`, bind in `prefix`, and bind five `mod+` mouse names in `root`.
- `crates/lua/src/bundled.rs` lists the presets in `KEY_STYLES` and every bundled module in `MODULES`. `crates/lua/src/directory.rs` writes `KEY_STYLES` to `defaults/keystyle/` and every other module to `defaults/lua/gband/`. `crates/lua/src/lib.rs` re-exports `KEY_STYLES` with its length in the type.
- The client enters `prefix` on the prefix key only while `prefix` holds a binding, and emits `KeyTableChanged` for each change of the active table. A `KeyTableChanged` handler is a callback, so it may call `gband.keymap.enter`, open plugin windows and dispatch actions. Binding `prefix` in `root` is a configuration error.
- `gband.keylist` already runs `prefix` bindings from inside a plugin window with `gband.keymap.run("prefix", key)`, closing itself when another floating plugin window takes focus.
- `gband.sidebar` draws row 0 from `gband.keymap.label` and handles its clicks in a `MousePressed` handler, since a bar cell reaches handlers with the target `outside`.
- `gband.settings` toggles the `keys` line between two styles, and shows the `I on new` line only for `modal`.
- From the prerequisites:
  - A mouse binding function that returns `false` declines the event. The event then takes its unbound default, starts no gesture, and the function's dispatches stand. The mouse payload and `on_mouse` carry `box_col`, `box_row`, `box_width` and `box_height`, also on `MouseReleased`.
  - `drag_resize_window({ edges = { ... } })` moves exactly the named edges.
  - `gband.core.provide("decorations", fn)` draws spans right-aligned on a top border, ending at the box's third-last column. Spans carry resolved styles. The client calls `fn` at most once per window per frame, and again only after other Lua ran or the info changed.
  - `gband.window.minimize(window)` and `minimize_window` hide a floating window in this client; focusing it restores it; `gband.layout()` marks it `minimized = true`.
  - `toggle_window_floating({ window = id, floating = true })` floats a tiled window and changes nothing for a floating one. The server applies it, so requests from several clients float a window once. `floating = false` tiles in the same way.
  - `LayoutChanged` also runs when the screen area's size changes, whichever client changed it, this client's own resize included once the server's layout arrives.
  - `gband.api_version` is 2.

## Goals / Non-Goals

**Goals:**
- A desktop-like key style built only from documented Lua API, readable and copyable as the other presets are.
- Content clicks never change meaning: every press the style does not use takes the interactive-mode defaults.
- One window list, whatever opens it.

**Non-Goals:**
- Any Rust, protocol or harness change beyond packaging the two new Lua files and the client fix that "Tiled plugin windows beside a floating focus" describes.
- Keyboard moving and resizing of floating windows, a double click on a title bar, and edge snapping by dragging.
- Changing what the modal and direct styles bind.

## Decisions

### A preset plus a bundled plugin
`gband.keystyle.floating` stays a file of top-level calls, as the key-style capability requires of a preset: it sets up `gband.keylist`, `gband.prompt` and `gband.desktop`, binds keys and mouse names, and registers the leader's `KeyTableChanged` handler. Everything with state lives in the new plugin `gband.desktop`: the float rules, the decorations function, the remembered boxes, the window list and the window menu, and the four actions `desktop.list`, `desktop.leader`, `desktop.press` and `desktop.menu`.

- *Alternative*: put everything in the preset. Rejected. The preset would hold several hundred lines of state and drawing, and would stop reading as a list of bindings. The sidebar could also not open the list without reaching into the preset.
- *Alternative*: let `gband.desktop` take options, such as `{ leader = true }`. Rejected. The key-style capability sets plugins up with no options, and what the leader does is a binding decision that belongs to the preset.

The leader handler stays in the preset, because the plugin may be set up under another style: a modal user can bind `prefix w` to `desktop.list` without losing navigation mode. Setting the plugin up does float windows, which is what the plugin is for, and the docs say so.

The mouse functions are registered actions bound to `leftmouse` and `rightmouse`, so `gband.keymap.list("root")` names them and a user can rebind them. A registered action bound to a mouse name receives the payload and can decline, as mouse-binding-fallthrough defines.

### Packaging is not API
Besides one client fix, described below, the only Rust edits add the two files to `MODULES` and the preset to `KEY_STYLES` in `crates/lua/src/bundled.rs`, change the length in the type of `KEY_STYLES` in `crates/lua/src/lib.rs`, and update the `first_run` unit test in `crates/lua/src/directory.rs`, which lists `defaults/keystyle/`. `directory.rs` already iterates `KEY_STYLES` and the bundled files, so it writes `defaults/keystyle/floating.lua` and `defaults/lua/gband/desktop.lua` with no new code. The lua-api capability's "Bundled sources on disk" already excludes every preset and includes every module, so it needs no delta.

### Tiled plugin windows beside a floating focus
With every window floating, the focused window is nearly always floating. `gband.win.open({ kind = "tiled" })` without `band` or `after` then opened nothing: the client named the focused window as `after`, and `after` must be tiled, so the server dropped the request. The plugin-windows capability already says the default resolves "as open window resolves against the view", which floating-target defines as the tiled window this client focused most recently in the viewed band. The client now resolves the default through the same path as `open_window`, in `crates/client/src/lib.rs`. No spec changes, and `tests/lua/floating_target_spec.lua` gains a case with the modal style.

### The API version stays 2
The change adds a style name to `use` and `saved`, the function `gband.keystyle.current`, a plugin, its actions and groups. `use("floating")` was an error, and the API version rule treats an addition that makes a former error valid as adding. `saved()` returning `"floating"` for a file that holds it is the same kind of widening: no file that returned `"modal"` or `"direct"` returns something else. The settings window's keys are interface, not API. So `gband.api_version` stays at the value mouse-binding-fallthrough set.

### The leader: `KeyTableChanged`, then back to `root`
The floating preset declares no mode and binds seven keys in `prefix`, so the prefix key enters `prefix` and emits `KeyTableChanged`. The preset's handler sees `table == "prefix"`, calls `gband.keymap.enter("root")` and dispatches `desktop.leader`. Every call is allowed in an event handler: `gband.keymap.enter` and action values in any callback, and `gband.win.open` wherever an action value is. The handler runs before the client reads the next key, so the next key reaches the window list.

`desktop.leader` opens the list, or, when the list is open and focused, closes it and runs `gband.keymap.run("prefix", "prefix")`. The preset binds `prefix prefix` to `send_prefix`, so the prefix key twice sends it to the focused window, as in the modal and direct styles. Without this, a program could never receive Ctrl+Space.

- *Alternative*: make `prefix` a mode whose bindings drive the list. Rejected. While a mode is active, presses go to the mode's bindings and `on_mouse` gets no press, so the list's clicks, the press outside it and the title bars would all need mode bindings, and the window menu would need a second mode.
- *Alternative*: bind the prefix key in `root`. Rejected. The configuration capability makes it an error.
- *Alternative*: set the `prefix` option to another key and bind Ctrl+Space in `root`. Rejected. A preset sets no option.

**Decision to revisit: shortcuts inside the list.** The user's item list has no detach, Lua prompt or key list. The list keeps exactly `New window`, `Settings` and the windows, and runs any `prefix` binding by its key, as the key list does: `D` detaches, `:` opens the Lua prompt, `?` opens the key list, `N` renames the focused window, and `n` and `s` repeat the first two entries. The bottom border shows `? keys` while `prefix` binds `?`, so the key list, which lists every shortcut with its description, is one key away. A user's own `prefix` binding becomes a list shortcut with no extra code. The list's own keys, `j`, `k`, the arrows, PageUp, PageDown, Home, End, Enter, Escape and `q`, win over a `prefix` binding of the same key. `prefix` binds none of them in the preset.

### Which windows float, and when
`toggle_window_floating({ window = id, floating = true })` is idempotent and ordered by the server, so every client that uses the style can float every window it sees appear, with no owner to pick:

- `WindowOpened` for a window that the client's layout holds tiled and does not mark with `plugin_window`, while no tiled plugin window of this client awaits its window. This covers `gband.spawn`, a binding that calls `open_window` without `floating`, a plugin's `open_window`, and a window that a client with another key style opens.
- `Attached`: every tiled window that runs a program, in every band. This is how a new session's first window floats, and how windows from before the attach float.
- `ConfigReloaded`: the same sweep, since the style may have just been chosen in the settings window. Every client of the configuration directory reloads together, and their requests now agree.
- The style's own new windows open with `open_window({ floating = true })` and need no request.

A window runs a program when its `gband.layout()` table holds `name`. A tiled plugin window's drawn window has none, so the sweeps leave it tiled: its plugin chose a tile, and a floating plugin window is the plugin's way to float.

`WindowOpened` cannot read `name`: the server sends a window's name in its own message after the layout that adds the window, and the client emits `WindowOpened` while it applies that layout. The rule instead leaves out what this client knows to be its own tiled plugin window: a window marked `plugin_window`, and any window that opens while one of its tiled plugin windows has no window yet, since the client may learn the mapping after the layout. The drawn window of another client's tiled plugin window looks like a spawned window at that moment, so it floats. It gets no buttons and `desktop.press` declines its border, because it has no `name`.

- *Alternative*: send a window's name before the layout that adds it, and keep it in the client until the window arrives. Rejected for this change: it reorders server messages and changes the client's name handling, for a case that needs two clients and a third-party tiled plugin window.

No rule floats a window when it is focused or on a later layout. With `WindowOpened` in every client, a tiled window that runs a program appears only when someone tiles a floating one, with a binding of their own or from a client with another style. That is a deliberate choice, and floating the window again when it is focused, or on any later layout, would fight it. Such a window floats again at the next attach or reload of a floating-style client, as the sweep defines.

**Shared layout.** The layout is shared, so every float the style sends changes every client's screen, whatever its key style. While a floating-style client is attached, a modal or direct user in the same session sees each new window float. Maximize, restore and tile also change shared boxes. Minimize does not: it is per client.

**Cascade.** Every first float and every `open_window({ floating = true })` gives the same box, centred at the default width, so without help each new window would cover the last one exactly, and the user could not tell that there are two. The plugin therefore places each window it saw open, or that its sweep named, once, in the first layout where it floats: when another floating window of its band has the same top-left cell, it moves the window two columns right and one row down, again until the cell is free, as long as the box still fits in the screen area. Otherwise the window keeps its box. Two columns for one row keeps the offset about square on a terminal's tall cells, and the title of the window below stays readable. The rule reads only the shared layout and sets absolute positions, so two floating-style clients that place the same window send the same `set_position`, and a client that sees the window already moved sends nothing. A window the user moves later is never placed again. A screen area that is barely taller than the box allows few steps, and the next window then keeps the centred box; that is accepted rather than wrapping the cascade back to the top-left corner, which would put new windows where the user does not look.

**Decision to revisit: existing tiled windows.** At attach and reload the style floats all of them, in every band, and cascades the ones that land on the same cell.

### Left press: decline first, then the four rules
`desktop.press` declines, by returning `false`, every press that is not on the border of a floating window that runs a program. Content presses, tiles, plugin windows, empty ribbon and bars keep their defaults. For a border press it uses `box_col`, `box_row`, `box_width` and `box_height` in this order:

1. A button: decorations end at column `w − 3`, each span is 3 cells, so the buttons are fixed for a given `w`. The plugin computes them with the same function that the decorations provider uses, so drawing and hit-testing cannot disagree.
2. A corner zone, the corner cell and the cell beside it along each side, as the user asked. A box of width or height 3 makes zones overlap, so they are checked top-left, top-right, bottom-left, bottom-right.
3. The rest of the top border moves the window with `drag_window`.
4. The rest of a side resizes that one edge with `drag_resize_window({ edges = { edge } })`.

The top edge resizes only from a corner, because the top border is the title bar.

**Buttons act on the release.** The press is taken and starts no gesture, so focus does not change. The plugin remembers the window and the button, and its `MouseReleased` handler acts only when the release lies on the same button of the same window. A release elsewhere does nothing, as on a desktop, so a mis-aimed press on `[X]` can be cancelled by moving away. The `MouseReleased` payload carries the box fields because mouse events and bindings share one payload.

**The `mod+` bindings stay.** The floating preset binds the five `mod+` mouse names as the other presets do, so Alt and a drag move or resize any window from its content, and Alt and the wheel switch bands. `leftmouse` and `rightmouse` never match a press with a modifier, and `mouse_mod` cannot be empty, so the two sets never compete.

### Right press: no focus change
`desktop.menu` takes a border press on a floating window that runs a program, and a press on empty ribbon. A press on a bar, the sidebar included, has the target `outside` and is declined: the sidebar has its own apps character, and a right press on a bar belongs to that bar. A press on a tiled window is declined, since it is not part of the desktop.

The menu opens without focusing the window. Focusing it would be a dispatched view action, applied after the callback, and a change of the focused window leaves no floating plugin window focused, so the new menu would lose focus at once.

The pointer's cell in the ribbon area is the press's column less the columns of the shown left bars, from `gband.bar.list()`. Bars are only on the sides, so the row needs no change. The plugin window's own placement then cuts the box to the ribbon area.

### Maximize, restore and tile from the box itself
The style sets boxes with `gband.window.set_width`, `set_height({ rows })` and `set_position`, in that order, so that a position is never clamped by the old size. Restore sets the width and height first for the same reason.

Whether a window is maximized, tiled left or tiled right is read from its `gband.layout()` table, not kept in Lua. So the `[❐]` glyph and the `Restore` entry survive a reload, and stay right when another client moves the window. Only the box to restore is kept, per window and per client, because it cannot be read from anywhere. A reload loses it, and Restore then gives a centred box of half the screen area's width and height.

A maximized box uses width 1, not full width, so a remembered box with full width on can be restored with `toggle_full_width` after `set_width`.

The plugin keeps the screen area's size of the last layout it saw. When `LayoutChanged` reports a layout of another size, whether this client, another client's resize or another client's attach changed it, the plugin gives each window that matched a state against the previous size the box of that state for the new size. Every floating-style client computes the same boxes from the same layout, so their requests agree. `TerminalResized` is not used: for this client's own resize, the new size reaches `gband.layout()` only with the server's layout, which `LayoutChanged` reports. A smaller area needs no request for a maximized window, since box geometry clamps it, but the plugin sends one anyway so the record matches the area; the rule stays the same for every state.

### Glyphs
- `[□]` keeps the user's `□`, U+25A1. It is East Asian ambiguous width, but so is every box-drawing character gband already draws borders with. A terminal that draws ambiguous characters two cells wide breaks every border first, so `□` adds no new requirement. `gband.ui.width` measures it as one cell, and so does the decorations placement. `▢`, U+25A2, is a neutral-width alternative that looks close, and the docs mention it for a user who wants to change it.
- `[❐]`, U+2750, marks a maximized window. It has neutral width and shows two stacked windows, the usual restore sign. Changing the middle button's glyph tells the user which action the button will take.
- `⊞`, U+229E, is the sidebar's apps character. It has neutral width and resembles the start button of a desktop. `≡` is ambiguous width, and `☰` is wide, which a one-column bar cannot hold.
- `[_]` and `[X]` are ASCII.

### Desktop menus draw their own frame
A floating plugin window can show a title on its border but nothing at the border's right end, and floating plugin windows get no decorations. So the window list and the window menu are borderless floating plugin windows that draw their frame in their lines, in the plain set that `border = true` uses, with `PluginWindowBorder` and `PluginWindowTitle`. The list's `[□][X]` sit where window decorations sit, ending at the third-last column, so the same placement rule and hit-test serve both. Without a border, `box_col` and `content_col` agree, and `on_mouse` reports every frame cell with its `line`.

The menus keep their own selection and scroll offset instead of `cursorline`. With `cursorline`, the cursor could land on a frame line, and scrolling would scroll the frame away. The selected row is drawn with `PluginWindowCursorLine` between the side borders, so it looks like every other bundled list.

- *Alternative*: a bordered list with the buttons on its first content line. Rejected. The user asked for the buttons on the frame.
- *Alternative*: a one-row plugin window laid over the list's top border. Rejected. Focusing the list raises it above the overlay.

Both menus share one implementation: frame, selection, keys, wheel, click on press and pick on release, and dismissal. The window list adds buttons, a hint and the `prefix` shortcuts. The window menu adds nothing. At most one is open, so a menu never covers another.

A press outside closes a menu. The plugin's `MousePressed` handler sees every press in every key table, after the press is handled. The function that opened a menu during that same press marks it, and the handler skips the marked press once. The press itself still takes its effect, binding or default, so a click on a window both dismisses the list and focuses the window.

### The window list's content
- Every band is listed, in layout order, because focusing a window views its band. Shown names repeat across bands, since ` #n` only counts within a band, so each entry starts with the band's sidebar label once windows sit in more than one band.
- Minimized windows end with ` (minimized)` and use `DesktopMinimized`, so they stay readable without colour.
- Only windows that run a program are listed. Drawn windows have no name, and their plugins give their own way to reach them.
- The list is built when it opens and again on `WindowOpened`, `WindowClosed`, `LayoutChanged`, `FocusChanged` and the plugin's own minimize. A name change alone emits no event, so the list shows it at the next rebuild.
- Width: at least 24 cells, so `windows` and its buttons fit with room. Natural height: at most 15 rows, as the key list. The `[□]` button gives the full ribbon height for a long list. Selection starts on `New window`.

### The sidebar asks `gband.keystyle.current()`
The sidebar must know the style without knowing the preset. `gband.keystyle.current()` returns what `use` picked in this load. It is nil when a configuration requires a preset by module name, and the sidebar then shows the mode letter, as before. With the floating style, row 0 always shows `⊞`, because `prefix` is active only for an instant and `root` would always show `I`. The click dispatches `desktop.list` only when `gband.action` holds it, so a failed desktop plugin costs the sidebar nothing.

**Decision to revisit:** a user mode entered under the floating style no longer shows its letter on row 0.

### Settings
The `keys` line cycles `modal`, `direct`, `floating`: Enter, `l` and Right forward, `h` and Left back, wrapping at both ends. Enter goes forward, as `l` does, because it was "the other style" with two. The `I on new` line stays modal only.

### Spec deltas
- New capability `floating-key-style` for everything the style does.
- `key-style`: "Key style presets", "Use a key style", "Saved key style" and "Preset parity" are modified, and "Floating preset" is added. The capability's Purpose sentence says "two key styles, modal and direct". A delta cannot change it, so the integrator edits it to "three key styles, modal, direct and floating" when this change is archived.
- `client-attach`: "Key bindings" and "Default mouse bindings".
- `configuration`: "Configuration directory" and "Defaults use the public API".
- `settings`: "Settings window" and "Settings window keys".
- `sidebar`: "Sidebar rows", "Mode letter" and "Band click" are modified, and "Apps character" is added. Its Purpose says the sidebar shows "the active mode", which stays true for modal and direct. The integrator adds "or the apps character" at archive time.
- `lua-api` "Bundled Lua consumes the API", `plugins` "Module lookup", `lua-prompt` "Default setup" and `tutorials` "Internals tutorial".

No delta modifies a requirement that a prerequisite modifies. In particular, mouse "Mouse names in key tables", "Interactive mode defaults", "Drag gestures", "Pointer targets" and "Resize by dragging", lua-control "Action targets" and "Read the layout", lua-api "Providers" and "API version rule", and configuration "Binding functions" stay as the prerequisites leave them. So do floating-target's floating-windows "Float a window" and "Tile a window", lua-control "Action targets", lua-events "Built-in events", server-runtime "Server session actions", session-server "Session actions" and wire-protocol "Handshake" and "Client messages".

### Documentation
- `README.md`: the third key style, its leader, the mouse table, the settings `keys` line, the saved file's values, `defaults/keystyle/floating.lua`, and the bundled plugin list.
- `docs/plugins.md`: "Mouse names" (which styles bind what), "Key styles" (`floating`, `current`), a new section "The desktop: `gband.desktop`" with its actions, menus and groups, "The sidebar" (the apps character), and "Settings" (the cycle).
- `docs/internals/08-key-styles.md` grows sections for the floating preset and `gband.desktop`, which quote every line of both files. Keeping them in chapter 08 avoids renumbering chapters 09 to 12 and their example directories.
- `docs/internals/00-boundary.md`, `07-settings.md`, `09-sidebar-and-errors.md`, `10-keylist-and-prompt.md`, `11-default-configs.md` and `README.md` follow the files they quote or describe.
- `docs/tutorial/00-setup.md`, `01-keys.md` and `03-options.md` mention the third style in text only, so no chapter directory changes.

### Tests
- `g.start`'s `keystyle` option accepts `"modal"`, `"direct"` and `false`. Lua cases write `user/keystyle.lua` through `files`, which the testing guide says replaces the saved file, so the harness needs no change. Accepting `"floating"` there would be a convenience for test authors, left for later. Cases that want no sidebar pass `config = 'gband.keystyle.use("floating")'`.
- `tests/lua/floating_spec.lua` covers every visual behaviour with screenshots: the title bar, the restore glyph, the window list, the maximized list, the window menu, the minimized entry, band labels and the sidebar's apps character.
- Rust tests cover the API and the tables: `crates/lua/tests/keystyle.rs`, `settings.rs`, `sidebar.rs`, `config.rs`, and a key table test in `crates/client/src/bindings.rs`.

## Risks / Trade-offs

- [A floating-style client in a session floats the windows of a modal or direct user too, as they open and at each of its attaches and reloads] → Stated in the docs. A window that user tiles afterwards stays tiled until the floating-style client attaches or reloads again.
- [Another client's tiled plugin window floats when it opens, because `WindowOpened` cannot tell it from a spawned window] → It shows no buttons and its border presses keep their defaults. Its user can tile it again, and it stays tiled until the next attach or reload of a floating-style client, whose sweep leaves it alone since it has no `name`.
- [Several floating-style clients each send a float, a refit or a cascade placement for the same window] → The server floats a window once, and the refit and the cascade compute the same absolute boxes from the same layout, so the requests agree. A client whose layout is older sends a request that the later layouts already satisfy, which changes nothing.
- [The `KeyTableChanged` handler depends on the event order: `prefix` must be reported before the next key is read] → The client emits the event while it handles the prefix key. The "Leader opens the list" and "Table changes of the leader" scenarios test it end to end.
- [The cascade runs out of room in a short screen area, and the next window keeps the centred box] → The window list reaches every window, and the cascade starts again from the free cells as windows close or move.
- [A cascaded window jumps one frame after it appears] → The placement is one `set_position` in the first layout where the window floats, so the jump is a single frame.
- [The decorations function reads `gband.layout()` once per call] → The client calls it at most once per window per frame, and only after Lua ran or the box width changed. The function builds no table beyond the three spans.
- [A reload forgets the box to restore] → Restore then gives a centred half-size box, and the `[❐]` glyph stays right because it is read from the box.
- [A `prefix` binding of `j`, `k`, `q` or another menu key cannot run from the list] → The key list still runs it, and the docs name the menu keys.
- [`⊞` and `❐` are missing from some fonts] → Both are in common monospace fonts, such as DejaVu Sans Mono. A user can copy `defaults/lua/gband/desktop.lua` or the sidebar under `user/lua/gband/` and change the glyph.

## Migration Plan

No migration. Nothing changes until a user saves the floating style in the settings window, calls `gband.keystyle.use("floating")`, or sets up `gband.desktop`. A rollback is a revert of the change. A `user/keystyle.lua` holding `return "floating"` then reads as nil, so the modal style applies. Windows the style floated stay floating, as any floated window does.
