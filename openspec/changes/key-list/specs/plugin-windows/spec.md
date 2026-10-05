## MODIFIED Requirements

### Requirement: Keys in a focused window
While a window is focused, every key that the key bindings would send to the focused pane SHALL go to the window instead, and SHALL NOT be sent to the server. Every paste SHALL be discarded. A key that matches an entry of the window's `keys`, as a key matches a binding, SHALL run that entry's function with the window's number, as a callback that belongs to the window's plugin. A key that matches no entry SHALL take its default, when it has one:

| key | default |
|---|---|
| Up or `k`, Down or `j` | move the cursor line up or down one line with `cursorline` on, otherwise scroll by one line |
| PageUp, PageDown | scroll by the content area's height, and move the cursor line by as many lines with `cursorline` on |
| Home, End | show the first or last line, and make it the cursor line with `cursorline` on |
| `q`, Escape | close the window when it is a float |

Any other key SHALL be discarded.

#### Scenario: Key entry runs
- **WHEN** a focused float has `keys = { enter = fn }` and `cursorline = true`, its cursor is on line 3, and the user presses Enter
- **THEN** `fn` runs with the window's number, and `gband.win.info(win).cursor` is 3 inside it

#### Scenario: Default scroll
- **WHEN** a focused float without `cursorline` shows lines 1 to 10 of 25 and the user presses Down
- **THEN** it shows lines 2 to 11

#### Scenario: j and k scroll like the arrow keys
- **WHEN** a focused float without `cursorline` shows lines 1 to 10 of 25 and the user presses `j`, then `j`, then `k`
- **THEN** it shows lines 2 to 11, then lines 3 to 12, then lines 2 to 11

#### Scenario: Escape closes a float
- **WHEN** a focused float binds no `escape` and the user presses Escape
- **THEN** the float closes and the focused pane receives nothing

#### Scenario: Bindings still win
- **WHEN** a float with `keys = { h = fn }` is focused over the second of two columns and the user presses Ctrl+Space then `h`
- **THEN** the first column is focused and `fn` does not run

#### Scenario: q closes a float
- **WHEN** a focused float binds no `q` and the user presses `q`
- **THEN** the float closes, its `on_close` runs, and the focused pane receives nothing

#### Scenario: A keys entry for q wins
- **WHEN** a focused float has `keys = { q = fn }` and the user presses `q`
- **THEN** `fn` runs and the float stays open

#### Scenario: Prefix q closes the focused float
- **WHEN** a float is focused over pane 1 and the user presses Ctrl+Space then `q`
- **THEN** the float closes, pane 1 stays open and focused, and the float's `keys` functions do not run

#### Scenario: q in a pane window
- **WHEN** a pane window is focused, binds no `q`, and the user presses `q`
- **THEN** the key is discarded and the pane window stays open

#### Scenario: Unmatched key
- **WHEN** a float is focused and the user types `x`
- **THEN** nothing reaches the focused pane

### Requirement: Closing windows
`gband.win.close(win)` SHALL close the window. For a pane window, it SHALL also dispatch close pane naming its plugin pane, or close that pane as soon as the server reports it when the pane is not yet known. Closing a window that is not open SHALL do nothing. Close pane resolved against the view while a float is focused SHALL close that float the same way, as the actions capability defines. A window SHALL also close when the plugin it belongs to is marked failed. A reload SHALL close every window and every plugin pane this client opened, before the new configuration's `ConfigReloaded` handlers run.

#### Scenario: Close a float
- **WHEN** a binding function closes the focused float
- **THEN** the float is no longer drawn and no float is focused

#### Scenario: Close a pane window
- **WHEN** a binding function closes a pane window whose plugin pane is pane 3
- **THEN** pane 3 leaves the layout

#### Scenario: Close pane on a focused float
- **WHEN** a focused float's `on_close` records its argument and a binding function calls `gband.action.close_pane()`
- **THEN** the float is no longer drawn, `on_close` runs with its number, and no pane closes

#### Scenario: Reload
- **WHEN** a float and a pane window are open and the user saves `user/init.lua`
- **THEN** after the reload no window is open, the plugin pane has left the layout, and no `on_close` has run
