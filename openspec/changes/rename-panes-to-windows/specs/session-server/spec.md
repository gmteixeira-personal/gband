## RENAMED Requirements

- FROM: `### Requirement: Pane program`
- TO: `### Requirement: Window program`

- FROM: `### Requirement: Input to the pane`
- TO: `### Requirement: Input to the window`

- FROM: `### Requirement: Close a pane`
- TO: `### Requirement: Close a window`

- FROM: `### Requirement: Session ends with its last pane`
- TO: `### Requirement: Session ends with its last window`

- FROM: `### Requirement: Pane resizes`
- TO: `### Requirement: Window resizes`

- FROM: `### Requirement: Plugin panes`
- TO: `### Requirement: Drawn windows`

## MODIFIED Requirements

### Requirement: Single server per socket
At most one server SHALL serve a socket path. Servers selected with different options that resolve to the same socket path SHALL share one guard. A server that finds another live server on its socket path SHALL print one line to standard error naming the socket path, and exit with status 1 without starting a shell. A socket file that no live server holds SHALL be replaced. Servers on different socket paths SHALL run side by side, each with its own windows.

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

### Requirement: Window program
On creating a session, the server SHALL open that session's first window. Every window other than a drawn window SHALL run, in its own new PTY, the program its open window action names, or the user's shell when the action names none and for the first window. The user's shell SHALL be `$SHELL` when it is set, otherwise the user's login shell. A program named as a command line SHALL run as the user's shell with the arguments `-c` and that command line. A program named as an argument list SHALL run as that list. The program SHALL start in its session's working directory. Its environment SHALL be the server's without `GBAND_TEST_SOCKET`, with `TERM=xterm-256color`, `COLORTERM=truecolor`, `GBAND` set to the socket path, `GBAND_SESSION` set to the session's name and `GBAND_WINDOW` set to the window's identifier in decimal. The PTY's initial size SHALL be the terminal size the layout gives the window for its session's current screen area.

#### Scenario: Environment of the window
- **WHEN** a client attaches and the user runs `echo $TERM $COLORTERM $GBAND` in the window
- **THEN** the window prints `xterm-256color truecolor` followed by the socket path

#### Scenario: Session named in the window
- **WHEN** a client attaches with `-s work` and the user runs `echo $GBAND_SESSION` in the window
- **THEN** the window prints `work`

#### Scenario: Windows are told apart
- **WHEN** a session holds two windows and the user runs `echo $GBAND_WINDOW` in each
- **THEN** the two windows print different identifiers

#### Scenario: Size before any client
- **WHEN** the server starts and no client has attached
- **THEN** the first window's PTY is 38 columns by 22 rows

#### Scenario: Command line program
- **WHEN** a client asks to open a window naming the command line `echo $GBAND_WINDOW; sleep 5`
- **THEN** the new window prints its identifier, as the user's shell expands it

#### Scenario: Argument list program
- **WHEN** a client asks to open a window naming the argument list `printf`, `%s-%s`, `a`, `b`
- **THEN** the new window prints `a-b` without a shell expanding it

#### Scenario: Program that cannot start
- **WHEN** a client asks to open a window naming the argument list `/nonexistent`
- **THEN** the server records the reason in its log and the layout is unchanged

#### Scenario: Test channel kept out of windows
- **WHEN** a server runs with `GBAND_TEST_SOCKET` set and the user runs `echo "[$GBAND_TEST_SOCKET]"` in a window
- **THEN** the window prints `[]`

### Requirement: Authoritative screen
The server SHALL keep each window's screen state by feeding everything the window's program writes to its PTY into a terminal grid of its own. A drawn window has no program, and its screen SHALL be the one its owner's content messages write, as "Drawn windows" defines. The server SHALL keep reading every window's PTY whether or not a client is attached, so no program ever blocks on output. Escape sequences a grid does not handle SHALL be recorded in the server log at `debug` level, naming the sequence.

#### Scenario: Output while detached
- **WHEN** no client is attached and a window's program writes 1 MiB of output
- **THEN** the program completes without blocking
- **AND** the server's screen of that window holds the last screenful of that output

