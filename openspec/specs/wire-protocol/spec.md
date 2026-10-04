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
The first frame a client sends SHALL be a hello that carries the client's protocol version and its terminal size. The first frame the server sends SHALL answer it. When the versions are equal, the answer SHALL accept the client and carry the server's protocol version. Otherwise the answer SHALL reject the client, carry the server's protocol version, and the server SHALL then close the connection. The hello and its answer SHALL keep the same encoding in every later protocol version, so that two versions can always detect each other. This change defines protocol version 2.

#### Scenario: Matching versions
- **WHEN** a version 2 client sends its hello to a version 2 server
- **THEN** the server accepts it
- **AND** sends server info as its next frame, then the layout, then a snapshot of each pane

#### Scenario: Mismatched versions
- **WHEN** a version 1 client sends its hello to a version 2 server
- **THEN** the server rejects it with version 2 and closes the connection
- **AND** the server records the rejected version in its log and keeps running

#### Scenario: First frame is not a hello
- **WHEN** a client's first frame decodes as any message other than a hello
- **THEN** the server closes the connection

### Requirement: Client messages
After the handshake, a client SHALL send only these messages:

| message | content |
|---|---|
| key | a pane identifier, and a key code with its Shift, Alt and Ctrl modifiers as the input-encoding capability defines them |
| paste | a pane identifier and the pasted text |
| resize | the client terminal's columns and rows |
| action | one session action, as the session-server capability defines it, with the pane or workspace it names |
| detach | nothing |

The open pane action SHALL name a workspace and, optionally, the pane whose column the new column follows. Every other action SHALL name a pane, and consume or expel SHALL also name its direction. A client that detaches SHALL send detach, then close the connection.

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
| exited | nothing; the session has ended |

The server SHALL send info exactly once, as its first message after accepting the hello, before the first layout. An executable's identity SHALL be the device and inode of the file the process was started from, taken when the process starts, so that replacing the file on disk, as a rebuild does, gives a new identity.

A client SHALL keep one grid per pane. It SHALL build a pane's grid by replacing it with a new empty grid of the snapshot's size on every snapshot of that pane, then feeding the snapshot's output into it, and by feeding each update's output for that pane into its current grid. It SHALL discard the grid of a pane that the latest layout does not hold.

#### Scenario: Info before snapshot
- **WHEN** a client is accepted
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

#### Scenario: Session ends
- **WHEN** the session's last pane leaves the layout
- **THEN** each attached client receives exited as its last message
