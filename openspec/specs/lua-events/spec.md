# lua-events Specification

## Purpose

Defines how Lua code reacts to what happens in the client: registering handlers with `gband.on`, grouping them with `gband.augroup`, emitting `User` events with `gband.emit`, and the built-in events the client emits from changes it already observes.

## Requirements

### Requirement: Event handlers
`gband.on(event, fn, opts)` SHALL register `fn` as a handler of `event`, which SHALL be the name of a built-in event of the process's side or `User`. `opts.group` SHALL be an optional group, given by the id or the name `gband.augroup` returned or took. `opts.once`, when `true`, SHALL remove the handler before its first run. `opts.pattern`, allowed only for `User` and `ServerEvent`, SHALL make the handler run only for events of that name. An unknown event, a `pattern` for another event, an unknown group, or an argument of the wrong type SHALL be an error at the line of the call.

The client SHALL run the handlers of an event in the order they were registered, each as a callback that belongs to the plugin that registered it, and each with its own copy of the event's payload table. The server SHALL run only the handlers registered in its own Lua state, for the events the server-runtime capability defines, and never a handler of a client.

#### Scenario: Handler receives the payload
- **WHEN** a handler of `TerminalResized` is registered and the client's terminal becomes 100×30
- **THEN** the handler runs once with a table whose `cols` is 100 and `rows` is 30

#### Scenario: Once
- **WHEN** a handler of `FocusChanged` is registered with `once = true` and focus changes twice
- **THEN** the handler runs once

#### Scenario: Payload copies are separate
- **WHEN** two handlers of `FocusChanged` run and the first sets `window` in its payload to nil
- **THEN** the second handler's payload still names the focused window

#### Scenario: Unknown event
- **WHEN** line 2 of `user/init.lua` calls `gband.on("FocusChange", fn)`
- **THEN** loading fails with an error at `user/init.lua` line 2 naming `FocusChange`

#### Scenario: Server event pattern
- **WHEN** a client handler is registered with `gband.on("ServerEvent", fn, { pattern = "agent.done" })` and the server emits `agent.waiting` then `agent.done`
- **THEN** the handler runs once, for `agent.done`

#### Scenario: Server event name in the client
- **WHEN** line 4 of `user/init.lua` calls `gband.on("WindowOutput", fn)`
- **THEN** loading fails with an error at `user/init.lua` line 4 naming `WindowOutput`

### Requirement: Handler groups
`gband.augroup(name, opts)` SHALL return the integer id of the group `name`, the same id each time within one load. When `opts.clear` is `true` or absent, it SHALL first remove every handler in the group. When `opts.clear` is `false`, it SHALL keep them.

#### Scenario: Clear on redefinition
- **WHEN** code calls `gband.augroup("g")`, registers a `FocusChanged` handler in group `g`, then calls `gband.augroup("g")` and registers a second handler in it
- **THEN** a focus change runs only the second handler

#### Scenario: Keep without clear
- **WHEN** the second call is `gband.augroup("g", { clear = false })`
- **THEN** a focus change runs both handlers

#### Scenario: Group by name
- **WHEN** a handler is registered with `group = "g"` after `gband.augroup("g")`
- **THEN** it belongs to the group whose id `gband.augroup("g")` returned

### Requirement: User events
`gband.emit(name, data)` SHALL emit a `User` event named `name`, a non-empty string, with the payload `{ name = name, data = data }`. It SHALL run every `User` handler with no pattern or with the pattern `name`, at once and before it returns.

#### Scenario: Pattern match
- **WHEN** one `User` handler has the pattern `hello.ready`, another the pattern `other`, and code calls `gband.emit("hello.ready", { n = 1 })`
- **THEN** only the first runs, with a payload whose `data.n` is 1

### Requirement: Built-in events
The client SHALL emit these events, and no other built-in events:

