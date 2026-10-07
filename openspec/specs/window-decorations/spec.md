# window-decorations Specification

## Purpose
Defines window decorations: styled spans that a Lua function supplies for a window's top border, and that the client draws at the right end of that border. It covers the decorations provider's info table, return value and call rules, where the spans go and when they are left out, their style, and how a failing provider is reported.

## Requirements

### Requirement: Decorations provider
The decorations provider SHALL be the function that `gband.core.provide("decorations", fn)` registers, as the lua-api capability's "Providers" defines. While one is registered and no failure has stopped it, as "Decorations provider failures" defines, the client SHALL call it for each window that it draws in a tile or in a floating window's box, when the window's border definition draws the top side and the box's top row lies inside the ribbon area. This SHALL include a lifted tile, the drawn window of a tiled plugin window, and the windows of every band drawn during a band switch. The client SHALL NOT call it for a floating plugin window or for a lifted tile's drop outline.

The client SHALL call it with one argument, a new table holding:

| field | value |
|---|---|
| `window` | the window's number |
| `floating` | `true` when the window floats, `false` when it is tiled |
| `focused` | `true` when the window's border is drawn in the focused style, as the client-attach capability defines it, and `false` otherwise |
| `width` | the width of the window's tile or box in cells, border included, as drawn in that frame |

The function SHALL return nil or a list. Each entry of the list SHALL be a span: a string, or a table holding `text`, a string, and `style`, a style table or nil. A style table SHALL hold the fields of a style that a plugin window frame takes, as the API documentation gives them: `fg` and `bg`, each `#rrggbb` or a palette index from 0 to 255, and the booleans `bold`, `italic`, `underline`, `reverse` and `dim`. A string SHALL be a span with that text and no style. Control characters, the code points U+0000 to U+001F, U+007F and U+0080 to U+009F, SHALL be removed from every span's text. A span's width SHALL be the width of its text as `gband.ui.width` measures it. The window's decorations SHALL be the spans, in list order. Nil and an empty list SHALL give the window no decorations.

Each call SHALL run outside a callback, as code of no plugin, with its own instruction limit, as the plugins capability's "Instruction limit" defines for one call of a callback. Calling an action value, `gband.spawn` or `gband.keymap.enter` in it SHALL be an error, as it is outside any callback.

The client SHALL call the function at most once per window for each frame it draws. It SHALL keep the result of each window's last call. When it draws the window again with an info table of the same values, and no other Lua code of the client has run since that call, it SHALL draw the kept result without calling the function. Other Lua code SHALL include every callback, every timer function, every function that `gband.core.on_state` or `gband.core.after_event` added, every function of another provider, and every load. A load SHALL drop every kept result.

#### Scenario: Buttons on floating windows only
- **WHEN** `user/init.lua` registers a decorations function that returns `{ "[_]", "[□]", "[X]" }` when `info.floating` is `true` and nil otherwise, the client's 80×24 terminal sets the screen area, the client has no bar, and the viewed band holds one column of width 1/2 and a floating window whose box spans columns 20 to 59 and rows 6 to 17
- **THEN** row 6 shows `[_][□][X]` from column 49 to column 57, `─` on column 58 and `╮` on column 59
- **AND** the tile's top row shows no span

#### Scenario: Info table
- **WHEN** the decorations function records each info table it receives, the client's 80×24 terminal sets the screen area, the client has no bar, and the viewed band holds windows 1 and 2 in two columns of width 1/2 with window 2 focused
- **THEN** it has received `{ window = 1, floating = false, focused = false, width = 40 }` and `{ window = 2, floating = false, focused = true, width = 40 }`

#### Scenario: Control characters removed
- **WHEN** the decorations function returns `{ "a\tb" }` for a tile 20 columns wide
- **THEN** the tile's top row shows `ab` on its columns 16 and 17

#### Scenario: Result kept between frames
- **WHEN** the decorations function counts its calls, the viewed band holds one tile, and the client draws three frames with no input, no message from the server and no timer between them
- **THEN** the function has run once

