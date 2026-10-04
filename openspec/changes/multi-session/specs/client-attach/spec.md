## MODIFIED Requirements

### Requirement: Connect to the session
`gband attach` SHALL connect to the socket path the session-server capability defines, resolved from its own environment. After the handshake, it SHALL ask to attach to the session `-s` names, sending its own working directory with the request. The server attaches it to that session, creating the session when it does not exist, as the session-server capability defines.

#### Scenario: Server already running
- **WHEN** a server is running and the user runs `gband attach` with the same environment
- **THEN** the client attaches to that server's session named `default`

#### Scenario: Attach to a named session
- **WHEN** a server hosts `default` and `work` and the user runs `gband attach -s work`
- **THEN** the client shows the panes of `work`

### Requirement: Start a server when none is running
When nothing accepts connections on the socket path, `gband attach` SHALL start `gband server` from its own executable, passing the session name `-s` names, detached from the client's terminal and session, with the client's working directory and environment, and with standard input, output and error on `/dev/null`. It SHALL then connect to it. If no connection succeeds within 5 seconds, the client SHALL print one line to standard error naming the socket path and the server log, and exit with status 1.

#### Scenario: First attach starts the session
- **WHEN** no server is running and the user runs `gband attach` in `~/repos/gband`
- **THEN** a shell prompt appears in the client
- **AND** `pwd` in the pane prints the path of `~/repos/gband`

#### Scenario: First attach names the session
- **WHEN** no server is running and the user runs `gband attach -s work`
- **THEN** the server it starts hosts exactly one session, named `work`
- **AND** the client is attached to it

#### Scenario: Started server survives its terminal
- **WHEN** `gband attach` started the server and the terminal emulator window running the client is closed
- **THEN** the server and its shell keep running

#### Scenario: Server cannot start
- **WHEN** the server that `gband attach` starts exits before accepting a connection
- **THEN** the client prints one line to standard error naming the socket path and the server log
- **AND** it exits with status 1 within 5 seconds