| event | payload | emitted when |
|---|---|---|
| `Attached` | `session`: the session's name | once, after the client attaches and before it handles its first key |
| `FocusChanged` | `window`, `previous`: window numbers, nil when no window | the focused window differs from before a server message, a dispatched action or a resize was handled |
| `BandChanged` | `band`, `previous`: band numbers | the viewed band differs from before a server message, a dispatched action or a resize was handled |
| `WindowOpened` | `window`, `band` | a layout holds a window the client's previous layout did not |
| `WindowClosed` | `window`, `band`: the band the window was in | the client's previous layout held a window a new layout does not |
| `LayoutChanged` | empty | a layout the client receives differs from the client's previous layout in its bands, columns, windows, widths or floating windows and their boxes |
| `TerminalResized` | `cols`, `rows` | the client's terminal changes size |
| `ConfigReloaded` | empty | a reload succeeded and the new configuration is in use, to the handlers of the new configuration |
| `KeyTableChanged` | `table`, `previous`: key table names | the active key table changes, as the client-attach capability defines |
| `HighlightChanged` | `group`: the group's name | a group's settings change after the configuration has loaded, as the highlights capability defines |
| `ColorschemeChanged` | `name`, `previous`: colorscheme names | a colorscheme loads after the configuration has loaded, as the colorschemes capability defines |
| `ServerEvent` | `name`, `data`, `queued`, `time` | the server sends an event, as the plugin-bridge capability defines |
| `WindowStateChanged` | `window`, `key`, `value`, `previous` | the server changes a window's state, as the plugin-bridge capability defines |
| `MousePressed` | `button`, and the mouse fields | the user presses a mouse button |
| `MouseReleased` | `button`, and the mouse fields | the user releases a mouse button |
| `MouseDragged` | `button`, and the mouse fields | the pointer moves to another cell while a button is held |
| `MouseScrolled` | `direction`, and the mouse fields | the user turns the wheel one step |

`button` SHALL be `"left"`, `"middle"` or `"right"`, and `direction` `"up"`, `"down"`, `"left"` or `"right"`. The mouse fields SHALL be:

- `col` and `row`: the terminal cell under the pointer, counted from 0.
- `ctrl`, `alt` and `shift`: booleans, the modifiers the terminal reports.
- `target`: `"window"`, `"plugin_window"`, `"ribbon"` or `"outside"`, the target of the cell under the pointer as the mouse capability's "Pointer targets" defines, with a floating plugin window or a tiled plugin window's drawn window as `"plugin_window"`.
- `window`: the number of the target's window, or of a tiled plugin window's drawn window, and nil otherwise.
- `plugin_window`: the target plugin window's number, and nil otherwise.
- `content_col` and `content_row`: the target's content cell, and nil without one.
- `box_col` and `box_row`: for the targets `"window"` and `"plugin_window"`, the cell's column and row less those of the top-left cell of the target's box, counted from 0, with the border cells included; nil for other targets.
- `box_width` and `box_height`: for the targets `"window"` and `"plugin_window"`, the width and height of the target's box, border included, as drawn in the frame the event was resolved against; nil for other targets.
- `table`: the name of the key table active when the event arrived.

The box of a target SHALL be the box the mouse capability's "Pointer targets" resolves the cell to: a tile's box at the position drawn, a floating window's box, a floating plugin window's box, or the tile of a tiled plugin window's drawn window. The box fields SHALL count the whole box, also the cells that the terminal or the ribbon area cuts off, so `box_col` and `box_row` give the same cell whether or not the box is wholly shown. For a cell with a content cell, `box_col` SHALL equal `content_col` plus the column of the content area's top-left cell within the box, and `box_row` SHALL equal `content_row` plus that cell's row within the box.

The client SHALL emit a mouse event for every press, release, motion with a button held to another cell, and wheel step the terminal reports, after handling it, whether a binding, a default, a gesture or a program took it. A motion with no button held SHALL emit none. Mouse events SHALL only report: a handler SHALL NOT keep an event from the window, plugin window or gesture that takes it.

The first layout after attaching SHALL emit no `WindowOpened` and no `LayoutChanged`, and establishing the client's first view SHALL emit neither `FocusChanged` nor `BandChanged`. A layout that opens or closes a window SHALL emit `LayoutChanged` after its `WindowOpened` and `WindowClosed` events.

#### Scenario: Focus change
- **WHEN** windows 1 and 2 are open with window 2 focused and the user focuses the column to the left
- **THEN** `FocusChanged` runs with `window` 1 and `previous` 2

#### Scenario: Band change
- **WHEN** the user views the band below the first band
- **THEN** `BandChanged` runs with the second band's number as `band` and the first's as `previous`

