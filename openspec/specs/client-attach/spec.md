# client-attach Specification

## Purpose

Defines `gband attach`: the client that connects the user's terminal to a session server, presents the pane full screen from its own copy of the server's screen, sends the user's input back, and leaves the session running when it goes.

## Requirements

### Requirement: Connect to the session
`gband attach` SHALL connect to the socket path the session-server capability defines, resolved from its own environment.

#### Scenario: Server already running
- **WHEN** a server is running and the user runs `gband attach` with the same environment
- **THEN** the client attaches to that server's pane

### Requirement: Start a server when none is running
When nothing accepts connections on the socket path, `gband attach` SHALL start `gband server` from its own executable, detached from the client's terminal and session, with the client's working directory and environment, and with standard input, output and error on `/dev/null`. It SHALL then connect to it. If no connection succeeds within 5 seconds, the client SHALL print one line to standard error naming the socket path and the server log, and exit with status 1.

#### Scenario: First attach starts the session
- **WHEN** no server is running and the user runs `gband attach` in `~/repos/gband`
- **THEN** a shell prompt appears in the client
- **AND** `pwd` in the pane prints the path of `~/repos/gband`

#### Scenario: Started server survives its terminal
- **WHEN** `gband attach` started the server and the terminal emulator window running the client is closed
- **THEN** the server and its shell keep running

#### Scenario: Server cannot start
- **WHEN** the server that `gband attach` starts exits before accepting a connection
- **THEN** the client prints one line to standard error naming the socket path and the server log
- **AND** it exits with status 1 within 5 seconds

### Requirement: Refuse to nest
`gband attach` SHALL refuse to run when the `GBAND` environment variable is set and not empty, because it is then running inside a gband pane. It SHALL print one line to standard error saying so, and exit with status 1 without connecting.

#### Scenario: Attach inside a pane
- **WHEN** the user runs `gband attach` in a gband pane
- **THEN** standard error says that the client is already inside a gband pane
- **AND** the process exits with status 1

### Requirement: Require a terminal
`gband attach` SHALL refuse to run when its standard input or standard output is not a terminal. It SHALL print one line to standard error saying so, and exit with status 1 without connecting or starting a server.

#### Scenario: Output redirected
- **WHEN** the user runs `gband attach > out.txt`
- **THEN** standard error says that `gband attach` needs a terminal
- **AND** the process exits with status 1
- **AND** no server is started

### Requirement: Version handshake
The client SHALL complete the wire protocol's handshake, and receive the server's info, before it changes the terminal. When the server speaks a different protocol version, a debug build of the client SHALL replace the server as the "Server from a different build" requirement defines. A release build SHALL print one line to standard error naming both versions and saying that `gband kill-server` stops the running server, and exit with status 1. The terminal SHALL be left as it was. In this capability, a debug build is one compiled with debug assertions, as cargo's default `dev` profile does, and a release build is any other.

#### Scenario: Incompatible server, debug build
- **WHEN** a server speaking protocol version 2 is running and a debug-build client speaking version 1 attaches
- **THEN** the version 2 server and its shell are stopped
- **AND** the client attaches to a new version 1 server

#### Scenario: Incompatible server, release build
- **WHEN** a server speaking protocol version 2 is running and a release-build client speaking version 1 attaches
- **THEN** standard error names versions 1 and 2 and `gband kill-server`
- **AND** the client exits with status 1
- **AND** the terminal is not left in raw mode or the alternate screen

### Requirement: Server from a different build
The client SHALL compare the executable identity in the server's info with the identity of its own executable, as the wire protocol defines it. When they differ, it SHALL record a warning in the client log naming the server's process id.

A debug build SHALL then replace the server before it changes the terminal. It SHALL send detach and close the connection, stop the server as `gband kill-server` does, start a new server from its own executable as the "Start a server when none is running" requirement defines, and attach to that one. A client SHALL replace a server at most once per invocation. When the server it reaches after replacing still differs, the client SHALL continue as a release build does. When the old server does not stop within 5 seconds, the client SHALL print one line to standard error naming its process id, and exit with status 1.

A release build SHALL attach to the server unchanged and, after printing the line that "Leaving the client" defines, print to standard error: `gband: the server runs a different gband build; stop it with gband kill-server and attach again`.

#### Scenario: Debug build after a rebuild
- **WHEN** a server started from a debug build is running, the user runs `cargo build`, which replaces the executable, and runs `gband attach` from the new build
- **THEN** the old server and its shell are stopped
- **AND** the client attaches to a new server whose info carries the client's own executable identity

#### Scenario: Debug build, same executable
- **WHEN** a debug-build client attaches to a server started from the same unchanged executable
- **THEN** the client attaches to that server and its shell keeps running

#### Scenario: Release build after an upgrade
- **WHEN** a server started from a release build is running and the user replaces the executable and runs `gband attach`
- **THEN** the client attaches to the running server and its shell
- **AND** after the user detaches, standard error holds `gband: the server runs a different gband build; stop it with gband kill-server and attach again` after `[detached]`

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
After the handshake, the client SHALL take the terminal full screen in raw mode with bracketed paste enabled. It SHALL keep its own grid of every pane, as the wire-protocol capability defines, and its own view, as the layout-view capability defines. It SHALL draw only the viewed workspace.

Each pane SHALL be drawn in its tile, as the layout capability's tile geometry gives it for the screen area in the latest layout. A tile SHALL be drawn at its strip position less the viewed workspace's camera position, from the terminal's top row. Each tile SHALL show a one-cell border around the pane's grid, which is drawn from its top-left corner. The focused pane's border SHALL be drawn in a style distinct from the other borders.

A tile that crosses the terminal's left, right or bottom edge SHALL be cut at that edge, and the part of the tile inside the terminal SHALL be drawn unchanged. No tile SHALL be resized to fit the terminal. A tile wholly outside the terminal SHALL NOT be drawn, and cells no tile covers SHALL be blank.

The terminal's cursor SHALL sit where the focused pane's cursor is. It SHALL be hidden when the focused pane hides its cursor, when that cell lies outside the terminal, or when no pane is focused.

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

### Requirement: Key bindings
Ctrl+A SHALL be the prefix key, and the client SHALL NOT send it to the server when it is pressed. The key pressed after the prefix SHALL act as the table below defines. Until a later change makes the bindings configurable, the table SHALL be fixed in the client.

| key after Ctrl+A | action | kind |
|---|---|---|
| `h` | focus the column to the left | view |
| `l` | focus the column to the right | view |
| `j` | focus the pane below | view |
| `k` | focus the pane above | view |
| `u` | view the workspace below | view |
| `i` | view the workspace above | view |
| Enter | open a pane right of the focused pane's column | session |
| `q` | close the focused pane | session |
| `[` | consume or expel the focused pane to the left | session |
| `]` | consume or expel the focused pane to the right | session |
| `r` | cycle the width of the focused pane's column | session |
| `f` | toggle full width of the focused pane's column | session |
| `D` | detach | client |
| Ctrl+A | send one Ctrl+A to the focused pane | client |
| any other key | discard both keys | — |

A view action SHALL change this client's view as the layout-view capability defines, and SHALL send nothing to the server. A session action SHALL be sent to the server as an action naming the focused pane. Open pane SHALL name the viewed workspace and the focused pane, or no pane when none is focused. Any other session action, and sending Ctrl+A, SHALL do nothing when no pane is focused. A character key SHALL match a table entry by its character, Ctrl and Alt, so a `D` matches whether or not the terminal reports Shift with it.

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
