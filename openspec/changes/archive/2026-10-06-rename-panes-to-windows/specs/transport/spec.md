## MODIFIED Requirements

### Requirement: Byte-stream transport
A client SHALL reach a server through a transport. A transport opens a connection made of one byte stream from the client to the server and one byte stream from the server to the client. Each stream SHALL deliver bytes reliably and in order. The two streams MAY be separate channels. The wire protocol SHALL be the same over every transport, and the server SHALL serve a connection the same way whatever carries it.

#### Scenario: Separate reader and writer
- **WHEN** a test transport hands the client a reader and a writer that are two different in-memory pipes, each copied to and from the server's socket
- **THEN** the client completes the handshake and shows the window's snapshot

### Requirement: Relay transparency
A transport that only copies bytes between the client and the server's Unix socket SHALL NOT change the session's behaviour. This is the shape a later SSH proxy will have.

#### Scenario: Attach through a relay
- **WHEN** a client attaches through a relay that copies bytes in both directions, runs `printf 'relayed\n'` in the window and detaches
- **THEN** the client shows `relayed` in the window
- **AND** the session keeps running after the detach
