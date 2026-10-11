## MODIFIED Requirements

### Requirement: Handshake
The first frame a client sends SHALL be a hello. The hello's payload SHALL begin with the client's protocol version, encoded the same way in every protocol version, followed by the client's terminal size. The first frame the server sends SHALL answer it. The server SHALL decode the leading version on its own, and compare it with its own version before it decodes anything that follows it. When the versions differ, the answer SHALL reject the client and carry the server's protocol version, whatever bytes follow the version, and the server SHALL then close the connection. When the versions are equal and the rest of the hello decodes, the answer SHALL accept the client and carry the server's protocol version. The leading version and the answer SHALL keep the same encoding in every later protocol version, so that two versions can always detect each other. The fields after the leading version MAY change in a later protocol version. The current protocol version SHALL be 13.

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