#### Scenario: Unhandled sequence logged
- **WHEN** the server runs with `GBAND_LOG=debug` and a window's program writes `\x1b[999z`
- **THEN** the server log holds a `debug` event naming the unhandled CSI sequence

### Requirement: Session outlives its clients
The windows' programs SHALL keep running when a client detaches, disconnects or is killed. Detaching or losing any number of clients SHALL NOT signal or restart any program. It SHALL NOT resize any program, except where the drawn windows the client owned leave the layout, as "Drawn windows" defines, and the windows that remain take their tiles' new sizes.

#### Scenario: Client killed
- **WHEN** a client attached to the server is killed with SIGKILL
- **THEN** every window's shell keeps running with the same process id
- **AND** a program one of them was running, such as `sleep 100`, keeps running

#### Scenario: Owner of a stacked drawn window detaches
- **WHEN** a column holds a shell window above a drawn window, and the client that owns the drawn window detaches
- **THEN** the shell keeps running with the same process id
- **AND** once the session settles, the shell's window is resized to the whole column

### Requirement: Snapshot on attach
When a client completes the handshake, the server SHALL send it the layout with the screen area, then one snapshot for each window in the layout. A window's snapshot SHALL reproduce that window's current screen: its size, every visible cell with its attributes, the cursor position and visibility, and the input modes the grid tracks, including application cursor keys and bracketed paste.

#### Scenario: Reattach restores the screen
- **WHEN** the user runs `ls` in a window, kills the client, and runs `gband attach` again
- **THEN** the new client shows the `ls` output and the prompt exactly as the old client showed them
- **AND** the cursor is at the same row and column of that window

#### Scenario: Reattach restores every window
- **WHEN** a session holds two windows, the user runs `echo one` in the first and `echo two` in the second, kills the client and attaches again
- **THEN** the new client receives a layout holding both windows and a snapshot of each
- **AND** the snapshots show `one` and `two` respectively

#### Scenario: Reattach restores input modes
- **WHEN** nvim is running in a window with application cursor keys enabled, the client is killed, and a new client attaches
- **THEN** pressing Up in the new client, with that window focused, sends `\x1bOA` to nvim

### Requirement: State updates
After the snapshots, the server SHALL send each attached client the layout whenever the layout or the screen area changes. Whenever a window's screen changes, the server SHALL send each attached client an update for that window that turns the screen of that window the client last received into its current screen. Updates SHALL carry state, not a replay of a program's output: when a client falls behind, intermediate states MAY be skipped, and the client SHALL still converge on every window's current screen. When a window's PTY size changes, and when a window joins the layout, the server SHALL send each client a snapshot of that window instead of an update, after the layout that holds the change.

#### Scenario: Live output
- **WHEN** a client is attached and the user runs `echo hello` in a window
- **THEN** the client's screen of that window shows `hello` without the client sending anything further

#### Scenario: Slow client converges
- **WHEN** a client stops reading its socket while two windows each print 10 000 lines, then resumes reading
- **THEN** the client's screen of each window matches the server's once the output has stopped

#### Scenario: New window arrives with a snapshot
- **WHEN** a client is attached and another client opens a window
- **THEN** the first client receives a layout holding the new window, followed by a snapshot of it

### Requirement: Input to the window
Every key and paste a client sends SHALL name a window. The server SHALL write each key to the PTY of the window it names, as the bytes the input-encoding capability defines, using the input modes of that window's screen on the server at the moment it writes. It SHALL write each paste the same way. A key or paste naming a window that is not in the layout, or naming a drawn window, SHALL be dropped. Input from every attached client to one window SHALL reach that window's PTY in the order the server receives it.

#### Scenario: Key encoded with current modes
- **WHEN** a client sends the Up key to a window whose program has application cursor keys enabled
- **THEN** the server writes `\x1bOA` to that window's PTY

#### Scenario: Key reaches only its window
- **WHEN** a session holds two windows and a client sends `x` naming the second
- **THEN** only the second window's program receives `x`

#### Scenario: Two clients type
- **WHEN** two clients are attached and each sends a key to the same window
- **THEN** both keys reach that window's program

