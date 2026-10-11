## Context

The shared layout lives in `crates/core/src/layout.rs`. A `Column` holds `windows`, `heights`, `width` and `full_width`, and is serialized whole into the layout message every client receives. `Layout::apply` turns a `SessionAction` into `LayoutEvent`s, and an empty event list means "nothing changed", so the server sends no new layout. Width actions go through `with_width`, height actions through `with_height`, which splits tiled and floating windows and calls `Column::step_height`, `Column::set_height` or a reset closure.

`crates/core/src/geometry.rs::tiles` gives every window of a band a `Tile`. Three consumers read it: the server sizes each PTY from it (`crates/server/src/session.rs`), `View::shown` filters it into shown windows (`crates/core/src/view.rs`), and the client draws it (`crates/client/src/render.rs::layers`, fed by `crates/client/src/animation.rs`). Focus is per client: `View` keeps the focused window and a last-focus map, and `View::recent_in` already picks the most recently focused window of a list for "Focus across columns".

`render::layers` builds one `Layer` per drawn tile, and `regions` turns layers into the hit-test `Region`s that `crates/client/src/mouse.rs` resolves presses against. `draw_tile` draws the border, then the title and decorations on the top row, then the grid.

The modal and direct presets bind `prefix f` to `toggle_full_width`. `w` is free in both.

## Goals / Non-Goals

**Goals:**
- One per-column flag and one per-column tab height in the shared layout, so every client and the server agree on tile sizes.
- The shown tab derived per client from focus alone, with no new view state to keep in sync.
- An indicator that costs no cell of width, so no PTY resizes when tabs turn on or off beyond the height change.

**Non-Goals:**
- No `default_column_display` option, as niri has. New columns are never tabbed.
- No indicator options: no position other than the left border, no configurable characters, no hiding when a column has one tab. The two highlight groups are the only knobs.
- No tab titles in the indicator. The shown tab's title stays on its top border, as for any tile.
- No binding in the floating key style. Its window list reaches windows directly and does not list column actions today.
- No animation of the switch between tabs. All tabs share one tile, so a switch draws the new tab in place.

## Decisions

### Two fields on `Column`, carried by every existing move

`Column` gains `tabbed: bool` and `tab_rows: Option<u16>`, `None` meaning the tabs fill the area. `Column::new` sets `false` and `None`, so open, expel, move to a new column and tiling a floating window give an untabbed column without new code. Consume and move above or below go through `push` and the existing insertion, so the entering window becomes a tab at its index. Move column swaps whole `Column` values, so both fields travel with it.

Alternative: a `ColumnDisplay` enum, as niri has. Rejected: there are two modes and the flag mirrors `full_width`. An enum can replace it if a third mode appears.

Alternative: reuse `WindowHeight` for the tab height. Rejected: a weight has no meaning for one shared height, and `Option<u16>` maps straight onto the Lua field `tab_rows`.

### Toggle as a session action with its own event

`SessionAction::ToggleTabbed(WindowId)` and `SessionCommand::ToggleTabbed` sit beside `ToggleFullWidth`. `Layout::apply` flips the flag of the window's column and returns a new `LayoutEvent::ColumnTabsChanged { band, column, tabbed, tab_rows }`. A floating or unknown window returns no event. The same event reports a tab height change, so the server broadcasts the layout and scripting sees one kind of event for both fields.

### Height actions branch on the column, inside `with_height`

`with_height` already separates tiled and floating windows. For a tiled window in a tabbed column it calls a new `Column::tab_height` path instead of the `tiled` closure: grow and shrink measure the current tab tile height, add or subtract `by.rows_of(area.rows)`, and clamp to `MIN_TILE_HEIGHT..=area.rows`; set rows clamps the same way; reset and set weight store `None`. `heights` is never touched, which is what makes "Turn tabs off gives the stack back" hold. The resize-by-drag of a top or bottom edge goes through set height, so it needs no change.

### `tiles` places tabs on top of one another

`geometry::tiles` gives every window of a tabbed column `y = 0` and `height = tab_rows.unwrap_or(area.rows).min(area.rows)`, the column's span as its width, and its real `row` index. The server sizes PTYs from `tiles`, but resizes only shown windows, and a hidden tab is not shown. The server's settle step therefore also resizes every tab of a tabbed column one of whose tabs some client shows, so every tab has the shown tab's terminal size and switching tabs resizes nothing. The client and the view filter hidden tabs out after `tiles`, so `tiles` stays per band and client-independent.

