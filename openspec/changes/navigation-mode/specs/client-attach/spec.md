## MODIFIED Requirements

### Requirement: Key bindings
The client SHALL take its key tables, its modes and its prefix key from the configuration, as the configuration capability defines them. The client SHALL keep one active key table, which SHALL be `root` outside a key sequence and outside a mode. While `root` is active, a key bound in `root` SHALL run its binding, and the prefix key SHALL make `prefix` the active table. The client SHALL send neither to the server. Any other key while `root` is active SHALL go to the focused window when there is one, as the plugin-windows capability defines, and SHALL otherwise be sent to the focused pane.

While a table that is not a mode is active, the next key SHALL end the sequence: a key bound in that table SHALL run its binding, and any other key SHALL be discarded together with the keys that began the sequence. When the sequence ends, the active table SHALL become the table its binding entered with `gband.keymap.enter`, or `root` when it entered none.

While a mode is active, a key bound in it SHALL run its binding, and any other key SHALL be discarded. Neither SHALL be sent to the server, to the focused pane or to a window. The mode SHALL stay active after each key, until a binding enters another table with `gband.keymap.enter`, which then becomes active. Entering `root` ends the mode, and the keys that follow go to the focused window or pane again. This is interactive mode.

A binding run while `root` is active MAY also enter a table, which then becomes active. Entering a table other than `root` that has no binding SHALL be an error raised by `gband.keymap.enter`. When `prefix` holds no binding, the prefix key SHALL be handled like any other key. A reload SHALL make `root` the active table.

`gband.keymap.current_table()` SHALL return the name of the active table, and `root` while the configuration loads. Each change of the active table SHALL emit `KeyTableChanged`, as the lua-events capability defines, naming the new and the previous table. A key after which the same mode stays active SHALL emit none.

With no configuration file, the bindings SHALL be those the default configuration makes: Ctrl+Space as the prefix, no binding in `root`, `prefix` declared a mode with the label `navigation`, and these bindings in `prefix`, in this order. A binding SHALL keep navigation mode active unless its row says it returns to interactive mode. A binding whose row gives a description is a Lua function with that description, and every other binding is bound to the action its row names:

| key in navigation mode | Lua binding | action | kind | description of a function |
|---|---|---|---|---|
| `h` | `prefix h` | focus the column to the left | view | |
| `l` | `prefix l` | focus the column to the right | view | |
| `j` | `prefix j` | focus the pane below | view | |
| `k` | `prefix k` | focus the pane above | view | |
| `u` | `prefix u` | view the band below | view | |
| `i` | `prefix i` | view the band above | view | |
| `n` | `prefix n` | open a pane right of the focused pane's column, then return to interactive mode | session | `open a window` |
| `q` | `prefix q` | close the focused window: the focused float when one is focused, otherwise the focused pane | session | |
| `[` | `prefix [` | consume or expel the focused pane to the left | session | |
| `]` | `prefix ]` | consume or expel the focused pane to the right | session | |
| `r` | `prefix r` | cycle the width of the focused pane's column | session | |
| `f` | `prefix f` | toggle full width of the focused pane's column | session | |
| `-` | `prefix -` | shrink the width of the focused pane's column | session | |
| `=` | `prefix =` | grow the width of the focused pane's column | session | |
| `_` | `prefix _` | shrink the height of the focused pane | session | |
| `+` | `prefix +` | grow the height of the focused pane | session | |
| `R` | `prefix R` | reset the height of the focused pane | session | |
| `v` | `prefix v` | float or tile the focused pane | session | |
| `V` | `prefix V` | switch focus between floating and tiled panes | view | |
| Ctrl+H | `prefix ctrl+h` | move the focused pane's column, or its floating box, to the left | session | |
| Ctrl+L | `prefix ctrl+l` | move the focused pane's column, or its floating box, to the right | session | |
| Ctrl+J | `prefix ctrl+j` | move the focused pane, or its floating box, down | session | |
| Ctrl+K | `prefix ctrl+k` | move the focused pane, or its floating box, up | session | |
| Ctrl+Left | `prefix ctrl+left` | move the focused pane's column, or its floating box, to the left | session | |
| Ctrl+Right | `prefix ctrl+right` | move the focused pane's column, or its floating box, to the right | session | |
| Ctrl+Down | `prefix ctrl+down` | move the focused pane, or its floating box, down | session | |
| Ctrl+Up | `prefix ctrl+up` | move the focused pane, or its floating box, up | session | |
| `?` | `prefix ?` | open the key list, as the key-list capability defines, which returns to interactive mode | client | |
| `D` | `prefix D` | detach | client | |
| Escape | `prefix escape` | return to interactive mode | — | `interactive mode` |
| Enter | `prefix enter` | return to interactive mode | — | `interactive mode` |
| Left | `prefix left` | focus the column to the left | view | |
| Right | `prefix right` | focus the column to the right | view | |
| Down | `prefix down` | focus the pane below | view | |
| Up | `prefix up` | focus the pane above | view | |
| Ctrl+Space | `prefix prefix` | send the prefix key to the focused pane, then return to interactive mode | client | `send the prefix key` |
| any other key | — | discard the key; navigation mode stays active | — | |

