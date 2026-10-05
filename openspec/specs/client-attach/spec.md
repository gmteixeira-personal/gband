# client-attach Specification

## Purpose

Defines `gband attach`: the client that connects the user's terminal to a session server, presents the pane full screen from its own copy of the server's screen, sends the user's input back, and leaves the session running when it goes.

## Requirements

### Requirement: Connect to the session
`gband attach` SHALL connect to the socket path the command-line capability's server selection resolves from its own environment and options. Before it sends anything, the client SHALL read the user id of the process listening on the socket from the operating system. When that user id differs from the client's own, the client SHALL print one line to standard error naming the socket path and the listening user id, and exit with status 1 without changing the terminal. After the handshake, it SHALL ask to attach to the session `-s` names, sending its own working directory with the request. The server attaches it to that session, creating the session when it does not exist, as the session-server capability defines.

#### Scenario: Server already running
- **WHEN** a server is running and the user runs `gband attach` with the same environment and options
- **THEN** the client attaches to that server's session named `default`

#### Scenario: Attach to a named session
- **WHEN** a server hosts `default` and `work` and the user runs `gband attach -s work`
- **THEN** the client shows the panes of `work`

#### Scenario: Named server
- **WHEN** servers named `a` and `b` are running and the user runs `gband -S b attach`
- **THEN** the client attaches to the session named `default` of the server named `b`

#### Scenario: Socket owned by another user
- **WHEN** a process of another user listens on `/tmp/s.sock` and the user runs `gband attach -p /tmp/s.sock`
- **THEN** standard error names `/tmp/s.sock` and that user's id
- **AND** the process exits with status 1
- **AND** nothing is sent on the socket

### Requirement: Start a server when none is running
When nothing accepts connections on the socket path, `gband attach` SHALL start `gband server` from its own executable, addressing the same socket path with `-p` and passing the session name `-s` names, detached from the client's terminal and session, with the client's working directory and environment, and with standard input, output and error on `/dev/null`. It SHALL then connect to it. If no connection succeeds within 5 seconds, the client SHALL print one line to standard error naming the socket path and the server log, and exit with status 1.

#### Scenario: First attach starts the session
- **WHEN** no server is running and the user runs `gband attach` in `~/repos/gband`
- **THEN** a shell prompt appears in the client
- **AND** `pwd` in the pane prints the path of `~/repos/gband`

#### Scenario: First attach names the session
- **WHEN** no server is running and the user runs `gband attach -s work`
- **THEN** the server it starts hosts exactly one session, named `work`
- **AND** the client is attached to it

#### Scenario: Named server started on demand
- **WHEN** no server named `feature` is running and the user runs `gband -S feature attach`
- **THEN** a server listening on `feature.sock` in the runtime directory starts
- **AND** `echo $GBAND` in its pane prints that socket path

#### Scenario: Started server survives its terminal
- **WHEN** `gband attach` started the server and the terminal emulator window running the client is closed
- **THEN** the server and its shell keep running

#### Scenario: Server cannot start
- **WHEN** the server that `gband attach` starts exits before accepting a connection
- **THEN** the client prints one line to standard error naming the socket path and the server log
- **AND** it exits with status 1 within 5 seconds

### Requirement: Refuse to nest
`gband attach` SHALL refuse to run when the `GBAND` environment variable is set, is not empty, and holds the socket path the client resolved, because the client would then show the pane's own server inside one of its panes. It SHALL print one line to standard error saying so, and exit with status 1 without connecting. Attaching from a pane to a server on another socket path SHALL be allowed.

#### Scenario: Attach inside a pane
- **WHEN** the user runs `gband attach` without an option in a gband pane
- **THEN** standard error says that the client is already inside a pane of that server
- **AND** the process exits with status 1

#### Scenario: Same server named explicitly
- **WHEN** the user runs `gband -S feature attach` in a pane of the server named `feature`
- **THEN** the process exits with status 1 without connecting

#### Scenario: Another server from a pane
- **WHEN** the user runs `gband -S feature attach` in a pane of the default server
- **THEN** the client attaches to the server named `feature`

