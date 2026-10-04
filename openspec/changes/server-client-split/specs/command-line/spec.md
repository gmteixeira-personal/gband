## ADDED Requirements

### Requirement: Server and attach run the session
The `gband` binary SHALL accept the subcommands `server` and `attach`. Each subcommand SHALL start logging for its role and record one log event stating that it started, before doing anything else. `gband server` SHALL run the session server the session-server capability defines, in the foreground, until the session's program exits. `gband attach` SHALL run the client the client-attach capability defines. Neither subcommand SHALL write log events to standard output or standard error.

#### Scenario: Run the server
- **WHEN** the user runs `gband server` with `SHELL=/bin/true`
- **THEN** the process exits with status 0
- **AND** it prints nothing to standard output or standard error
- **AND** the server log file holds an event stating that the server started

#### Scenario: Server runs until its program exits
- **WHEN** the user runs `gband server` with the default shell
- **THEN** the process keeps running until that shell exits

#### Scenario: Run attach
- **WHEN** the user runs `gband attach` and then detaches
- **THEN** the client log file holds an event stating that the client started

## REMOVED Requirements

### Requirement: Server and attach subcommands
**Reason**: Both subcommands were stubs that logged and exited. They now run the session server and the client.
**Migration**: See "Server and attach run the session" in this capability, and the `session-server` and `client-attach` capabilities.
