## Context

See proposal.md for the motivation. This change starts after window-peek is archived on `dev`. The state it builds on:

- `crates/lua/src/runtime/gband/desktop.lua` holds both desktop menus in one implementation. `open_menu` opens a borderless floating plugin window with `keys` and `on_mouse`, and `menu` holds the open menu's rows, selection, first shown row (`top`), buttons, hint and `press`. `draw` moves `top` to show the selection on every redraw. `list_rows` builds the window list in layout order, with band labels in front of names. `list_shortcuts` turns every `prefix` binding outside `MENU_KEYS` into a list key that closes the list and runs the binding. `rebuild` runs on `WindowOpened`, `WindowClosed`, `LayoutChanged`, `FocusChanged` and the plugin's own minimize. `dismiss`, a `MousePressed` handler, closes a menu on any press outside it, skipping the press that opened it.
- `gband.desktop` defines `DesktopClose` as `{ fg = 1, bold = true }`. No bundled colorscheme in `crates/lua/src/runtime/gband/colors/`, nor `theme.lua` or `theme/catppuccin.lua`, sets a `Desktop` group.
- `crates/lua/src/runtime/gband/sidebar.lua` draws `APPS = "⊞"` on row 0 with the floating style.
- From window-peek:
  - `gband.window.focus(window, { peek = true })` focuses a window and records nothing. While it lasts, the peeked window is the focused window, drawn above its band's floating windows, also when minimized, and a pointer target. `gband.view()` holds `peek = true`.
  - The peek ends with a recorded focus when the view focuses any window by other means, when another window is peeked, and when the viewed band changes. A focus of the peeked window itself records it, raises and restores it, and emits no `FocusChanged` and no `BandChanged`, since the focused window does not change.
  - The peek ends with no recorded focus when the client views the band it already views by name, when the peeked window is minimized or leaves the layout, and when a reload loads a new configuration. Focus then returns to the window this client last focused in the viewed band, or goes where "Switch band" sends it. A failed reload keeps the peek.
  - Every other focus records: focus actions, focus by number, viewing a band, presses and gestures, the attach, and focus that moves because a window left. `gband.layout()` gives each window `last_focus`, higher for a more recent focus, and none for a window never focused. The stacking order follows `last_focus`.
  - `gband.win.open` takes `hover`, `false` by default, fixed at opening: `gband.win.info` reports it and `gband.win.set_config` does not take it. Only a plugin window opened with `hover = true` gets `on_mouse` with the kind `"move"` when the pointer moves with no button held to another cell over it. A focused floating plugin window stays focused through a focus or band change after its `keys` functions run, and, when it was opened with `hover = true`, after its `on_mouse` runs for any kind, until the user presses another key.
  - `gband.api_version` stays 2, and `desktop.lua` keeps `api = 2`.
- Event order, from the lua-events and mouse capabilities: the client emits `MousePressed` after it has handled the press, so the press's binding or default has already taken effect. The actions that a handler dispatches run after that handler returns. The actions that a declining mouse binding dispatches take effect before the fallback. `FocusChanged` runs only when the focused window differs.
- From the plugin-windows capability: a change of the focused window or the viewed band takes focus from a floating plugin window, unless that plugin window's own `keys`, or the `on_mouse` of a hover window, caused it.

## Goals / Non-Goals

**Goals:**
- Escape restores exactly the view at opening: the viewed band, the focused window, the stacking order of every band and the minimized windows.
- Every way to move the selection previews the same way, and every way to close the list either keeps or cancels.
- Only Lua changes: `desktop.lua`, `sidebar.lua` and `win.lua`, with their tests and docs.

**Non-Goals:**
- Any change to the window menu, the title bar buttons' placement, or the floating preset's bindings.
- Restoring a camera that a peek of a tiled window moved, in a band other than the band at opening. window-peek leaves the camera out of scope.
- Hover or a scrolling wheel in the window menu.

## Decisions

### The close button takes the border's colour
`DesktopClose` defaults to `{ bold = true }`, the same as `DesktopButton`. The group stays, so a theme or a user can still colour `[X]`, as "Restyle the close button" shows. A search of `crates/lua/src/runtime/gband/colors/`, `theme.lua` and `theme/` finds no `Desktop` group, so only the default changes. The spec states that no bundled theme sets it, and a Lua case checks every theme that `gband.settings.themes()` lists, so a future theme cannot bring the red back unnoticed.

