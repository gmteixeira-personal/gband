## ADDED Requirements

### Requirement: Session option
The `gband` binary SHALL accept the option `-s <name>`, long form `--session <name>`, before or after the subcommand. It names a session as the session-server capability defines session names. When it is absent, the session name SHALL be `default`. `gband server`, `gband attach` and `gband kill-session` SHALL use it. When the name is not a valid session name, `gband` SHALL print one line to standard error stating the rule for session names and exit with status 2, without creating or writing a log file. When `-s` is given to `kill-server` or `list-sessions`, which do not select a session, `gband` SHALL print one line to standard error saying that the option does not apply to that subcommand and exit with status 2, without creating or writing a log file.

#### Scenario: Option after the subcommand
- **WHEN** the user runs `gband attach -s work`
- **THEN** the client attaches to the session named `work`

#### Scenario: Option before the subcommand
- **WHEN** the user runs `gband -s work attach`
- **THEN** the client attaches to the session named `work`

#### Scenario: Invalid name
- **WHEN** the user runs `gband attach -s 'a/b'`
- **THEN** standard error states which characters a session name may hold
- **AND** the process exits with status 2
- **AND** no log file is created

#### Scenario: Option that does not apply
- **WHEN** the user runs `gband kill-server -s work`
- **THEN** standard error says that `-s` does not apply to `kill-server`
- **AND** the process exits with status 2
- **AND** no server is stopped

## MODIFIED Requirements

### Requirement: Subcommands run the session
The `gband` binary SHALL accept the subcommands `server`, `attach`, `list-sessions`, `kill-session` and `kill-server`. Each subcommand SHALL start logging for its role and record one log event stating that it started, before doing anything else. `gband server` SHALL run the session server the session-server capability defines, in the foreground, until its last session ends. `gband attach` SHALL run the client the client-attach capability defines. `gband list-sessions`, `gband kill-session` and `gband kill-server` SHALL list the sessions, end a session and stop the running server, as the session-server capability defines. No subcommand SHALL write log events to standard output or standard error.

#### Scenario: Run the server
- **WHEN** the user runs `gband server` with `SHELL=/bin/true`
- **THEN** the process exits with status 0
- **AND** it prints nothing to standard output or standard error
- **AND** the server log file holds an event stating that the server started

#### Scenario: Server runs until its program exits
- **WHEN** the user runs `gband server` with the default shell
- **THEN** the process keeps running until every pane's shell in every session has exited

#### Scenario: Run attach
- **WHEN** the user runs `gband attach` and then detaches
- **THEN** the client log file holds an event stating that the client started

#### Scenario: Run kill-server
- **WHEN** the user runs `gband kill-server` with no server running
- **THEN** the client log file holds an event stating that the client started
- **AND** the process exits with status 1

#### Scenario: Run list-sessions
- **WHEN** the user runs `gband list-sessions` with no server running
- **THEN** the client log file holds an event stating that the client started
- **AND** the process exits with status 1

### Requirement: Help and version
`gband --help` SHALL print usage that lists the `server`, `attach`, `list-sessions`, `kill-session` and `kill-server` subcommands and the `-s` option, and exit with status 0. `gband --version` SHALL print the binary's version and exit with status 0.

#### Scenario: Print help
- **WHEN** the user runs `gband --help`
- **THEN** standard output lists the `server`, `attach`, `list-sessions`, `kill-session` and `kill-server` subcommands and the `-s` option
- **AND** the process exits with status 0

#### Scenario: Print version
- **WHEN** the user runs `gband --version`
- **THEN** standard output holds `gband` followed by the package version
- **AND** the process exits with status 0
