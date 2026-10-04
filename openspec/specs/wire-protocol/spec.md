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
The first frame a client sends SHALL be a hello that carries the client's protocol version and its terminal size. The first frame the server sends SHALL answer it. When the versions are equal, the answer SHALL accept the client and carry the server's protocol version. Otherwise the answer SHALL reject the client, carry the server's protocol version, and the server SHALL then close the connection. The hello and its answer SHALL keep the same encoding in every later protocol version, so that two versions can always detect each other. The terminal size in the hello SHALL apply only when the client then asks to attach. This change defines protocol version 3.

#### Scenario: Matching versions
- **WHEN** a version 3 client sends its hello to a version 3 server
- **THEN** the server accepts it
- **AND** sends server info as its next frame, then waits for the client's request

#### Scenario: Mismatched versions
- **WHEN** a version 2 client sends its hello to a version 3 server
- **THEN** the server rejects it with version 3 and closes the connection
- **AND** the server records the rejected version in its log and keeps running

#### Scenario: First frame is not a hello
- **WHEN** a client's first frame decodes as any message other than a hello
- **THEN** the server closes the connection

### Requirement: Client messages
After an attach request, a client SHALL send only these messages:

| message | content |
|---|---|
| key | a pane identifier, and a key code with its Shift, Alt and Ctrl modifiers as the input-encoding capability defines them |
| paste | a pane identifier and the pasted text |
| resize | the client terminal's columns and rows |
| action | one session action, as the session-server capability defines it, with the pane or workspace it names |
| detach | nothing |

Pane and workspace identifiers SHALL name panes and workspaces of the client's session. The open pane action SHALL name a workspace and, optionally, the pane whose column the new column follows. Every other action SHALL name a pane, and consume or expel SHALL also name its direction. A client that detaches SHALL send detach, then close the connection.

#### Scenario: Detach message
- **WHEN** a client sends detach
- **THEN** the server stops sending to it and closes the connection
- **AND** every pane's program keeps running

#### Scenario: Action round trip
- **WHEN** a client sends consume or expel naming pane 3 and the direction left
- **THEN** the server decodes the same action, pane and direction

### Requirement: Server messages
After the handshake, a server SHALL send only these messages:

| message | content |
|---|---|
| info | the server's process id and the identity of the executable it runs from |
| layout | the screen area's columns and rows, and the layout: its workspaces in order with their identifiers, each workspace's columns in order with their widths and full-width flags, and each column's panes in order with their identifiers |
| snapshot | a pane identifier, the pane's columns and rows, and terminal output that, fed into an empty terminal grid of that size, reproduces that pane's screen on the server |
| update | a pane identifier, and terminal output that, fed into the grid the client built for that pane from every earlier snapshot and update of it, reproduces that pane's current screen on the server |
| focus | a pane identifier: the pane the client asked to open, which the client focuses |
| exited | nothing; the client's session has ended |
| sessions | for each session, its name, pane count and attached-client count |
| killed | nothing; the session a kill request named has ended |
| no such session | nothing; the server hosts no session of the name a kill request named |

The server SHALL send info exactly once, as its first message after accepting the hello, before any answer to a request. An executable's identity SHALL be the device and inode of the file the process was started from, taken when the process starts, so that replacing the file on disk, as a rebuild does, gives a new identity. The layout, snapshot, update, focus and exited messages SHALL concern only the session the client attached to.

A client SHALL keep one grid per pane. It SHALL build a pane's grid by replacing it with a new empty grid of the snapshot's size on every snapshot of that pane, then feeding the snapshot's output into it, and by feeding each update's output for that pane into its current grid. It SHALL discard the grid of a pane that the latest layout does not hold.

#### Scenario: Info before snapshot
- **WHEN** a client is accepted and sends an attach request
- **THEN** the first message it receives is info, carrying the server's process id
- **AND** the second is the layout, followed by a snapshot of each pane it holds

#### Scenario: Rebuilt executable has a new identity
- **WHEN** a server is started from `target/debug/gband` and `cargo build` then replaces that file with a new build
- **THEN** the identity the server reports differs from the identity a client started from the new file computes for itself

#### Scenario: Snapshot then updates
- **WHEN** a client applies every snapshot and every update it receives for a pane
- **THEN** its grid of that pane has the same visible cells, cursor and input modes as the server's

#### Scenario: Layout round trip
- **WHEN** a layout of two workspaces, one holding a full-width column of width 1/3 with two panes, is framed and decoded
- **THEN** the decoded layout equals the original

#### Scenario: Sessions round trip
- **WHEN** a sessions message holding `default` with 1 pane and 0 clients and `work` with 2 panes and 1 client is framed and decoded
- **THEN** the decoded message equals the original

#### Scenario: Session ends
- **WHEN** the last pane of a client's session leaves the layout
- **THEN** that client receives exited as its last message

### Requirement: Requests
After the server's info, the first message a client sends SHALL be one request:

| request | content | server's answer |
|---|---|---|
| attach | a session name, and the client's working directory | the layout, then a snapshot of each pane, of the session it attaches the client to |
| list sessions | nothing | one sessions message, then it closes the connection |
| kill session | a session name | killed once the session has ended, or no such session, then it closes the connection |

A sessions message SHALL hold, for each session, its name, the number of panes in its layout and the number of clients attached to it. The server SHALL close the connection, and record the reason in its log, when the first message after info is not a request, or when a request names an invalid session name. After a list or kill request, the client SHALL send nothing more. After an attach request, the client SHALL send only the messages "Client messages" defines.

#### Scenario: Attach request
- **WHEN** a client sends an attach request naming `work`
- **THEN** the server's next messages are the layout of `work` and a snapshot of each of its panes

#### Scenario: List request
- **WHEN** a client sends a list request to a server hosting `default` and `work`
- **THEN** the server sends one sessions message naming `default` and `work`, and closes the connection

#### Scenario: Key before a request
- **WHEN** a client's first message after info is a key
- **THEN** the server closes the connection

#### Scenario: Request round trip
- **WHEN** an attach request naming `work` with the working directory `/tmp` is framed and decoded
- **THEN** the decoded request equals the original