#### Scenario: Key to a drawn window
- **WHEN** a client sends `x` naming a drawn window
- **THEN** the key is dropped and the drawn window's screen is unchanged

### Requirement: Stop on SIGTERM
When the server receives SIGTERM, it SHALL send SIGHUP to every window's program in every session. A program still running 2 seconds later SHALL receive SIGKILL, along with its window's foreground process group. Each session SHALL end as the "Session ends with its last window" requirement defines, and the server SHALL exit when the last one has ended.

#### Scenario: Terminated with a client attached
- **WHEN** a client is attached to a session of two windows and the server receives SIGTERM
- **THEN** both shells exit
- **AND** the client prints `[exited]`
- **AND** the server exits with status 0 and its socket file is removed

#### Scenario: Terminated with two sessions
- **WHEN** a client is attached to each of the sessions `default` and `work` and the server receives SIGTERM
- **THEN** both clients print `[exited]`
- **AND** the server exits with status 0

#### Scenario: Program ignores SIGHUP
- **WHEN** a window runs `trap '' HUP; sleep 100` and the server receives SIGTERM
- **THEN** the program is killed within 3 seconds and the server exits with status 0

### Requirement: Screen area follows the latest client
Each session SHALL have its own screen area. A session's screen area SHALL be the terminal size reported by the client that most recently attached to that session or reported a resize from it. It SHALL be 80 columns by 24 rows before any client has, unless the session was created on attach. A client detaching SHALL NOT change the screen area. A client SHALL NOT change the screen area of a session it is not attached to. Each window's terminal size SHALL be the terminal size the layout capability's tile geometry gives that window for its session's screen area. The server SHALL bring each window's PTY to its terminal size as "Window resizes" defines.

#### Scenario: Newer client sets the area
- **WHEN** a client with a 120×40 terminal is attached and a second client with a 100×30 terminal attaches, and the session holds one shown window in a column of width 1/2
- **THEN** that window's PTY becomes 48 columns by 28 rows

#### Scenario: Resize from either client
- **WHEN** two clients are attached and the earlier one reports a resize to 90×25, and the session holds one shown window in a column of width 1/2
- **THEN** that window's PTY becomes 43 columns by 23 rows

#### Scenario: Other session keeps its area
- **WHEN** a client with a 120×40 terminal is attached to `default` and a client with a 100×30 terminal attaches to `work`
- **THEN** the screen area of `default` stays 120×40

#### Scenario: Opening a window keeps other sizes
- **WHEN** the screen area is 80×24, one window runs in a column of width 1/2, and a client opens a new window
- **THEN** the first window's PTY stays 38 columns by 22 rows
- **AND** its program receives no SIGWINCH

#### Scenario: Width change resizes the window
- **WHEN** the screen area is 90×30 and a client cycles the width of a shown window's column from 1/2 to 2/3
- **THEN** that window's PTY becomes 58 columns by 28 rows

### Requirement: Session actions
The server SHALL own the session's layout, as the layout capability defines it, and change it only through session actions. It SHALL apply the session actions of every client, and those the server's Lua calls as the server-runtime capability defines, in the order it receives them, and send the resulting layout to every attached client. An action that names a window or a band no longer in the layout SHALL be ignored. The session actions SHALL be:

| action | effect |
|---|---|
| open window | start a window program, as "Window program" defines, or open a drawn window, as "Drawn windows" defines, and place the window as the layout capability's "Open a window" defines, with the column width the action names, tiled or floating as the action names |
| close window | close the named window, as "Close a window" defines |
| consume or expel | as the layout capability defines, left or right |
| move column | move the named window's column, or its floating box, left or right, as the layout capability defines |
| move window | move the named window, or its floating box, down or up, as the layout capability defines |
| toggle floating | float the named tiled window, or tile the named floating window after the tiled window the action names, as the floating-windows capability defines |
| set position | place the named floating window at the column and row the action names, as the floating-windows capability defines |
| cycle width | cycle the width of the named window's column or floating box |
| toggle full width | toggle full width of the named window's column or floating box |
| grow width | grow the width of the named window's column or floating box |
| shrink width | shrink the width of the named window's column or floating box |
| grow height | grow the height of the named window |
| shrink height | shrink the height of the named window |
| reset height | reset the height of the named window |
| set width | set the width of the named window's column or floating box to the width the action names |
| set height | set the height of the named window to the rows or the weight the action names |

