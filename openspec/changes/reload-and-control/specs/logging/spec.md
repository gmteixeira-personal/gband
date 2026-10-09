## MODIFIED Requirements

### Requirement: Log to a file per role
The `server`, `attach`, `list-sessions`, `kill-session`, `kill-server`, `reload`, `errors`, `eval` and `cmd` subcommands SHALL write their logs to files in the log directory. No `gband` subcommand SHALL write log events to standard output or standard error. The `server` subcommand SHALL log to the `server` file series, and the `attach`, `list-sessions`, `kill-session`, `kill-server`, `reload`, `errors`, `eval` and `cmd` subcommands to the `client` file series. Subcommands that other capabilities define, such as shell-completions' `completions` and `install-completions`, SHALL log only as those capabilities state. Each line SHALL carry a timestamp, the level, the event's source module and its message, with no terminal colour codes.

#### Scenario: Server logs to its own series
- **WHEN** the user runs `gband server`
- **THEN** a file whose name starts with `server.` and ends with `.log` exists in the log directory
- **AND** no file of the `client` series was written by that run

#### Scenario: Kill-server logs to the client series
- **WHEN** the user runs `gband kill-server`
- **THEN** a file whose name starts with `client.` and ends with `.log` exists in the log directory
- **AND** no file of the `server` series was written by that run

#### Scenario: List-sessions logs to the client series
- **WHEN** the user runs `gband list-sessions`
- **THEN** a file whose name starts with `client.` and ends with `.log` exists in the log directory
- **AND** no file of the `server` series was written by that run

#### Scenario: Kill-session logs to the client series
- **WHEN** the user runs `gband kill-session`
- **THEN** a file whose name starts with `client.` and ends with `.log` exists in the log directory
- **AND** no file of the `server` series was written by that run

#### Scenario: Reload logs to the client series
- **WHEN** the user runs `gband reload`
- **THEN** a file whose name starts with `client.` and ends with `.log` exists in the log directory
- **AND** no file of the `server` series was written by that run

#### Scenario: Completion subcommands write no log
- **WHEN** the user runs `gband completions bash` with `XDG_STATE_HOME` naming an empty directory
- **THEN** the process exits with status 0
- **AND** the directory stays empty

#### Scenario: No colour codes in the file
- **WHEN** any subcommand writes a log line
- **THEN** the line contains no ANSI escape sequence

### Requirement: Events name their server
Servers on different socket paths share one log directory and one file series per role. Every log event that the `server`, `attach`, `list-sessions`, `kill-session`, `kill-server`, `reload`, `errors`, `eval` and `cmd` subcommands write after resolving their socket path SHALL carry that socket path, so the lines of one server can be told from another's.

#### Scenario: Two servers in one file
- **WHEN** servers named `a` and `b` run on the same day
- **THEN** every event in that day's `server` file that either wrote after start names `a.sock` or `b.sock`

#### Scenario: Client events
- **WHEN** the user runs `gband -S a kill-server`
- **THEN** the `client` file's events from that run name `a.sock`

#### Scenario: List-sessions events
- **WHEN** the user runs `gband -S a list-sessions` with no server running
- **THEN** the `client` file holds at least one event from that run after start
- **AND** every such event names `a.sock`

#### Scenario: Kill-session events
- **WHEN** the user runs `gband -S a kill-session` with no server running
- **THEN** the `client` file holds at least one event from that run after start
- **AND** every such event names `a.sock`
