## ADDED Requirements

### Requirement: Named sessions
A server SHALL host one or more sessions, each with a name that is unique within the server. A session name SHALL be 1 to 64 bytes long and hold only ASCII letters, digits, `.`, `_` and `-`. Each session SHALL own its own layout, panes, screen area and working directory. A client SHALL attach to exactly one session. The layout, snapshots and updates the server sends a client, and the keys, pastes, resizes and session actions it receives from that client, SHALL concern only the client's session. Every other requirement of this capability that speaks of the session SHALL apply to each session on its own.

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

## MODIFIED Requirements

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

### Requirement: Screen area follows the latest client
Each session SHALL have its own screen area. A session's screen area SHALL be the terminal size reported by the client that most recently attached to that session or reported a resize from it. It SHALL be 80 columns by 24 rows before any client has, unless the session was created on attach. A client detaching SHALL NOT change the screen area. A client SHALL NOT change the screen area of a session it is not attached to. Each pane's PTY size SHALL be the terminal size the layout capability's tile geometry gives that pane for its session's screen area. Whenever the screen area or the layout changes a pane's terminal size, the server SHALL resize that pane's PTY. A pane whose terminal size did not change SHALL NOT be resized.

#### Scenario: Newer client sets the area
- **WHEN** a client with a 120×40 terminal is attached and a second client with a 100×30 terminal attaches, and the session holds one pane in a column of width 1/2
- **THEN** that pane's PTY becomes 48 columns by 28 rows

#### Scenario: Resize from either client
- **WHEN** two clients are attached and the earlier one reports a resize to 90×25, and the session holds one pane in a column of width 1/2
- **THEN** that pane's PTY becomes 43 columns by 23 rows

#### Scenario: Other session keeps its area
- **WHEN** a client with a 120×40 terminal is attached to `default` and a client with a 100×30 terminal attaches to `work`
- **THEN** the screen area of `default` stays 120×40

#### Scenario: Opening a pane keeps other sizes
- **WHEN** the screen area is 80×24, one pane runs in a column of width 1/2, and a client opens a new pane
- **THEN** the first pane's PTY stays 38 columns by 22 rows
- **AND** its program receives no SIGWINCH

#### Scenario: Width change resizes the pane
- **WHEN** the screen area is 90×30 and a client cycles the width of a pane's column from 1/2 to 2/3
- **THEN** that pane's PTY becomes 58 columns by 28 rows

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
`gband kill-server` SHALL stop the server for the socket path it resolves, with every session it hosts, using only the server's pid record and signals, so that it stops a server of any protocol version. When no server holds the single-server guard, it SHALL print one line to standard error naming the socket path and exit with status 1. Otherwise it SHALL send SIGTERM to the recorded process and wait up to 5 seconds for the guard to be released. It SHALL exit with status 0, printing nothing, when the guard is released. When the guard is still held after 5 seconds, it SHALL print one line to standard error naming the process id and exit with status 1. It SHALL NOT create the runtime directory.

#### Scenario: Stop a running server
- **WHEN** a server is running and the user runs `gband kill-server`
- **THEN** the server and its shell are no longer running
- **AND** `gband kill-server` exits with status 0 and prints nothing

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
