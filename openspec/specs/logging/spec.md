# logging Specification

## Purpose

Defines how `gband` processes record diagnostics. The server runs without a terminal and the client owns the terminal in raw mode, so a log file is the only place either can report what it did.

## Requirements

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

### Requirement: Log directory location
The log directory SHALL be `gband/log` under the XDG state directory: `$XDG_STATE_HOME/gband/log/` when `XDG_STATE_HOME` is set to an absolute path, and `~/.local/state/gband/log/` otherwise. On a platform with no state directory, it SHALL be `log` under gband's local data directory. `gband` SHALL create the directory and any missing parents when a subcommand starts.

#### Scenario: XDG_STATE_HOME set
- **WHEN** the user runs `gband server` with `XDG_STATE_HOME=/tmp/s`
- **THEN** the server log file is written under `/tmp/s/gband/log/`

#### Scenario: Directory missing
- **WHEN** the log directory does not exist and the user runs `gband attach`
- **THEN** `gband` creates it and writes the client log there

### Requirement: Daily rotation with bounded retention
Each file series SHALL rotate daily, with the date in the file name, as `<role>.<YYYY-MM-DD>.log`. `gband` SHALL keep at most 7 files per series and delete the oldest of that series when a new one is created.

#### Scenario: File named by role and date
- **WHEN** the user runs `gband server` on 2026-10-04
- **THEN** the server log is written to `server.2026-10-04.log`

#### Scenario: Retention limit
- **WHEN** the log directory already holds 7 `server` files for 7 earlier days and `gband server` starts a new day's file
- **THEN** the directory holds at most 7 `server` files afterwards
- **AND** the `client` files are untouched

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

### Requirement: Panics are logged
A panic in any thread of a `gband` process SHALL be recorded in that process's log at `error` level with the panic message and its source location, and the record SHALL reach the file before the process exits.

#### Scenario: Panic reaches the log
- **WHEN** a thread of `gband server` panics with the message `boom`
- **THEN** the server log holds an `error` event containing `boom` and the file and line of the panic

### Requirement: Log setup failure
When the log directory cannot be created or its file cannot be opened, `gband` SHALL print one line to standard error naming the directory and the cause, and exit with status 1 without running the subcommand.

#### Scenario: Unwritable state directory
- **WHEN** the user runs `gband server` with `XDG_STATE_HOME` pointing at a read-only directory
- **THEN** standard error names the log directory and the permission error
- **AND** the process exits with status 1

### Requirement: Events name their server
Servers on different socket paths share one log directory and one file series per role. Every log event that the `server`, `attach` and `kill-server` subcommands write after resolving their socket path SHALL carry that socket path, so the lines of one server can be told from another's.

#### Scenario: Two servers in one file
- **WHEN** servers named `a` and `b` run on the same day
- **THEN** every event in that day's `server` file that either wrote after start names `a.sock` or `b.sock`

#### Scenario: Client events
- **WHEN** the user runs `gband -S a kill-server`
- **THEN** the `client` file's events from that run name `a.sock`
