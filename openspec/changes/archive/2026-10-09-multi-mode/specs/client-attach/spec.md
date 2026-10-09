## MODIFIED Requirements

### Requirement: Key bindings
The client SHALL take its key tables, its modes and its prefix key from the configuration, as the configuration capability defines them. The client SHALL keep one active key table, which SHALL be `root` outside a key sequence and outside a mode. While `root` is active, a key bound in `root` SHALL run its binding, and the prefix key SHALL make `prefix` the active table. The client SHALL send neither to the server. Any other key while `root` is active SHALL go to the focused plugin window when there is one, as the plugin-windows capability defines, and SHALL otherwise be sent to the focused window, or, while multi mode is on, to every window of the viewed band, as the multi-mode capability defines.

While a table that is not a mode is active, the next key SHALL end the sequence: a key bound in that table SHALL run its binding, and any other key SHALL be discarded together with the keys that began the sequence. When the sequence ends, the active table SHALL become the table its binding entered with `gband.keymap.enter`, or `root` when it entered none.

While a mode is active, a key bound in it SHALL run its binding, and any other key SHALL be discarded. Neither SHALL be sent to the server, to the focused window or to a plugin window. The mode SHALL stay active after each key, until a binding enters another table with `gband.keymap.enter`, which then becomes active. Entering `root` ends the mode, and the keys that follow go to the focused plugin window or window again. This is interactive mode.

A binding run while `root` is active MAY also enter a table, which then becomes active. Entering a table other than `root` that has no binding SHALL be an error raised by `gband.keymap.enter`. When `prefix` holds no binding, the prefix key SHALL be handled like any other key. A reload SHALL make `root` the active table.

`gband.keymap.current_table()` SHALL return the name of the active table, and `root` while the configuration loads. Each change of the active table SHALL emit `KeyTableChanged`, as the lua-events capability defines, naming the new and the previous table. A key after which the same mode stays active SHALL emit none.

With no configuration file, the bindings SHALL be those the default configuration makes. They SHALL follow the key style that the key-style capability's `gband.keystyle.use()` selects: the saved key style, or the modal key style when none is saved.

With the modal key style, the bindings SHALL be: Ctrl+Space as the prefix, no binding in `root`, `prefix` declared a mode with the label `navigation`, and these bindings in `prefix`, in this order. A binding SHALL keep navigation mode active unless its row says it returns to interactive mode. A binding whose row gives a description is a Lua function with that description, and every other binding is bound to the action its row names:

