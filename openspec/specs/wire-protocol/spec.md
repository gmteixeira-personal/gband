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
The first frame a client sends SHALL be a hello that carries the client's protocol version and its terminal size. The first frame the server sends SHALL answer it. When the versions are equal, the answer SHALL accept the client and carry the server's protocol version. Otherwise the answer SHALL reject the client, carry the server's protocol version, and the server SHALL then close the connection. The hello and its answer SHALL keep the same encoding in every later protocol version, so that two versions can always detect each other. This change defines protocol version 1.

#### Scenario: Matching versions
- **WHEN** a version 1 client sends its hello to a version 1 server
- **THEN** the server accepts it
- **AND** sends server info as its next frame and the pane snapshot after that

#### Scenario: Mismatched versions
- **WHEN** a version 2 client sends its hello to a version 1 server
- **THEN** the server rejects it with version 1 and closes the connection
- **AND** the server records the rejected version in its log and keeps running

#### Scenario: First frame is not a hello
- **WHEN** a client's first frame decodes as any message other than a hello
- **THEN** the server closes the connection

### Requirement: Client messages
After the handshake, a client SHALL send only these messages:

| message | content |
|---|---|
| key | a key code and its Shift, Alt and Ctrl modifiers, as the input-encoding capability defines them |
| paste | the pasted text |
| resize | the client terminal's columns and rows |
| detach | nothing |

A client that detaches SHALL send detach, then close the connection.

#### Scenario: Detach message
- **WHEN** a client sends detach
- **THEN** the server stops sending to it and closes the connection
- **AND** the pane's program keeps running

### Requirement: Server messages
After the handshake, a server SHALL send only these messages:

| message | content |
|---|---|
| info | the server's process id and the identity of the executable it runs from |
| snapshot | the pane's columns and rows, and terminal output that, fed into an empty terminal grid of that size, reproduces the server's screen |
| update | terminal output that, fed into the grid the client built from every earlier snapshot and update, reproduces the server's current screen |
| exited | nothing; the session's program has exited |

The server SHALL send info exactly once, as its first message after accepting the hello, before the first snapshot. An executable's identity SHALL be the device and inode of the file the process was started from, taken when the process starts, so that replacing the file on disk, as a rebuild does, gives a new identity.

A client SHALL build its grid by replacing it with a new empty grid of the snapshot's size on every snapshot, then feeding the snapshot's output into it, and by feeding each update's output into its current grid.

#### Scenario: Info before snapshot
- **WHEN** a client is accepted
- **THEN** the first message it receives is info, carrying the server's process id
- **AND** the second is a snapshot

#### Scenario: Rebuilt executable has a new identity
- **WHEN** a server is started from `target/debug/gband` and `cargo build` then replaces that file with a new build
- **THEN** the identity the server reports differs from the identity a client started from the new file computes for itself

#### Scenario: Snapshot then updates
- **WHEN** a client applies a snapshot and then every update it receives
- **THEN** its grid's visible cells, cursor and input modes equal the server's

#### Scenario: Session ends
- **WHEN** the session's program exits
- **THEN** each attached client receives exited as its last message
