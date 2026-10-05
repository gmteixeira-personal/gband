## MODIFIED Requirements

### Requirement: Built-in events
The client SHALL emit these events, and no other built-in events:

| event | payload | emitted when |
|---|---|---|
| `Attached` | `session`: the session's name | once, after the client attaches and before it handles its first key |
| `FocusChanged` | `pane`, `previous`: pane numbers, nil when no pane | the focused pane differs from before a server message, a dispatched action or a resize was handled |
| `BandChanged` | `band`, `previous`: band numbers | the viewed band differs from before a server message, a dispatched action or a resize was handled |
| `PaneOpened` | `pane`, `band` | a layout holds a pane the client's previous layout did not |
| `PaneClosed` | `pane`, `band`: the band the pane was in | the client's previous layout held a pane a new layout does not |
| `LayoutChanged` | empty | a layout the client receives differs from the client's previous layout in its bands, columns, panes or widths |
| `TerminalResized` | `cols`, `rows` | the client's terminal changes size |
| `ConfigReloaded` | empty | a reload succeeded and the new configuration is in use, to the handlers of the new configuration |
| `KeyTableChanged` | `table`, `previous`: key table names | the active key table changes, as the client-attach capability defines |
| `HighlightChanged` | `group`: the group's name | a group's settings change after the configuration has loaded, as the highlights capability defines |
| `ColorschemeChanged` | `name`, `previous`: colorscheme names | a colorscheme loads after the configuration has loaded, as the colorschemes capability defines |

The first layout after attaching SHALL emit no `PaneOpened` and no `LayoutChanged`, and establishing the client's first view SHALL emit neither `FocusChanged` nor `BandChanged`. A layout that opens or closes a pane SHALL emit `LayoutChanged` after its `PaneOpened` and `PaneClosed` events.

#### Scenario: Focus change
- **WHEN** panes 1 and 2 are open with pane 2 focused and the user focuses the column to the left
- **THEN** `FocusChanged` runs with `pane` 1 and `previous` 2

#### Scenario: Band change
- **WHEN** the user views the band below the first band
- **THEN** `BandChanged` runs with the second band's number as `band` and the first's as `previous`

#### Scenario: Pane opened and closed
- **WHEN** the user opens a pane and then closes it
- **THEN** `PaneOpened` runs once naming the new pane and its band, then `PaneClosed` runs once naming it

#### Scenario: Layout change without a pane change
- **WHEN** two panes sit in two columns and the user consumes the focused pane into the column to its left
- **THEN** `LayoutChanged` runs once, and neither `PaneOpened` nor `PaneClosed` runs

#### Scenario: Nothing at attach
- **WHEN** a client attaches to a session holding three panes
- **THEN** `Attached` runs once and no `PaneOpened`, `LayoutChanged`, `FocusChanged` or `BandChanged` runs

#### Scenario: Reload
- **WHEN** `user/init.lua` registers a `ConfigReloaded` handler and the user saves it unchanged
- **THEN** the handler registered by the reloaded file runs once
