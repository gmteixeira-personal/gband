## Purpose

Defines `gband attach`: the client that connects the user's terminal to a session server, presents the pane full screen from its own copy of the server's screen, sends the user's input back, and leaves the session running when it goes.

## ADDED Requirements

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
The client SHALL complete the wire protocol's handshake before it changes the terminal. When the server speaks a different protocol version, the client SHALL print one line to standard error naming both versions and saying that the running server must be stopped first, and exit with status 1. The terminal SHALL be left as it was.

#### Scenario: Incompatible server
- **WHEN** a server speaking protocol version 2 is running and a client speaking version 1 attaches
- **THEN** standard error names versions 1 and 2
- **AND** the client exits with status 1
- **AND** the terminal is not left in raw mode or the alternate screen

### Requirement: Present the pane
After the handshake, the client SHALL take the terminal full screen in raw mode with bracketed paste enabled. It SHALL keep its own copy of the pane's screen, built from the server's snapshot and updated by every update the server sends. It SHALL draw that copy from the top-left corner of the terminal, cut to the terminal's size, and place the terminal's cursor where the pane's cursor is, hidden when the pane hides it.

#### Scenario: Snapshot drawn
- **WHEN** the client attaches to a pane that shows a prompt
- **THEN** the client draws the prompt with its colours
- **AND** the cursor sits after the prompt

#### Scenario: Pane larger than the terminal
- **WHEN** the PTY is 120×40 because of another client and this client's terminal is 100×30
- **THEN** this client draws the pane's top-left 100×30 cells

### Requirement: Send input
The client SHALL send each key press and repeat to the server as a key with its modifiers, each paste as text, and each change of its terminal's size as a resize. Keys the input-encoding capability cannot represent SHALL be dropped. On attach, the client SHALL report its terminal size.

#### Scenario: Typing runs a command
- **WHEN** the user types `echo hi` and Enter
- **THEN** the pane prints `hi`

#### Scenario: Resize reaches the program
- **WHEN** the user resizes the terminal to 70 columns and runs `tput cols`
- **THEN** the pane prints `70`

### Requirement: Prefix key
Ctrl+A SHALL be the prefix key, and the client SHALL NOT send it to the server when it is pressed. The key pressed after the prefix SHALL act as follows:

| key after Ctrl+A | action |
|---|---|
| `d` | detach |
| Ctrl+A | send one Ctrl+A to the pane |
| any other key | discard both keys |

#### Scenario: Detach
- **WHEN** the user presses Ctrl+A then `d`
- **THEN** the client detaches

#### Scenario: Literal Ctrl+A
- **WHEN** the user presses Ctrl+A twice at a bash prompt with text typed
- **THEN** the pane receives `\x01` once
- **AND** bash moves the cursor to the start of the line

#### Scenario: Unbound key after the prefix
- **WHEN** the user presses Ctrl+A then `x`
- **THEN** nothing is sent to the pane

### Requirement: Leaving the client
Whenever the client exits after taking the terminal, it SHALL first leave the alternate screen, disable raw mode and bracketed paste, and show the cursor. It SHALL then print one line and exit as follows:

| cause | line printed | exit status |
|---|---|---|
| the user detached | `[detached]` | 0 |
| the session's program exited | `[exited]` | 0 |
| the connection closed without the server saying why | `[lost server]` | 1 |

#### Scenario: Detach and reattach
- **WHEN** the user runs `sleep 100`, detaches with Ctrl+A then `d`, and runs `gband attach` again
- **THEN** the first client printed `[detached]` and exited with status 0
- **AND** the new client shows `sleep 100` still running

#### Scenario: Shell exits
- **WHEN** the user runs `exit` in the pane
- **THEN** the client restores the terminal, prints `[exited]` and exits with status 0

#### Scenario: Server killed
- **WHEN** the server is killed with SIGKILL while a client is attached
- **THEN** the client restores the terminal, prints `[lost server]` and exits with status 1