| key in navigation mode | Lua binding | action | kind | description of a function |
|---|---|---|---|---|
| `h` | `prefix h` | focus the column to the left | view | |
| `l` | `prefix l` | focus the column to the right | view | |
| `j` | `prefix j` | focus the window below | view | |
| `k` | `prefix k` | focus the window above | view | |
| `u` | `prefix u` | view the band below | view | |
| `i` | `prefix i` | view the band above | view | |
| `c` | `prefix c` | center the focused column, or the focused floating window | view | |
| `n` | `prefix n` | open a window right of the focused window's column, then return to interactive mode when the `I on new` setting is on, as the settings capability defines | session | `open a window` |
| `q` | `prefix q` | close the focused floating plugin window when one is focused, otherwise the focused window | session | |
| `[` | `prefix [` | consume or expel the focused window to the left | session | |
| `]` | `prefix ]` | consume or expel the focused window to the right | session | |
| `r` | `prefix r` | cycle the width of the focused window's column | session | |
| `f` | `prefix f` | toggle full width of the focused window's column | session | |
| `-` | `prefix -` | shrink the width of the focused window's column | session | |
| `=` | `prefix =` | grow the width of the focused window's column | session | |
| `_` | `prefix _` | shrink the height of the focused window | session | |
| `+` | `prefix +` | grow the height of the focused window | session | |
| `R` | `prefix R` | reset the height of the focused window | session | |
| `v` | `prefix v` | float or tile the focused window | session | |
| `V` | `prefix V` | switch focus between floating and tiled windows | view | |
| Ctrl+H | `prefix ctrl+h` | move the focused window's column, or its floating box, to the left | session | |
| Ctrl+L | `prefix ctrl+l` | move the focused window's column, or its floating box, to the right | session | |
| Ctrl+J | `prefix ctrl+j` | move the focused window, or its floating box, down | session | |
| Ctrl+K | `prefix ctrl+k` | move the focused window, or its floating box, up | session | |
| Ctrl+Left | `prefix ctrl+left` | move the focused window's column, or its floating box, to the left | session | |
| Ctrl+Right | `prefix ctrl+right` | move the focused window's column, or its floating box, to the right | session | |
| Ctrl+Down | `prefix ctrl+down` | move the focused window, or its floating box, down | session | |
| Ctrl+Up | `prefix ctrl+up` | move the focused window, or its floating box, up | session | |
| `?` | `prefix ?` | open the key list, as the key-list capability defines, which returns to interactive mode | client | |
| `:` | `prefix :` | open the Lua prompt, as the lua-prompt capability defines, which returns to interactive mode | client | |
| `N` | `prefix N` | rename the focused window, as the window-names capability's "Rename prompt" defines, which returns to interactive mode | client | |
| `s` | `prefix s` | open the settings window, as the settings capability defines, which returns to interactive mode | client | `settings` |
| `!` | `prefix !` | reload the configuration, as the configuration capability's "Forced reload" defines, which returns to interactive mode | client | |
| `m` | `prefix m` | toggle multi mode, as the multi-mode capability defines, then return to interactive mode | client | `multi mode` |
| `D` | `prefix D` | detach | client | |
| Escape | `prefix escape` | return to interactive mode | — | `interactive mode` |
| Enter | `prefix enter` | return to interactive mode | — | `interactive mode` |
| Left | `prefix left` | focus the column to the left | view | |
| Right | `prefix right` | focus the column to the right | view | |
| Down | `prefix down` | focus the window below | view | |
| Up | `prefix up` | focus the window above | view | |
| Ctrl+Space | `prefix prefix` | send the prefix key to the focused window, or to every window of the viewed band while multi mode is on, then return to interactive mode | client | `send the prefix key` |
| any other key | — | discard the key; navigation mode stays active | — | |

With the direct key style, the bindings SHALL be: Ctrl+Space as the prefix, no binding in `root`, no mode, and in `prefix` every row of the table above except Escape and Enter, in the same order. `n` SHALL be bound to open window, `m` to toggle multi mode and Ctrl+Space to send the prefix key, each to the action itself, with that action's description. Every other row SHALL be bound as with the modal key style. Each key after the prefix key SHALL end the key sequence, as in any table that is not a mode, so `root` is active again after it. Any other key, Escape and Enter included, SHALL be discarded together with the prefix key.

With the floating key style, the bindings SHALL be: Ctrl+Space as the prefix, no key binding in `root`, no mode, and in `prefix` the bindings that the floating-key-style capability's "Floating bindings" gives, in its order. The floating preset's handler of `KeyTableChanged` SHALL make `root` active again as soon as the prefix key makes `prefix` active, and SHALL open the window list, as that capability's "Leader" defines. The keys that follow SHALL therefore reach the window list, and a key bound in `prefix` SHALL run from the window list as its "Window list" defines.

A binding to an action SHALL dispatch it. A binding to a Lua function SHALL run the function, as the configuration capability defines. A view action SHALL change this client's view as the layout-view capability defines, and SHALL send nothing to the server, except `center_column` on the floating layer, which sends the placing the layout-view capability defines. A session action SHALL be sent to the server as an action naming the focused window, resolved as the actions capability defines, except close window while a floating plugin window is focused, which closes that floating plugin window in the client. Open window SHALL name the viewed band and the tiled window this client focused most recently there, or no window when there is none. Any other session action, and sending the prefix key while multi mode is off, SHALL do nothing when no window is focused, except close window while a floating plugin window is focused. A character key SHALL match a binding by its character, Ctrl and Alt, so a `D` matches whether or not the terminal reports Shift with it.

Where a scenario of this requirement names no key style, the modal key style is saved.

#### Scenario: Detach
- **WHEN** the user presses Ctrl+Space then Shift+D
- **THEN** the client detaches

#### Scenario: Lowercase d does not detach
- **WHEN** the user presses Ctrl+Space then `d`
- **THEN** the client stays attached, nothing is sent to the window, and navigation mode stays active

