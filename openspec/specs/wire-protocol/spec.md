# wire-protocol Specification

## Purpose

Defines the byte format and message exchange between a gband client and a gband server, so that each side can tell when the other side is incompatible and a later protocol version can be told apart from this one.

## Requirements

### Requirement: Framing
Every message in either direction SHALL be sent as one frame. A frame is a 4-byte big-endian unsigned length followed by that many bytes of payload, and the payload is the message encoded with `postcard`. A side that receives a frame whose length exceeds 16 MiB, or whose payload does not decode, SHALL close the connection and record the reason in its log.

#### Scenario: Round trip
- **WHEN** any message is framed and the frame is decoded
- **THEN** the decoded message equals the original

#### Scenario: Frame split across reads
- **WHEN** a frame arrives in several pieces, split at arbitrary byte boundaries
- **THEN** the receiver decodes exactly one message once the last piece arrives

#### Scenario: Oversized frame
- **WHEN** a server receives a frame header announcing 17 MiB
- **THEN** it closes that connection without reading the payload
- **AND** it keeps serving its other clients

### Requirement: Handshake
The first frame a client sends SHALL be a hello. The hello's payload SHALL begin with the client's protocol version, encoded the same way in every protocol version, followed by the client's terminal size. The first frame the server sends SHALL answer it. The server SHALL decode the leading version on its own, and compare it with its own version before it decodes anything that follows it. When the versions differ, the answer SHALL reject the client and carry the server's protocol version, whatever bytes follow the version, and the server SHALL then close the connection. When the versions are equal and the rest of the hello decodes, the answer SHALL accept the client and carry the server's protocol version. The leading version and the answer SHALL keep the same encoding in every later protocol version, so that two versions can always detect each other. The fields after the leading version MAY change in a later protocol version. The current protocol version SHALL be 12.

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
| mouse | a window identifier, a mouse event kind, press, release, motion or wheel, its button or wheel direction, its content cell, and its Shift, Alt and Ctrl modifiers, as the input-encoding capability defines them |
| rename | a window identifier, and the window's new manual name, or nothing to clear it |
| detach | nothing |
| reload | nothing; it asks the server for a forced reload, as the configuration capability's "Forced reload" defines |
| control answer | a call number from a control message, and the answer to its operation, as "Control operations" in "Requests" defines |

Window and band identifiers SHALL name windows and bands of the client's session. The server SHALL ignore a window identifier in a shown message that names no window of the client's session. The open window action SHALL name a band, optionally the window whose column the new column follows, optionally the new column's width, whether the window floats, whether the client asks to focus the new window, and what the window holds: either a program, which is optionally named as a command line or as an argument list, or plugin content with a request number. Every other action SHALL name a window. Consume or expel and move column SHALL also name their direction, left or right. Move window SHALL also name its direction, down or up. Toggle floating SHALL also name optionally the tiled window to tile after, and SHALL name either no layer, the floating layer or the tiled layer, as the session-server capability's "Session actions" defines. Set position SHALL also name a column and a row. Set width SHALL also name a width. Set height SHALL also name either a number of rows or a weight. Grow and shrink of a width and of a height SHALL also name a step. Move to place SHALL also name a reference window and a place: a new column left of the reference window's column, a new column right of it, above the reference window, or below it. A rename SHALL name a window that runs a program. A width, a weight and a step SHALL each be a fraction in lowest terms. A client SHALL NOT reuse a call number while its call is unanswered. A client SHALL answer each control message with exactly one control answer that carries its call number. A client that detaches SHALL send detach, then close the connection.

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
- **WHEN** a client sends grow height naming window 2 and the step 1/10
- **THEN** the server decodes the same action, window and step

#### Scenario: Open window with a program round trip
- **WHEN** a client sends open window naming band 1, window 2 and the argument list `htop`, `-d`, `10`
- **THEN** the server decodes the same band, window and argument list

#### Scenario: Open window without a program round trip
- **WHEN** a client sends open window naming band 1 and no window or program
- **THEN** the server decodes open window naming band 1 with no window and no program

#### Scenario: Open drawn window round trip
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

#### Scenario: Toggle floating round trip
- **WHEN** a client sends toggle floating naming window 3 and window 1 as the window to tile after
- **THEN** the server decodes toggle floating naming window 3 and window 1

#### Scenario: Toggle floating with a layer round trip
- **WHEN** a client sends toggle floating naming window 3, no window to tile after and the floating layer, and toggle floating naming window 4, window 1 as the window to tile after and the tiled layer
- **THEN** the server decodes the same windows and layers
- **AND** toggle floating naming window 3 and no layer decodes with no layer

#### Scenario: Move round trip
- **WHEN** a client sends move column naming window 2 and the direction right, and move window naming window 2 and the direction up
- **THEN** the server decodes the same actions, window and directions

#### Scenario: Set position round trip
- **WHEN** a client sends set position naming window 4, column 12 and row 3
- **THEN** the server decodes set position naming window 4, column 12 and row 3

#### Scenario: Open floating window round trip
- **WHEN** a client sends open window naming band 1, no window, the width 1/3, floating, and no program
- **THEN** the server decodes the same band, width and floating flag

#### Scenario: Mouse round trip
- **WHEN** a client sends mouse naming window 2, a press of the left button at content column 4 and row 2, with Ctrl
- **THEN** the server decodes the same window, kind, button, cell and modifiers

