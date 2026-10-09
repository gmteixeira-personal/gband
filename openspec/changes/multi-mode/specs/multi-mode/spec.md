## Purpose

Defines multi mode, a per-client state in which the keys and pastes that the user types reach every window of the viewed band instead of the focused window alone, and the action that turns it on and off.

## ADDED Requirements

### Requirement: Multi mode state
Each client SHALL keep one multi mode state, on or off, of its own. It SHALL be off when the client attaches. Only the client action `toggle_multi` SHALL change it, except a reload, which SHALL turn it off. A reload is any load of a new configuration, as the configuration capability defines, including the forced reload. Multi mode SHALL be independent of the active key table: entering or leaving a table or a mode SHALL NOT change it.

`toggle_multi` SHALL turn multi mode on when it is off, and off when it is on. It SHALL send nothing to the server, and SHALL change neither the view nor the active key table.

Turning multi mode on or off in one client SHALL change nothing in another client attached to the same session.

#### Scenario: Off at attach
- **WHEN** a client attaches and a binding function calls `gband.view()`
- **THEN** the result holds no `multi`

#### Scenario: Toggle twice
- **WHEN** a binding function dispatches `gband.action.toggle_multi()`, and a later binding function dispatches it again
- **THEN** multi mode is on after the first and off after the second
- **AND** the active key table and the focused window are those before the first

#### Scenario: Navigation mode keeps multi mode
- **WHEN** the modal key style is in use, multi mode is on, and the user presses Ctrl+Space, `l`, then Escape
- **THEN** the column to the right is focused and multi mode is still on

#### Scenario: Reload turns it off
- **WHEN** multi mode is on and the configuration reloads
- **THEN** multi mode is off

#### Scenario: Another client is unaffected
- **WHEN** two clients are attached to one session, viewing the same band of two windows, and the first turns multi mode on, and the user types `echo hi` and Enter in the second client
- **THEN** only the window focused in the second client prints `hi`

### Requirement: Input to every window of the band
While multi mode is on, each key and each paste that the client would send to the focused window, as the client-attach capability's "Input to the server" defines, SHALL instead be sent once to each window of the band that the client views when the key or paste arrives. The windows of a band are its tiled windows and its floating windows, including those this client has minimized. The client SHALL send to them in layout order: the columns from left to right with their windows from top to bottom, then the floating windows in the order of the band's floating list. Each key SHALL be encoded for each window as the input-encoding capability defines for that window, so two windows in different input modes may receive different bytes for the same key.

This SHALL hold whether or not a window is focused: a band with windows and no focused window SHALL still receive the input. A key or paste SHALL be dropped when the viewed band holds no window.

While a plugin window is focused, it SHALL take the keys and the pastes, as the plugin-windows capability defines, and no window SHALL receive them. A key bound in the active key table SHALL run its binding and reach no window, as with multi mode off. Sending the prefix key, by the client action `send_prefix`, SHALL send it to every window of the viewed band while multi mode is on.

Mouse input SHALL NOT change with multi mode: a press, release, drag or wheel step SHALL reach only the window that it reaches with multi mode off. Input that a Lua function sends to a named window, as the lua-control capability's "Input to a named window" defines, SHALL reach only that window.

Viewing another band while multi mode is on SHALL make the input that follows reach the windows of the newly viewed band.

#### Scenario: Typing reaches every window
- **WHEN** the viewed band holds two columns of one shell window each and a floating shell window, multi mode is on, and the user types `echo hi` and Enter
- **THEN** each of the three windows prints `hi`

#### Scenario: Other bands receive nothing
- **WHEN** band 1 holds windows 1 and 2, band 2 holds window 3, the client views band 1 with multi mode on, and the user types `echo hi` and Enter
- **THEN** windows 1 and 2 print `hi`, and the screen of window 3 is unchanged

#### Scenario: Paste reaches every window
- **WHEN** the viewed band holds two shell windows, multi mode is on, and the user pastes `echo pasted` and presses Enter
- **THEN** both windows print `pasted`

#### Scenario: Minimized window receives the input
- **WHEN** the viewed band holds floating windows 1 and 2, this client has minimized window 1, multi mode is on, and the user types `echo hi` and Enter
- **THEN** window 2 prints `hi`, and when the user restores window 1 it shows `hi` too

#### Scenario: Band change moves the input
- **WHEN** the modal key style is in use, band 1 holds windows 1 and 2, band 2 holds windows 3 and 4, multi mode is on while the client views band 1, and the user presses Ctrl+Space, `u`, Escape, then types `echo hi` and Enter
- **THEN** windows 3 and 4 print `hi`, and the screens of windows 1 and 2 are unchanged

#### Scenario: Plugin window takes the keys
- **WHEN** the viewed band holds two shell windows, multi mode is on, and the user opens the Lua prompt and types `1`
- **THEN** the prompt shows `1` and neither window receives a key

#### Scenario: Prefix key to every window
- **WHEN** the modal key style is in use, the viewed band holds two windows running `cat -v`, multi mode is on, and the user presses Ctrl+Space twice, then Enter
- **THEN** each window prints `^@` on a line of its own

#### Scenario: Mouse reaches one window
- **WHEN** the viewed band holds two windows that report the mouse, multi mode is on, and the user clicks the second window
- **THEN** only the second window receives the press and the release

#### Scenario: Empty band
- **WHEN** the client views the empty band, multi mode is on, and the user types `ls`
- **THEN** nothing is sent to the server

### Requirement: Multi mode action
The built-in client action `toggle_multi` SHALL have the description `toggle multi mode`. Dispatching it from a binding, from `gband.action.toggle_multi()` or from any other source SHALL toggle multi mode as "Multi mode state" defines. It SHALL take no target.

#### Scenario: Listed with its description
- **WHEN** a binding function reads `gband.action.list()`
- **THEN** it holds `toggle_multi` with the description `toggle multi mode`

#### Scenario: Bound by a user
- **WHEN** `user/init.lua` binds `alt+m` in `root` to `gband.action.toggle_multi`, and the user presses Alt+M, then types `echo hi` and Enter, with two windows in the viewed band
- **THEN** both windows print `hi`
