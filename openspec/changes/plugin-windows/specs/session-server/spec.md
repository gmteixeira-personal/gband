## MODIFIED Requirements

### Requirement: Pane program
On creating a session, the server SHALL open that session's first pane. Every pane other than a plugin pane SHALL run, in its own new PTY, the program its open pane action names, or the user's shell when the action names none and for the first pane. The user's shell SHALL be `$SHELL` when it is set, otherwise the user's login shell. A program named as a command line SHALL run as the user's shell with the arguments `-c` and that command line. A program named as an argument list SHALL run as that list. The program SHALL start in its session's working directory. Its environment SHALL be the server's, with `TERM=xterm-256color`, `COLORTERM=truecolor`, `GBAND` set to the socket path, `GBAND_SESSION` set to the session's name and `GBAND_PANE` set to the pane's identifier in decimal. The PTY's initial size SHALL be the terminal size the layout gives the pane for its session's current screen area.

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

#### Scenario: Command line program
- **WHEN** a client asks to open a pane naming the command line `echo $GBAND_PANE; sleep 5`
- **THEN** the new pane prints its identifier, as the user's shell expands it

#### Scenario: Argument list program
- **WHEN** a client asks to open a pane naming the argument list `printf`, `%s-%s`, `a`, `b`
- **THEN** the new pane prints `a-b` without a shell expanding it

#### Scenario: Program that cannot start
- **WHEN** a client asks to open a pane naming the argument list `/nonexistent`
- **THEN** the server records the reason in its log and the layout is unchanged

### Requirement: Input to the pane
Every key and paste a client sends SHALL name a pane. The server SHALL write each key to the PTY of the pane it names, as the bytes the input-encoding capability defines, using the input modes of that pane's screen on the server at the moment it writes. It SHALL write each paste the same way. A key or paste naming a pane that is not in the layout, or naming a plugin pane, SHALL be dropped. Input from every attached client to one pane SHALL reach that pane's PTY in the order the server receives it.

#### Scenario: Key encoded with current modes
- **WHEN** a client sends the Up key to a pane whose program has application cursor keys enabled
- **THEN** the server writes `\x1bOA` to that pane's PTY

#### Scenario: Key reaches only its pane
- **WHEN** a session holds two panes and a client sends `x` naming the second
- **THEN** only the second pane's program receives `x`

#### Scenario: Two clients type
- **WHEN** two clients are attached and each sends a key to the same pane
- **THEN** both keys reach that pane's program

#### Scenario: Key to a plugin pane
- **WHEN** a client sends `x` naming a plugin pane
- **THEN** the key is dropped and the plugin pane's screen is unchanged

### Requirement: Session actions
The server SHALL own the session's layout, as the layout capability defines it, and change it only through session actions. It SHALL apply the session actions of every client in the order it receives them, and send the resulting layout to every attached client. An action that names a pane or a band no longer in the layout SHALL be ignored. The session actions SHALL be:

| action | effect |
|---|---|
| open pane | start a pane program, as "Pane program" defines, or open a plugin pane, as "Plugin panes" defines, and place the pane as the layout capability's "Open a pane" defines, with the column width the action names |
| close pane | close the named pane, as "Close a pane" defines |
| consume or expel | as the layout capability defines, left or right |
| cycle width | cycle the width of the named pane's column |
| toggle full width | toggle full width of the named pane's column |
| grow width | grow the width of the named pane's column |
| shrink width | shrink the width of the named pane's column |
| grow height | grow the height of the named pane |
| shrink height | shrink the height of the named pane |
| reset height | reset the height of the named pane |
| set width | set the width of the named pane's column to the width the action names |
| set height | set the height of the named pane to the rows or the weight the action names |

After placing an opened pane whose action asks for focus, the server SHALL send the client that asked for it, after the layout that holds the pane, a message telling it to focus that pane. When the program of a new pane cannot be started, the server SHALL record the reason in its log and leave the layout unchanged.

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

#### Scenario: Open without focus
- **WHEN** a client asks to open a pane and the action does not ask for focus
- **THEN** every client receives a layout holding the new pane
- **AND** no client is told to focus it

#### Scenario: Open with a width
- **WHEN** a client asks to open a pane with the column width 1/4
- **THEN** every client receives a layout whose new column has width 1/4

#### Scenario: Set a column's width
- **WHEN** a client asks to set the width of pane 1's column to 2/5
- **THEN** every attached client receives a layout in which pane 1's column has width 2/5 and full width off

