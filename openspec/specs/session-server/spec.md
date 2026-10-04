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
On start, the server SHALL open the session's first pane. Every pane SHALL run the user's shell in its own new PTY: `$SHELL` when it is set, otherwise the user's login shell. The shell SHALL start in the server's working directory. Its environment SHALL be the server's, with `TERM=xterm-256color`, `COLORTERM=truecolor`, `GBAND` set to the socket path and `GBAND_PANE` set to the pane's identifier in decimal. The PTY's initial size SHALL be the terminal size the layout gives the pane for the current screen area.

#### Scenario: Environment of the pane
- **WHEN** a client attaches and the user runs `echo $TERM $COLORTERM $GBAND` in the pane
- **THEN** the pane prints `xterm-256color truecolor` followed by the socket path

#### Scenario: Panes are told apart
- **WHEN** a session holds two panes and the user runs `echo $GBAND_PANE` in each
- **THEN** the two panes print different identifiers

#### Scenario: Size before any client
- **WHEN** the server starts and no client has attached
- **THEN** the first pane's PTY is 38 columns by 22 rows

### Requirement: Authoritative screen
The server SHALL keep each pane's screen state by feeding everything the pane's program writes to its PTY into a terminal grid of its own. It SHALL keep reading every pane's PTY whether or not a client is attached, so no program ever blocks on output. Escape sequences a grid does not handle SHALL be recorded in the server log at `debug` level, naming the sequence.

#### Scenario: Output while detached
- **WHEN** no client is attached and a pane's program writes 1 MiB of output
- **THEN** the program completes without blocking
- **AND** the server's screen of that pane holds the last screenful of that output

#### Scenario: Unhandled sequence logged
- **WHEN** the server runs with `GBAND_LOG=debug` and a pane's program writes `\x1b[999z`
- **THEN** the server log holds a `debug` event naming the unhandled CSI sequence

### Requirement: Session outlives its clients
The panes' programs SHALL keep running when a client detaches, disconnects or is killed. Detaching or losing any number of clients SHALL NOT signal, restart or resize any program.

#### Scenario: Client killed
- **WHEN** a client attached to the server is killed with SIGKILL
- **THEN** every pane's shell keeps running with the same process id
- **AND** a program one of them was running, such as `sleep 100`, keeps running

### Requirement: Snapshot on attach
When a client completes the handshake, the server SHALL send it the layout with the screen area, then one snapshot for each pane in the layout. A pane's snapshot SHALL reproduce that pane's current screen: its size, every visible cell with its attributes, the cursor position and visibility, and the input modes the grid tracks, including application cursor keys and bracketed paste.

#### Scenario: Reattach restores the screen
- **WHEN** the user runs `ls` in a pane, kills the client, and runs `gband attach` again
- **THEN** the new client shows the `ls` output and the prompt exactly as the old client showed them
- **AND** the cursor is at the same row and column of that pane

#### Scenario: Reattach restores every pane
- **WHEN** a session holds two panes, the user runs `echo one` in the first and `echo two` in the second, kills the client and attaches again
- **THEN** the new client receives a layout holding both panes and a snapshot of each
- **AND** the snapshots show `one` and `two` respectively

#### Scenario: Reattach restores input modes
- **WHEN** nvim is running in a pane with application cursor keys enabled, the client is killed, and a new client attaches
- **THEN** pressing Up in the new client, with that pane focused, sends `\x1bOA` to nvim

### Requirement: State updates
After the snapshots, the server SHALL send each attached client the layout whenever the layout or the screen area changes. Whenever a pane's screen changes, the server SHALL send each attached client an update for that pane that turns the screen of that pane the client last received into its current screen. Updates SHALL carry state, not a replay of a program's output: when a client falls behind, intermediate states MAY be skipped, and the client SHALL still converge on every pane's current screen. When a pane's PTY size changes, and when a pane joins the layout, the server SHALL send each client a snapshot of that pane instead of an update, after the layout that holds the change.

#### Scenario: Live output
- **WHEN** a client is attached and the user runs `echo hello` in a pane
- **THEN** the client's screen of that pane shows `hello` without the client sending anything further

#### Scenario: Slow client converges
- **WHEN** a client stops reading its socket while two panes each print 10 000 lines, then resumes reading
- **THEN** the client's screen of each pane matches the server's once the output has stopped

#### Scenario: New pane arrives with a snapshot
- **WHEN** a client is attached and another client opens a pane
- **THEN** the first client receives a layout holding the new pane, followed by a snapshot of it

### Requirement: Input to the pane
Every key and paste a client sends SHALL name a pane. The server SHALL write each key to the PTY of the pane it names, as the bytes the input-encoding capability defines, using the input modes of that pane's screen on the server at the moment it writes. It SHALL write each paste the same way. A key or paste naming a pane that is not in the layout SHALL be dropped. Input from every attached client to one pane SHALL reach that pane's PTY in the order the server receives it.

#### Scenario: Key encoded with current modes
- **WHEN** a client sends the Up key to a pane whose program has application cursor keys enabled
- **THEN** the server writes `\x1bOA` to that pane's PTY

#### Scenario: Key reaches only its pane
- **WHEN** a session holds two panes and a client sends `x` naming the second
- **THEN** only the second pane's program receives `x`

#### Scenario: Two clients type
- **WHEN** two clients are attached and each sends a key to the same pane
- **THEN** both keys reach that pane's program

