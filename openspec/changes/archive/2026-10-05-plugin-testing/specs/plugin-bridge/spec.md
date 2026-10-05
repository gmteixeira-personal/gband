## MODIFIED Requirements

### Requirement: No code crosses the connection
No message between client and server SHALL carry Lua code, a function or bytecode. A client SHALL evaluate only Lua from its own machine: the files the plugins capability defines, and the chunks a test runner sends through a test channel that the client's own environment named, as the test-channel capability defines. A client SHALL treat every value it receives from a server as data. Attaching to a server SHALL NOT cause a client to load, evaluate or call anything the server sent. A server SHALL NOT forward a test channel chunk to a client.

#### Scenario: Code as data
- **WHEN** a server emits the string `os.exit(1)` and a client handler stores it
- **THEN** the client keeps running and holds the string unchanged

#### Scenario: Chunk from a local runner
- **WHEN** a runner starts a client with `GBAND_TEST_SOCKET` naming its socket and sends it `return gband.side`
- **THEN** the client evaluates the chunk and answers `"client"`

#### Scenario: Server sends no chunk
- **WHEN** a runner holds a test channel to the server only, and a client attaches without `GBAND_TEST_SOCKET`
- **THEN** the client evaluates no chunk the runner sends the server