Set position naming a tiled window SHALL leave the layout unchanged.

After placing an opened window whose action asks for focus, the server SHALL send the client that asked for it, after the layout that holds the window, a message telling it to focus that window. A window opened by the server's Lua SHALL NOT change any client's focus. When the program of a new window cannot be started, the server SHALL record the reason in its log and leave the layout unchanged.

#### Scenario: Open a window
- **WHEN** two clients are attached and the first asks to open a window next to the window it focuses
- **THEN** both clients receive a layout holding both windows
- **AND** only the first client is told to focus the new window

#### Scenario: Concurrent actions
- **WHEN** two clients each ask to open a window at the same moment
- **THEN** the layout holds three windows, and every client receives the same layout

#### Scenario: Action on a closed window
- **WHEN** a client asks to cycle the width of a window that has already left the layout
- **THEN** the layout is unchanged

#### Scenario: Grow a window's height
- **WHEN** the screen area is 80×24 and a client asks to grow the height of the top window of a column holding two windows with automatic heights of weight 1
- **THEN** every attached client receives a layout in which the top window has a fixed height of 14 rows and the bottom window an automatic height of weight 1

#### Scenario: Open without focus
- **WHEN** a client asks to open a window and the action does not ask for focus
- **THEN** every client receives a layout holding the new window
- **AND** no client is told to focus it

#### Scenario: Open with a width
- **WHEN** a client asks to open a window with the column width 1/4
- **THEN** every client receives a layout whose new column has width 1/4

#### Scenario: Set a column's width
- **WHEN** a client asks to set the width of window 1's column to 2/5
- **THEN** every attached client receives a layout in which window 1's column has width 2/5 and full width off

#### Scenario: Action from the server's Lua
- **WHEN** a server handler of `WindowOpened` calls `gband.action.grow_column_width` for the new window
- **THEN** every attached client receives a layout in which that window's column is wider than the default width

#### Scenario: Float a window for every client
- **WHEN** two clients are attached and the first asks to toggle floating on tiled window 2
- **THEN** both clients receive a layout in which window 2 is in its band's floating list

#### Scenario: Open a floating window with focus
- **WHEN** a client asks to open a floating window that asks for focus
- **THEN** every client receives a layout whose band's floating list ends with the new window
- **AND** only that client is told to focus it

#### Scenario: Set position on a tiled window
- **WHEN** a client asks to set the position of tiled window 1
- **THEN** the layout is unchanged

### Requirement: Close a window
When a client asks to close a drawn window, the window SHALL leave the layout at once. When a client asks to close any other window, the server SHALL send SIGHUP to that window's program. If the program is still running 2 seconds later, the server SHALL send SIGKILL to the window's foreground process group and to its program. The window SHALL leave the layout when its program exits, as "Session ends with its last window" defines.

#### Scenario: Close a shell
- **WHEN** two windows run shells and a client asks to close the second
- **THEN** the second shell exits
- **AND** every client receives a layout without the second window

#### Scenario: Program ignores SIGHUP
- **WHEN** a window runs `trap '' HUP; sleep 100` and a client asks to close it
- **THEN** the program is killed within 3 seconds and the window leaves the layout

#### Scenario: Close a drawn window
- **WHEN** a client asks to close a drawn window
- **THEN** every client receives a layout without that window, with no delay

### Requirement: Session ends with its last window
When a window's program exits, the window SHALL leave its session's layout once its remaining output has been read, and the server SHALL send the new layout to every client of that session. When the last window of a session that is not a drawn window leaves, every drawn window SHALL leave the layout with it, and the server SHALL tell every client attached to that session that the session ended, record the last program's exit status and the session's name in its log, and remove the session. Clients of other sessions SHALL NOT be told anything. When the server's last session has been removed, the server SHALL remove its socket file and exit with status 0.

