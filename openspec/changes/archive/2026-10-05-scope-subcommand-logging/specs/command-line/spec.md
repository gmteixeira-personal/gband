## MODIFIED Requirements

### Requirement: Subcommands run the session
The `gband` binary SHALL accept the subcommands `server`, `attach`, `list-sessions`, `kill-session` and `kill-server`. When no subcommand is given, `gband` SHALL run `attach`, with every option given applied to it as if `attach` had been named. Each of these five subcommands SHALL start logging for its role and record one log event stating that it started, before doing anything else. Subcommands that other capabilities define, such as shell-completions' `completions` and `install-completions`, SHALL log only as those capabilities state. `gband server` SHALL run the session server the session-server capability defines, in the foreground, until its last session ends. `gband attach` SHALL run the client the client-attach capability defines. `gband list-sessions`, `gband kill-session` and `gband kill-server` SHALL list the sessions, end a session and stop the running server, as the session-server capability defines. No subcommand SHALL write log events to standard output or standard error.

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

#### Scenario: Run kill-session
- **WHEN** the user runs `gband kill-session` with no server running
- **THEN** the client log file holds an event stating that the client started
- **AND** the process exits with status 1