#### Scenario: Literal Ctrl+Space
- **WHEN** the user runs `cat -v`, presses Ctrl+Space twice, then presses Enter
- **THEN** the focused window receives `\x00` once
- **AND** `cat -v` prints `^@` on a line of its own

#### Scenario: Literal Ctrl+A
- **WHEN** no `user/init.lua` exists and the user presses Ctrl+A at a bash prompt with text typed
- **THEN** the focused window receives `\x01`
- **AND** bash moves the cursor to the start of the line

#### Scenario: Unbound key after the prefix
- **WHEN** the user presses Ctrl+Space, `x`, then `l` with the first of two columns focused
- **THEN** nothing is sent to either window
- **AND** the second column is focused

#### Scenario: Repeated focus moves
- **WHEN** the viewed band holds four columns with the first focused, and the user presses Ctrl+Space, `l`, `l`, `l`
- **THEN** the fourth column is focused and no window receives a key
- **AND** the sidebar shows `N` on row 0

#### Scenario: Repeated resize
- **WHEN** the client's 80×24 terminal sets the screen area, the only window sits in a column of width 1/2, and the user presses Ctrl+Space, `=`, `=`, then Escape, waits, and runs `tput cols`
- **THEN** the column's width is 7/10 and the tile is 56 columns wide
- **AND** the window prints `54`, so the keys after Escape reached it

#### Scenario: Arrow keys move focus
- **WHEN** the second of two columns is focused and the user presses Ctrl+Space then Left
- **THEN** the first column is focused and no window receives a key

#### Scenario: Focus back to the left
- **WHEN** the user presses Ctrl+Space, `h`, Escape, then types `echo left` and Enter, after opening a second window
- **THEN** `left` appears in the first tile only
- **AND** the focused window receives no Escape

#### Scenario: Enter returns to interactive mode
- **WHEN** the user presses Ctrl+Space then Enter with one window open
- **THEN** no window opens and the focused window receives nothing
- **AND** `root` is the active table

#### Scenario: Open a window
- **WHEN** `user/interactive_on_new.lua` holds `return true`, no `user/init.lua` exists, one window is focused, and the user presses Ctrl+Space then `n`
- **THEN** a second tile with a shell prompt appears right of the first
- **AND** the new window is focused and `root` is active, so `echo $GBAND_WINDOW` runs in it

#### Scenario: Open a window and stay in navigation mode
- **WHEN** neither `user/interactive_on_new.lua` nor `user/init.lua` exists, one window is focused, and the user presses Ctrl+Space, `n`, then `h`
- **THEN** a second tile with a shell prompt appears right of the first and is focused after `n`
- **AND** row 0 of the sidebar shows `N` after `n`, and `h` focuses the first window without reaching either window

#### Scenario: Close the focused window
- **WHEN** two windows are open with the second focused and the user presses Ctrl+Space then `q`
- **THEN** the second tile disappears and the first window is focused
- **AND** navigation mode stays active

#### Scenario: Another band
- **WHEN** the user presses Ctrl+Space, `u`, `n`, then `i`
- **THEN** the client shows the first band with its original window focused
- **AND** the second band holds the new window

#### Scenario: Grow the column
- **WHEN** the client's 80×24 terminal sets the screen area, the only window sits in a column of width 1/2, and the user presses Ctrl+Space, `=`, then Escape, waits, and runs `tput cols`
- **THEN** the tile is 48 columns wide
- **AND** the window prints `46`

#### Scenario: Grow the window's height
- **WHEN** the client's 80×24 terminal sets the screen area, a column holds two windows with automatic heights with the top one focused, and the user presses Ctrl+Space then `+`
- **THEN** the top tile is 14 rows high and the bottom tile is 10 rows high

#### Scenario: Center the column
- **WHEN** the client's terminal is 80 columns wide, the client has no bar, the only window sits in a column 40 cells wide at strip position 0, and the user presses Ctrl+Space then `c`
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
- **WHEN** `user/init.lua` is a copy of `defaults/init.lua` that then sets `prefix` to `"ctrl+b"`, and the user presses Ctrl+B, `q`, then Escape with two windows open
- **THEN** the focused window closes
- **AND** pressing Ctrl+Space then sends `\x00` to the focused window

