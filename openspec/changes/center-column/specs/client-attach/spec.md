## MODIFIED Requirements

### Requirement: Key bindings
The client SHALL take its key tables and its prefix key from the configuration, as the configuration capability defines them. The client SHALL keep one active key table, which SHALL be `root` outside a key sequence. While `root` is active, a key bound in `root` SHALL run its binding, and the prefix key SHALL make `prefix` the active table. The client SHALL send neither to the server. Any other key while `root` is active SHALL go to the focused plugin window when there is one, as the plugin-windows capability defines, and SHALL otherwise be sent to the focused window. While another table is active, the next key SHALL end the sequence: a key bound in that table SHALL run its binding, and any other key SHALL be discarded together with the keys that began the sequence. When the sequence ends, the active table SHALL become the table its binding entered with `gband.keymap.enter`, or `root` when it entered none. A binding run while `root` is active MAY also enter a table, which then becomes active. Entering a table with no binding SHALL be an error raised by `gband.keymap.enter`. When `prefix` holds no binding, the prefix key SHALL be handled like any other key. A reload SHALL make `root` the active table.

`gband.keymap.current_table()` SHALL return the name of the active table, and `root` while the configuration loads. Each change of the active table SHALL emit `KeyTableChanged`, as the lua-events capability defines, naming the new and the previous table.

With no configuration file, the bindings SHALL be those the default configuration makes: Ctrl+Space as the prefix, no binding in `root`, and these bindings in `prefix`:

| key after Ctrl+Space | Lua binding | action | kind |
|---|---|---|---|
| `h` | `prefix h` | focus the column to the left | view |
| `l` | `prefix l` | focus the column to the right | view |
| `j` | `prefix j` | focus the window below | view |
| `k` | `prefix k` | focus the window above | view |
| `u` | `prefix u` | view the band below | view |
| `i` | `prefix i` | view the band above | view |
| `c` | `prefix c` | center the focused column | view |
| Enter | `prefix enter` | open a window right of the focused window's column | session |
| `q` | `prefix q` | close the focused window | session |
| `[` | `prefix [` | consume or expel the focused window to the left | session |
| `]` | `prefix ]` | consume or expel the focused window to the right | session |
| `r` | `prefix r` | cycle the width of the focused window's column | session |
| `f` | `prefix f` | toggle full width of the focused window's column | session |
| `-` | `prefix -` | shrink the width of the focused window's column | session |
| `=` | `prefix =` | grow the width of the focused window's column | session |
| `_` | `prefix _` | shrink the height of the focused window | session |
| `+` | `prefix +` | grow the height of the focused window | session |
| `R` | `prefix R` | reset the height of the focused window | session |
| `D` | `prefix D` | detach | client |
| Ctrl+Space | `prefix prefix` | send the prefix key to the focused window | client |
| any other key | — | discard both keys | — |

A binding to an action SHALL dispatch it. A binding to a Lua function SHALL run the function, as the configuration capability defines. A view action SHALL change this client's view as the layout-view capability defines, and SHALL send nothing to the server. A session action SHALL be sent to the server as an action naming the focused window. Open window SHALL name the viewed band and the focused window, or no window when none is focused. Any other session action, and sending the prefix key, SHALL do nothing when no window is focused. A character key SHALL match a binding by its character, Ctrl and Alt, so a `D` matches whether or not the terminal reports Shift with it.

#### Scenario: Detach
- **WHEN** the user presses Ctrl+Space then Shift+D
- **THEN** the client detaches

#### Scenario: Lowercase d does not detach
- **WHEN** the user presses Ctrl+Space then `d`
- **THEN** the client stays attached and nothing is sent to the window

#### Scenario: Literal Ctrl+Space
- **WHEN** the user runs `cat -v`, presses Ctrl+Space twice, then presses Enter
- **THEN** the focused window receives `\x00` once
- **AND** `cat -v` prints `^@` on a line of its own

#### Scenario: Literal Ctrl+A
- **WHEN** no `user/init.lua` exists and the user presses Ctrl+A at a bash prompt with text typed
- **THEN** the focused window receives `\x01`
- **AND** bash moves the cursor to the start of the line