#### Scenario: One of two shells exits
- **WHEN** a client is attached, two windows run shells, and the user runs `exit` in one of them
- **THEN** the server keeps running
- **AND** the client receives a layout holding only the other window

#### Scenario: Last shell exits
- **WHEN** a client is attached to the server's only session and the user runs `exit` in the only window
- **THEN** the client is told the session ended
- **AND** the server exits with status 0
- **AND** the socket file no longer exists

#### Scenario: One of two sessions ends
- **WHEN** a server hosts `default` and `work`, a client is attached to each, and the user runs `exit` in the only window of `work`
- **THEN** the client of `work` is told the session ended
- **AND** the client of `default` keeps running and is told nothing
- **AND** the server keeps running and hosts only `default`

#### Scenario: Program exits with no client
- **WHEN** `gband server` runs with `SHELL=/bin/true` and no client attaches
- **THEN** the server exits with status 0 without waiting for a client

#### Scenario: Only a drawn window remains
- **WHEN** a client is attached to the server's only session, which holds one shell window and one drawn window, and the user runs `exit` in the shell
- **THEN** the drawn window leaves the layout, the client is told the session ended, and the server exits with status 0

### Requirement: Terminal query replies
The server SHALL answer the terminal queries in the table below when the window's program writes them to its PTY. It SHALL write each reply to that PTY after the grid has processed every byte the program wrote before the query. It SHALL answer whether or not a client is attached, and SHALL NOT send replies to clients. Replies and client input SHALL reach the PTY as whole sequences, never interleaved within one another. Positions are 1-based, counted from the top-left cell of the server's own screen of the window. `<version>` is the package version of the running server.

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
- **WHEN** the window's program is fish 4, which waits for a Primary Device Attributes reply before drawing its prompt
- **THEN** the prompt appears within 1 second of the server starting
- **AND** fish prints no warning about the Primary Device Attributes query

#### Scenario: Device attributes with no client attached
- **WHEN** no client is attached and the window's program writes `\e[0c`
- **THEN** the program reads `\e[?62;22c` from its terminal

#### Scenario: Cursor position
- **WHEN** the window's program moves the cursor to row 5, column 10 with `\e[5;10H` and writes `\e[6n`
- **THEN** the program reads `\e[5;10R` from its terminal

#### Scenario: Operating status
- **WHEN** the window's program writes `\e[5n`
- **THEN** the program reads `\e[0n` from its terminal

