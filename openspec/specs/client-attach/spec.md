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
Whenever the client exits after taking the terminal, it SHALL first leave the alternate screen, disable raw mode and bracketed paste, and show the cursor. It SHALL then print one line to standard output, followed by the note for a server from a different build when one applies, and exit as follows:

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
