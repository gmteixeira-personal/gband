## MODIFIED Requirements

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