### Requirement: Require a terminal
`gband attach` SHALL refuse to run when its standard input or standard output is not a terminal. It SHALL print one line to standard error saying so, and exit with status 1 without connecting or starting a server.

#### Scenario: Output redirected
- **WHEN** the user runs `gband attach > out.txt`
- **THEN** standard error says that `gband attach` needs a terminal
- **AND** the process exits with status 1
- **AND** no server is started

### Requirement: Version handshake
The client SHALL complete the wire protocol's handshake, and receive the server's info, before it changes the terminal. When the server speaks a different protocol version, a debug build of the client SHALL replace the server as the "Server from a different build" requirement defines, when that requirement allows it. Otherwise the client SHALL print one line to standard error naming both versions and the `gband kill-server` command that stops the running server, carrying the selection option the client was given, and exit with status 1. The terminal SHALL be left as it was. In this capability, a debug build is one compiled with debug assertions, as cargo's default `dev` profile does, and a release build is any other.

#### Scenario: Incompatible server, debug build
- **WHEN** a server speaking protocol version 2 is running from the client's executable path and a debug-build client speaking version 1 attaches
- **THEN** the version 2 server and its shell are stopped
- **AND** the client attaches to a new version 1 server

#### Scenario: Incompatible server, release build
- **WHEN** a server speaking protocol version 2 is running and a release-build client speaking version 1 attaches
- **THEN** standard error names versions 1 and 2 and `gband kill-server`
- **AND** the client exits with status 1
- **AND** the terminal is not left in raw mode or the alternate screen

#### Scenario: Incompatible named server
- **WHEN** a server named `feature` speaks another protocol version and a release-build client runs `gband -S feature attach`
- **THEN** standard error names `gband -S feature kill-server`

### Requirement: Server from a different build
The client SHALL compare the executable identity in the server's info with the identity of its own executable, as the wire protocol defines it. When they differ, it SHALL record a warning in the client log naming the server's process id.

A debug build SHALL replace the server only when the server's process runs from the same executable path as the client, which is the case after a rebuild of that executable. It SHALL read that path from the operating system, using the process id in the server's pid record. A server from any other path, or whose path cannot be read, SHALL be treated as a release build treats it. This keeps a client built in one worktree from stopping a server built in another.

To replace the server, the client SHALL, before it changes the terminal, send detach and close the connection, stop the server as `gband kill-server` does for the same socket path, start a new server from its own executable as the "Start a server when none is running" requirement defines, and attach to that one. A client SHALL replace a server at most once per invocation. When the server it reaches after replacing still differs, the client SHALL continue as a release build does. When the old server does not stop within 5 seconds, the client SHALL print one line to standard error naming its process id, and exit with status 1.

A release build, and a debug build that does not replace, SHALL attach to the server unchanged and, after printing the line that "Leaving the client" defines, print to standard error: `gband: the server runs a different gband build; stop it with <command> and attach again`, where `<command>` is `gband kill-server` carrying the selection option the client was given.

#### Scenario: Debug build after a rebuild
- **WHEN** a server started from a debug build is running, the user runs `cargo build`, which replaces the executable, and runs `gband attach` from the new build
- **THEN** the old server and its shell are stopped
- **AND** the client attaches to a new server whose info carries the client's own executable identity

#### Scenario: Debug build, same executable
- **WHEN** a debug-build client attaches to a server started from the same unchanged executable
- **THEN** the client attaches to that server and its shell keeps running

#### Scenario: Debug build from another worktree
- **WHEN** a server started from `/w/a/target/debug/gband` is running and a debug-build client from `/w/b/target/debug/gband` attaches to it
- **THEN** the client attaches to that server and its shell keeps running
- **AND** after the user detaches, standard error holds the different-build note

#### Scenario: Release build after an upgrade
- **WHEN** a server started from a release build is running and the user replaces the executable and runs `gband attach`
- **THEN** the client attaches to the running server and its shell
- **AND** after the user detaches, standard error holds `gband: the server runs a different gband build; stop it with gband kill-server and attach again` after `[detached]`

