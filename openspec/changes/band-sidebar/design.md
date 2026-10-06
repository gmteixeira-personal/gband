## Context

This change lands after sidebars-borders-steps, clear-errors, lua-prompt, key-style and mouse. In that state:

- **The status line is a bar.** `gband.statusline` adds a left bar, `statusline`, 20 to 40 columns wide, and lays components out in it vertically. `gband.statusline.band`, `.mode`, `.hints` and `.position` are set up by the default configuration. `clock` is optional.
- **Bars belong to the client.** `gband.bar` adds a bar of any width on either side. The ribbon area is the columns the bars leave, at the terminal's full height. Bars never change the reported size, the screen area or any window's size.
- **The client pushes its state to Lua.** Before each frame, the client pushes a view state to the Lua host: the band number, index and count, the column, the window, the windows, the active table and the errors. The status line lays itself out again in the `on_state` hook. `statusline.lua` reports through `host.error_item(shown)` whether its error item is drawn. The client draws the banner when an error is reported and no error item is drawn.
- **Modes are labelled.** `gband.keymap.label(table)` returns a mode's label, or the table's name. With the modal key style, `prefix` is the mode `navigation`. With the direct style it is a plain table whose label is `prefix`.
- **The last band is always empty.** A session with one window has bands 1 and 2.

## Goals / Non-Goals

**Goals:**
- Replace the status line with a one-column sidebar that shows the mode, the bands and the viewed band.
- Build it on `gband.bar` as an ordinary bar, so the client learns nothing new and plugins keep wider bars.
- Remove the status line and the key hints from the code, the specs and the docs, not just from the defaults.
- Keep error indication on screen.

**Non-Goals:**
- A migration path for configurations that use the status line.
- Mouse interaction with the sidebar beyond viewing a band.
- A component API for the sidebar. A plugin that wants its own content adds its own bar.

## Decisions

### The sidebar is a bundled plugin on `gband.bar`
`gband/sidebar.lua` is a bundled plugin named `sidebar`. Its setup calls `gband.bar.add({ side = opts.side or "left", size = 1, order = opts.order or 0 })`, so the bar's full id is `sidebar`. Placement, the ribbon area, reloads and failed plugins all follow the bars capability with no new rule.

The alternative was a sidebar drawn by the client in Rust. That would duplicate bar placement and make the sidebar the one bar that is not a bar.

### Rows
On a bar of height `h`:

| row | content | group |
|---|---|---|
| 0 | mode letter | `SidebarMode` |
| 1 | blank | `Bar` |
| 2 to `h − 2` | one band each, in layout order | `SidebarBandActive` for the viewed band, `SidebarBand` otherwise |
| `h − 1` | `!` while an error is reported, otherwise blank | `SidebarError` |

The blank row keeps the mode letter `I` apart from the band label `1`, which look alike. The last row is kept for the error marker even when no error is reported, so the band rows do not move when an error comes and goes. A band whose row would fall on or below the marker row is not drawn. A bar of 3 rows or fewer shows the mode letter and the marker, and no band. A bar of 1 row shows the marker while an error is reported, and the mode letter otherwise.

### Band labels
The label is the band's position in the layout, counted from 1, not the band's number. Band numbers are ids that skip after a band is removed, while the user expects `1 2 3`. Positions 1 to 9 are `1` to `9`. Positions 10 to 35 are `a` to `z`, which the user chose over a last digit or an overflow mark. Positions past 35 are not drawn.

The viewed band is shown by intensity alone. `SidebarBandActive` is bold and bright, and `SidebarBand` is dim. The labels stay the same, so a screenshot reads `1 2` whatever the view, and a test tells them apart by style.

### Mode letter
The active table `root` shows `I`. Any other table shows the first character of `gband.keymap.label(table)`, uppercased when it is an ASCII letter. Navigation mode shows `N`. With the direct key style, the prefix sequence shows `P` until it ends. A user mode labelled `RESIZE` shows `R`. An empty label cannot happen, because `gband.keymap.mode` rejects one.

### Reading state through the pushed view state
The sidebar redraws in the same `on_state` hook the status line used. The client already pushes the band index and count, the active table and the errors before each frame, so the sidebar is current in the frame that shows the change. It needs no new Lua event and works while `gband.layout()` would still be an error. The hook runs only when the pushed state differs, which covers every trigger: `BandChanged`, `LayoutChanged` changing the band count, `KeyTableChanged`, a reported error and a clear.

The view state drops the `column` field and the `windows` list, which only status line components read. It keeps the band `number` and the focused `window`, because `gband.win` and the control API read them too, and the band index, band count, active table and errors, which the sidebar reads.

The alternative was public events, with a new `ErrorsChanged`. That adds an API that only the sidebar would use, and leaves the first frame blank until `Attached`.

### Error marker and the banner
The sidebar reports `host.error_item(shown)`, as `statusline.lua` did, where `shown` is true while its marker is drawn. The client keeps its rule: it draws the banner when an error is reported and no error item is drawn. The banner therefore still appears without a sidebar, with a sidebar too short for its marker, and with a hidden sidebar. The marker shows no text, so the error list stays the way to read the message.

### Clicking a band
The sidebar registers a `MousePressed` handler. A press of the left button on its own column, on a row that shows a band label, views the band at that position with `gband.band.view`, the band's id read from `gband.layout()`. Every other press on the sidebar does nothing. The column comes from `gband.bar.info("sidebar")`, so a sidebar on the right works the same.

The client already reports a press on a bar cell as a mouse event with the target `outside`, and takes no default action for it, so a handler sees every such press and nothing else competes for it. The client learns nothing new.

