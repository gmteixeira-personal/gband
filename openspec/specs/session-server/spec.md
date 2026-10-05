# session-server Specification

## Purpose

Defines the gband session server: the process that owns the pane's program, PTY and authoritative screen, keeps them alive while no client is attached, and keeps every attached client's copy of the screen in step with its own.

## Requirements

### Requirement: Socket location
The server SHALL listen on a Unix stream socket at the socket path the command-line capability's server selection resolves. Without a selection option, that path is `default.sock` in the gband runtime directory. The runtime directory SHALL be `$XDG_RUNTIME_DIR/gband/` when `XDG_RUNTIME_DIR` is set to an absolute path, and `/tmp/gband-<uid>/` otherwise, where `<uid>` is the user's numeric user id. When the socket path lies in the runtime directory, the server SHALL create the runtime directory with mode `0700` when it is missing. When the socket path was given with `-p`, the server SHALL NOT create its parent directory, and a missing parent SHALL be refused with one line on standard error naming the directory, and exit status 1. The server SHALL create the socket file with mode `0600`. The server and `gband attach` SHALL resolve the same path from the same environment and options.

#### Scenario: Runtime directory from the environment
- **WHEN** the user runs `gband server` with `XDG_RUNTIME_DIR=/run/user/1000`
- **THEN** the server listens on `/run/user/1000/gband/default.sock`

#### Scenario: No runtime directory in the environment
- **WHEN** the user with uid 1000 runs `gband server` with `XDG_RUNTIME_DIR` unset
- **THEN** the server listens on `/tmp/gband-1000/default.sock`
- **AND** `/tmp/gband-1000/` has mode `0700`

#### Scenario: Named server
- **WHEN** the user runs `gband server -S feature` with `XDG_RUNTIME_DIR=/run/user/1000`
- **THEN** the server listens on `/run/user/1000/gband/feature.sock`

#### Scenario: Socket is private
- **WHEN** a server is listening
- **THEN** its socket file has mode `0600`

#### Scenario: Missing parent of an explicit socket
- **WHEN** the user runs `gband server -p /tmp/missing/s.sock` and `/tmp/missing/` does not exist
- **THEN** standard error names `/tmp/missing/`
- **AND** the process exits with status 1
- **AND** `/tmp/missing/` is not created

### Requirement: Runtime directory ownership
The socket gives whoever can open it control of a shell. When the socket path lies in the runtime directory, the server SHALL refuse to start when the runtime directory exists but is not owned by the user, or grants any permission to group or others. It SHALL print one line to standard error that names the directory and the problem, and exit with status 1. A socket path given with `-p` SHALL rely on the socket's own mode and on the client's check of the server's user instead.

#### Scenario: Directory open to others
- **WHEN** `/tmp/gband-1000/` exists with mode `0777` and the user with uid 1000 runs `gband server` with `XDG_RUNTIME_DIR` unset
- **THEN** standard error names `/tmp/gband-1000/` and its permissions
- **AND** the process exits with status 1
- **AND** no shell is started

#### Scenario: Explicit socket in a shared directory
- **WHEN** the user runs `gband server -p /tmp/s.sock`
- **THEN** the server listens on `/tmp/s.sock` with mode `0600`

### Requirement: Single server per socket
At most one server SHALL serve a socket path. Servers selected with different options that resolve to the same socket path SHALL share one guard. A server that finds another live server on its socket path SHALL print one line to standard error naming the socket path, and exit with status 1 without starting a shell. A socket file that no live server holds SHALL be replaced. Servers on different socket paths SHALL run side by side, each with its own panes.

#### Scenario: Second server refused
- **WHEN** a server is running and the user runs `gband server` again with the same environment and options
- **THEN** the second process prints that a server is already running on the socket path
- **AND** it exits with status 1
- **AND** the first server and its shell keep running

#### Scenario: Same socket through both options
- **WHEN** a server started with `-S feature` is running and the user runs `gband server -p "$XDG_RUNTIME_DIR/gband/feature.sock"`
- **THEN** the second process prints that a server is already running on that socket path
- **AND** it exits with status 1

#### Scenario: Stale socket replaced
- **WHEN** a socket file remains from a server that was killed with SIGKILL and the user runs `gband server`
- **THEN** the new server starts and listens on that path

#### Scenario: Two named servers
- **WHEN** a server named `a` is running and the user runs `gband server -S b`
- **THEN** both servers run, each with its own shell