#### Scenario: No prefix binding left
- **WHEN** `user/init.lua` makes no prefix binding and the user presses Ctrl+Space then `h`
- **THEN** the focused window receives `\x00` and then `h`

#### Scenario: Prefix table that is not a mode
- **WHEN** `user/init.lua` binds `prefix h` to `gband.action.focus_column_left` and declares no mode, and the user presses Ctrl+Space, `h`, then `h` with the second of two columns focused
- **THEN** the first column is focused
- **AND** the second `h` reaches the focused window

#### Scenario: Direct key style
- **WHEN** the direct key style is saved, no `user/init.lua` exists, the first of three columns is focused, and the user presses Ctrl+Space, `l`, then `l`
- **THEN** the second column is focused
- **AND** the focused window receives the second `l`

#### Scenario: Open a window with the direct key style
- **WHEN** the direct key style is saved, no `user/init.lua` exists, one window is focused, and the user presses Ctrl+Space then `n`
- **THEN** a second tile with a shell prompt appears right of the first
- **AND** the new window is focused and `root` is active

#### Scenario: Direct key style ignores the setting
- **WHEN** the direct key style is saved, `user/interactive_on_new.lua` holds `return false`, no `user/init.lua` exists, one window is focused, and the user presses Ctrl+Space then `n`
- **THEN** the new window is focused and `root` is active

#### Scenario: Enter after the prefix with the direct key style
- **WHEN** the direct key style is saved, no `user/init.lua` exists, one window is open, and the user presses Ctrl+Space then Enter
- **THEN** no window opens and the focused window receives nothing
- **AND** `root` is the active table

#### Scenario: Named key table
- **WHEN** `user/init.lua` binds `h` and `l` in the table `move`, and binds `prefix m` to a function that calls `gband.keymap.enter("move")`, and the user presses Ctrl+Space, `m`, then `l`, with the first of two columns focused
- **THEN** the second column is focused
- **AND** a further `l` reaches the focused window

#### Scenario: Unbound key in a named table
- **WHEN** the user enters the table `move` and presses `x`
- **THEN** nothing is sent to the window and `root` is active again

#### Scenario: A user mode
- **WHEN** `user/init.lua` declares `resize` a mode, binds `=` in it to `gband.action.grow_column_width` and `escape` to a function that calls `gband.keymap.enter("root")`, binds `alt+r` in `root` to a function that calls `gband.keymap.enter("resize")`, and the user presses Alt+R, `=`, `=`, Escape, then `x`
- **THEN** the focused column grew twice
- **AND** only `x` reaches the focused window

#### Scenario: Key table change events
- **WHEN** a `KeyTableChanged` handler records each payload and the user presses Ctrl+Space, `h`, `l`, then Escape
- **THEN** the handler records `prefix` with previous `root`, then `root` with previous `prefix`, and nothing else

#### Scenario: Current table
- **WHEN** a binding in `prefix` calls `gband.keymap.current_table()`
- **THEN** it returns `prefix`

#### Scenario: Unbound key goes to the focused plugin window
- **WHEN** a floating plugin window with `keys = { j = fn }` is focused and the user presses `j`
- **THEN** `fn` runs and the focused window receives nothing

#### Scenario: Navigation mode before the plugin window
- **WHEN** the default configuration is in use, a floating plugin window with `keys = { x = fn }` is focused, and the user presses Ctrl+Space then `x`
- **THEN** `fn` does not run, nothing reaches the focused window, and navigation mode stays active

#### Scenario: Root binding before the plugin window
- **WHEN** `user/init.lua` binds `alt+h` in `root` to `gband.action.focus_column_left`, a floating plugin window binding `alt+h` in its `keys` is focused, and the user presses Alt+H
- **THEN** the root binding runs and the floating plugin window's function does not

#### Scenario: Prefix q closes the focused floating plugin window
- **WHEN** two windows are open, a floating plugin window is focused, and the user presses Ctrl+Space then `q`
- **THEN** the floating plugin window closes and both tiles stay

#### Scenario: Prefix q closes a floating plugin window on the empty band
- **WHEN** the viewed band holds no window, a floating plugin window is focused, and the user presses Ctrl+Space then `q`
- **THEN** the floating plugin window closes

