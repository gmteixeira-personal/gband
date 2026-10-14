## MODIFIED Requirements

### Requirement: Handshake
The first frame a client sends SHALL be a hello. The hello's payload SHALL begin with the client's protocol version, encoded the same way in every protocol version, followed by the client's terminal size. The first frame the server sends SHALL answer it. The server SHALL decode the leading version on its own, and compare it with its own version before it decodes anything that follows it. When the versions differ, the answer SHALL reject the client and carry the server's protocol version, whatever bytes follow the version, and the server SHALL then close the connection. When the versions are equal and the rest of the hello decodes, the answer SHALL accept the client and carry the server's protocol version. The leading version and the answer SHALL keep the same encoding in every later protocol version, so that two versions can always detect each other. The fields after the leading version MAY change in a later protocol version. The current protocol version SHALL be 15.

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

#### Scenario: Version 6 client meets a version 7 server
- **WHEN** a client speaking protocol version 6 sends its hello to a server speaking version 7
- **THEN** the server rejects it with version 7 and closes the connection

#### Scenario: Version 7 client meets a version 8 server
- **WHEN** a client speaking protocol version 7 sends its hello to a server speaking version 8
- **THEN** the server rejects it with version 8 and closes the connection

#### Scenario: Version 8 client meets a version 9 server
- **WHEN** a client speaking protocol version 8 sends its hello to a server speaking version 9
- **THEN** the server rejects it with version 9 and closes the connection

#### Scenario: Version 9 client meets a version 10 server
- **WHEN** a client speaking protocol version 9 sends its hello to a server speaking version 10
- **THEN** the server rejects it with version 10 and closes the connection

#### Scenario: Version 10 client meets a version 11 server
- **WHEN** a client speaking protocol version 10 sends its hello to a server speaking version 11
- **THEN** the server rejects it with version 11 and closes the connection

#### Scenario: Version 11 client meets a version 12 server
- **WHEN** a client speaking protocol version 11 sends its hello to a server speaking version 12
- **THEN** the server rejects it with version 12 and closes the connection

#### Scenario: Version 12 client meets a version 13 server
- **WHEN** a client speaking protocol version 12 sends its hello to a server speaking version 13
- **THEN** the server rejects it with version 13 and closes the connection

#### Scenario: Version 13 client meets a version 14 server
- **WHEN** a client speaking protocol version 13 sends its hello to a server speaking version 14
- **THEN** the server rejects it with version 14 and closes the connection

#### Scenario: Version 14 client meets a version 15 server
- **WHEN** a client speaking protocol version 14 sends its hello to a server speaking version 15
- **THEN** the server rejects it with version 15 and closes the connection

### Requirement: Server messages
After the handshake, a server SHALL send only these messages:

| message | content |
|---|---|
| info | the server's process id and the identity of the executable it runs from |
| layout | the screen area's columns and rows, and the layout: its bands in order with their identifiers, each band's columns in order with their widths, full-width flags, tabbed flags and tab heights, as the tabbed-columns capability defines them, and each column's windows in order with their identifiers and heights, each either automatic with its weight or fixed with its rows, and each band's floating windows in order with their identifiers and box records |
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
| window name | a window identifier, the window's automatic name, and its manual name, or nothing when it has none |
| clipboard | a window identifier, and the text that window's program wrote to the clipboard, as the window-clipboard capability defines |
| result | a call number from a command message, and either the command's result as a plain data value or an error message |
| requirements | for each plugin the server requires in the client, its name and its requirement |
| server error | the text of a configuration or plugin error of the server's Lua |
| reloaded | nothing; the server's load that a reload message asked for has ended, and any error of it has been sent as a server error |
| control | a call number chosen by the server, and one control operation, as "Control operations" in "Requests" defines |
| control results | the answers to a control request, as "Requests" defines |

The server SHALL send info exactly once, as its first message after accepting the hello, before any answer to a request. An executable's identity SHALL be the device and inode of the file the process was started from, taken when the process starts, so that replacing the file on disk, as a rebuild does, gives a new identity. The layout, snapshot, update, focus, opened, exited, window state, window name and clipboard messages SHALL concern only the session the client attached to, and the result messages only the client's own calls. After the layout and the snapshots of an attach, the server SHALL send, in this order, a window name message for each window that runs a program, a window state message for each key of each window state of the session, one requirements message, the latest server error if any, and the queued events for the client; and only then any later message.

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

#### Scenario: Window name round trip
- **WHEN** a window name message naming window 4, the automatic name `vim` and the manual name `notes` is framed and decoded
- **THEN** the decoded message equals the original

#### Scenario: Attach order with window names
- **WHEN** windows 1 and 2 run programs, window 1's state holds `agent`, and a client attaches
- **THEN** after the snapshots the client receives a window name message for window 1 and one for window 2, then the window state message for window 1

#### Scenario: Result round trip
- **WHEN** a result message for call 7 holding the error `unknown command absent` is framed and decoded
- **THEN** the decoded message equals the original

#### Scenario: Floating windows in the layout
- **WHEN** band 1 holds a column with window 1 and floating windows 2 and 3, window 2 with column 5, row 3, width 1/3, full width off and 10 rows, and the server sends the layout
- **THEN** the client decodes band 1 with that column and floating windows 2 and 3 in that order, window 2 with the same box record

#### Scenario: Clipboard round trip
- **WHEN** a clipboard message naming window 2 and the text `hi` is framed and decoded
- **THEN** the decoded message equals the original