#### Scenario: Called again after a callback
- **WHEN** the decorations function returns `{ mark }`, where `mark` is a global that `user/init.lua` sets to `"[a]"`, and a binding function sets `mark` to `"[b]"`
- **THEN** the first frame after the binding function runs shows `[b]` on the tile's top row

#### Scenario: No call without a top side
- **WHEN** `user/init.lua` sets `tile_border_sides` to `{ "left", "right", "bottom" }`, its decorations function counts its calls, and the viewed band holds one tile and no floating window
- **THEN** the function has not run

#### Scenario: Not a callback
- **WHEN** the decorations function records what `gband.core.dispatching()` returns
- **THEN** it records `false`

### Requirement: Decoration placement
Let `w` be the width of a window's tile or box, border included, as drawn, and `d` the sum of the widths of its decorations' spans. The columns of the box SHALL be counted from 0 at its left border.

When `d` is greater than 0 and at most `w − 4`, the client SHALL draw the spans on the box's top row, after the border and the title, one after another in list order with no gap. The first span SHALL start at column `w − 2 − d`, so the last span's last cell is column `w − 3`. The spans SHALL NOT cover columns 0, 1, `w − 2` and `w − 1`, so both top corners and the cell beside each stay as the border and the title draw them.

When `d` is greater than `w − 4`, the client SHALL draw none of the window's spans, and SHALL cut the window's title as for a window with no decorations. The client SHALL NOT cut a span or leave out some spans and draw the others.

When the window's border definition, as the borders capability defines it, does not draw the top side, or the box's top row lies outside the ribbon area, the client SHALL draw no decorations. The client SHALL draw decorations whether window titles are on or off, as the window-names capability's "Titles turned off" defines.

The spans SHALL be placed against the box, whatever part of it is shown. A box that crosses the ribbon area's edge SHALL show the cells of its spans that lie inside the area, and a cell that a floating window, a floating plugin window or the error banner covers SHALL stay covered, as for the border. While an animation runs, `w` SHALL be the drawn width, and the spans SHALL move with the box as its border does.

The API documentation SHALL state this rule with an example, so that Lua can find the span under a mouse press from the press's cell in the box and the box's width.

#### Scenario: Three buttons in a 40-column box
- **WHEN** a floating window's box is 40 columns wide and its spans are `[_]`, `[□]` and `[X]`
- **THEN** on the box's top row `[_]` takes columns 29 to 31, `[□]` columns 32 to 34 and `[X]` columns 35 to 37
- **AND** column 38 shows `─` and column 39 shows `╮`

#### Scenario: Spans that just fit
- **WHEN** titles are on, a tile is 13 columns wide, its shown name is `vim`, and its spans are `[_]`, `[□]` and `[X]`
- **THEN** its top row shows `╭─[_][□][X]─╮`, with no title

#### Scenario: Box too narrow
- **WHEN** titles are on, a tile is 12 columns wide, its shown name is `vim`, and its spans are `[_]`, `[□]` and `[X]`
- **THEN** its top row shows `╭`, `vim` from column 1 to column 3, `─` from column 4 to column 10 and `╮` on column 11, and no span

#### Scenario: No top side
- **WHEN** `user/init.lua` sets `floating_border_sides` to `{ "left", "right", "bottom" }`, and its decorations function returns `{ "[X]" }` for every window
- **THEN** no floating window's top row shows `[X]`
- **AND** every tile's top row shows `[X]`

#### Scenario: Column cut at the left edge
- **WHEN** the client's 80×24 terminal sets the screen area, the client has no bar, the viewed band holds two columns of width 2/3, the client focuses the second, and the decorations function returns `{ "[X]" }` for every window
- **THEN** screen columns 22 to 24 show the first tile's `[X]`, and columns 25 and 26 show `─` and `╮`
- **AND** screen columns 75 to 77 show the second tile's `[X]`