### The apps character is `∷`
The user chose `∷`, U+2237 PROPORTION. It is East Asian ambiguous width. A terminal set to draw ambiguous characters two cells wide draws it over column 1, the ribbon's first column, in the one-column sidebar. The floating-key-style design rejected `≡` for the same reason, and kept `⊞`, which has neutral width. The user's choice wins here, for these reasons:

- `gband.ui.width` measures `∷` as one cell, as it measures every ambiguous character, so the bar's layout does not change.
- Such a terminal already draws gband's box-drawing borders and `□` two cells wide, so it breaks the screen before it breaks the sidebar.
- `⸬`, U+2E2C SQUARED FOUR DOT PUNCTUATION, looks close and has neutral width, but fewer fonts hold it. The docs name it for a user who copies the sidebar to change the glyph.

### Order by `last_focus`, fixed at opening
The window entries follow `last_focus` from the highest down, then the windows this client never focused, in layout order. window-peek records the attach, every focus and every band view, so the focused window is first, with shortcut `1`, without a special rule. A rule that put the focused window first would only differ while a peek that other code started is running, and is left out.

The list keeps the window ids in this order when it opens, and every rebuild keeps them: a closed window drops out, and a window that opened since then follows the others, in layout order. Peeks change no `last_focus`, so the order would not move anyway, but other code can focus a window while the list is open. A fixed order keeps the rows and their shortcuts still under the user's eyes.

- *Alternative*: keep layout order and add only the shortcuts. Rejected: the user asked for recent windows on top.
- *Alternative*: keep grouping by band, recent first in each band. Rejected: the most recent window of another band would not be near the top.

### A shortcut column, drawn in `DesktopShortcut`
Each entry starts with its shortcut and a space, so the keys form a column at the left edge. `New window` and `Settings` show `n` and `s`, which already ran those two from the list through `prefix`. The digits follow the sidebar's habit of `1` to `9` and add `0` for the tenth window, as a keyboard's number row runs. Windows past the tenth get a blank cell, which keeps the labels aligned.

The shortcut is drawn in a new group, `DesktopShortcut`, which defaults to `{ link = "KeyListKey" }`. A key the user can press then looks the same in the key list and in the window list, and every bundled theme colours it through `KeyListKey` with no new line in `theme.lua`. The selected row stays one bar of `PluginWindowCursorLine`, shortcut included, as every bundled list draws its cursor line.

- *Alternative*: use `KeyListKey` directly. Rejected: it ties a group name of another plugin into this one, and a user could not style the two lists apart.
- *Alternative*: a plain `{ bold = true }` group. Rejected: themes would leave it uncoloured unless `theme.lua` and the documented list of theme groups grew.

### The band moves to the end of the line, as `band <label>`
The sidebar's band labels are `1` to `9` and then letters, the same characters as the new shortcuts. In front of the name, `1 2 logs` would read as two shortcuts. So the band part moves to the end of the entry, right-aligned, and says `band 2`. It shows, as before, only while windows sit in two or more bands. The label is cut first, so the band part and the shortcut always stay readable.

- *Alternative*: the bare label at the end, `logs   2`. Rejected: it still reads as a key.
- *Alternative*: the band only as a dim style. Rejected: it is lost without colour and in screenshots without styles.
- *Cost*: `band 2` adds 7 cells. The list widens only when an entry no longer fits the 24-cell minimum.

### Preview: a peek, or a view of the band at opening
When the selection lands on a window entry, the list calls `gband.window.focus(id, { peek = true })`. When it lands on `New window` or `Settings`, the list views its band at opening with `gband.band.view`, also when the client views it already. window-peek makes that view end a peek in the band and return focus to the window this client last focused there, with no record. That window is the window focused at opening, because previews recorded nothing. From another band, the view is a band switch, which records a focus of that same window, already the most recent, so no order changes.

Cancel uses the same step, so one function serves the preview of the first two entries and the cancel. It also covers the windows that close meanwhile: when the window focused at opening has left, focus goes where window-peek and "Switch band" send it, which is where it would have gone without the list. When the band itself has left, the list views the band now at its position, or the last band, as "Follow layout changes" does for a removed viewed band.

- *Alternative*, from the brief: peek the window focused at opening, and view the band only when no window was focused. Rejected: it gives the same view in the common case, but needs a second path, and a peek of that window leaves a peek running, which a later keep has to end with a recorded focus.

Opening the list previews nothing: its first entry is `New window`, which shows the state at opening already. A preview that would show what the client already shows dispatches nothing, so opening and closing an untouched list never emits `FocusChanged`.

