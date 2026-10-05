## MODIFIED Requirements

### Requirement: Subcommands run the session
The `gband` binary SHALL accept the subcommands `server`, `attach`, `list-sessions`, `kill-session` and `kill-server`. When no subcommand is given, `gband` SHALL run `attach`, with every option given applied to it as if `attach` had been named. Each subcommand SHALL start logging for its role and record one log event stating that it started, before doing anything else. `gband server` SHALL run the session server the session-server capability defines, in the foreground, until its last session ends. `gband attach` SHALL run the client the client-attach capability defines. `gband list-sessions`, `gband kill-session` and `gband kill-server` SHALL list the sessions, end a session and stop the running server, as the session-server capability defines. No subcommand SHALL write log events to standard output or standard error.

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

#### Scenario: No subcommand runs attach
- **WHEN** the user runs `gband` with no arguments and then detaches
- **THEN** the client attaches to the session named `default` on the default server
- **AND** the client log file holds an event stating that the client started

#### Scenario: No subcommand with options
- **WHEN** the user runs `gband -S feature -s work`
- **THEN** the client attaches to the session named `work` on the server named `feature`

#### Scenario: No subcommand without a terminal
- **WHEN** the user runs `gband` with no arguments and standard input is not a terminal
- **THEN** standard error holds the one line `gband attach` prints when it has no terminal
- **AND** the process exits with status 1
- **AND** the client log file holds an event stating that the client started

#### Scenario: Run kill-server
- **WHEN** the user runs `gband kill-server` with no server running
- **THEN** the client log file holds an event stating that the client started
- **AND** the process exits with status 1

#### Scenario: Run list-sessions
- **WHEN** the user runs `gband list-sessions` with no server running
- **THEN** the client log file holds an event stating that the client started
- **AND** the process exits with status 1

### Requirement: Help and version
`gband -h` and `gband --help` SHALL each print usage to standard output that lists the `server`, `attach`, `list-sessions`, `kill-session` and `kill-server` subcommands and the `-s` option, marks `attach` as the subcommand run when none is given, and exit with status 0 without creating or writing a log file. `gband --version` SHALL print the binary's version and exit with status 0.

#### Scenario: Print help
- **WHEN** the user runs `gband --help`
- **THEN** standard output lists the `server`, `attach`, `list-sessions`, `kill-session` and `kill-server` subcommands and the `-s` option
- **AND** the `attach` entry states that it is the default subcommand
- **AND** the process exits with status 0
- **AND** no log file is created

#### Scenario: Print help with the short flag
- **WHEN** the user runs `gband -h`
- **THEN** standard output lists the `server`, `attach`, `list-sessions`, `kill-session` and `kill-server` subcommands and the `-s` option
- **AND** the process exits with status 0
- **AND** no log file is created

#### Scenario: Print version
- **WHEN** the user runs `gband --version`
- **THEN** standard output holds `gband` followed by the package version
- **AND** the process exits with status 0

### Requirement: Invalid invocations
`gband` run with a subcommand or option it does not recognise SHALL print usage to standard error and exit with status 2. It SHALL NOT create or write a log file. A missing subcommand is not an invalid invocation: it runs `attach`.

#### Scenario: No subcommand
- **WHEN** the user runs `gband` with no arguments
- **THEN** it runs `attach` rather than printing usage
- **AND** the process does not exit with status 2

#### Scenario: Unknown subcommand
- **WHEN** the user runs `gband frobnicate`
- **THEN** standard error names `frobnicate` as unrecognised
- **AND** the process exits with status 2
- **AND** no log file is created

#### Scenario: Unknown option without a subcommand
- **WHEN** the user runs `gband --frobnicate`
- **THEN** standard error names `--frobnicate` as unrecognised
- **AND** the process exits with status 2
- **AND** no log file is created
