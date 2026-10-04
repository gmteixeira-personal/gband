## ADDED Requirements

### Requirement: Subcommands run the session
The `gband` binary SHALL accept the subcommands `server`, `attach` and `kill-server`. Each subcommand SHALL start logging for its role and record one log event stating that it started, before doing anything else. `gband server` SHALL run the session server the session-server capability defines, in the foreground, until the session's program exits. `gband attach` SHALL run the client the client-attach capability defines. `gband kill-server` SHALL stop the running server as the session-server capability defines. No subcommand SHALL write log events to standard output or standard error.

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

#### Scenario: Run kill-server
- **WHEN** the user runs `gband kill-server` with no server running
- **THEN** the client log file holds an event stating that the client started
- **AND** the process exits with status 1

## MODIFIED Requirements

### Requirement: Help and version
`gband --help` SHALL print usage that lists the `server`, `attach` and `kill-server` subcommands and exit with status 0. `gband --version` SHALL print the binary's version and exit with status 0.

#### Scenario: Print help
- **WHEN** the user runs `gband --help`
- **THEN** standard output lists the `server`, `attach` and `kill-server` subcommands
- **AND** the process exits with status 0

#### Scenario: Print version
- **WHEN** the user runs `gband --version`
- **THEN** standard output holds `gband` followed by the package version
- **AND** the process exits with status 0

## REMOVED Requirements

### Requirement: Server and attach subcommands
**Reason**: Both subcommands were stubs that logged and exited. They now run the session server and the client.
**Migration**: See "Subcommands run the session" in this capability, and the `session-server` and `client-attach` capabilities.
