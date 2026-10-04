## Purpose

Defines the `gband` binary's command line: the subcommands it accepts, the help it prints and the exit status each invocation returns.

## ADDED Requirements

### Requirement: Server and attach subcommands
The `gband` binary SHALL accept the subcommands `server` and `attach`. Until a later change gives them behavior, each subcommand SHALL start logging for its role, record one log event stating that it started, print nothing to standard output or standard error, and exit with status 0.

#### Scenario: Run the server stub
- **WHEN** the user runs `gband server`
- **THEN** the process exits with status 0
- **AND** it prints nothing to standard output or standard error
- **AND** the server log file holds an event stating that the server started

#### Scenario: Run the attach stub
- **WHEN** the user runs `gband attach`
- **THEN** the process exits with status 0
- **AND** it prints nothing to standard output or standard error
- **AND** the client log file holds an event stating that the client started

### Requirement: Help and version
`gband --help` SHALL print usage that lists the `server` and `attach` subcommands and exit with status 0. `gband --version` SHALL print the binary's version and exit with status 0.

#### Scenario: Print help
- **WHEN** the user runs `gband --help`
- **THEN** standard output lists the `server` and `attach` subcommands
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