### Requirement: Pane program
On creating a session, the server SHALL open that session's first pane. Every pane SHALL run the user's shell in its own new PTY: `$SHELL` when it is set, otherwise the user's login shell. The shell SHALL start in its session's working directory. Its environment SHALL be the server's, with `TERM=xterm-256color`, `COLORTERM=truecolor`, `GBAND` set to the socket path, `GBAND_SESSION` set to the session's name and `GBAND_PANE` set to the pane's identifier in decimal. The PTY's initial size SHALL be the terminal size the layout gives the pane for its session's current screen area.

#### Scenario: Environment of the pane
- **WHEN** a client attaches and the user runs `echo $TERM $COLORTERM $GBAND` in the pane
- **THEN** the pane prints `xterm-256color truecolor` followed by the socket path

#### Scenario: Session named in the pane
- **WHEN** a client attaches with `-s work` and the user runs `echo $GBAND_SESSION` in the pane
- **THEN** the pane prints `work`

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
While it holds the single-server guard, the server SHALL record its process id, in decimal followed by a newline, in its lock file. The lock file's path SHALL be the socket path with a final `.sock` replaced by `.lock`, or with `.lock` appended when the socket path does not end in `.sock`. The record SHALL be in place before the server accepts its first connection.

#### Scenario: Pid readable while running
- **WHEN** a server is running without a selection option
- **THEN** `default.lock` in the runtime directory holds that server's process id

#### Scenario: Named server record
- **WHEN** a server named `feature` is running
- **THEN** `feature.lock` in the runtime directory holds that server's process id

#### Scenario: Explicit socket record
- **WHEN** a server started with `-p /tmp/s` is running
- **THEN** `/tmp/s.lock` holds that server's process id

### Requirement: Stop on SIGTERM
When the server receives SIGTERM, it SHALL send SIGHUP to every pane's program in every session. A program still running 2 seconds later SHALL receive SIGKILL, along with its pane's foreground process group. Each session SHALL end as the "Session ends with its last pane" requirement defines, and the server SHALL exit when the last one has ended.

#### Scenario: Terminated with a client attached
- **WHEN** a client is attached to a session of two panes and the server receives SIGTERM
- **THEN** both shells exit
- **AND** the client prints `[exited]`
- **AND** the server exits with status 0 and its socket file is removed

#### Scenario: Terminated with two sessions
- **WHEN** a client is attached to each of the sessions `default` and `work` and the server receives SIGTERM
- **THEN** both clients print `[exited]`
- **AND** the server exits with status 0

#### Scenario: Program ignores SIGHUP
- **WHEN** a pane runs `trap '' HUP; sleep 100` and the server receives SIGTERM
- **THEN** the program is killed within 3 seconds and the server exits with status 0

### Requirement: Kill-server subcommand
`gband kill-server` SHALL stop the server for the socket path it resolves, with every session it hosts, using only the server's pid record and signals, so that it stops a server of any protocol version. It SHALL leave every server on another socket path running. When no server holds the single-server guard, it SHALL print one line to standard error naming the socket path and exit with status 1. Otherwise it SHALL send SIGTERM to the recorded process and wait up to 5 seconds for the guard to be released. It SHALL exit with status 0, printing nothing, when the guard is released. When the guard is still held after 5 seconds, it SHALL print one line to standard error naming the process id and exit with status 1. It SHALL NOT create the runtime directory or any directory of a socket path.

#### Scenario: Stop a running server
- **WHEN** a server is running and the user runs `gband kill-server`
- **THEN** the server and its shell are no longer running
- **AND** `gband kill-server` exits with status 0 and prints nothing

#### Scenario: Stop one named server
- **WHEN** servers named `a` and `b` are running and the user runs `gband -S a kill-server`
- **THEN** the server named `a` is no longer running
- **AND** the server named `b` and its shell keep running

#### Scenario: Stop every session
- **WHEN** a server hosts `default` and `work` and the user runs `gband kill-server`
- **THEN** the shells of both sessions are no longer running

#### Scenario: No server running
- **WHEN** no server is running and the user runs `gband kill-server`
- **THEN** standard error names the socket path and says no server is running
- **AND** the process exits with status 1
- **AND** the runtime directory is not created

#### Scenario: Next attach starts a fresh server
- **WHEN** the user runs `gband kill-server` and then `gband attach`
- **THEN** the client shows a new shell with a different process id

### Requirement: Screen area follows the latest client
Each session SHALL have its own screen area. A session's screen area SHALL be the terminal size reported by the client that most recently attached to that session or reported a resize from it. It SHALL be 80 columns by 24 rows before any client has, unless the session was created on attach. A client detaching SHALL NOT change the screen area. A client SHALL NOT change the screen area of a session it is not attached to. Each pane's terminal size SHALL be the terminal size the layout capability's tile geometry gives that pane for its session's screen area. The server SHALL bring each pane's PTY to its terminal size as "Pane resizes" defines.