### Requirement: Pid record
While it holds the single-server guard, the server SHALL record its process id, in decimal followed by a newline, in the file `default.lock` in the runtime directory. The record SHALL be in place before the server accepts its first connection.

#### Scenario: Pid readable while running
- **WHEN** a server is running
- **THEN** `default.lock` holds that server's process id

### Requirement: Stop on SIGTERM
When the server receives SIGTERM, it SHALL send SIGHUP to every pane's program. A program still running 2 seconds later SHALL receive SIGKILL, along with its pane's foreground process group. When every program has exited, the server SHALL end the session as the "Session ends with its last pane" requirement defines.

#### Scenario: Terminated with a client attached
- **WHEN** a client is attached to a session of two panes and the server receives SIGTERM
- **THEN** both shells exit
- **AND** the client prints `[exited]`
- **AND** the server exits with status 0 and its socket file is removed

#### Scenario: Program ignores SIGHUP
- **WHEN** a pane runs `trap '' HUP; sleep 100` and the server receives SIGTERM
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

### Requirement: Screen area follows the latest client
The screen area SHALL be the terminal size reported by the client that most recently attached or reported a resize, and SHALL be 80 columns by 24 rows before any client has. A client detaching SHALL NOT change the screen area. Each pane's PTY size SHALL be the terminal size the layout capability's tile geometry gives that pane for the screen area. Whenever the screen area or the layout changes a pane's terminal size, the server SHALL resize that pane's PTY. A pane whose terminal size did not change SHALL NOT be resized.

#### Scenario: Newer client sets the area
- **WHEN** a client with a 120×40 terminal is attached and a second client with a 100×30 terminal attaches, and the session holds one pane in a column of width 1/2
- **THEN** that pane's PTY becomes 48 columns by 28 rows

#### Scenario: Resize from either client
- **WHEN** two clients are attached and the earlier one reports a resize to 90×25, and the session holds one pane in a column of width 1/2
- **THEN** that pane's PTY becomes 43 columns by 23 rows

#### Scenario: Opening a pane keeps other sizes
- **WHEN** the screen area is 80×24, one pane runs in a column of width 1/2, and a client opens a new pane
- **THEN** the first pane's PTY stays 38 columns by 22 rows
- **AND** its program receives no SIGWINCH

#### Scenario: Width change resizes the pane
- **WHEN** the screen area is 90×30 and a client cycles the width of a pane's column from 1/2 to 2/3
- **THEN** that pane's PTY becomes 58 columns by 28 rows

### Requirement: Session actions
The server SHALL own the session's layout, as the layout capability defines it, and change it only through session actions. It SHALL apply the session actions of every client in the order it receives them, and send the resulting layout to every attached client. An action that names a pane or a workspace no longer in the layout SHALL be ignored. The session actions SHALL be:

| action | effect |
|---|---|
| open pane | start a pane program, as "Pane programs" defines, and place the pane as the layout capability's "Open a pane" defines |
| close pane | close the named pane, as "Close a pane" defines |
| consume or expel | as the layout capability defines, left or right |
| cycle width | cycle the width of the named pane's column |
| toggle full width | toggle full width of the named pane's column |

After placing an opened pane, the server SHALL send the client that asked for it, after the layout that holds the pane, a message telling it to focus that pane. When the program of a new pane cannot be started, the server SHALL record the reason in its log and leave the layout unchanged.

#### Scenario: Open a pane
- **WHEN** two clients are attached and the first asks to open a pane next to the pane it focuses
- **THEN** both clients receive a layout holding both panes
- **AND** only the first client is told to focus the new pane

#### Scenario: Concurrent actions
- **WHEN** two clients each ask to open a pane at the same moment
- **THEN** the layout holds three panes, and every client receives the same layout

#### Scenario: Action on a closed pane
- **WHEN** a client asks to cycle the width of a pane that has already left the layout
- **THEN** the layout is unchanged

### Requirement: Close a pane
When a client asks to close a pane, the server SHALL send SIGHUP to that pane's program. If the program is still running 2 seconds later, the server SHALL send SIGKILL to the pane's foreground process group and to its program. The pane SHALL leave the layout when its program exits, as "Session ends with its last pane" defines.

#### Scenario: Close a shell
- **WHEN** two panes run shells and a client asks to close the second
- **THEN** the second shell exits
- **AND** every client receives a layout without the second pane

#### Scenario: Program ignores SIGHUP
- **WHEN** a pane runs `trap '' HUP; sleep 100` and a client asks to close it
- **THEN** the program is killed within 3 seconds and the pane leaves the layout

### Requirement: Session ends with its last pane
When a pane's program exits, the pane SHALL leave the layout once its remaining output has been read, and the server SHALL send the new layout to every client. When the last pane leaves, the server SHALL tell every attached client that the session ended, remove its socket file, record the last program's exit status in its log, and exit with status 0.

#### Scenario: One of two shells exits
- **WHEN** a client is attached, two panes run shells, and the user runs `exit` in one of them
- **THEN** the server keeps running
- **AND** the client receives a layout holding only the other pane

#### Scenario: Last shell exits
- **WHEN** a client is attached and the user runs `exit` in the only pane
- **THEN** the client is told the session ended
- **AND** the server exits with status 0
- **AND** the socket file no longer exists

#### Scenario: Program exits with no client
- **WHEN** `gband server` runs with `SHELL=/bin/true` and no client attaches
- **THEN** the server exits with status 0 without waiting for a client