The alternative was an `on_click` field in `gband.bar.add`, so every bar could take clicks. Nothing but the sidebar needs one yet, and the event already carries the cell.

### Setup options
`side` and `order` are passed to the bar. Any other option, or a wrong type or value, raises an error from `setup`. Width is not an option: the sidebar is one column by design, and a plugin that wants more adds its own bar.

### Highlight groups
The plugin sets these as defaults, in the defaults layer:

| group | default | `default` colorscheme |
|---|---|---|
| `SidebarMode` | `{ bold = true }` | `{ fg = "#7aa2f7", bold = true }` |
| `SidebarBand` | `{ dim = true }` | `{ fg = "#565f89" }` |
| `SidebarBandActive` | `{ bold = true }` | `{ fg = "#c0caf5", bold = true }` |
| `SidebarError` | `{ fg = 1, bold = true }` | `{ fg = "#f7768e", bold = true }` |

### Background
The bar keeps the default `hl`, `Bar`, and the `default` colorscheme no longer sets `Bar`. `Bar` therefore resolves to its default, `{}`, and the sidebar's cells take the terminal's default foreground and background, like the ribbon's empty cells. A terminal with a translucent background shows through the sidebar as it does through the rest of the screen. A colorscheme that wants a coloured sidebar sets `Bar`.

The alternative was a separate `Sidebar` base group, so plugin bars could keep a coloured `Bar`. Nothing in the default configuration draws a plugin bar, so one group for every bar is enough until a theme needs both.

The key list loses the status line groups it linked to. `keylist.lua` sets `KeyListKey` to `{ bold = true }` and `KeyListMuted` to `{ dim = true }` as defaults. The `default` colorscheme sets them to `{ fg = "#7aa2f7", bold = true }` and `{ fg = "#9aa5ce" }`, the colours they had through `StatusLineAccent` and `StatusLineMuted`.

### Removing the status line
The removal covers code, specs and docs:

- `ui.rs` loses the component registry, its layout and `gband.ui.statusline`. `gband.ui` keeps `width` and `truncate`.
- `statusline.lua` and `statusline/*.lua` are deleted, and their entries leave `bundled.rs`.
- The `status-line` and `key-hints` specs are retired with `retire_capabilities: true`.
- "Display width helpers" moves to plugin-windows. The control character rule, which plugin-windows cited from status-line, is written there in full.
- "Key form" moves to key-list, which uses it. `gband.keyform` stays bundled. Its scenarios move from hints to key list lines.
- "Hint labels" and its short label table are removed. The key list shows binding descriptions, and nothing else uses the short labels.
- The removed options `statusline_position`, `statusline_height` and `statusline_separator` lose their migration message, which told the user to set up `gband.statusline`. `options.rs` drops their list, and setting one is the error of an unknown option, which names it. The scenarios that tested those names are removed from the specs, because "Unknown option" covers the behaviour.
- OpenSpec does not let a MODIFIED block drop or rename a scenario the main spec holds. The requirements whose scenarios name the status line are therefore removed and added back under new names: client-attach's "Send input", "Ribbon area" and "Present the ribbon" become "Input to the server", "Ribbon area beside the bars" and "Ribbon presentation"; configuration's "Options" and "Configuration errors" become "Options and their values" and "Reporting configuration errors"; key-list's "Key list floating plugin window" becomes "Key list window"; key-style's "Choose a key style" becomes "Key style chooser"; and plugin-testing's "Observing a case" becomes "Case observation". "Binding functions" and "Key list plugin" are modified only to cite the new names. settings-themes and window-names, which wait on this change, are updated here to target the new names.

### Examples
- `window` adds a right bar 12 columns wide that shows `window <n>` for the focused window. It redraws on `FocusChanged` with `gband.bar.set_lines`, and keeps the `WindowSegment` group, now linked to `SidebarMode`.
- `agent-status` adds a right bar 3 columns wide that shows the number of waiting agents, or nothing. The plugin keeps the set of waiting windows itself. It updates the set from each `WindowStateChanged` with `key` `agent`, drops a window on `WindowClosed`, and redraws after each change. Attaching and reloading emit no `WindowStateChanged`, so on `Attached` and in the bar's `on_resize` it refills the set from `gband.layout()` and `gband.window_state(window)`, and a reattach or a reload still counts the windows already waiting.
- `hello` needs only its screenshot updated.

## Risks / Trade-offs

- **Every screen test moves.** The ribbon starts at column 1 instead of 20 and is 79 columns wide, so screenshots and column numbers change across the five dependencies' tests. → Tasks name each spec scenario and test that moves. The screenshots are regenerated and read one by one.
- **Errors are less visible.** A single `!` is easier to miss than `error`. → It is bold red in the bottom row, and the error list shows the message.
- **Dropping key hints loses discoverability.** → The key list, on `prefix ?`, lists every binding of navigation mode with its description.
- **Bands past 35 are invisible.** → That is beyond any terminal height the sidebar can fill. The viewed band is still shown in the ribbon.
- **A long dependency chain.** This change waits on five changes. → Each of them edits a status line requirement, and landing after them avoids replanning all five.

## Migration Plan

A user file that sets up `gband.statusline` or a segment, or calls `gband.ui.statusline`, fails at that line and loads the rest. Users remove those lines, and set up `gband.sidebar` themselves if their file replaces the defaults. A plugin that drew a status line component adds its own bar with `gband.bar.add`, as the examples show. There is no rollback beyond reverting the change.

## Open Questions

None.