#### Scenario: Unbound key after the prefix
- **WHEN** the user presses Ctrl+Space then `x`
- **THEN** nothing is sent to the window

#### Scenario: Open a window
- **WHEN** one window is focused and the user presses Ctrl+Space then Enter
- **THEN** a second tile with a shell prompt appears right of the first
- **AND** the new window is focused, so `echo $GBAND_WINDOW` runs in it

#### Scenario: Focus back to the left
- **WHEN** the user has opened a second window and presses Ctrl+Space then `h`, then types `echo left` and Enter
- **THEN** `left` appears in the first tile only

#### Scenario: Close the focused window
- **WHEN** two windows are open with the second focused and the user presses Ctrl+Space then `q`
- **THEN** the second tile disappears and the first window is focused

#### Scenario: Another band
- **WHEN** the user presses Ctrl+Space then `u`, then Ctrl+Space then Enter, then Ctrl+Space then `i`
- **THEN** the client shows the first band with its original window focused
- **AND** the second band holds the new window

#### Scenario: Grow the column
- **WHEN** the client's 80×24 terminal sets the screen area, the only window sits in a column of width 1/2, and the user presses Ctrl+Space then `=`, waits, and runs `tput cols`
- **THEN** the tile is 48 columns wide
- **AND** the window prints `46`

#### Scenario: Grow the window's height
- **WHEN** the client's 80×25 terminal, whose status line takes one row, sets the screen area, a column holds two windows with automatic heights with the top one focused, and the user presses Ctrl+Space then `+`
- **THEN** the top tile is 14 rows high and the bottom tile is 10 rows high

#### Scenario: Center the column
- **WHEN** the client's terminal is 80 columns wide, the only window sits in a column 40 cells wide at strip position 0, and the user presses Ctrl+Space then `c`
- **THEN** the tile is drawn from the terminal's column 20
- **AND** the 20 columns left of it are drawn empty

#### Scenario: Direct binding acts without the prefix
- **WHEN** `user/init.lua` binds `alt+h` to `gband.action.focus_column_left`, the second of two windows is focused, and the user presses Alt+H
- **THEN** the first window is focused
- **AND** neither window receives the key

#### Scenario: Unbound Alt key reaches the window
- **WHEN** `user/init.lua` binds `alt+h` and the user presses Alt+X
- **THEN** the focused window receives `\x1bx`

#### Scenario: Another prefix key
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua` that then sets `prefix` to `"ctrl+b"`, and the user presses Ctrl+B then `q` with two windows open
- **THEN** the focused window closes
- **AND** pressing Ctrl+Space sends `\x00` to the focused window

#### Scenario: No prefix binding left
- **WHEN** `user/init.lua` makes no prefix binding and the user presses Ctrl+Space then `h`
- **THEN** the focused window receives `\x00` and then `h`

#### Scenario: Named key table
- **WHEN** `user/init.lua` binds `h` and `l` in the table `move`, and binds `prefix m` to a function that calls `gband.keymap.enter("move")`, and the user presses Ctrl+Space, `m`, then `l`, with the first of two columns focused
- **THEN** the second column is focused
- **AND** a further `l` reaches the focused window

#### Scenario: Unbound key in a named table
- **WHEN** the user enters the table `move` and presses `x`
- **THEN** nothing is sent to the window and `root` is active again

#### Scenario: Key table change events
- **WHEN** a `KeyTableChanged` handler records each payload and the user presses Ctrl+Space then `h`
- **THEN** the handler records `prefix` with previous `root`, then `root` with previous `prefix`

#### Scenario: Current table
- **WHEN** a binding in `prefix` calls `gband.keymap.current_table()`
- **THEN** it returns `prefix`

#### Scenario: Unbound key goes to the focused plugin window
- **WHEN** a floating plugin window with `keys = { j = fn }` is focused and the user presses `j`
- **THEN** `fn` runs and the focused window receives nothing

#### Scenario: Root binding before the plugin window
- **WHEN** `user/init.lua` binds `alt+h` in `root` to `gband.action.focus_column_left`, a floating plugin window binding `alt+h` in its `keys` is focused, and the user presses Alt+H
- **THEN** the root binding runs and the floating plugin window's function does not

