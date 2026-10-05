## MODIFIED Requirements

### Requirement: Events name their server
Servers on different socket paths share one log directory and one file series per role. Every log event that the `server`, `attach`, `list-sessions`, `kill-session` and `kill-server` subcommands write after resolving their socket path SHALL carry that socket path, so the lines of one server can be told from another's.

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