A binding to an action SHALL dispatch it. A binding to a Lua function SHALL run the function, as the configuration capability defines. A view action SHALL change this client's view as the layout-view capability defines, and SHALL send nothing to the server. A session action SHALL be sent to the server as an action naming the focused pane, resolved as the actions capability defines, except close pane while a float is focused, which closes that float in the client. Open pane SHALL name the viewed band and the tiled pane this client focused most recently there, or no pane when there is none. Any other session action, and sending the prefix key, SHALL do nothing when no pane is focused. A character key SHALL match a binding by its character, Ctrl and Alt, so a `D` matches whether or not the terminal reports Shift with it.

#### Scenario: Detach
- **WHEN** the user presses Ctrl+Space then Shift+D
- **THEN** the client detaches

#### Scenario: Lowercase d does not detach
- **WHEN** the user presses Ctrl+Space then `d`
- **THEN** the client stays attached, nothing is sent to the pane, and navigation mode stays active

#### Scenario: Literal Ctrl+Space
- **WHEN** the user runs `cat -v`, presses Ctrl+Space twice, then presses Enter
- **THEN** the focused pane receives `\x00` once
- **AND** `cat -v` prints `^@` on a line of its own

#### Scenario: Literal Ctrl+A
- **WHEN** no `user/init.lua` exists and the user presses Ctrl+A at a bash prompt with text typed
- **THEN** the focused pane receives `\x01`
- **AND** bash moves the cursor to the start of the line

#### Scenario: Unbound key after the prefix
- **WHEN** the user presses Ctrl+Space, `x`, then `l` with the first of two columns focused
- **THEN** nothing is sent to either pane
- **AND** the second column is focused

#### Scenario: Repeated focus moves
- **WHEN** the viewed band holds four columns with the first focused, and the user presses Ctrl+Space, `l`, `l`, `l`
- **THEN** the fourth column is focused and no pane receives a key
- **AND** the status line's mode segment shows `navigation`

#### Scenario: Repeated resize
- **WHEN** the client's 80×24 terminal sets the screen area, the only pane sits in a column of width 1/2, and the user presses Ctrl+Space, `=`, `=`, then Escape, waits, and runs `tput cols`
- **THEN** the column's width is 7/10 and the tile is 56 columns wide
- **AND** the pane prints `54`, so the keys after Escape reached it

#### Scenario: Arrow keys move focus
- **WHEN** the second of two columns is focused and the user presses Ctrl+Space then Left
- **THEN** the first column is focused and no pane receives a key

#### Scenario: Focus back to the left
- **WHEN** the user presses Ctrl+Space, `h`, Escape, then types `echo left` and Enter, after opening a second pane
- **THEN** `left` appears in the first tile only
- **AND** the focused pane receives no Escape

#### Scenario: Enter returns to interactive mode
- **WHEN** the user presses Ctrl+Space then Enter with one pane open
- **THEN** no pane opens and the focused pane receives nothing
- **AND** `root` is the active table

#### Scenario: Open a pane
- **WHEN** one pane is focused and the user presses Ctrl+Space then `n`
- **THEN** a second tile with a shell prompt appears right of the first
- **AND** the new pane is focused and `root` is active, so `echo $GBAND_PANE` runs in it

#### Scenario: Close the focused pane
- **WHEN** two panes are open with the second focused and the user presses Ctrl+Space then `q`
- **THEN** the second tile disappears and the first pane is focused
- **AND** navigation mode stays active

#### Scenario: Another band
- **WHEN** the user presses Ctrl+Space, `u`, `n`, then Ctrl+Space, `i`
- **THEN** the client shows the first band with its original pane focused
- **AND** the second band holds the new pane

#### Scenario: Grow the column
- **WHEN** the client's 80×24 terminal sets the screen area, the only pane sits in a column of width 1/2, and the user presses Ctrl+Space, `=`, then Escape, waits, and runs `tput cols`
- **THEN** the tile is 48 columns wide
- **AND** the pane prints `46`

#### Scenario: Grow the pane's height
- **WHEN** the client's 80×25 terminal, whose status line takes one row, sets the screen area, a column holds two panes with automatic heights with the top one focused, and the user presses Ctrl+Space then `+`
- **THEN** the top tile is 14 rows high and the bottom tile is 10 rows high

#### Scenario: Direct binding acts without the prefix
- **WHEN** `user/init.lua` binds `alt+h` to `gband.action.focus_column_left`, the second of two panes is focused, and the user presses Alt+H
- **THEN** the first pane is focused
- **AND** neither pane receives the key

#### Scenario: Unbound Alt key reaches the pane
- **WHEN** `user/init.lua` binds `alt+h` and the user presses Alt+X
- **THEN** the focused pane receives `\x1bx`

