## MODIFIED Requirements

### Requirement: Server session actions
The server's `gband.action` SHALL hold one function for each session action the actions capability names, under its Lua name. Each SHALL take one target table naming `session` and the pane the action acts on as `pane`; `open_pane` SHALL instead take `session`, `band`, an optional `after` pane, an optional `program`, a command line string or a list of argument strings, and an optional `floating`, a boolean. `floating = true` SHALL open the pane floating in `band`, as the layout capability's "Open a pane" defines, and SHALL be an error together with `after`. `toggle_pane_floating` SHALL also take an optional `after` pane: a floating pane SHALL be tiled after `after`, as the floating-panes capability's "Tile a pane" defines, and as the band's first column without it. A missing session, a wrong type or an unknown field SHALL be an error at the line of the call. The actions SHALL be applied to their sessions after the callback returns, in the order called, as the session-server capability's "Session actions" defines, and SHALL tell no client to focus. An action naming a session, pane or band that no longer exists SHALL change nothing. `gband.action` SHALL be callable only in a callback.

#### Scenario: Close from a handler
- **WHEN** a `PaneOutput` handler calls `gband.action.close_pane({ session = ev.session, pane = ev.pane })` on seeing `bye`, and the pane prints `bye`
- **THEN** the pane's program receives SIGHUP and the pane leaves the layout

#### Scenario: Open without stealing focus
- **WHEN** a client focuses pane 1 and a server command opens a pane in band 1 after pane 1 running `htop`
- **THEN** every client receives a layout holding the new pane
- **AND** the client's focus stays on pane 1

#### Scenario: Float from a handler
- **WHEN** a `PaneOpened` handler calls `gband.action.toggle_pane_floating({ session = ev.session, pane = ev.pane })` for tiled pane 2, then `gband.action.open_pane({ session = ev.session, band = ev.band, floating = true })`
- **THEN** every client receives a layout whose band's floating list holds pane 2 and then the new pane

#### Scenario: Tile from the server
- **WHEN** band 1 of `work` holds columns with panes 1 and 2 and floating pane 3, and a server command calls `gband.action.toggle_pane_floating({ session = "work", pane = 3, after = 1 })`
- **THEN** band 1 holds the columns of pane 1, pane 3 and pane 2, in that order

#### Scenario: Floating with after
- **WHEN** a handler calls `gband.action.open_pane({ session = "work", band = 1, after = 1, floating = true })`
- **THEN** the call raises an error naming `after`

### Requirement: Reading structure
`gband.sessions()` SHALL return the names of the sessions in ascending byte order. `gband.session(name)` SHALL return `{ name, bands, clients }` for the session `name`, or nil for an unknown session. `bands` SHALL list its bands from the top, each `{ band, columns, floating }`, each column `{ width, full_width, panes }` from the left, and each pane `{ pane }` from the top. `floating` SHALL list the band's floating panes in its floating list order, each `{ pane, width, full_width, rows, col, row }`, where `width` and `full_width` are its box record's, `rows` is its box record's height, and `col` and `row` are its box's top-left cell as the floating-panes capability places it in the session's screen area. `clients` SHALL list the numbers of the clients attached to it in ascending order. Each call SHALL return new tables.

#### Scenario: Layout of a session
- **WHEN** the session `work` holds one band with two columns, holding panes 1 and 2, and one client is attached
- **THEN** `gband.session("work").bands[1].columns[2].panes[1].pane` is 2
- **AND** `gband.session("work").clients` holds one number

#### Scenario: Floating pane in the structure
- **WHEN** the screen area of `work` is 80×24, and band 1 holds a column with pane 1 and floating pane 2 with `col` 70, `row` 3, width 1/2, full width off and `rows` 24
- **THEN** `gband.session("work").bands[1].floating` is `{ { pane = 2, width = 0.5, full_width = false, rows = 24, col = 40, row = 0 } }`