The list keeps focus through its previews because they run from its `keys` functions, or from its `on_mouse`, which keeps a floating plugin window focused only when it was opened with `hover = true`.

### Keep and cancel
Every way to close the window list either keeps what it shows or cancels:

| close | effect |
|---|---|
| Enter, a click, a shortcut on a window entry | keep: focus the peeked window as `gband.window.focus` does |
| Enter, a click, `n` or `s` on `New window` or `Settings` | cancel, then open the window or the settings |
| a `prefix` binding, the leader | keep, then run the binding |
| a right press that opens a desktop menu | keep |
| a press on a window | focus that window |
| any other press outside, Escape, `q`, `[X]`, closing by other code | cancel, unless the press itself changed the view |
| a successful reload | neither: window-peek ends the peek, with focus on the window last focused in the viewed band and nothing recorded |

Keep is a normal focus of the peeked window: it records the focus, which raises the window and restores a minimized one. window-peek's "Peek a window" states that this focus emits no `FocusChanged` and no `BandChanged`, because the focused window and the band do not change.

**`prefix` bindings keep.** A binding run with `gband.keymap.run` from the list reads `gband.view()` as of the call, and the view shows the preview, so `N` would rename the previewed window. A cancel dispatched before the binding would apply after that read, and the binding would act on one window while the view returned to another. Keeping makes what the binding reads and what stays agree. The leader keeps for the same reason: `send_prefix` goes to the previewed window.

**A right press that opens a menu keeps.** The new menu opens in the same callback, and a focus change in that callback would take focus from it, as the plugin-windows capability defines. A cancel changes the focused window. Keep does not: it is a focus of the window already focused, which emits no `FocusChanged` and no `BandChanged`, as window-peek's "Peek a window" states.

**Presses outside.** `MousePressed` runs after the press has taken its effect, so `dismiss` sees its result:

1. A press whose payload names a `window` focuses that window. The default of a press on a window already did, and then this changes nothing. A press on a title bar button, which `desktop.press` takes without focusing, then also focuses the window under it, so a release on the previewed window's `[X]` still finds it.
2. Otherwise, when `gband.view()` no longer shows the band and window that the list's last preview showed, something else changed the view, such as a sidebar band label whose handler ran first. The list leaves it.
3. Otherwise it cancels. Each handler's dispatches run after it returns, and the desktop's handler is registered before the sidebar's in the default configuration, so a click on a band label cancels and then views that band.

The list stores what its last preview showed for rule 2, as the band and the window, and sets it again after each preview.

**Closing by other code cancels.** The window's `on_close` runs only when other code closes it: the plugin clears `menu` before its own closes. A successful reload closes plugin windows without `on_close`, and window-peek ends the peek in the same step, so no previewed window outlives the list. A failed reload keeps the list open and the peek with it.

### The hold passes to a window the held window opens
`s` from a preview selects `Settings`, which dispatches the return to the band at opening, then picks it, which cancels and opens the settings window, all in one `keys` function. The view change takes effect after the function returns, and its `FocusChanged` or `BandChanged` would take focus from the settings window: the plugin-windows capability held only the window whose `keys` function ran, and that window, the list, has closed. Enter and a click do not meet this, because their selection previewed in an earlier key or press.

So `gband.win` passes the hold on: while a floating plugin window is held, a floating plugin window that becomes the focused one, by opening with `focus` or by `gband.win.focus`, while the held window is still focused or after it has closed, is held in its place until the user presses another key. The plugin-windows delta states it as a rule of "Focused plugin window".

- *Alternative*: open the settings window from a `FocusChanged` handler once the view has returned. Rejected: the plugin windows module unfocuses after every handler of that event has run, so the window would lose focus all the same.
- *Alternative*: let `s` keep instead of cancel while a window is previewed. Rejected: `Settings` would then act on what the list shows rather than the state at opening, unlike Enter and a click.

### Keys of the list
The list's own keys come first: the menu keys of "Desktop menus", then `n`, `s` and the ten digits. The digits are reserved even when no entry has one, so `5` with three windows does nothing rather than run a `prefix 5` binding: a key that changes meaning with the number of open windows would surprise. Every other `prefix` binding keeps and runs, as before. A `prefix` binding of a digit stays reachable from the key list, which `?` opens.

### Pointer and wheel
A move over an entry selects it, so the pointer previews as the keys do. A move over the separator or the frame changes nothing, so crossing the border does not jump the selection. A press still selects and the release still picks.