#### Scenario: Note names the selected server
- **WHEN** a release-build client run as `gband -S feature attach` reaches a server from a different build and the user detaches
- **THEN** standard error holds `gband: the server runs a different gband build; stop it with gband -S feature kill-server and attach again`

#### Scenario: Server does not stop
- **WHEN** a debug-build client must replace a server that is still running 5 seconds after being told to stop
- **THEN** the client prints one line to standard error naming that server's process id
- **AND** it exits with status 1 without changing the terminal

### Requirement: Send input
The client SHALL send each key press and repeat that the key bindings do not consume to the server as a key naming the focused pane, each paste as a paste naming the focused pane, and each change of its terminal's size as a resize. Keys and pastes SHALL be dropped while no pane is focused. Keys the input-encoding capability cannot represent SHALL be dropped. On attach, the client SHALL report its terminal size.

#### Scenario: Typing runs a command
- **WHEN** the user types `echo hi` and Enter
- **THEN** the focused pane prints `hi`

#### Scenario: Typing reaches only the focused pane
- **WHEN** two panes are open with the second focused and the user types `echo hi` and Enter
- **THEN** the second pane prints `hi`
- **AND** the first pane's screen is unchanged

#### Scenario: Resize reaches the program
- **WHEN** the only pane sits in a column of width 1/2, the user resizes the terminal to 70 columns and runs `tput cols`
- **THEN** the pane prints `33`

### Requirement: Leaving the client
Whenever the client exits after taking the terminal, it SHALL first leave the alternate screen, disable raw mode and bracketed paste, and show the cursor. It SHALL then print one line to standard output, followed by the note for a server from a different build when one applies, and exit as follows:

| cause | line printed | exit status |
|---|---|---|
| the user detached | `[detached]` | 0 |
| the session ended | `[exited]` | 0 |
| the connection closed without the server saying why | `[lost server]` | 1 |

#### Scenario: Detach and reattach
- **WHEN** the user runs `sleep 100`, detaches with Ctrl+A then `D`, and runs `gband attach` again
- **THEN** the first client printed `[detached]` and exited with status 0
- **AND** the new client shows `sleep 100` still running

#### Scenario: Shell exits
- **WHEN** the user runs `exit` in the only pane
- **THEN** the client restores the terminal, prints `[exited]` and exits with status 0

#### Scenario: Server killed
- **WHEN** the server is killed with SIGKILL while a client is attached
- **THEN** the client restores the terminal, prints `[lost server]` and exits with status 1

### Requirement: Present the ribbon
After the handshake, the client SHALL take the terminal full screen in raw mode with bracketed paste enabled. It SHALL keep its own grid of every pane, as the wire-protocol capability defines, and its own view, as the layout-view capability defines. It SHALL draw only the viewed workspace, except during a workspace switch, when it SHALL draw the workspaces the animations capability places on screen. While the configuration capability shows a configuration error, the client SHALL draw that error over the bottom row, after the tiles.

Each pane SHALL be drawn in its tile, as the layout capability's tile geometry gives it for the screen area in the latest layout. A tile SHALL be drawn at its strip position less the viewed workspace's camera position, from the terminal's top row. While an animation runs, the tile's position and size, the camera and the workspace's top row SHALL be the drawn values the animations capability defines. At rest they equal the values above. Each tile SHALL show a one-cell border around the pane's grid, which is drawn from its top-left corner. The focused pane's border SHALL be drawn in a style distinct from the other borders.

A pane's grid MAY differ in size from its tile's interior while the server has not yet resized the pane. The border SHALL still follow the tile, at its drawn size while an animation runs. A grid larger than the interior SHALL be cut at the interior's right and bottom edges, and interior cells the grid does not cover SHALL be blank.

A tile that crosses the terminal's left, right or bottom edge SHALL be cut at that edge, and the part of the tile inside the terminal SHALL be drawn unchanged, except where a configuration error covers the bottom row. No tile SHALL be resized to fit the terminal. A tile wholly outside the terminal SHALL NOT be drawn, and cells no tile or configuration error covers SHALL be blank.

