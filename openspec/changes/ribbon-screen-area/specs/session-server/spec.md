## MODIFIED Requirements

### Requirement: Screen area follows the latest client
Each session SHALL have its own screen area. A session's screen area SHALL be the reported size of the client that most recently attached to that session or reported a resize from it, as the client-attach capability defines the reported size: the client's terminal less the columns its bars take. It SHALL be 80 columns by 24 rows before any client has, unless the session was created on attach. A client detaching SHALL NOT change the screen area. A client SHALL NOT change the screen area of a session it is not attached to. Each window's terminal size SHALL be the terminal size the layout capability's tile geometry gives that window for its session's screen area. The server SHALL bring each window's PTY to its terminal size as "Window resizes" defines.

#### Scenario: Newer client sets the area
- **WHEN** a client reporting the size 120×40 is attached and a second client reporting the size 100×30 attaches, and the session holds one shown window in a column of width 1/2
- **THEN** that window's PTY becomes 48 columns by 28 rows

#### Scenario: Resize from either client
- **WHEN** two clients are attached and the earlier one reports a resize to 90×25, and the session holds one shown window in a column of width 1/2
- **THEN** that window's PTY becomes 43 columns by 23 rows

#### Scenario: Sidebar narrows the area
- **WHEN** a client with an 80×24 terminal and the default 1-column sidebar is the only client attached, and the session holds one shown window in a column of width 1/2
- **THEN** the session's screen area is 79×24 and that window's PTY becomes 37 columns by 22 rows

#### Scenario: Other session keeps its area
- **WHEN** a client reporting the size 120×40 is attached to `default` and a client reporting the size 100×30 attaches to `work`
- **THEN** the screen area of `default` stays 120×40

#### Scenario: Opening a window keeps other sizes
- **WHEN** the screen area is 80×24, one window runs in a column of width 1/2, and a client opens a new window
- **THEN** the first window's PTY stays 38 columns by 22 rows
- **AND** its program receives no SIGWINCH

#### Scenario: Width change resizes the window
- **WHEN** the screen area is 90×30 and a client cycles the width of a shown window's column from 1/2 to 2/3
- **THEN** that window's PTY becomes 58 columns by 28 rows

### Requirement: Attach creates a missing session
When a client asks to attach to a session the server does not host, the server SHALL create that session and attach the client to it. The new session's working directory SHALL be the working directory the client sent with its request, and its screen area SHALL start as the size the client's hello carries, which is its reported size. A client attaching to a session that exists SHALL NOT create one.

#### Scenario: Second session
- **WHEN** a server hosts `default` and the user runs `gband attach -s work` in `/tmp`
- **THEN** the server hosts `default` and `work`
- **AND** `pwd` in the window of `work` prints `/tmp`
- **AND** the windows of `default` keep running

#### Scenario: Reattach by name
- **WHEN** the user runs `sleep 100` in session `work`, detaches, and runs `gband attach -s work` again
- **THEN** the client shows `sleep 100` still running