### Requirement: Close a pane
When a client asks to close a plugin pane, the pane SHALL leave the layout at once. When a client asks to close any other pane, the server SHALL send SIGHUP to that pane's program. If the program is still running 2 seconds later, the server SHALL send SIGKILL to the pane's foreground process group and to its program. The pane SHALL leave the layout when its program exits, as "Session ends with its last pane" defines.

#### Scenario: Close a shell
- **WHEN** two panes run shells and a client asks to close the second
- **THEN** the second shell exits
- **AND** every client receives a layout without the second pane

#### Scenario: Program ignores SIGHUP
- **WHEN** a pane runs `trap '' HUP; sleep 100` and a client asks to close it
- **THEN** the program is killed within 3 seconds and the pane leaves the layout

#### Scenario: Close a plugin pane
- **WHEN** a client asks to close a plugin pane
- **THEN** every client receives a layout without that pane, with no delay

### Requirement: Session ends with its last pane
When a pane's program exits, the pane SHALL leave its session's layout once its remaining output has been read, and the server SHALL send the new layout to every client of that session. When the last pane of a session that is not a plugin pane leaves, every plugin pane SHALL leave the layout with it, and the server SHALL tell every client attached to that session that the session ended, record the last program's exit status and the session's name in its log, and remove the session. Clients of other sessions SHALL NOT be told anything. When the server's last session has been removed, the server SHALL remove its socket file and exit with status 0.

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

#### Scenario: Only a plugin pane remains
- **WHEN** a client is attached to the server's only session, which holds one shell pane and one plugin pane, and the user runs `exit` in the shell
- **THEN** the plugin pane leaves the layout, the client is told the session ended, and the server exits with status 0

## ADDED Requirements

### Requirement: Plugin panes
An open pane action MAY name plugin content and a request number instead of a program. The server SHALL then place a **plugin pane**: a pane with no program and no PTY, owned by the client that asked for it. Its screen SHALL start blank, at the terminal size the layout gives it. The server SHALL send the owning client, after the layout that holds the pane and before any focus message for it, an opened message naming the request number and the pane. When the server ignores the action, it SHALL send the owning client an opened message naming the request number and no pane.

A content message from the owning client SHALL replace the plugin pane's screen. The server SHALL feed the message's terminal output into a new blank screen of the pane's current terminal size, and that screen SHALL become the pane's screen, which "State updates" sends to every client. A content message naming a pane the client does not own, a pane that is not a plugin pane, or a pane not in the layout SHALL be ignored.

A plugin pane's screen SHALL be resized as "Pane resizes" defines for a PTY, keeping its cells where they fit, and no signal SHALL be sent. The server SHALL send each client a snapshot of the pane after each resize, as it does when a PTY's size changes. When the owning client detaches, disconnects or is killed, every plugin pane it owns SHALL leave the layout. A plugin pane SHALL count as a pane in the pane count that a sessions message reports.

#### Scenario: Open a plugin pane
- **WHEN** a client asks to open a pane naming plugin content and request number 7, after pane 1, with focus
- **THEN** every client receives a layout holding a new pane right of pane 1's column, followed by a blank snapshot of it
- **AND** the asking client then receives opened naming request 7 and the new pane, then a message to focus it

#### Scenario: Ignored request
- **WHEN** a client asks to open a plugin pane with request number 8 in a band no longer in the layout
- **THEN** the layout is unchanged and the client receives opened naming request 8 and no pane

#### Scenario: Content reaches every client
- **WHEN** two clients are attached and the owner of a 20×5 plugin pane sends content whose output writes `hello` at the top-left
- **THEN** both clients' grids of that pane show `hello` on the first row and nothing else

#### Scenario: Content replaces the screen
- **WHEN** the owner sends content writing `one`, then content writing `two`
- **THEN** the pane's screen shows `two` and no `one`

#### Scenario: Content from another client
- **WHEN** a client that does not own a plugin pane sends content naming it
- **THEN** the pane's screen is unchanged

#### Scenario: Owner leaves
- **WHEN** two clients are attached, the first owns a plugin pane, and the first client detaches
- **THEN** the second client receives a layout without the plugin pane

#### Scenario: Resized like a PTY
- **WHEN** a shown plugin pane's column grows from width 1/2 to 3/5 in an 80×24 area
- **THEN** after the session settles, every client receives a snapshot of the pane at 46 columns by 22 rows