The wheel scrolls the list by one row and leaves the selection. With hover selection, a wheel that moved the selection would pull it away from the row under a pointer that has not moved, and the next small motion would snap it back. A wheel step that changes the shown rows does not select the row now under the pointer: the next move does. The first shown row therefore changes only when the selection changes or moves to another row, on a wheel step, and when the rows or the height change. `draw` stops snapping `top` to the selection on every redraw.

The window menu keeps its wheel as `j` and `k` and gets no hover. Its rows always fit unless the ribbon is very short, and the user did not ask to change it.

### Hover on the window list only
window-peek makes hover opt-in: only a plugin window opened with `hover = true` receives the kind `"move"`, and only such a window stays focused after its `on_mouse` changes the focus or the band, for every kind, the wheel included. The window list opens with `hover = true`, because both halves serve it: a move selects and previews, and a press that selects previews a window from `on_mouse`.

The window menu opens without `hover`. It has no hover behaviour, and none of its `on_mouse` events changes the focus while it stays open: a press only selects, a release picks after the menu has closed, and its wheel acts as `j` and `k`. Without `hover` it keeps its present behaviour exactly, as window-peek defines for a window that does not opt in. `open_menu` takes `hover` from the menu's spec, and `gband.win.info` reports it, which a Lua case checks.

- *Alternative*: `hover = true` on both menus, for one shared rule. Rejected: the window menu would receive moves it ignores and a hold it does not need.

### Rebuilding the rows
A rebuild keeps the selected entry by its key. When the selected window has left, the selection takes the entry now on the same row, or the last entry. A rebuild previews nothing: it runs on the `FocusChanged` of the list's own peeks, and a preview from it would start a new peek in a handler, outside the list's own code. The next key or move previews the new selection. When the previewed window closes, window-peek ends the peek and returns focus to the window last focused in its band. The list stays focused, because the hold that its last `keys` function or `on_mouse` started lasts until the user presses another key.

### The API version stays 2
The change adds a highlight group to a bundled plugin, opts its window list into `hover`, and changes how the list behaves. It adds and changes no API, and window-peek adds its own as additions, so this change does not raise `gband.api_version`, which stays 2, and `desktop.lua` keeps `api = 2`.

### Documentation
- `README.md`: the floating key style's bullet on the window list, and the sidebar's `∷`.
- `docs/plugins.md`: "The sidebar" (`∷`), and "The desktop" (the menus table and wheel sentence, `hover` on the window list only, the window list paragraphs, the leader, the groups).
- `docs/internals/08-key-styles.md`: the quotes of every changed line of `defaults/lua/gband/desktop.lua`, with prose for the order, the shortcuts, the preview, keep and cancel, the pointer and the wheel.
- `docs/internals/09-sidebar-and-errors.md`: the quote of `APPS` and the two sentences that name `⊞`.
- `docs/tutorial/00-setup.md`: `∷`. `docs/tutorial/01-keys.md`: the list's shortcuts and the `prefix` keys that it does not run.

## Risks / Trade-offs

- [A peek of a tiled window in another band moves that band's camera, and the cancel does not move it back] → The floating style floats every window that runs a program, so the list rarely holds a tiled window. window-peek lists the camera as out of scope.
- [A successful reload while the list previews a window of another band leaves the client on that band] → window-peek ends the peek at the reload, with focus on the window last focused in that band and nothing recorded, so the stacking order and the minimized windows stay as they were at opening. Only the viewed band differs, and the sidebar shows it.
- [The pointer crossing the list on its way elsewhere previews each window it passes] → Only a move onto an entry changes the selection. Each preview is one client-side view action, drawn once per frame.
- [`∷` is two cells wide in a terminal that draws ambiguous characters wide] → Recorded above. The same terminal already breaks the borders, and the docs name `⸬` as a neutral-width lookalike.
- [A `prefix` binding of a digit, `n` or `s` no longer runs from the list] → The key list still runs it, and the docs name the reserved keys.
- [The keep rule for `prefix` bindings and right presses commits a preview the user may have meant only to look at] → Escape, `q`, `[X]` and a press on empty ribbon still cancel. Listed as a decision to revisit.
- [A plugin whose `MousePressed` handler runs after the desktop's and changes the view sees the cancel first] → Its own change applies after the cancel and stands.
- [Band parts widen the list by 7 cells] → Only while windows sit in two or more bands, and only past the 24-cell minimum.

## Migration Plan

No migration. A user who set `DesktopClose` keeps that setting, since an explicit setting wins over the default. A user who wants the red button back sets `gband.hl.set("DesktopClose", { fg = 1, bold = true })`. A rollback is a revert of the change.
