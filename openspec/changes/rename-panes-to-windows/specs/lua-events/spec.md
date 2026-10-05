## MODIFIED Requirements

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

### Requirement: Built-in events
The client SHALL emit these events, and no other built-in events:

| event | payload | emitted when |
|---|---|---|
| `Attached` | `session`: the session's name | once, after the client attaches and before it handles its first key |
| `FocusChanged` | `window`, `previous`: window numbers, nil when no window | the focused window differs from before a server message, a dispatched action or a resize was handled |
| `BandChanged` | `band`, `previous`: band numbers | the viewed band differs from before a server message, a dispatched action or a resize was handled |
| `WindowOpened` | `window`, `band` | a layout holds a window the client's previous layout did not |
| `WindowClosed` | `window`, `band`: the band the window was in | the client's previous layout held a window a new layout does not |
| `LayoutChanged` | empty | a layout the client receives differs from the client's previous layout in its bands, columns, windows or widths |
| `TerminalResized` | `cols`, `rows` | the client's terminal changes size |
| `ConfigReloaded` | empty | a reload succeeded and the new configuration is in use, to the handlers of the new configuration |
| `KeyTableChanged` | `table`, `previous`: key table names | the active key table changes, as the client-attach capability defines |
| `HighlightChanged` | `group`: the group's name | a group's settings change after the configuration has loaded, as the highlights capability defines |
| `ColorschemeChanged` | `name`, `previous`: colorscheme names | a colorscheme loads after the configuration has loaded, as the colorschemes capability defines |
| `ServerEvent` | `name`, `data`, `queued`, `time` | the server sends an event, as the plugin-bridge capability defines |
| `WindowStateChanged` | `window`, `key`, `value`, `previous` | the server changes a window's state, as the plugin-bridge capability defines |

The first layout after attaching SHALL emit no `WindowOpened` and no `LayoutChanged`, and establishing the client's first view SHALL emit neither `FocusChanged` nor `BandChanged`. A layout that opens or closes a window SHALL emit `LayoutChanged` after its `WindowOpened` and `WindowClosed` events.

#### Scenario: Focus change
- **WHEN** windows 1 and 2 are open with window 2 focused and the user focuses the column to the left
- **THEN** `FocusChanged` runs with `window` 1 and `previous` 2

#### Scenario: Band change
- **WHEN** the user views the band below the first band
- **THEN** `BandChanged` runs with the second band's number as `band` and the first's as `previous`

#### Scenario: Pane opened and closed
- **WHEN** the user opens a window and then closes it
- **THEN** `WindowOpened` runs once naming the new window and its band, then `WindowClosed` runs once naming it

#### Scenario: Layout change without a pane change
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

### Requirement: Actions from handlers
A handler MAY call action values, `gband.spawn` and `gband.keymap.enter` as a binding function does. The actions it dispatches SHALL run after it returns, in the order dispatched. Events that those actions cause SHALL be emitted in turn. An event emitted while ten events are already being delivered, each caused by the one before, SHALL NOT be delivered, and the process SHALL record a warning in its log.

#### Scenario: Handler dispatches an action
- **WHEN** a `WindowOpened` handler calls `gband.action.focus_column_left()` and the user opens a window right of the only window
- **THEN** the first window is focused

#### Scenario: Event loop is cut
- **WHEN** a `FocusChanged` handler always calls `gband.action.focus_column_left()` if the right column is focused and `gband.action.focus_column_right()` otherwise, and the user changes focus once
- **THEN** the client keeps handling keys
- **AND** the log records a warning about dropped events