#### Scenario: Newer client sets the area
- **WHEN** a client with a 120×40 terminal is attached and a second client with a 100×30 terminal attaches, and the session holds one shown pane in a column of width 1/2
- **THEN** that pane's PTY becomes 48 columns by 28 rows

#### Scenario: Resize from either client
- **WHEN** two clients are attached and the earlier one reports a resize to 90×25, and the session holds one shown pane in a column of width 1/2
- **THEN** that pane's PTY becomes 43 columns by 23 rows

#### Scenario: Other session keeps its area
- **WHEN** a client with a 120×40 terminal is attached to `default` and a client with a 100×30 terminal attaches to `work`
- **THEN** the screen area of `default` stays 120×40

#### Scenario: Opening a pane keeps other sizes
- **WHEN** the screen area is 80×24, one pane runs in a column of width 1/2, and a client opens a new pane
- **THEN** the first pane's PTY stays 38 columns by 22 rows
- **AND** its program receives no SIGWINCH

#### Scenario: Width change resizes the pane
- **WHEN** the screen area is 90×30 and a client cycles the width of a shown pane's column from 1/2 to 2/3
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
| grow width | grow the width of the named pane's column |
| shrink width | shrink the width of the named pane's column |
| grow height | grow the height of the named pane |
| shrink height | shrink the height of the named pane |
| reset height | reset the height of the named pane |

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

#### Scenario: Grow a pane's height
- **WHEN** the screen area is 80×24 and a client asks to grow the height of the top pane of a column holding two panes with automatic heights of weight 1
- **THEN** every attached client receives a layout in which the top pane has a fixed height of 14 rows and the bottom pane an automatic height of weight 1

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
When a pane's program exits, the pane SHALL leave its session's layout once its remaining output has been read, and the server SHALL send the new layout to every client of that session. When the last pane of a session leaves, the server SHALL tell every client attached to that session that the session ended, record the last program's exit status and the session's name in its log, and remove the session. Clients of other sessions SHALL NOT be told anything. When the server's last session has been removed, the server SHALL remove its socket file and exit with status 0.

#### Scenario: One of two shells exits
- **WHEN** a client is attached, two panes run shells, and the user runs `exit` in one of them
- **THEN** the server keeps running
- **AND** the client receives a layout holding only the other pane

#### Scenario: Last shell exits
- **WHEN** a client is attached to the server's only session and the user runs `exit` in the only pane
- **THEN** the client is told the session ended
- **AND** the server exits with status 0
- **AND** the socket file no longer exists

#### Scenario: One of two sessions ends
- **WHEN** a server hosts `default` and `work`, a client is attached to each, and the user runs `exit` in the only pane of `work`
- **THEN** the client of `work` is told the session ended
- **AND** the client of `default` keeps running and is told nothing
- **AND** the server keeps running and hosts only `default`

#### Scenario: Program exits with no client
- **WHEN** `gband server` runs with `SHELL=/bin/true` and no client attaches
- **THEN** the server exits with status 0 without waiting for a client

### Requirement: Terminal query replies
The server SHALL answer the terminal queries in the table below when the pane's program writes them to its PTY. It SHALL write each reply to that PTY after the grid has processed every byte the program wrote before the query. It SHALL answer whether or not a client is attached, and SHALL NOT send replies to clients. Replies and client input SHALL reach the PTY as whole sequences, never interleaved within one another. Positions are 1-based, counted from the top-left cell of the server's own screen of the pane. `<version>` is the package version of the running server.