### The shown tab is a `View` query

`View::shown_tab(&Column) -> WindowId` returns the focused window when the column holds it, else `recent_in(&column.windows)`, else `column.windows[0]`. The peeked window is the focused window, so a peek shows its tab with no special case. `View::shown` drops every tab of a tabbed column other than its shown tab, which also keeps "Report shown windows" correct: focus moves between tabs change the shown set, and the existing comparison sends a new shown message.

`render::layers` applies the same filter. While a window is lifted, `layers` already leaves the lifted window out; `shown_tab` is then asked about the column without the lifted window, so the column shows the next remembered tab under the drag.

Alternative: store an active tab index per column in the shared layout, as niri does. Rejected: gband's focus is per client, so a shared active tab would make one client's focus move switch the tab another client sees.

### Indicator segments come from one pure function

`geometry::tab_segments(rows: u16, tabs: usize) -> Vec<Option<Range<u16>>>` returns each tab's rows within `1..rows - 1` of a tile `rows` high, by the rule of "Tab indicator". Both the renderer and the hit test call it with the drawn height, so a click always lands on the segment that is drawn, animations included.

The shown tab's segment is `█` and every other segment `░`. The shape tells the shown tab apart in every column, focused or not, so the styles only have to mark focus. Alternative: `┃` and `│`, closer to niri's thin bar. Rejected: in a dim, unfocused column a `│` segment reads as an ordinary border, so the column does not look tabbed.

`draw_tile` gains an optional indicator in its `Edge`: the segments, the shown tab's index and the two resolved styles. It draws after the border and before the grid, over column 0 rows 1 to `h − 2`, honouring the tile's clip as the border does. A lifted tile and the drop outline pass no indicator.

### Segment cells become regions above their tile

For each drawn tabbed tile, `regions` emits extra hit-test regions, one per drawn segment, 1 cell wide, of a new kind `RegionKind::TabSegment { column, row }`, with `window` set to the segment's tab and `border: true`. They sit in the region list directly after the tile's own region, so the hit test reaches them before the tile, and floating windows and plugin windows, later in the list, keep covering them. Target resolution in `mouse.rs` reads a `TabSegment` as a tile border cell of its window, so a press focuses that tab and is not forwarded, and `drag_window` on a segment lifts that tab, which is how a tab is dragged out of its column.

`mouse::drop_columns` builds drop places from the `Tile` regions and skips `TabSegment` ones. Hidden tabs have no region, so a tabbed column offers only its shown tab's tile, picked without the lifted window, which gives "Drop into a tabbed column" with no extra rule in the drop code.

### Highlight groups as links

`hl.lua` defines `TabIndicatorActive` as `{ link = "WindowBorderFocused" }` and `TabIndicator` as `{ link = "WindowBorder" }` in the defaults layer. Every bundled theme sets the two border groups, so the indicator follows each theme with no theme file edited, and the colorschemes requirement "every theme sets every theme group" keeps its list.

### `w` beside `f`

Both presets bind `prefix w` to `action.toggle_column_tabbed` right after `prefix f`, matching the row order of the client-attach table. The modal binding keeps navigation mode, as `f` does, so the user can toggle and then move between tabs with `j` and `k`.

## Risks / Trade-offs

- [The left border cell of a tabbed tile no longer shows focus by its character, only by the indicator's style] → `TabIndicatorActive` links to `WindowBorderFocused`, so the shown tab's segment of the focused column keeps the focused colour, and the other three sides keep the focused characters.
- [A tab the client has never focused is never shown in another client's view until that client focuses it] → It is the same rule that picks the window when focus enters a column today, and `j`/`k` or a click reach every tab.
- [Wire format change: the layout message carries two more fields] → The version handshake refuses a server from another build, as the client-attach capability defines, so mixed builds never exchange layouts.
- [`window-names` "Border title" says it draws the name of every tiled window] → A hidden tab is not drawn at all, as "Shown tab" says, so it has no border to draw a title on. The rule stays true of every drawn tile.

## Migration Plan

None. Layouts live only in a running server, so no saved state has the old shape. A server and client from this build start with every column untabbed.