The terminal's cursor SHALL sit where the focused pane's cursor is. It SHALL be hidden when the focused pane hides its cursor, when that cell lies outside the terminal or outside the tile's interior, when no pane is focused, or while the animations capability hides it during motion.

#### Scenario: Two columns side by side
- **WHEN** the client's 80×24 terminal sets the screen area, and the viewed workspace holds two columns of width 1/2 with the second focused
- **THEN** the first tile fills screen columns 0 to 39 and the second fills 40 to 79, each with a border
- **AND** only the second tile's border has the focused style
- **AND** the cursor sits in the second tile

#### Scenario: Column clipped at the left edge
- **WHEN** the client's 80×24 terminal sets the screen area, the viewed workspace holds two columns of width 2/3, and the client focuses the second
- **THEN** the second tile fills screen columns 27 to 79
- **AND** screen columns 0 to 26 show the rightmost 27 columns of the first tile, without its left border

#### Scenario: Tile larger than the terminal
- **WHEN** the screen area is 120×40 because of another client, this client's terminal is 100×30, and the viewed workspace holds one column of width 1/2
- **THEN** this client draws the top 30 rows of the 60-column tile
- **AND** the tile's bottom border is not drawn

#### Scenario: Empty workspace
- **WHEN** the client views an empty workspace
- **THEN** the screen is blank and the cursor is hidden

#### Scenario: Grid smaller than its tile
- **WHEN** a pane's tile is 50×30 and its grid is still 38×22
- **THEN** the border is drawn around the 50×30 tile
- **AND** the grid fills the top-left 38×22 cells of the interior, and the rest of the interior is blank

#### Scenario: Grid larger than its tile
- **WHEN** a pane's tile is 40×24 and its grid is still 48×28
- **THEN** the interior shows the grid's top-left 38×22 cells
- **AND** the border is drawn unbroken around the 40×24 tile

#### Scenario: Mid-scroll frame
- **WHEN** the camera is drawn at 20 while it scrolls toward 40, and the viewed workspace holds three columns of 40 cells
- **THEN** the second tile fills screen columns 20 to 59
- **AND** the cursor is hidden

#### Scenario: Configuration error over the ribbon
- **WHEN** the client's 40×6 terminal sets the screen area, the viewed workspace holds one column of width 1/2, and the client shows a configuration error
- **THEN** the bottom row shows the error, cut at the terminal's width
- **AND** the rows above it show the tile unchanged

### Requirement: Key bindings
The client SHALL take its key bindings and its prefix key from the configuration, as the configuration capability defines them. Outside a prefix sequence, a key that a direct binding names SHALL run that binding, and the prefix key SHALL start a prefix sequence. The client SHALL send neither to the server. Any other key outside a prefix sequence SHALL be sent to the focused pane. The key pressed next in a prefix sequence SHALL end it: a key that a prefix binding names SHALL run that binding, and any other key SHALL discard both keys. When no prefix binding exists, the prefix key SHALL be sent to the focused pane like any other key.

With no configuration file, the bindings SHALL be those `defaults.lua` makes: Ctrl+A as the prefix, no direct binding, and these prefix bindings:

| key after Ctrl+A | Lua binding | action | kind |
|---|---|---|---|
| `h` | `prefix h` | focus the column to the left | view |
| `l` | `prefix l` | focus the column to the right | view |
| `j` | `prefix j` | focus the pane below | view |
| `k` | `prefix k` | focus the pane above | view |
| `u` | `prefix u` | view the workspace below | view |
| `i` | `prefix i` | view the workspace above | view |
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
| Ctrl+A | `prefix prefix` | send the prefix key to the focused pane | client |
| any other key | — | discard both keys | — |

A binding to an action SHALL dispatch it. A binding to a Lua function SHALL run the function, as the configuration capability defines. A view action SHALL change this client's view as the layout-view capability defines, and SHALL send nothing to the server. A session action SHALL be sent to the server as an action naming the focused pane. Open pane SHALL name the viewed workspace and the focused pane, or no pane when none is focused. Any other session action, and sending the prefix key, SHALL do nothing when no pane is focused. A character key SHALL match a binding by its character, Ctrl and Alt, so a `D` matches whether or not the terminal reports Shift with it.

