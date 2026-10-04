# command-line Specification

## Purpose

Defines the `gband` binary's command line: the subcommands it accepts, the help it prints and the exit status each invocation returns.

## Requirements

### Requirement: Subcommands run the session
The `gband` binary SHALL accept the subcommands `server`, `attach` and `kill-server`. Each subcommand SHALL start logging for its role and record one log event stating that it started, before doing anything else. `gband server` SHALL run the session server the session-server capability defines, in the foreground, until the session ends when its last pane's program exits. `gband attach` SHALL run the client the client-attach capability defines. `gband kill-server` SHALL stop the running server as the session-server capability defines. No subcommand SHALL write log events to standard output or standard error.

#### Scenario: Run the server
- **WHEN** the user runs `gband server` with `SHELL=/bin/true`
- **THEN** the process exits with status 0
- **AND** it prints nothing to standard output or standard error
- **AND** the server log file holds an event stating that the server started

#### Scenario: Server runs until its program exits
- **WHEN** the user runs `gband server` with the default shell
- **THEN** the process keeps running until every pane's shell has exited

#### Scenario: Run attach
- **WHEN** the user runs `gband attach` and then detaches
- **THEN** the client log file holds an event stating that the client started

#### Scenario: Run kill-server
- **WHEN** the user runs `gband kill-server` with no server running
- **THEN** the client log file holds an event stating that the client started
- **AND** the process exits with status 1

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

### Requirement: Invalid invocations
`gband` run with no subcommand, or with a subcommand or option it does not recognise, SHALL print usage to standard error and exit with status 2. It SHALL NOT create or write a log file.

#### Scenario: No subcommand
- **WHEN** the user runs `gband` with no arguments
- **THEN** standard error holds the usage, listing the `server` and `attach` subcommands
- **AND** the process exits with status 2

#### Scenario: Unknown subcommand
- **WHEN** the user runs `gband frobnicate`
- **THEN** standard error names `frobnicate` as unrecognised
- **AND** the process exits with status 2
- **AND** no log file is created

### Requirement: Server selection
The `server`, `attach` and `kill-server` subcommands SHALL address one server, identified by its socket path. Two options select it, and each SHALL be accepted before or after the subcommand:

| option | selects |
|---|---|
| `-S <name>`, `--server <name>` | the server named `<name>` |
| `-p <path>`, `--socket <path>` | the server listening on `<path>` |

A server name SHALL be 1 to 64 bytes long and hold only ASCII letters, digits, `_` and `-`. A relative `<path>` SHALL be resolved against the working directory. A command SHALL resolve its socket path as follows, first match winning:

1. `-p <path>`: that path.
2. `-S <name>`: `<name>.sock` in the gband runtime directory the session-server capability defines.
3. The `GBAND` environment variable, when it is set and not empty: the path it holds.
4. `default.sock` in the gband runtime directory.

Giving both options, a name that breaks the rule, or an empty path SHALL be an invalid invocation. A resolved socket path longer than 107 bytes SHALL be refused with one line on standard error that names the path and the limit, and exit status 1.

#### Scenario: Named server
- **WHEN** the user runs `gband -S feature attach` with `XDG_RUNTIME_DIR=/run/user/1000`
- **THEN** the client connects to `/run/user/1000/gband/feature.sock`

#### Scenario: Option after the subcommand
- **WHEN** the user runs `gband kill-server -S feature`
- **THEN** it stops the server named `feature`

#### Scenario: Explicit socket
- **WHEN** the user runs `gband attach -p target/gband.sock` in `/home/u/repo`
- **THEN** the client connects to `/home/u/repo/target/gband.sock`

#### Scenario: Default server
- **WHEN** the user runs `gband attach` outside a gband pane with `XDG_RUNTIME_DIR=/run/user/1000`
- **THEN** the client connects to `/run/user/1000/gband/default.sock`

#### Scenario: Inside a pane
- **WHEN** the user runs `gband kill-server` without an option in a pane of the server named `feature`
- **THEN** it stops the server named `feature`

#### Scenario: Both options
- **WHEN** the user runs `gband -S a -p /tmp/b.sock attach`
- **THEN** standard error holds the usage
- **AND** the process exits with status 2

#### Scenario: Invalid name
- **WHEN** the user runs `gband -S a.b attach`
- **THEN** standard error names `a.b` as an invalid server name
- **AND** the process exits with status 2

#### Scenario: Path too long
- **WHEN** the user runs `gband server -p` with a path of 120 bytes
- **THEN** standard error names the path and the 107-byte limit
- **AND** the process exits with status 1
- **AND** no shell is started
