## MODIFIED Requirements

### Requirement: Handshake
The first frame a client sends SHALL be a hello. The hello's payload SHALL begin with the client's protocol version, encoded the same way in every protocol version, followed by the client's terminal size. The first frame the server sends SHALL answer it. The server SHALL decode the leading version on its own, and compare it with its own version before it decodes anything that follows it. When the versions differ, the answer SHALL reject the client and carry the server's protocol version, whatever bytes follow the version, and the server SHALL then close the connection. When the versions are equal and the rest of the hello decodes, the answer SHALL accept the client and carry the server's protocol version. The leading version and the answer SHALL keep the same encoding in every later protocol version, so that two versions can always detect each other. The fields after the leading version MAY change in a later protocol version. The current protocol version SHALL be 6.

#### Scenario: Matching versions
- **WHEN** a version 2 client sends its hello to a version 2 server
- **THEN** the server accepts it
- **AND** sends server info as its next frame, then the layout, then a snapshot of each window

#### Scenario: Mismatched versions
- **WHEN** a version 1 client sends its hello to a version 2 server
- **THEN** the server rejects it with version 2 and closes the connection
- **AND** the server records the rejected version in its log and keeps running

#### Scenario: Later hello with other fields
- **WHEN** a client sends a hello frame holding version 3 followed by bytes that do not decode as a version 2 terminal size
- **THEN** the server rejects it with version 2 and closes the connection

#### Scenario: First frame is not a hello
- **WHEN** a client's first frame begins with version 2 but the rest does not decode as a hello
- **THEN** the server closes the connection without accepting it

#### Scenario: Empty first frame
- **WHEN** a client's first frame has an empty payload
- **THEN** the server closes the connection without answering

#### Scenario: Version 3 client meets a version 4 server
- **WHEN** a client speaking protocol version 3 sends its hello to a server speaking version 4
- **THEN** the server rejects it with version 4 and closes the connection

#### Scenario: Version 4 client meets a version 5 server
- **WHEN** a client speaking protocol version 4 sends its hello to a server speaking version 5
- **THEN** the server rejects it with version 5 and closes the connection

#### Scenario: Version 5 client meets a version 6 server
- **WHEN** a client speaking protocol version 5 sends its hello to a server speaking version 6
- **THEN** the server rejects it with version 6 and closes the connection

### Requirement: Client messages
After an attach request, a client SHALL send only these messages:

| message | content |
|---|---|
| key | a window identifier, and a key code with its Shift, Alt and Ctrl modifiers as the input-encoding capability defines them |
| paste | a window identifier and the pasted text |
| resize | the client terminal's columns and rows |
| shown | the identifiers of every window the client shows, as the layout-view capability defines; it replaces the set the client reported before |
| action | one session action, as the session-server capability defines it, with the window or band it names |
| content | a window identifier, and terminal output for the drawn window's screen, as the session-server capability defines |
| command | a call number chosen by the client, a server command's full name, and its arguments as a plain data value |
| detach | nothing |

Window and band identifiers SHALL name windows and bands of the client's session. The server SHALL ignore a window identifier in a shown message that names no window of the client's session. The open window action SHALL name a band, optionally the window whose column the new column follows, optionally the new column's width, whether the client asks to focus the new window, and what the window holds: either a program, which is optionally named as a command line or as an argument list, or plugin content with a request number. Every other action SHALL name a window. Consume or expel SHALL also name its direction. Set width SHALL also name a width. Set height SHALL also name either a number of rows or a weight. A width and a weight SHALL each be a fraction in lowest terms. A client SHALL NOT reuse a call number while its call is unanswered. A client that detaches SHALL send detach, then close the connection.

#### Scenario: Detach message
- **WHEN** a client sends detach
- **THEN** the server stops sending to it and closes the connection
- **AND** every window's program keeps running

#### Scenario: Action round trip
- **WHEN** a client sends consume or expel naming window 3 and the direction left
- **THEN** the server decodes the same action, window and direction

#### Scenario: Shown round trip
- **WHEN** a client sends shown naming windows 1 and 4
- **THEN** the server decodes a shown message naming windows 1 and 4

#### Scenario: Grow height round trip
- **WHEN** a client sends grow height naming window 2
- **THEN** the server decodes the same action and window

#### Scenario: Open pane with a program round trip
- **WHEN** a client sends open window naming band 1, window 2 and the argument list `htop`, `-d`, `10`
- **THEN** the server decodes the same band, window and argument list

#### Scenario: Open pane without a program round trip
- **WHEN** a client sends open window naming band 1 and no window or program
- **THEN** the server decodes open window naming band 1 with no window and no program

#### Scenario: Open plugin pane round trip
- **WHEN** a client sends open window naming band 1, window 2, the width 1/4, no focus, and plugin content with request number 7
- **THEN** the server decodes the same band, window, width, focus flag, content kind and request number

#### Scenario: Set height round trip
- **WHEN** a client sends set height naming window 2 and the weight 3/2
- **THEN** the server decodes set height naming window 2 and the weight 3/2

#### Scenario: Content round trip
- **WHEN** a client sends content naming window 4 and the output `\x1b[1;1Hhello`
- **THEN** the server decodes content naming window 4 and the same bytes

#### Scenario: Command round trip
- **WHEN** a client sends command number 7 naming `agents.next_waiting` with the arguments `{ state = "agent" }`
- **THEN** the server decodes the same number, name and arguments

### Requirement: Server messages
After the handshake, a server SHALL send only these messages:

| message | content |
|---|---|
| info | the server's process id and the identity of the executable it runs from |
| layout | the screen area's columns and rows, and the layout: its bands in order with their identifiers, each band's columns in order with their widths and full-width flags, and each column's windows in order with their identifiers and heights, each either automatic with its weight or fixed with its rows |
| snapshot | a window identifier, the window's columns and rows, and terminal output that, fed into an empty terminal grid of that size, reproduces that window's screen on the server |
| update | a window identifier, and terminal output that, fed into the grid the client built for that window from every earlier snapshot and update of it, reproduces that window's current screen on the server |
| focus | a window identifier: the window the client asked to open, or the window a server command focused for it, which the client focuses |
| opened | a request number, and the identifier of the drawn window opened for it, or none |
| exited | nothing; the client's session has ended |
| sessions | for each session, its name, window count and attached-client count |
| killed | nothing; the session a kill request named has ended |
| no such session | nothing; the server hosts no session of the name a kill request named |
| event | an event's name, its data as a plain data value, whether it was queued, and its emission time in milliseconds since the Unix epoch |
| window state | a window identifier, a key, and the key's new value as a plain data value, or nothing when the key was removed |
| result | a call number from a command message, and either the command's result as a plain data value or an error message |
| requirements | for each plugin the server requires in the client, its name and its requirement |
| server error | the text of a configuration or plugin error of the server's Lua |

The server SHALL send info exactly once, as its first message after accepting the hello, before any answer to a request. An executable's identity SHALL be the device and inode of the file the process was started from, taken when the process starts, so that replacing the file on disk, as a rebuild does, gives a new identity. The layout, snapshot, update, focus, opened, exited and window state messages SHALL concern only the session the client attached to, and the result messages only the client's own calls. After the layout and the snapshots of an attach, the server SHALL send, in this order, a window state message for each key of each window state of the session, one requirements message, the latest server error if any, and the queued events for the client; and only then any later message.

A client SHALL keep one grid per window. It SHALL build a window's grid by replacing it with a new empty grid of the snapshot's size on every snapshot of that window, then feeding the snapshot's output into it, and by feeding each update's output for that window into its current grid. It SHALL discard the grid of a window that the latest layout does not hold.

#### Scenario: Info before snapshot
- **WHEN** a client is accepted and sends an attach request
- **THEN** the first message it receives is info, carrying the server's process id
- **AND** the second is the layout, followed by a snapshot of each window it holds

#### Scenario: Rebuilt executable has a new identity
- **WHEN** a server is started from `target/debug/gband` and `cargo build` then replaces that file with a new build
- **THEN** the identity the server reports differs from the identity a client started from the new file computes for itself

#### Scenario: Snapshot then updates
- **WHEN** a client applies every snapshot and every update it receives for a window
- **THEN** its grid of that window has the same visible cells, cursor and input modes as the server's

#### Scenario: Layout round trip
- **WHEN** a layout of two bands, one holding a full-width column of width 1/3 with two windows, is framed and decoded
- **THEN** the decoded layout equals the original

#### Scenario: Sessions round trip
- **WHEN** a sessions message holding `default` with 1 window and 0 clients and `work` with 2 windows and 1 client is framed and decoded
- **THEN** the decoded message equals the original

#### Scenario: Session ends
- **WHEN** the last window of a client's session leaves the layout
- **THEN** that client receives exited as its last message

#### Scenario: Heights in the layout
- **WHEN** a column holds window 1 with a fixed height of 14 rows and window 2 with an automatic height of weight 10/7, and the server sends the layout
- **THEN** the client decodes that column with window 1 fixed at 14 rows and window 2 automatic with weight 10/7

#### Scenario: Opened round trip
- **WHEN** an opened message naming request 7 and window 9, and one naming request 8 and no window, are framed and decoded
- **THEN** each decoded message equals the original

#### Scenario: Attach order with plugin state
- **WHEN** window 1's state holds `agent`, the server loaded a plugin requiring `agent-status >= 0.1` in the client, one event is queued, and a client attaches
- **THEN** after the snapshots the client receives a window state message for window 1, then requirements, then the queued event

#### Scenario: Result round trip
- **WHEN** a result message for call 7 holding the error `unknown command absent` is framed and decoded
- **THEN** the decoded message equals the original

### Requirement: Requests
After the server's info, the first message a client sends SHALL be one request:

| request | content | server's answer |
|---|---|---|
| attach | a session name, and the client's working directory | the layout, then a snapshot of each window, of the session it attaches the client to |
| list sessions | nothing | one sessions message, then it closes the connection |
| kill session | a session name | killed once the session has ended, or no such session, then it closes the connection |

A sessions message SHALL hold, for each session, its name, the number of windows in its layout and the number of clients attached to it. The server SHALL close the connection, and record the reason in its log, when the first message after info is not a request, or when a request names an invalid session name. After a list or kill request, the client SHALL send nothing more. After an attach request, the client SHALL send only the messages "Client messages" defines.

#### Scenario: Attach request
- **WHEN** a client sends an attach request naming `work`
- **THEN** the server's next messages are the layout of `work` and a snapshot of each of its windows

#### Scenario: List request
- **WHEN** a client sends a list request to a server hosting `default` and `work`
- **THEN** the server sends one sessions message naming `default` and `work`, and closes the connection

#### Scenario: Key before a request
- **WHEN** a client's first message after info is a key
- **THEN** the server closes the connection

#### Scenario: Request round trip
- **WHEN** an attach request naming `work` with the working directory `/tmp` is framed and decoded
- **THEN** the decoded request equals the original