#### Scenario: Detach
- **WHEN** the user presses Ctrl+A then Shift+D
- **THEN** the client detaches

#### Scenario: Lowercase d does not detach
- **WHEN** the user presses Ctrl+A then `d`
- **THEN** the client stays attached and nothing is sent to the pane

#### Scenario: Literal Ctrl+A
- **WHEN** the user presses Ctrl+A twice at a bash prompt with text typed
- **THEN** the focused pane receives `\x01` once
- **AND** bash moves the cursor to the start of the line

#### Scenario: Unbound key after the prefix
- **WHEN** the user presses Ctrl+A then `x`
- **THEN** nothing is sent to the pane

#### Scenario: Open a pane
- **WHEN** one pane is focused and the user presses Ctrl+A then Enter
- **THEN** a second tile with a shell prompt appears right of the first
- **AND** the new pane is focused, so `echo $GBAND_PANE` runs in it

#### Scenario: Focus back to the left
- **WHEN** the user has opened a second pane and presses Ctrl+A then `h`, then types `echo left` and Enter
- **THEN** `left` appears in the first tile only

#### Scenario: Close the focused pane
- **WHEN** two panes are open with the second focused and the user presses Ctrl+A then `q`
- **THEN** the second tile disappears and the first pane is focused

#### Scenario: Another workspace
- **WHEN** the user presses Ctrl+A then `u`, then Ctrl+A then Enter, then Ctrl+A then `i`
- **THEN** the client shows the first workspace with its original pane focused
- **AND** the second workspace holds the new pane

#### Scenario: Grow the column
- **WHEN** the client's 80×24 terminal sets the screen area, the only pane sits in a column of width 1/2, and the user presses Ctrl+A then `=`, waits, and runs `tput cols`
- **THEN** the tile is 48 columns wide
- **AND** the pane prints `46`

#### Scenario: Grow the pane's height
- **WHEN** the client's 80×24 terminal sets the screen area, a column holds two panes with automatic heights with the top one focused, and the user presses Ctrl+A then `+`
- **THEN** the top tile is 14 rows high and the bottom tile is 10 rows high

#### Scenario: Direct binding acts without the prefix
- **WHEN** `init.lua` binds `alt+h` to `gband.action.focus_column_left`, the second of two panes is focused, and the user presses Alt+H
- **THEN** the first pane is focused
- **AND** neither pane receives the key

#### Scenario: Unbound Alt key reaches the pane
- **WHEN** `init.lua` binds `alt+h` and the user presses Alt+X
- **THEN** the focused pane receives `\x1bx`

#### Scenario: Another prefix key
- **WHEN** `init.lua` sets `prefix` to `"ctrl+b"` and the user presses Ctrl+B then `q` with two panes open
- **THEN** the focused pane closes
- **AND** pressing Ctrl+A sends `\x01` to the focused pane

#### Scenario: No prefix binding left
- **WHEN** `init.lua` unbinds every prefix binding and the user presses Ctrl+A then `h`
- **THEN** the focused pane receives `\x01` and then `h`

### Requirement: Report shown panes
The client SHALL send the server a shown message naming its shown panes, as the layout-view capability defines them, once it has received the first layout after attaching. It SHALL send a new shown message whenever its shown panes change, whether a layout, a focus message, a view action or a change of its terminal's size changed them. It SHALL NOT send a shown message that names the same panes as the last one it sent.

#### Scenario: Shown after attach
- **WHEN** a client with an 80×24 terminal attaches to a session whose first workspace holds three columns of width 1/2
- **THEN** the client's first shown message names the panes of the first two columns

#### Scenario: Shown follows the camera
- **WHEN** that client then focuses the third column
- **THEN** it sends a shown message naming the panes of the second and third columns

#### Scenario: No repeated report
- **WHEN** the client focuses the pane below within a column whose panes are all shown
- **THEN** it sends no shown message
