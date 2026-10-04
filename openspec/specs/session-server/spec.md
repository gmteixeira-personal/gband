# session-server Specification

## Purpose

Defines the gband session server: the process that owns the pane's program, PTY and authoritative screen, keeps them alive while no client is attached, and keeps every attached client's copy of the screen in step with its own.

## Requirements

### Requirement: Socket location
The server SHALL listen on a Unix stream socket named `default.sock` in the gband runtime directory. The runtime directory SHALL be `$XDG_RUNTIME_DIR/gband/` when `XDG_RUNTIME_DIR` is set to an absolute path, and `/tmp/gband-<uid>/` otherwise, where `<uid>` is the user's numeric user id. The server SHALL create the runtime directory with mode `0700` when it is missing. The server and `gband attach` SHALL resolve the same path from the same environment.

#### Scenario: Runtime directory from the environment
- **WHEN** the user runs `gband server` with `XDG_RUNTIME_DIR=/run/user/1000`
- **THEN** the server listens on `/run/user/1000/gband/default.sock`

#### Scenario: No runtime directory in the environment
- **WHEN** the user with uid 1000 runs `gband server` with `XDG_RUNTIME_DIR` unset
- **THEN** the server listens on `/tmp/gband-1000/default.sock`
- **AND** `/tmp/gband-1000/` has mode `0700`

### Requirement: Runtime directory ownership
The socket gives whoever can open it control of a shell. The server SHALL refuse to start when the runtime directory exists but is not owned by the user, or grants any permission to group or others. It SHALL print one line to standard error that names the directory and the problem, and exit with status 1.

#### Scenario: Directory open to others
- **WHEN** `/tmp/gband-1000/` exists with mode `0777` and the user with uid 1000 runs `gband server` with `XDG_RUNTIME_DIR` unset
- **THEN** standard error names `/tmp/gband-1000/` and its permissions
- **AND** the process exits with status 1
- **AND** no shell is started

### Requirement: Single server per socket
At most one server SHALL serve a socket path. A server that finds another live server on its socket path SHALL print one line to standard error naming the socket path, and exit with status 1 without starting a shell. A socket file that no live server holds SHALL be replaced.

#### Scenario: Second server refused
- **WHEN** a server is running and the user runs `gband server` again with the same environment
- **THEN** the second process prints that a server is already running on the socket path
- **AND** it exits with status 1
- **AND** the first server and its shell keep running

#### Scenario: Stale socket replaced
- **WHEN** a socket file remains from a server that was killed with SIGKILL and the user runs `gband server`
- **THEN** the new server starts and listens on that path

### Requirement: Pane program
On start, the server SHALL run the user's shell in a new PTY: `$SHELL` when it is set, otherwise the user's login shell. The shell SHALL start in the server's working directory. Its environment SHALL be the server's, with `TERM=xterm-256color`, `COLORTERM=truecolor` and `GBAND` set to the socket path. The PTY SHALL be 80 columns by 24 rows until a client sets its size.

#### Scenario: Environment of the pane
- **WHEN** a client attaches and the user runs `echo $TERM $COLORTERM $GBAND` in the pane
- **THEN** the pane prints `xterm-256color truecolor` followed by the socket path

#### Scenario: Size before any client
- **WHEN** the server starts and no client has attached
- **THEN** the PTY is 80 columns by 24 rows

### Requirement: Authoritative screen
The server SHALL keep the pane's screen state by feeding everything the program writes to the PTY into its own terminal grid. It SHALL keep reading the PTY whether or not a client is attached, so the program never blocks on output. Escape sequences the grid does not handle SHALL be recorded in the server log at `debug` level, naming the sequence.

#### Scenario: Output while detached
- **WHEN** no client is attached and the pane's program writes 1 MiB of output
- **THEN** the program completes without blocking
- **AND** the server's screen holds the last screenful of that output

#### Scenario: Unhandled sequence logged
- **WHEN** the server runs with `GBAND_LOG=debug` and the pane's program writes `\x1b[999z`
- **THEN** the server log holds a `debug` event naming the unhandled CSI sequence

### Requirement: Session outlives its clients
The pane's program SHALL keep running when a client detaches, disconnects or is killed. Detaching or losing any number of clients SHALL NOT signal, restart or resize the program.

#### Scenario: Client killed
- **WHEN** a client attached to the server is killed with SIGKILL
- **THEN** the shell keeps running with the same process id
- **AND** a program it was running, such as `sleep 100`, keeps running

### Requirement: Snapshot on attach
When a client completes the handshake, the server SHALL send it one snapshot that reproduces the current screen: its size, every visible cell with its attributes, the cursor position and visibility, and the input modes the grid tracks, including application cursor keys and bracketed paste.

#### Scenario: Reattach restores the screen
- **WHEN** the user runs `ls` in the pane, kills the client, and runs `gband attach` again
- **THEN** the new client shows the `ls` output and the prompt exactly as the old client showed them
- **AND** the cursor is at the same row and column

