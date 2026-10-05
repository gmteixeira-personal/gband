## MODIFIED Requirements

### Requirement: Key bindings
The client SHALL take its key tables and its prefix key from the configuration, as the configuration capability defines them. The client SHALL keep one active key table, which SHALL be `root` outside a key sequence. While `root` is active, a key bound in `root` SHALL run its binding, and the prefix key SHALL make `prefix` the active table. The client SHALL send neither to the server. Any other key while `root` is active SHALL be sent to the focused pane. While another table is active, the next key SHALL end the sequence: a key bound in that table SHALL run its binding, and any other key SHALL be discarded together with the keys that began the sequence. When the sequence ends, the active table SHALL become the table its binding entered with `gband.keymap.enter`, or `root` when it entered none. A binding run while `root` is active MAY also enter a table, which then becomes active. Entering a table with no binding SHALL be an error raised by `gband.keymap.enter`. When `prefix` holds no binding, the prefix key SHALL be sent to the focused pane like any other key. A reload SHALL make `root` the active table.

`gband.keymap.current_table()` SHALL return the name of the active table, and `root` while the configuration loads. Each change of the active table SHALL emit `KeyTableChanged`, as the lua-events capability defines, naming the new and the previous table.

With no configuration file, the bindings SHALL be those the default configuration makes: Ctrl+Space as the prefix, no binding in `root`, and these bindings in `prefix`:

| key after Ctrl+Space | Lua binding | action | kind |
|---|---|---|---|
| `h` | `prefix h` | focus the column to the left | view |
| `l` | `prefix l` | focus the column to the right | view |
| `j` | `prefix j` | focus the pane below | view |
| `k` | `prefix k` | focus the pane above | view |
| `u` | `prefix u` | view the band below | view |
| `i` | `prefix i` | view the band above | view |
| Enter | `prefix enter` | open a pane right of the focused pane's column | session |
| `q` | `prefix q` | close the focused pane | session |
| `[` | `prefix [` | consume or expel the focused pane to the left | session |
| `]` | `prefix ]` | consume or expel the focused pane to the right | session |
| `r` | `prefix r` | cycle the width of the focused pane's column | session |
| `f` | `prefix f` | toggle full width of the focused pane's column | session |
| `-` | `prefix -` | shrink the width of the focused pane's column | session |
| `=` | `prefix =` | grow the width of the focused pane's column | session |
| `_` | `prefix _` | shrink the height of the focused pane | session |
| `+` | `prefix +` | grow the height of the focused pane | session |
| `R` | `prefix R` | reset the height of the focused pane | session |
| `D` | `prefix D` | detach | client |
| Ctrl+Space | `prefix prefix` | send the prefix key to the focused pane | client |
| any other key | — | discard both keys | — |

A binding to an action SHALL dispatch it. A binding to a Lua function SHALL run the function, as the configuration capability defines. A view action SHALL change this client's view as the layout-view capability defines, and SHALL send nothing to the server. A session action SHALL be sent to the server as an action naming the focused pane. Open pane SHALL name the viewed band and the focused pane, or no pane when none is focused. Any other session action, and sending the prefix key, SHALL do nothing when no pane is focused. A character key SHALL match a binding by its character, Ctrl and Alt, so a `D` matches whether or not the terminal reports Shift with it.

#### Scenario: Detach
- **WHEN** the user presses Ctrl+Space then Shift+D
- **THEN** the client detaches

#### Scenario: Lowercase d does not detach
- **WHEN** the user presses Ctrl+Space then `d`
- **THEN** the client stays attached and nothing is sent to the pane

#### Scenario: Literal Ctrl+Space
- **WHEN** the user runs `cat -v`, presses Ctrl+Space twice, then presses Enter
- **THEN** the focused pane receives `\x00` once
- **AND** `cat -v` prints `^@` on a line of its own

#### Scenario: Literal Ctrl+A
- **WHEN** no `user/init.lua` exists and the user presses Ctrl+A at a bash prompt with text typed
- **THEN** the focused pane receives `\x01`
- **AND** bash moves the cursor to the start of the line

#### Scenario: Unbound key after the prefix
- **WHEN** the user presses Ctrl+Space then `x`
- **THEN** nothing is sent to the pane

#### Scenario: Open a pane
- **WHEN** one pane is focused and the user presses Ctrl+Space then Enter
- **THEN** a second tile with a shell prompt appears right of the first
- **AND** the new pane is focused, so `echo $GBAND_PANE` runs in it

#### Scenario: Focus back to the left
- **WHEN** the user has opened a second pane and presses Ctrl+Space then `h`, then types `echo left` and Enter
- **THEN** `left` appears in the first tile only

#### Scenario: Close the focused pane
- **WHEN** two panes are open with the second focused and the user presses Ctrl+Space then `q`
- **THEN** the second tile disappears and the first pane is focused

#### Scenario: Another workspace
- **WHEN** the user presses Ctrl+Space then `u`, then Ctrl+Space then Enter, then Ctrl+Space then `i`
- **THEN** the client shows the first band with its original pane focused
- **AND** the second band holds the new pane

#### Scenario: Grow the column
- **WHEN** the client's 80×24 terminal sets the screen area, the only pane sits in a column of width 1/2, and the user presses Ctrl+Space then `=`, waits, and runs `tput cols`
- **THEN** the tile is 48 columns wide
- **AND** the pane prints `46`

#### Scenario: Grow the pane's height
- **WHEN** the client's 80×24 terminal sets the screen area, a column holds two panes with automatic heights with the top one focused, and the user presses Ctrl+Space then `+`
- **THEN** the top tile is 14 rows high and the bottom tile is 10 rows high

#### Scenario: Direct binding acts without the prefix
- **WHEN** `user/init.lua` binds `alt+h` to `gband.action.focus_column_left`, the second of two panes is focused, and the user presses Alt+H
- **THEN** the first pane is focused
- **AND** neither pane receives the key

#### Scenario: Unbound Alt key reaches the pane
- **WHEN** `user/init.lua` binds `alt+h` and the user presses Alt+X
- **THEN** the focused pane receives `\x1bx`

#### Scenario: Another prefix key
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua` that then sets `prefix` to `"ctrl+b"`, and the user presses Ctrl+B then `q` with two panes open
- **THEN** the focused pane closes
- **AND** pressing Ctrl+Space sends `\x00` to the focused pane

#### Scenario: No prefix binding left
- **WHEN** `user/init.lua` makes no prefix binding and the user presses Ctrl+Space then `h`
- **THEN** the focused pane receives `\x00` and then `h`

#### Scenario: Named key table
- **WHEN** `user/init.lua` binds `h` and `l` in the table `move`, and binds `prefix m` to a function that calls `gband.keymap.enter("move")`, and the user presses Ctrl+Space, `m`, then `l`, with the first of two columns focused
- **THEN** the second column is focused
- **AND** a further `l` reaches the focused pane

#### Scenario: Unbound key in a named table
- **WHEN** the user enters the table `move` and presses `x`
- **THEN** nothing is sent to the pane and `root` is active again

#### Scenario: Key table change events
- **WHEN** a `KeyTableChanged` handler records each payload and the user presses Ctrl+Space then `h`
- **THEN** the handler records `prefix` with previous `root`, then `root` with previous `prefix`

#### Scenario: Current table
- **WHEN** a binding in `prefix` calls `gband.keymap.current_table()`
- **THEN** it returns `prefix`