#### Scenario: Open the key list
- **WHEN** no `user/init.lua` exists and the user presses Ctrl+Space then `?`
- **THEN** a floating plugin window titled `navigation keys` lists the prefix bindings and has focus
- **AND** `root` is active, so a following Down moves the list's cursor line

#### Scenario: Open the Lua prompt
- **WHEN** no `user/init.lua` exists and the user presses Ctrl+Space then `:`
- **THEN** a floating plugin window titled `lua` shows `:` and has focus
- **AND** `root` is active, so a following `j` is typed into the prompt and does not focus the window below

#### Scenario: Rename the focused window
- **WHEN** no `user/init.lua` exists and the user presses Ctrl+Space then Shift+N
- **THEN** a floating plugin window titled `rename` has focus
- **AND** `root` is active, so a following `j` is typed into the prompt and does not focus the window below

#### Scenario: Rename with the direct key style
- **WHEN** the direct key style is saved, no `user/init.lua` exists, and the user presses Ctrl+Space then Shift+N
- **THEN** a floating plugin window titled `rename` has focus and `root` is active

#### Scenario: Open the settings window
- **WHEN** no `user/init.lua` exists and the user presses Ctrl+Space then `s`
- **THEN** a floating plugin window titled `settings` has focus
- **AND** `root` is active, so a following `j` moves the window's cursor line and does not focus the window below

#### Scenario: Settings with the direct key style
- **WHEN** the direct key style is saved, no `user/init.lua` exists, and the user presses Ctrl+Space then `s`
- **THEN** the settings window is open and focused

#### Scenario: Reload the configuration
- **WHEN** no `user/init.lua` exists, a plugin's `client.lua` is rewritten from drawing `old` to drawing `new`, and the user presses Ctrl+Space then `!`
- **THEN** the client draws `new`
- **AND** `root` is active

#### Scenario: Reload with the direct key style
- **WHEN** the direct key style is saved, no `user/init.lua` exists, a plugin's `client.lua` is rewritten from drawing `old` to drawing `new`, and the user presses Ctrl+Space then `!`
- **THEN** the client draws `new`

#### Scenario: Multi mode from navigation mode
- **WHEN** no `user/init.lua` exists, the viewed band holds two shell windows, and the user presses Ctrl+Space then `m`, and types `echo hi` and Enter
- **THEN** `root` is active and both windows print `hi`
- **AND** after Ctrl+Space, `m`, `echo solo` and Enter, only the focused window prints `solo`

#### Scenario: Multi mode with the direct key style
- **WHEN** the direct key style is saved, no `user/init.lua` exists, the viewed band holds two shell windows, and the user presses Ctrl+Space then `m`, and types `echo hi` and Enter
- **THEN** both windows print `hi`
- **AND** `gband.keymap.list("prefix")` holds `m` bound to the action `toggle_multi`

#### Scenario: Reset height keeps its key
- **WHEN** no `user/init.lua` exists, the focused window has a fixed height, and the user presses Ctrl+Space then Shift+R
- **THEN** the focused window's height is automatic again and no configuration loads

#### Scenario: Colon reaches the focused window
- **WHEN** no `user/init.lua` exists, `root` is active, and the user types `:`
- **THEN** the focused window receives `:` and no prompt opens

#### Scenario: Float the focused window
- **WHEN** the client's 80×24 terminal sets the screen area, the client has no bar, the only window sits in a column of width 1/2, and the user presses Ctrl+Space then `v`
- **THEN** the window is drawn in a box spanning columns 20 to 59 and rows 2 to 21, and stays focused

#### Scenario: Switch to the tiled layer and back
- **WHEN** two windows are open, the second floats and is focused, and the user presses Ctrl+Space, `V`, Escape, then types `echo tiled` and Enter
- **THEN** `tiled` appears in the first window only
- **AND** pressing Ctrl+Space then `V` again focuses the floating window

#### Scenario: Move a column with Ctrl+H and Ctrl+Right
- **WHEN** the viewed band holds columns A and B with B focused, and the user presses Ctrl+Space then Ctrl+H
- **THEN** the band holds B and A, in that order, and B stays focused, and neither window receives a key
- **AND** pressing Ctrl+Right next, still in navigation mode, restores A and B