#### Scenario: Window opened and closed
- **WHEN** the user opens a window and then closes it
- **THEN** `WindowOpened` runs once naming the new window and its band, then `WindowClosed` runs once naming it

#### Scenario: Layout change without a window change
- **WHEN** two windows sit in two columns and the user consumes the focused window into the column to its left
- **THEN** `LayoutChanged` runs once, and neither `WindowOpened` nor `WindowClosed` runs

#### Scenario: Nothing at attach
- **WHEN** a client attaches to a session holding three windows, one of which has a non-empty state
- **THEN** `Attached` runs once and no `WindowOpened`, `LayoutChanged`, `FocusChanged`, `BandChanged` or `WindowStateChanged` runs

#### Scenario: Reload
- **WHEN** `user/init.lua` registers a `ConfigReloaded` handler and the user saves it unchanged
- **THEN** the handler registered by the reloaded file runs once

#### Scenario: State change
- **WHEN** the server sets window 1's `agent` to `"waiting"`
- **THEN** `WindowStateChanged` runs once with `window` 1, `key` `"agent"`, `value` `"waiting"` and `previous` nil

#### Scenario: Floating box moved
- **WHEN** a floating window's box moves one step right and the client receives the new layout
- **THEN** `LayoutChanged` runs once

#### Scenario: Click event
- **WHEN** a handler of `MousePressed` is registered and the user clicks content column 4 and row 2 of window 1 with the left button
- **THEN** the handler runs once with `button` `"left"`, `target` `"window"`, `window` 1, `content_col` 4, `content_row` 2, `box_col` 5, `box_row` 3 and `table` `"root"`

#### Scenario: Box cell on a border
- **WHEN** the ribbon area starts at the terminal's top-left cell, window 2's tile has its box at columns 40 to 79 and rows 0 to 23, a handler of `MousePressed` is registered, and the user presses the left button at column 79 and row 0
- **THEN** the handler runs once with `target` `"window"`, `window` 2, `content_col` nil, `content_row` nil, `box_col` 39, `box_row` 0, `box_width` 40 and `box_height` 24

#### Scenario: Box cell of a cut tile
- **WHEN** the ribbon area starts at the terminal's top-left cell, a tile's box is 40×24 and starts 10 columns left of the terminal's first column, a handler of `MousePressed` is registered, and the user presses at column 0 and row 5
- **THEN** the handler runs once with `box_col` 10, `box_row` 5, `box_width` 40 and `box_height` 24

#### Scenario: Box cell of a floating plugin window
- **WHEN** a floating plugin window with a border has its box of 20×10 at column 5 and row 3 of a ribbon area that starts at the terminal's top-left cell, a handler of `MousePressed` is registered, and the user presses at column 5 and row 12
- **THEN** the handler runs once with `target` `"plugin_window"`, `box_col` 0, `box_row` 9, `box_width` 20 and `box_height` 10

#### Scenario: Drag events
- **WHEN** a handler of `MouseDragged` is registered and the user drags with the left button across three cells of one row and releases
- **THEN** the handler runs three times, and `MouseReleased` runs once after them

#### Scenario: Wheel event
- **WHEN** a handler of `MouseScrolled` is registered and the user turns the wheel two steps up over empty ribbon
- **THEN** the handler runs twice with `direction` `"up"`, `target` `"ribbon"`, and `box_col`, `box_row`, `box_width` and `box_height` nil

### Requirement: Actions from handlers
A handler MAY call action values, `gband.spawn` and `gband.keymap.enter` as a binding function does. The actions it dispatches SHALL run after it returns, in the order dispatched. Events that those actions cause SHALL be emitted in turn. An event emitted while ten events are already being delivered, each caused by the one before, SHALL NOT be delivered, and the process SHALL record a warning in its log.

#### Scenario: Handler dispatches an action
- **WHEN** a `WindowOpened` handler calls `gband.action.focus_column_left()` and the user opens a window right of the only window
- **THEN** the first window is focused

#### Scenario: Event loop is cut
- **WHEN** a `FocusChanged` handler always calls `gband.action.focus_column_left()` if the right column is focused and `gband.action.focus_column_right()` otherwise, and the user changes focus once
- **THEN** the client keeps handling keys
- **AND** the log records a warning about dropped events