#### Scenario: Version
- **WHEN** the window's program writes `\e[>q`
- **THEN** the program reads `\eP>|gband ` followed by the package version and `\e\` from its terminal

#### Scenario: Mode set
- **WHEN** the window's program enables bracketed paste with `\e[?2004h` and writes `\e[?2004$p`
- **THEN** the program reads `\e[?2004;1$y` from its terminal

#### Scenario: Mode reset
- **WHEN** the window's program writes `\e[?1$p` without having enabled application cursor keys
- **THEN** the program reads `\e[?1;2$y` from its terminal

#### Scenario: Mode not recognised
- **WHEN** the window's program writes `\e[?7727$p`
- **THEN** the program reads `\e[?7727;0$y` from its terminal

#### Scenario: Unsupported query stays unanswered
- **WHEN** the window's program writes the kitty keyboard query `\e[?u` followed by `\e[c`
- **THEN** the only bytes the program reads from its terminal are `\e[?62;22c`

### Requirement: Named sessions
A server SHALL host one or more sessions, each with a name that is unique within the server. A session name SHALL be 1 to 64 bytes long and hold only ASCII letters, digits, `_` and `-`. Each session SHALL own its own layout, windows, screen area and working directory. A client SHALL attach to exactly one session. The layout, snapshots and updates the server sends a client, and the keys, pastes, resizes and session actions it receives from that client, SHALL concern only the client's session. Every other requirement of this capability that speaks of the session SHALL apply to each session on its own.

#### Scenario: Two sessions are independent
- **WHEN** a server hosts the sessions `work` and `play`, a client attached to `work` opens a window, and a client is attached to `play`
- **THEN** the client of `play` receives no layout holding the new window
- **AND** the layout of `play` is unchanged

#### Scenario: Keys stay in their session
- **WHEN** a client attached to `work` types `echo hi` and Enter
- **THEN** no window of `play` receives those keys

### Requirement: Attach creates a missing session
When a client asks to attach to a session the server does not host, the server SHALL create that session and attach the client to it. The new session's working directory SHALL be the working directory the client sent with its request, and its screen area SHALL start as the client's terminal size. A client attaching to a session that exists SHALL NOT create one.

#### Scenario: Second session
- **WHEN** a server hosts `default` and the user runs `gband attach -s work` in `/tmp`
- **THEN** the server hosts `default` and `work`
- **AND** `pwd` in the window of `work` prints `/tmp`
- **AND** the windows of `default` keep running

#### Scenario: Reattach by name
- **WHEN** the user runs `sleep 100` in session `work`, detaches, and runs `gband attach -s work` again
- **THEN** the client shows `sleep 100` still running

### Requirement: Kill-session subcommand
`gband kill-session` SHALL end the session `-s` names, in the server the client selects. The server SHALL send SIGHUP to every window's program in that session. A program still running 2 seconds later SHALL receive SIGKILL, along with its window's foreground process group. The session SHALL then end as "Session ends with its last window" defines. `gband kill-session` SHALL exit with status 0, printing nothing, once the session has ended. When the session has not ended 5 seconds after the request, it SHALL print one line to standard error naming the session and exit with status 1. When the server hosts no session of that name, it SHALL print one line to standard error naming the session and exit with status 1. When no server is running, it SHALL print one line to standard error naming the socket path and exit with status 1. When the server speaks a different protocol version, it SHALL print one line to standard error naming both versions and `gband kill-server`, and exit with status 1. It SHALL NOT replace or start a server, and SHALL NOT create the runtime directory.

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
`gband list-sessions` SHALL print one line to standard output for each session in the server the client selects, sorted by name in byte order. Each line SHALL hold the session's name, the number of windows in its layout and the number of clients attached to it, separated by single tab characters. It SHALL exit with status 0. When no server is running, it SHALL print one line to standard error naming the socket path and exit with status 1. When the server speaks a different protocol version, it SHALL print one line to standard error naming both versions and `gband kill-server`, and exit with status 1. It SHALL NOT replace or start a server, and SHALL NOT create the runtime directory.

#### Scenario: Two sessions listed
- **WHEN** a server hosts `work` with two windows and one attached client, and `default` with one window and no client, and the user runs `gband list-sessions`
- **THEN** standard output is `default\t1\t0\nwork\t2\t1\n`
- **AND** the process exits with status 0

#### Scenario: No server for list-sessions
- **WHEN** no server is running and the user runs `gband list-sessions`
- **THEN** standard error names the socket path and says no server is running
- **AND** the process exits with status 1
- **AND** no server is started

### Requirement: Window resizes
The server SHALL send every change of the screen area or the layout to the attached clients at once, as "State updates" defines, without waiting for the PTYs.

A window SHALL be shown when at least one client attached to its session reports it among its shown windows, as the wire-protocol capability defines. A client that detaches or disconnects SHALL show no window. A window SHALL be unshown until a client reports it.

A window's terminal size SHALL be its tile's, as the layout capability gives it, when the window is tiled, and its box's, as the floating-windows capability gives it, when the window floats.

The server SHALL resize PTYs only when the session has settled: 100 ms have passed since the last change of the screen area, the layout or the set of shown windows. It SHALL then resize every shown window whose PTY size differs from its terminal size, once, to that terminal size. A window that is not shown SHALL keep its PTY size, whatever the screen area and the layout give it. A new window's PTY SHALL start at its terminal size.

#### Scenario: Burst of terminal resizes
- **WHEN** a client attached to an 80×24 session showing one window in a column of width 1/2 reports resizes to 90×25, 95×28 and 100×30, each less than 100 ms after the last
- **THEN** the window's program receives exactly one SIGWINCH
- **AND** its PTY becomes 48 columns by 28 rows

#### Scenario: Held resize key
- **WHEN** a client showing one window in a column of width 1/2 asks to grow that column's width four times, each less than 100 ms after the last
- **THEN** every attached client receives a layout after each action
- **AND** the window's program receives exactly one SIGWINCH, after the last action

#### Scenario: Offscreen window keeps its size
- **WHEN** a client with an 80×24 terminal views a band holding columns A, B and C, all of width 1/2, shows A and B, and resizes its terminal to 100×30
- **THEN** the PTYs of A and B become 48 columns by 28 rows
- **AND** C's PTY stays 38 columns by 22 rows, and C's program receives no SIGWINCH

#### Scenario: Offscreen window resized when shown
- **WHEN** C's PTY has kept 38 columns by 22 rows while offscreen in a 100×30 area, and the client focuses C so that it shows C
- **THEN** C's PTY becomes 48 columns by 28 rows

#### Scenario: Shown by another client
- **WHEN** two clients are attached, the first shows only window A, the second shows only window C, and the screen area changes
- **THEN** both A and C are resized

#### Scenario: Window leaves a stack while detached
- **WHEN** no client is attached, a column holds P1 and P2, and P2's program exits
- **THEN** P1's PTY keeps its size
- **AND** when a client attaches and shows P1, P1's PTY becomes the size of its tile, which now spans the column's height

#### Scenario: Floating window takes its box size
- **WHEN** the screen area is 80×24, a shown floating window has width 1/3 and `rows` 12, and its height is grown
- **THEN** once the session settles, its PTY becomes 24 columns by 12 rows

### Requirement: Drawn windows
An open window action MAY name plugin content and a request number instead of a program. The server SHALL then place a **drawn window**: a window with no program and no PTY, owned by the client that asked for it. Its screen SHALL start blank, at the terminal size the layout gives it. The server SHALL send the owning client, after the layout that holds the window and before any focus message for it, an opened message naming the request number and the window. When the server ignores the action, it SHALL send the owning client an opened message naming the request number and no window.

A content message from the owning client SHALL replace the drawn window's screen. The server SHALL feed the message's terminal output into a new blank screen of the window's current terminal size, and that screen SHALL become the window's screen, which "State updates" sends to every client. A content message naming a window the client does not own, a window that is not a drawn window, or a window not in the layout SHALL be ignored.

A drawn window's screen SHALL be resized as "Window resizes" defines for a PTY, keeping its cells where they fit, and no signal SHALL be sent. The server SHALL send each client a snapshot of the window after each resize, as it does when a PTY's size changes. When the owning client detaches, disconnects or is killed, every drawn window it owns SHALL leave the layout. A drawn window SHALL count as a window in the window count that a sessions message reports.

#### Scenario: Open a drawn window
- **WHEN** a client asks to open a window naming plugin content and request number 7, after window 1, with focus
- **THEN** every client receives a layout holding a new window right of window 1's column, followed by a blank snapshot of it
- **AND** the asking client then receives opened naming request 7 and the new window, then a message to focus it

#### Scenario: Ignored request
- **WHEN** a client asks to open a drawn window with request number 8 in a band no longer in the layout
- **THEN** the layout is unchanged and the client receives opened naming request 8 and no window

#### Scenario: Content reaches every client
- **WHEN** two clients are attached and the owner of a 20×5 drawn window sends content whose output writes `hello` at the top-left
- **THEN** both clients' grids of that window show `hello` on the first row and nothing else

#### Scenario: Content replaces the screen
- **WHEN** the owner sends content writing `one`, then content writing `two`
- **THEN** the window's screen shows `two` and no `one`

#### Scenario: Content from another client
- **WHEN** a client that does not own a drawn window sends content naming it
- **THEN** the window's screen is unchanged

#### Scenario: Owner leaves
- **WHEN** two clients are attached, the first owns a drawn window, and the first client detaches
- **THEN** the second client receives a layout without the drawn window

#### Scenario: Resized like a PTY
- **WHEN** a shown drawn window's column grows from width 1/2 to 3/5 in an 80×24 area
- **THEN** after the session settles, every client receives a snapshot of the window at 46 columns by 22 rows
