## MODIFIED Requirements

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
- `table`: the name of the key table active when the event arrived.

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
- **THEN** the handler runs once with `button` `"left"`, `target` `"window"`, `window` 1, `content_col` 4, `content_row` 2 and `table` `"root"`

#### Scenario: Drag events
- **WHEN** a handler of `MouseDragged` is registered and the user drags with the left button across three cells of one row and releases
- **THEN** the handler runs three times, and `MouseReleased` runs once after them

#### Scenario: Wheel event
- **WHEN** a handler of `MouseScrolled` is registered and the user turns the wheel two steps up over empty ribbon
- **THEN** the handler runs twice with `direction` `"up"` and `target` `"ribbon"`
