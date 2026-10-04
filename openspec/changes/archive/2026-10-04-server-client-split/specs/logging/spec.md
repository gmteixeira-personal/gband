## MODIFIED Requirements

### Requirement: Log to a file per role
Every `gband` subcommand SHALL write its log to files in the log directory and SHALL NOT write log events to standard output or standard error. The `server` subcommand SHALL log to the `server` file series, and the `attach` and `kill-server` subcommands to the `client` file series. Each line SHALL carry a timestamp, the level, the event's source module and its message, with no terminal colour codes.

#### Scenario: Server logs to its own series
- **WHEN** the user runs `gband server`
- **THEN** a file whose name starts with `server.` and ends with `.log` exists in the log directory
- **AND** no file of the `client` series was written by that run

#### Scenario: Kill-server logs to the client series
- **WHEN** the user runs `gband kill-server`
- **THEN** a file whose name starts with `client.` and ends with `.log` exists in the log directory
- **AND** no file of the `server` series was written by that run

#### Scenario: No colour codes in the file
- **WHEN** any subcommand writes a log line
- **THEN** the line contains no ANSI escape sequence

### Requirement: Level from the environment
The log level SHALL default to `info`. When the `GBAND_LOG` environment variable is set, `gband` SHALL use it as a filter directive in the `tracing` `EnvFilter` syntax, such as `debug` or `gband=trace,info`. When `GBAND_LOG` cannot be parsed, `gband` SHALL use the default level and record a warning in the log that quotes the rejected value.

#### Scenario: Default level
- **WHEN** `GBAND_LOG` is unset and a subcommand runs
- **THEN** the log holds `info` events and no `debug` or `trace` events

#### Scenario: Raised level
- **WHEN** the user runs `gband server` with `GBAND_LOG=debug`
- **THEN** `debug` events from `gband` appear in the server log

#### Scenario: Unparseable filter
- **WHEN** the user runs `gband server` with `GBAND_LOG=[[[` and `SHELL=/bin/true`
- **THEN** the process still exits with status 0
- **AND** the server log holds a warning quoting `[[[`
