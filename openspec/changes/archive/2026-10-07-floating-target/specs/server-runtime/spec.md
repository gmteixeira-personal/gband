## MODIFIED Requirements

### Requirement: Server session actions
The server's `gband.action` SHALL hold one function for each session action the actions capability names, under its Lua name. Each SHALL take one target table naming `session` and the window the action acts on as `window`; `open_window` SHALL instead take `session`, `band`, an optional `after` window, an optional `program`, a command line string or a list of argument strings, and an optional `floating`, a boolean. `floating = true` SHALL open the window floating in `band`, as the layout capability's "Open a window" defines, and SHALL be an error together with `after`. `toggle_window_floating` SHALL also take an optional `after` window: a floating window SHALL be tiled after `after`, as the floating-windows capability's "Tile a window" defines, and as the band's first column without it. `toggle_window_floating` SHALL also take an optional `floating`, a boolean. `floating = true` SHALL name the floating layer, so the action SHALL float the window when it is tiled and change nothing when it already floats. `floating = false` SHALL name the tiled layer, so the action SHALL tile the window when it floats and change nothing when it is already tiled. Each SHALL behave as the floating-windows capability's "Float a window" and "Tile a window" define. Without `floating`, the action SHALL toggle the window. `floating = true` SHALL be an error together with `after`.

`grow_column_width`, `shrink_column_width`, `grow_window_height` and `shrink_window_height` SHALL also take an optional `step`: a number greater than 0 and at most 10000 for the two width actions, and greater than 0 and at most 1 for the two height actions, read as the configuration capability reads a step. The action SHALL then name that step, and SHALL name 1/10 when the target holds no `step`, as the actions capability defines. No other action SHALL take `step`.

A missing session, a wrong type, an unknown field or a `step` out of its range SHALL be an error at the line of the call. The actions SHALL be applied to their sessions after the callback returns, in the order called, as the session-server capability's "Session actions" defines, and SHALL tell no client to focus. An action naming a session, window or band that no longer exists SHALL change nothing. `gband.action` SHALL be callable only in a callback.

#### Scenario: Close from a handler
- **WHEN** a `WindowOutput` handler calls `gband.action.close_window({ session = ev.session, window = ev.window })` on seeing `bye`, and the window prints `bye`
- **THEN** the window's program receives SIGHUP and the window leaves the layout

#### Scenario: Open without stealing focus
- **WHEN** a client focuses window 1 and a server command opens a window in band 1 after window 1 running `htop`
- **THEN** every client receives a layout holding the new window
- **AND** the client's focus stays on window 1

#### Scenario: Float from a handler
- **WHEN** a `WindowOpened` handler calls `gband.action.toggle_window_floating({ session = ev.session, window = ev.window })` for tiled window 2, then `gband.action.open_window({ session = ev.session, band = ev.band, floating = true })`
- **THEN** every client receives a layout whose band's floating list holds window 2 and then the new window

#### Scenario: Tile from the server
- **WHEN** band 1 of `work` holds columns with windows 1 and 2 and floating window 3, and a server command calls `gband.action.toggle_window_floating({ session = "work", window = 3, after = 1 })`
- **THEN** band 1 holds the columns of window 1, window 3 and window 2, in that order

#### Scenario: Floating with after
- **WHEN** a handler calls `gband.action.open_window({ session = "work", band = 1, after = 1, floating = true })`
- **THEN** the call raises an error naming `after`

#### Scenario: Two handlers float one window
- **WHEN** `user/server.lua` registers two `WindowOpened` handlers, each of which calls `gband.action.toggle_window_floating({ session = ev.session, window = ev.window, floating = true })`, and a client opens window 2 tiled in band 1
- **THEN** every client receives a layout whose band 1 floating list holds window 2 once
- **AND** no column of band 1 holds window 2

#### Scenario: Tile a tiled window from the server
- **WHEN** band 1 of `work` holds columns with windows 1 and 2, and a server command calls `gband.action.toggle_window_floating({ session = "work", window = 2, floating = false })`
- **THEN** the layout of `work` is unchanged

#### Scenario: Float with after from the server
- **WHEN** a handler calls `gband.action.toggle_window_floating({ session = "work", window = 3, after = 1, floating = true })`
- **THEN** the call raises an error naming `after`, and the layout is unchanged

#### Scenario: Floating that is not a boolean
- **WHEN** line 6 of `user/server.lua` calls `gband.action.toggle_window_floating({ session = "work", window = 3, floating = 1 })` in a handler, and the handler runs
- **THEN** the call raises an error at `user/server.lua` line 6 naming `floating`, and the layout is unchanged

#### Scenario: Grow by a step from the server
- **WHEN** the only column of band 1 of `work` holds window 1 at width 1/2, and a server command calls `gband.action.grow_column_width({ session = "work", window = 1, step = 1/4 })`
- **THEN** every attached client receives a layout in which window 1's column has width 3/4

#### Scenario: Server default step
- **WHEN** the only column of band 1 of `work` holds window 1 at width 1/2, and a server command calls `gband.action.grow_column_width({ session = "work", window = 1 })`
- **THEN** every attached client receives a layout in which window 1's column has width 3/5

#### Scenario: Server step out of range
- **WHEN** line 4 of `user/server.lua` calls `gband.action.grow_window_height({ session = "work", window = 1, step = 2 })` in a handler, and the handler runs
- **THEN** the call raises an error at `user/server.lua` line 4 naming `step`, and the layout is unchanged

#### Scenario: Step on another action
- **WHEN** a handler calls `gband.action.close_window({ session = "work", window = 1, step = 1/4 })`
- **THEN** the call raises an error naming `step`, and window 1 stays open

