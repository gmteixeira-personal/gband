## MODIFIED Requirements

### Requirement: Subcommands run the session
The `gband` binary SHALL accept the subcommands `server`, `attach` and `kill-server`. Each subcommand SHALL start logging for its role and record one log event stating that it started, before doing anything else. `gband server` SHALL run the session server the session-server capability defines, in the foreground, until the session ends when its last pane's program exits. `gband attach` SHALL run the client the client-attach capability defines. `gband kill-server` SHALL stop the running server as the session-server capability defines. No subcommand SHALL write log events to standard output or standard error.

#### Scenario: Run the server
- **WHEN** the user runs `gband server` with `SHELL=/bin/true`
- **THEN** the process exits with status 0
- **AND** it prints nothing to standard output or standard error
- **AND** the server log file holds an event stating that the server started

#### Scenario: Server runs until its program exits
- **WHEN** the user runs `gband server` with the default shell
- **THEN** the process keeps running until every pane's shell has exited

#### Scenario: Run attach
- **WHEN** the user runs `gband attach` and then detaches
- **THEN** the client log file holds an event stating that the client started

#### Scenario: Run kill-server
- **WHEN** the user runs `gband kill-server` with no server running
- **THEN** the client log file holds an event stating that the client started
- **AND** the process exits with status 1