#### Scenario: Reattach restores input modes
- **WHEN** nvim is running in the pane with application cursor keys enabled, the client is killed, and a new client attaches
- **THEN** pressing Up in the new client sends `\x1bOA` to nvim

### Requirement: State updates
After the snapshot, whenever the server's screen changes, the server SHALL send each attached client an update that turns the screen that client last received into the current screen. Updates SHALL carry state, not a replay of the program's output: when a client falls behind, intermediate states MAY be skipped, and the client SHALL still converge on the current screen. When the PTY size changes, the server SHALL send each client a new snapshot instead of an update.

#### Scenario: Live output
- **WHEN** a client is attached and the user runs `echo hello`
- **THEN** the client's screen shows `hello` without the client sending anything further

#### Scenario: Slow client converges
- **WHEN** a client stops reading its socket while the pane prints 10 000 lines, then resumes reading
- **THEN** the client's screen matches the server's screen once the output has stopped

### Requirement: Input to the pane
The server SHALL write each key a client sends to the PTY as the bytes the input-encoding capability defines, using the input modes of the server's own screen at the moment it writes. It SHALL write each paste the same way. Input from every attached client SHALL reach the same PTY, in the order the server receives it.

#### Scenario: Key encoded with current modes
- **WHEN** a client sends the Up key while the pane's program has application cursor keys enabled
- **THEN** the server writes `\x1bOA` to the PTY

#### Scenario: Two clients type
- **WHEN** two clients are attached and each sends a key
- **THEN** both keys reach the pane's program

### Requirement: PTY size follows the latest client
The PTY size SHALL be the terminal size reported by the client that most recently attached or reported a resize. A client detaching SHALL NOT change the PTY size.

#### Scenario: Newer client sets the size
- **WHEN** a client with a 120×40 terminal is attached and a second client with a 100×30 terminal attaches
- **THEN** the PTY becomes 100 columns by 30 rows

#### Scenario: Resize from either client
- **WHEN** two clients are attached and the earlier one reports a resize to 90×25
- **THEN** the PTY becomes 90 columns by 25 rows

### Requirement: Server exits with its program
When the pane's program exits, the server SHALL tell every attached client that the session ended, remove its socket file, record the program's exit status in its log, and exit with status 0.

#### Scenario: Shell exits
- **WHEN** a client is attached and the user runs `exit` in the pane
- **THEN** the client is told the session ended
- **AND** the server exits with status 0
- **AND** the socket file no longer exists

#### Scenario: Program exits with no client
- **WHEN** `gband server` runs with `SHELL=/bin/true` and no client attaches
- **THEN** the server exits with status 0 without waiting for a client

### Requirement: Pid record
While it holds the single-server guard, the server SHALL record its process id, in decimal followed by a newline, in the file `default.lock` in the runtime directory. The record SHALL be in place before the server accepts its first connection.

#### Scenario: Pid readable while running
- **WHEN** a server is running
- **THEN** `default.lock` holds that server's process id

### Requirement: Stop on SIGTERM
When the server receives SIGTERM, it SHALL send SIGHUP to the pane's program. If the program is still running 2 seconds later, the server SHALL send it SIGKILL. When the program has exited, the server SHALL end the session as the "Server exits with its program" requirement defines.

#### Scenario: Terminated with a client attached
- **WHEN** a client is attached and the server receives SIGTERM
- **THEN** the shell exits
- **AND** the client prints `[exited]`
- **AND** the server exits with status 0 and its socket file is removed

#### Scenario: Program ignores SIGHUP
- **WHEN** the pane runs `trap '' HUP; sleep 100` and the server receives SIGTERM
- **THEN** the program is killed within 3 seconds and the server exits with status 0

### Requirement: Kill-server subcommand
`gband kill-server` SHALL stop the server for the socket path it resolves, using only the server's pid record and signals, so that it stops a server of any protocol version. When no server holds the single-server guard, it SHALL print one line to standard error naming the socket path and exit with status 1. Otherwise it SHALL send SIGTERM to the recorded process and wait up to 5 seconds for the guard to be released. It SHALL exit with status 0, printing nothing, when the guard is released. When the guard is still held after 5 seconds, it SHALL print one line to standard error naming the process id and exit with status 1. It SHALL NOT create the runtime directory.

#### Scenario: Stop a running server
- **WHEN** a server is running and the user runs `gband kill-server`
- **THEN** the server and its shell are no longer running
- **AND** `gband kill-server` exits with status 0 and prints nothing

#### Scenario: No server running
- **WHEN** no server is running and the user runs `gband kill-server`
- **THEN** standard error names the socket path and says no server is running
- **AND** the process exits with status 1
- **AND** the runtime directory is not created

#### Scenario: Next attach starts a fresh server
- **WHEN** the user runs `gband kill-server` and then `gband attach`
- **THEN** the client shows a new shell with a different process id