#### Scenario: Another prefix key
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua` that then sets `prefix` to `"ctrl+b"`, and the user presses Ctrl+B, `q`, then Escape with two panes open
- **THEN** the focused pane closes
- **AND** pressing Ctrl+Space then sends `\x00` to the focused pane

#### Scenario: No prefix binding left
- **WHEN** `user/init.lua` makes no prefix binding and the user presses Ctrl+Space then `h`
- **THEN** the focused pane receives `\x00` and then `h`

#### Scenario: Prefix table that is not a mode
- **WHEN** `user/init.lua` binds `prefix h` to `gband.action.focus_column_left` and declares no mode, and the user presses Ctrl+Space, `h`, then `h` with the second of two columns focused
- **THEN** the first column is focused
- **AND** the second `h` reaches the focused pane

#### Scenario: Named key table
- **WHEN** `user/init.lua` binds `h` and `l` in the table `move`, and binds `prefix m` to a function that calls `gband.keymap.enter("move")`, and the user presses Ctrl+Space, `m`, then `l`, with the first of two columns focused
- **THEN** the second column is focused
- **AND** a further `l` reaches the focused pane

#### Scenario: Unbound key in a named table
- **WHEN** the user enters the table `move` and presses `x`
- **THEN** nothing is sent to the pane and `root` is active again

#### Scenario: A user mode
- **WHEN** `user/init.lua` declares `resize` a mode, binds `=` in it to `gband.action.grow_column_width` and `escape` to a function that calls `gband.keymap.enter("root")`, binds `alt+r` in `root` to a function that calls `gband.keymap.enter("resize")`, and the user presses Alt+R, `=`, `=`, Escape, then `x`
- **THEN** the focused column grew twice
- **AND** only `x` reaches the focused pane

#### Scenario: Key table change events
- **WHEN** a `KeyTableChanged` handler records each payload and the user presses Ctrl+Space, `h`, `l`, then Escape
- **THEN** the handler records `prefix` with previous `root`, then `root` with previous `prefix`, and nothing else

#### Scenario: Current table
- **WHEN** a binding in `prefix` calls `gband.keymap.current_table()`
- **THEN** it returns `prefix`

#### Scenario: Unbound key goes to the focused window
- **WHEN** a float with `keys = { j = fn }` is focused and the user presses `j`
- **THEN** `fn` runs and the focused pane receives nothing

#### Scenario: Navigation mode before the window
- **WHEN** the default configuration is in use, a float with `keys = { x = fn }` is focused, and the user presses Ctrl+Space then `x`
- **THEN** `fn` does not run, nothing reaches the focused pane, and navigation mode stays active

#### Scenario: Root binding before the window
- **WHEN** `user/init.lua` binds `alt+h` in `root` to `gband.action.focus_column_left`, a float binding `alt+h` in its `keys` is focused, and the user presses Alt+H
- **THEN** the root binding runs and the float's function does not

#### Scenario: Prefix q closes the focused float
- **WHEN** two panes are open, a float is focused, and the user presses Ctrl+Space then `q`
- **THEN** the float closes and both tiles stay

#### Scenario: Open the key list
- **WHEN** no `user/init.lua` exists and the user presses Ctrl+Space then `?`
- **THEN** a float titled `prefix keys` lists the prefix bindings and has focus
- **AND** `root` is active, so a following `j` moves the list's cursor line

#### Scenario: Float the focused pane
- **WHEN** the client's 80×24 terminal sets the screen area, the status line is off, the only pane sits in a column of width 1/2, and the user presses Ctrl+Space then `v`
- **THEN** the pane is drawn in a box spanning columns 20 to 59 and rows 0 to 23, and stays focused

#### Scenario: Switch to the tiled layer and back
- **WHEN** two panes are open, the second floats and is focused, and the user presses Ctrl+Space, `V`, Escape, then types `echo tiled` and Enter
- **THEN** `tiled` appears in the first pane only
- **AND** pressing Ctrl+Space then `V` again focuses the floating pane

#### Scenario: Move a column with Ctrl+H and Ctrl+Right
- **WHEN** the viewed band holds columns A and B with B focused, and the user presses Ctrl+Space then Ctrl+H
- **THEN** the band holds B and A, in that order, and B stays focused, and neither pane receives a key
- **AND** pressing Ctrl+Right next, still in navigation mode, restores A and B

#### Scenario: Move a pane with Ctrl+J
- **WHEN** a column holds P1 above P2 with P1 focused, and the user presses Ctrl+Space then Ctrl+J
- **THEN** the column holds P2 above P1, and P1 stays focused

#### Scenario: Move a floating pane
- **WHEN** the client's 80×24 terminal sets the screen area, a floating pane is focused with its box starting at column 20, and the user presses Ctrl+Space then Ctrl+L
- **THEN** the box is drawn from column 28