#### Scenario: Move a window with Ctrl+J
- **WHEN** a column holds P1 above P2 with P1 focused, and the user presses Ctrl+Space then Ctrl+J
- **THEN** the column holds P2 above P1, and P1 stays focused

#### Scenario: Move a floating window
- **WHEN** the client's 80×24 terminal sets the screen area, the client has no bar, a floating window is focused with its box starting at column 20, and the user presses Ctrl+Space then Ctrl+L
- **THEN** the box is drawn from column 28

#### Scenario: Floating key style
- **WHEN** the floating key style is saved, no `user/init.lua` exists, and the user presses Ctrl+Space
- **THEN** `gband.keymap.label("prefix")` returns `prefix`, the window list is open and focused, and `root` is active
- **AND** a following Escape closes the window list, and `echo hi` and Enter then print `hi` in the focused window

#### Scenario: Prefix binding from the window list
- **WHEN** the floating key style is saved, no `user/init.lua` exists, and the user presses Ctrl+Space then Shift+D
- **THEN** the client detaches

#### Scenario: Reload from the window list
- **WHEN** the floating key style is saved, no `user/init.lua` exists, a plugin's `client.lua` is rewritten from drawing `old` to drawing `new`, and the user presses Ctrl+Space then `!`
- **THEN** the window list is closed and the client draws `new`

### Requirement: Input to the server
The client SHALL send each key press and repeat that the key bindings do not consume to the server as a key naming the focused window, each paste as a paste naming the focused window, or, while multi mode is on, each such key and paste once to every window of the viewed band, as the multi-mode capability defines, and each change of its reported size, as "Ribbon area beside the bars" defines it, as a resize carrying the reported size. While a plugin window is focused, the plugin-windows capability SHALL take the keys and the pastes instead, whether or not a window is focused. Otherwise, with multi mode off, keys and pastes SHALL be dropped while no window is focused. Keys the input-encoding capability cannot represent SHALL be dropped. On attach, the client SHALL send its reported size as the terminal size its hello carries, as the wire-protocol capability defines the hello. Its bars SHALL already be placed for that report, as the configuration loaded before the handshake adds them.

#### Scenario: Typing runs a command
- **WHEN** the user types `echo hi` and Enter
- **THEN** the focused window prints `hi`

#### Scenario: Typing reaches only the focused window
- **WHEN** two windows are open with the second focused and the user types `echo hi` and Enter
- **THEN** the second window prints `hi`
- **AND** the first window's screen is unchanged

#### Scenario: Resize reaches the program
- **WHEN** the client has no bar, the only window sits in a column of width 1/2, the user resizes the terminal to 70 columns and runs `tput cols`
- **THEN** the window prints `33`

#### Scenario: Resize beside the sidebar
- **WHEN** the sidebar is a left bar 1 column wide, the only window sits in a column of width 1/2, the user resizes the terminal to 71 columns and runs `tput cols`
- **THEN** the client reports the size 70×24 and the window prints `33`

#### Scenario: First report beside the sidebar
- **WHEN** the default configuration is in use and the user runs `gband attach -s fresh` in an 80×24 terminal, creating the session
- **THEN** the client's hello carries the size 79×24
- **AND** the session's first window starts with a PTY of 37 columns by 22 rows

#### Scenario: Height beside the sidebar
- **WHEN** the client's terminal is 80×24, the sidebar is a left bar 1 column wide, and the only window runs `tput lines`
- **THEN** the window prints `22`

#### Scenario: Typing into a focused floating plugin window
- **WHEN** a floating plugin window is focused and the user types `ls`
- **THEN** nothing is sent to the server

#### Scenario: Paste into a focused floating plugin window
- **WHEN** a floating plugin window without `on_input` is focused and the user pastes `hello`
- **THEN** the paste is discarded and nothing is sent to the server

#### Scenario: Paste into a plugin window that takes text
- **WHEN** a floating plugin window whose `on_input` records its arguments is focused and the user pastes `hello`
- **THEN** `on_input` runs with `hello` and nothing is sent to the server

#### Scenario: Paste on a band with no window
- **WHEN** the viewed band holds no window, a floating plugin window whose `on_input` records its arguments is focused, and the user pastes `hi`
- **THEN** `on_input` runs with `hi`

