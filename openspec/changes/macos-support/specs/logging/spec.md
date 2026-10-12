## MODIFIED Requirements

### Requirement: Log directory location
The log directory SHALL be `gband/log` under the XDG state directory, on Linux and on macOS: `$XDG_STATE_HOME/gband/log/` when `XDG_STATE_HOME` is set to an absolute path, and `~/.local/state/gband/log/` otherwise. `gband` SHALL create the directory and any missing parents when a subcommand that logs starts.

#### Scenario: XDG_STATE_HOME set
- **WHEN** the user runs `gband server` with `XDG_STATE_HOME=/tmp/s`
- **THEN** the server log file is written under `/tmp/s/gband/log/`

#### Scenario: XDG_STATE_HOME set on macOS
- **WHEN** gband runs on macOS and the user runs `gband server` with `XDG_STATE_HOME=/tmp/s`
- **THEN** the server log file is written under `/tmp/s/gband/log/`
- **AND** nothing is written under `~/Library/`

#### Scenario: XDG_STATE_HOME unset on macOS
- **WHEN** gband runs on macOS for the user whose home directory is `/Users/u`, and the user runs `gband server` with `XDG_STATE_HOME` unset
- **THEN** the server log file is written under `/Users/u/.local/state/gband/log/`

#### Scenario: Directory missing
- **WHEN** the log directory does not exist and the user runs `gband attach`
- **THEN** `gband` creates it and writes the client log there
