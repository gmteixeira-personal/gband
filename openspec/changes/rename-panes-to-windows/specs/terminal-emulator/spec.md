## MODIFIED Requirements

### Requirement: One emulator behind every grid
Every terminal grid in gband SHALL be an instance of a single emulator interface. This covers the server's authoritative grid for each window, the client's grid for each window, and the grids tests use to read a window or a client's terminal. Only the emulator's implementation SHALL depend on the emulation library. Replacing the library SHALL change only that implementation.

#### Scenario: Library confined to the emulator
- **WHEN** the workspace's manifests are searched for the emulation library
- **THEN** only the emulator's own manifest names it

### Requirement: Output for the program
An emulator SHALL collect the bytes that processing a program's output requires it to write back to that program, such as replies to terminal queries, in the order the queries arrived. The server SHALL write a window emulator's collected bytes to that window's PTY, in order with the keys and pastes that clients send. A client's emulator and a test's emulator SHALL discard them. An emulator SHALL NOT produce write-back bytes for output that asks no question.

#### Scenario: Plain output asks nothing
- **WHEN** an emulator processes `printf 'hello\n'`
- **THEN** it has no bytes to write back
