## MODIFIED Requirements

### Requirement: Keys in a focused plugin window
While a plugin window is focused, every key that the key bindings would send to the focused window SHALL go to the plugin window instead, and SHALL NOT be sent to the server. Every paste SHALL be discarded. A key that matches an entry of the plugin window's `keys`, as a key matches a binding, SHALL run that entry's function with the plugin window's number, as a callback that belongs to the plugin window's plugin. A key that matches no entry SHALL take its default, when it has one:

| key | default |
|---|---|
| Up or `k`, Down or `j` | move the cursor line up or down one line with `cursorline` on, otherwise scroll by one line |
| PageUp, PageDown | scroll by the content area's height, and move the cursor line by as many lines with `cursorline` on |
| Home, End | show the first or last line, and make it the cursor line with `cursorline` on |
| `q`, Escape | close the plugin window when it is a floating plugin window |

Any other key SHALL be discarded.

#### Scenario: Key entry runs
- **WHEN** a focused floating plugin window has `keys = { enter = fn }` and `cursorline = true`, its cursor is on line 3, and the user presses Enter
- **THEN** `fn` runs with the plugin window's number, and `gband.win.info(win).cursor` is 3 inside it

#### Scenario: Default scroll
- **WHEN** a focused floating plugin window without `cursorline` shows lines 1 to 10 of 25 and the user presses Down
- **THEN** it shows lines 2 to 11

#### Scenario: j and k scroll like the arrow keys
- **WHEN** a focused floating plugin window without `cursorline` shows lines 1 to 10 of 25 and the user presses `j`, then `j`, then `k`
- **THEN** it shows lines 2 to 11, then lines 3 to 12, then lines 2 to 11

#### Scenario: Escape closes a floating plugin window
- **WHEN** a focused floating plugin window binds no `escape` and the user presses Escape
- **THEN** the floating plugin window closes and the focused window receives nothing

#### Scenario: Bindings still win
- **WHEN** a floating plugin window with `keys = { h = fn }` is focused over the second of two columns and the user presses Ctrl+Space then `h`
- **THEN** the first column is focused and `fn` does not run

#### Scenario: q closes a floating plugin window
- **WHEN** a focused floating plugin window binds no `q` and the user presses `q`
- **THEN** the floating plugin window closes, its `on_close` runs, and the focused window receives nothing

#### Scenario: A keys entry for q wins
- **WHEN** a focused floating plugin window has `keys = { q = fn }` and the user presses `q`
- **THEN** `fn` runs and the floating plugin window stays open

#### Scenario: Prefix q closes the focused floating plugin window
- **WHEN** a floating plugin window is focused over window 1 and the user presses Ctrl+Space then `q`
- **THEN** the floating plugin window closes, window 1 stays open and focused, and the floating plugin window's `keys` functions do not run

#### Scenario: q in a tiled plugin window
- **WHEN** a tiled plugin window is focused, binds no `q`, and the user presses `q`
- **THEN** the key is discarded and the tiled plugin window stays open

#### Scenario: Unmatched key
- **WHEN** a floating plugin window is focused and the user types `x`
- **THEN** nothing reaches the focused window

### Requirement: Closing plugin windows
`gband.win.close(win)` SHALL close the plugin window. For a tiled plugin window, it SHALL also dispatch close window naming its drawn window, or close that window as soon as the server reports it when the window is not yet known. Closing a plugin window that is not open SHALL do nothing. Close window resolved against the view while a floating plugin window is focused SHALL close that floating plugin window the same way, as the actions capability defines. A plugin window SHALL also close when the plugin it belongs to is marked failed. A reload SHALL close every plugin window and every drawn window this client opened, before the new configuration's `ConfigReloaded` handlers run.

#### Scenario: Close a floating plugin window
- **WHEN** a binding function closes the focused floating plugin window
- **THEN** the floating plugin window is no longer drawn and no floating plugin window is focused

#### Scenario: Close a tiled plugin window
- **WHEN** a binding function closes a tiled plugin window whose drawn window is window 3
- **THEN** window 3 leaves the layout

#### Scenario: Close window on a focused floating plugin window
- **WHEN** a focused floating plugin window's `on_close` records its argument and a binding function calls `gband.action.close_window()`
- **THEN** the floating plugin window is no longer drawn, `on_close` runs with its number, and no window closes

#### Scenario: Reload
- **WHEN** a floating plugin window and a tiled plugin window are open and the user saves `user/init.lua`
- **THEN** after the reload no plugin window is open, the drawn window has left the layout, and no `on_close` has run