#### Scenario: Rename round trip
- **WHEN** a client sends rename naming window 2 and `logs`, and rename naming window 3 and nothing
- **THEN** the server decodes rename naming window 2 and `logs`, and rename naming window 3 and nothing

#### Scenario: Move to place round trip
- **WHEN** a client sends move to place naming window 3, the reference window 5 and the place right of its column
- **THEN** the server decodes the same windows and place

#### Scenario: Reload message answered
- **WHEN** an attached client sends a reload message
- **THEN** the server loads its configuration again and then sends that client a reloaded message

#### Scenario: Control message answered
- **WHEN** the server sends an attached client a control message with call number 4 and the operation errors
- **THEN** the client sends one control answer with call number 4

### Requirement: Server messages
After the handshake, a server SHALL send only these messages:

| message | content |
|---|---|
| info | the server's process id and the identity of the executable it runs from |
| layout | the screen area's columns and rows, and the layout: its bands in order with their identifiers, each band's columns in order with their widths and full-width flags, and each column's windows in order with their identifiers and heights, each either automatic with its weight or fixed with its rows, and each band's floating windows in order with their identifiers and box records |
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
| result | a call number from a command message, and either the command's result as a plain data value or an error message |
| requirements | for each plugin the server requires in the client, its name and its requirement |
| server error | the text of a configuration or plugin error of the server's Lua |
| reloaded | nothing; the server's load that a reload message asked for has ended, and any error of it has been sent as a server error |
| control | a call number chosen by the server, and one control operation, as "Control operations" in "Requests" defines |
| control results | the answers to a control request, as "Requests" defines |

The server SHALL send info exactly once, as its first message after accepting the hello, before any answer to a request. An executable's identity SHALL be the device and inode of the file the process was started from, taken when the process starts, so that replacing the file on disk, as a rebuild does, gives a new identity. The layout, snapshot, update, focus, opened, exited, window state and window name messages SHALL concern only the session the client attached to, and the result messages only the client's own calls. After the layout and the snapshots of an attach, the server SHALL send, in this order, a window name message for each window that runs a program, a window state message for each key of each window state of the session, one requirements message, the latest server error if any, and the queued events for the client; and only then any later message.

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

### Requirement: Requests
After the server's info, the first message a client sends SHALL be one request:

| request | content | server's answer |
|---|---|---|
| attach | a session name, and the client's working directory | the layout, then a snapshot of each window, of the session it attaches the client to |
| list sessions | nothing | one sessions message, then it closes the connection |
| kill session | a session name | killed once the session has ended, or no such session, then it closes the connection |
| control | a session name, a target, and one control operation | control results once every addressed process has answered or timed out, or no such session, then it closes the connection |

A sessions message SHALL hold, for each session, its name, the number of windows in its layout and the number of clients attached to it. The server SHALL close the connection, and record the reason in its log, when the first message after info is not a request, or when a request names an invalid session name. A control request's target SHALL be one of: the server; every client attached to the session; the server and every client attached to the session; one client attached to the session, named by its client number; or the client of the session that the remote-control capability's "Chosen client" defines. The server SHALL give each client that attaches a client number: a positive integer that no other client of that server has had while it runs. The control operations SHALL be:

| operation | content | answer |
|---|---|---|
| reload | nothing | the process's load number after the load, and the load's error, or nothing when it succeeded |
| errors | nothing | the process's load number, and its error list, as the remote-control capability's "Errors subcommand" defines |
| eval | a chunk of Lua source, and a list of plain data arguments | the chunk's return values as plain data values, or an error message |
| command | a command's full name, and its arguments as a plain data value | the command's result as a plain data value, or an error message |

The server SHALL answer a control request with one control results message that holds one entry for each addressed process: the server first when the target includes the server, then the clients in ascending client number. Each entry SHALL name the process, the server or a client by its client number, and hold its answer, or no answer when the process did not answer in time. A target with no client, the chosen client included, SHALL give an empty list of entries. The server SHALL forward an operation for a client to that client as a control message, and SHALL NOT forward it to a client of another session. After a list, kill or control request, the client SHALL send nothing more. After an attach request, the client SHALL send only the messages "Client messages" defines.

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

#### Scenario: Control request answered by every client
- **WHEN** two clients are attached to `work` and a third connection sends a control request for `work` with the target every client and the operation errors
- **THEN** the server sends each attached client a control message, and then the third connection one control results message with two entries in ascending client number, and closes the connection

#### Scenario: Control request for an absent session
- **WHEN** a connection sends a control request naming `absent` to a server that hosts only `default`
- **THEN** the server sends no such session and closes the connection

#### Scenario: Control round trip
- **WHEN** a control request naming `work`, the target client 2 and the operation eval with the source `return 1` and no argument is framed and decoded
- **THEN** the decoded request equals the original

### Requirement: Plain data values
Every plain data value, as the plugin-bridge capability defines it, SHALL be carried in a message as one encoding of its own: nil, a boolean, a signed 64-bit integer, a 64-bit float, a byte string, or a table as a list of key and value pairs in which each key is an integer or a byte string. The encoding SHALL be the same over every transport. A side that receives a value nested more than 32 tables deep SHALL close the connection and record the reason in its log.

#### Scenario: Value round trip
- **WHEN** a value holding the integer 3, the float 0.5, the bytes 0 and 255, and a table nesting a list of two strings is framed and decoded
- **THEN** the decoded value equals the original, with 3 still an integer and 0.5 still a float