| query | written by the program | reply |
|---|---|---|
| Primary Device Attributes | `\e[c` or `\e[0c` | `\e[?62;22c` |
| operating status | `\e[5n` | `\e[0n` |
| cursor position | `\e[6n` | `\e[<row>;<column>R`, the cursor's position |
| XTVERSION | `\e[>q` or `\e[>0q` | `\eP>\|gband <version>\e\` |
| DEC private mode report | `\e[?<mode>$p` | `\e[?<mode>;<state>$y` |
| ANSI mode report | `\e[<mode>$p` | `\e[<mode>;0$y` |

In a DEC private mode report, `<state>` SHALL be `1` when the mode is set and `2` when it is reset, for the modes 1, 9, 25, 47, 1000, 1002, 1003, 1005, 1006, 1049 and 2004, read from the server's screen. For any other mode, `<state>` SHALL be `0`, meaning not recognised.

A query the server answers SHALL NOT be recorded in the log as an unhandled sequence. Every other query SHALL stay unanswered.

#### Scenario: fish starts without waiting
- **WHEN** the pane's program is fish 4, which waits for a Primary Device Attributes reply before drawing its prompt
- **THEN** the prompt appears within 1 second of the server starting
- **AND** fish prints no warning about the Primary Device Attributes query

#### Scenario: Device attributes with no client attached
- **WHEN** no client is attached and the pane's program writes `\e[0c`
- **THEN** the program reads `\e[?62;22c` from its terminal

#### Scenario: Cursor position
- **WHEN** the pane's program moves the cursor to row 5, column 10 with `\e[5;10H` and writes `\e[6n`
- **THEN** the program reads `\e[5;10R` from its terminal

#### Scenario: Operating status
- **WHEN** the pane's program writes `\e[5n`
- **THEN** the program reads `\e[0n` from its terminal

#### Scenario: Version
- **WHEN** the pane's program writes `\e[>q`
- **THEN** the program reads `\eP>|gband ` followed by the package version and `\e\` from its terminal

#### Scenario: Mode set
- **WHEN** the pane's program enables bracketed paste with `\e[?2004h` and writes `\e[?2004$p`
- **THEN** the program reads `\e[?2004;1$y` from its terminal

#### Scenario: Mode reset
- **WHEN** the pane's program writes `\e[?1$p` without having enabled application cursor keys
- **THEN** the program reads `\e[?1;2$y` from its terminal

#### Scenario: Mode not recognised
- **WHEN** the pane's program writes `\e[?7727$p`
- **THEN** the program reads `\e[?7727;0$y` from its terminal

#### Scenario: Unsupported query stays unanswered
- **WHEN** the pane's program writes the kitty keyboard query `\e[?u` followed by `\e[c`
- **THEN** the only bytes the program reads from its terminal are `\e[?62;22c`

### Requirement: Named sessions
A server SHALL host one or more sessions, each with a name that is unique within the server. A session name SHALL be 1 to 64 bytes long and hold only ASCII letters, digits, `_` and `-`. Each session SHALL own its own layout, panes, screen area and working directory. A client SHALL attach to exactly one session. The layout, snapshots and updates the server sends a client, and the keys, pastes, resizes and session actions it receives from that client, SHALL concern only the client's session. Every other requirement of this capability that speaks of the session SHALL apply to each session on its own.

#### Scenario: Two sessions are independent
- **WHEN** a server hosts the sessions `work` and `play`, a client attached to `work` opens a pane, and a client is attached to `play`
- **THEN** the client of `play` receives no layout holding the new pane
- **AND** the layout of `play` is unchanged

#### Scenario: Keys stay in their session
- **WHEN** a client attached to `work` types `echo hi` and Enter
- **THEN** no pane of `play` receives those keys

### Requirement: Session on start
On start, the server SHALL create one session, named by the `-s` option the command-line capability defines. That session's working directory SHALL be the server's working directory.

#### Scenario: Default session
- **WHEN** the user runs `gband server` without `-s` and a client lists the sessions
- **THEN** the list holds exactly one session, named `default`

#### Scenario: Named session
- **WHEN** the user runs `gband server -s work` and a client lists the sessions
- **THEN** the list holds exactly one session, named `work`

### Requirement: Attach creates a missing session
When a client asks to attach to a session the server does not host, the server SHALL create that session and attach the client to it. The new session's working directory SHALL be the working directory the client sent with its request, and its screen area SHALL start as the client's terminal size. A client attaching to a session that exists SHALL NOT create one.

#### Scenario: Second session
- **WHEN** a server hosts `default` and the user runs `gband attach -s work` in `/tmp`
- **THEN** the server hosts `default` and `work`
- **AND** `pwd` in the pane of `work` prints `/tmp`
- **AND** the panes of `default` keep running

#### Scenario: Reattach by name
- **WHEN** the user runs `sleep 100` in session `work`, detaches, and runs `gband attach -s work` again
- **THEN** the client shows `sleep 100` still running

### Requirement: Kill-session subcommand
`gband kill-session` SHALL end the session `-s` names, in the server the client selects. The server SHALL send SIGHUP to every pane's program in that session. A program still running 2 seconds later SHALL receive SIGKILL, along with its pane's foreground process group. The session SHALL then end as "Session ends with its last pane" defines. `gband kill-session` SHALL exit with status 0, printing nothing, once the session has ended. When the session has not ended 5 seconds after the request, it SHALL print one line to standard error naming the session and exit with status 1. When the server hosts no session of that name, it SHALL print one line to standard error naming the session and exit with status 1. When no server is running, it SHALL print one line to standard error naming the socket path and exit with status 1. When the server speaks a different protocol version, it SHALL print one line to standard error naming both versions and `gband kill-server`, and exit with status 1. It SHALL NOT replace or start a server, and SHALL NOT create the runtime directory.

#### Scenario: Kill one of two sessions
- **WHEN** a server hosts `default` and `work`, a client is attached to `work`, and the user runs `gband kill-session -s work`
- **THEN** the client of `work` prints `[exited]`
- **AND** `gband kill-session` exits with status 0 and prints nothing
- **AND** the server keeps running and hosts only `default`

#### Scenario: Kill the last session
- **WHEN** a server hosts only `default` and the user runs `gband kill-session`
- **THEN** the server exits with status 0 and its socket file is removed

#### Scenario: Unknown session
- **WHEN** a server hosts only `default` and the user runs `gband kill-session -s nope`
- **THEN** standard error names `nope`
- **AND** the process exits with status 1
- **AND** the session `default` keeps running

#### Scenario: No server for kill-session
- **WHEN** no server is running and the user runs `gband kill-session`
- **THEN** standard error names the socket path and says no server is running
- **AND** the process exits with status 1
- **AND** no server is started

### Requirement: List-sessions subcommand
`gband list-sessions` SHALL print one line to standard output for each session in the server the client selects, sorted by name in byte order. Each line SHALL hold the session's name, the number of panes in its layout and the number of clients attached to it, separated by single tab characters. It SHALL exit with status 0. When no server is running, it SHALL print one line to standard error naming the socket path and exit with status 1. When the server speaks a different protocol version, it SHALL print one line to standard error naming both versions and `gband kill-server`, and exit with status 1. It SHALL NOT replace or start a server, and SHALL NOT create the runtime directory.

#### Scenario: Two sessions listed
- **WHEN** a server hosts `work` with two panes and one attached client, and `default` with one pane and no client, and the user runs `gband list-sessions`
- **THEN** standard output is `default\t1\t0\nwork\t2\t1\n`
- **AND** the process exits with status 0

#### Scenario: No server for list-sessions
- **WHEN** no server is running and the user runs `gband list-sessions`
- **THEN** standard error names the socket path and says no server is running
- **AND** the process exits with status 1
- **AND** no server is started

### Requirement: Pane resizes
The server SHALL send every change of the screen area or the layout to the attached clients at once, as "State updates" defines, without waiting for the PTYs.

A pane SHALL be shown when at least one client attached to its session reports it among its shown panes, as the wire-protocol capability defines. A client that detaches or disconnects SHALL show no pane. A pane SHALL be unshown until a client reports it.

The server SHALL resize PTYs only when the session has settled: 100 ms have passed since the last change of the screen area, the layout or the set of shown panes. It SHALL then resize every shown pane whose PTY size differs from its terminal size, once, to that terminal size. A pane that is not shown SHALL keep its PTY size, whatever the screen area and the layout give it. A new pane's PTY SHALL start at its terminal size.

#### Scenario: Burst of terminal resizes
- **WHEN** a client attached to an 80×24 session showing one pane in a column of width 1/2 reports resizes to 90×25, 95×28 and 100×30, each less than 100 ms after the last
- **THEN** the pane's program receives exactly one SIGWINCH
- **AND** its PTY becomes 48 columns by 28 rows

#### Scenario: Held resize key
- **WHEN** a client showing one pane in a column of width 1/2 asks to grow that column's width four times, each less than 100 ms after the last
- **THEN** every attached client receives a layout after each action
- **AND** the pane's program receives exactly one SIGWINCH, after the last action

#### Scenario: Offscreen pane keeps its size
- **WHEN** a client with an 80×24 terminal views a workspace holding columns A, B and C, all of width 1/2, shows A and B, and resizes its terminal to 100×30
- **THEN** the PTYs of A and B become 48 columns by 28 rows
- **AND** C's PTY stays 38 columns by 22 rows, and C's program receives no SIGWINCH

#### Scenario: Offscreen pane resized when shown
- **WHEN** C's PTY has kept 38 columns by 22 rows while offscreen in a 100×30 area, and the client focuses C so that it shows C
- **THEN** C's PTY becomes 48 columns by 28 rows

#### Scenario: Shown by another client
- **WHEN** two clients are attached, the first shows only pane A, the second shows only pane C, and the screen area changes
- **THEN** both A and C are resized

#### Scenario: Pane leaves a stack while detached
- **WHEN** no client is attached, a column holds P1 and P2, and P2's program exits
- **THEN** P1's PTY keeps its size
- **AND** when a client attaches and shows P1, P1's PTY becomes the size of its tile, which now spans the column's height