#### Scenario: During an animation
- **WHEN** a tile widens from 20 to 40 columns, a frame draws it 30 columns wide, and the decorations function returns `{ "[X]" }`
- **THEN** the function receives `width = 30` for that frame, and `[X]` takes the tile's columns 25 to 27 in it

#### Scenario: Titles off
- **WHEN** `user/init.lua` sets `window_titles` to `false`, and its decorations function returns `{ "[X]" }` for a tile 20 columns wide
- **THEN** the tile's top row shows `─` from column 1 to column 14, `[X]` from column 15 to column 17, `─` on column 18 and `╮` on column 19

#### Scenario: Lifted tile
- **WHEN** the decorations function returns `{ "[X]" }` for every window, and the user lifts a tile 40 columns wide with `drag_window`
- **THEN** the lifted tile shows `[X]` on its columns 35 to 37 wherever it is drawn
- **AND** the drop outline shows no span

### Requirement: Decoration style
Each cell a span covers SHALL be drawn in the window's border style, as the client-attach capability defines it, distinct for the focused window, with the span's style applied over it. Each field that the span's style sets SHALL replace that field of the border style, and the other fields of the border style SHALL stay. A span without a style SHALL be drawn in the border style. The terminal palette, as the colorschemes capability defines it, SHALL apply to these cells as to every other cell.

#### Scenario: Plain spans take the border style
- **WHEN** the active colorscheme is `default`, two tiles are drawn with the first focused, and the decorations function returns `{ "[X]" }` for every window
- **THEN** the first tile's `[X]` is drawn in `WindowBorderFocused`'s resolved style and the second tile's in `WindowBorder`'s

#### Scenario: Style over the border style
- **WHEN** `WindowBorderFocused` resolves to `{ fg = "#b1b9f9", bold = true }`, and the focused tile's spans are `{ { text = "[X]", style = { fg = 1 } } }`
- **THEN** `[X]` is drawn bold with the foreground palette index 1

#### Scenario: Highlight group as a style
- **WHEN** `user/init.lua` sets the group `CloseButton` to `{ fg = "red" }`, and its decorations function returns `{ { text = "[X]", style = require("gband.hl").drawn("CloseButton") } }` for the focused tile
- **THEN** `[X]` is drawn with the foreground palette index 1 and the other fields of `WindowBorderFocused`'s resolved style

### Requirement: Decorations provider failures
A call of the decorations function SHALL fail when it raises an error, when it reaches the instruction limit, or when it returns a value other than nil or a list of spans, as "Decorations provider" defines. The client SHALL report the failure as the configuration capability defines, preceded by `decorations` and a colon. The message of a wrong return value SHALL name the first wrong entry by its place in the list, counted from 1. A failure SHALL NOT mark a plugin failed.

After a failure, the client SHALL draw no decorations, and SHALL NOT call the function again, until `gband.core.provide("decorations", fn)` registers a function or a load succeeds. So a function that fails at every call SHALL be reported once.

#### Scenario: Error in the function
- **WHEN** the decorations function raises `boom` on line 3 of `user/init.lua`, and the user then focuses another window and views another band
- **THEN** the client reports one error, preceded by `decorations: `, at `user/init.lua` line 3 with the message `boom`
- **AND** no window's top row shows a span

#### Scenario: Wrong span
- **WHEN** the decorations function returns `{ "[_]", 7 }`
- **THEN** the client reports an error preceded by `decorations: ` that names span 2
- **AND** no window's top row shows `[_]`

#### Scenario: Action in the function
- **WHEN** the second of two columns is focused and the decorations function calls `gband.action.focus_column_left()`
- **THEN** the client reports an error preceded by `decorations: `, and the second column stays focused

#### Scenario: Endless loop
- **WHEN** the decorations function runs `while true do end`
- **THEN** the client reports an error preceded by `decorations: ` that names the instruction limit
- **AND** the client handles the next key

#### Scenario: Registered again
- **WHEN** the decorations function has failed, and a binding function calls `gband.core.provide("decorations", fn)` with a function that returns `{ "[X]" }`
- **THEN** the next frame shows `[X]` on the top row of each window
