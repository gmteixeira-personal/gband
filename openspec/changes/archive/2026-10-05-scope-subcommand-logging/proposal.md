## Why

The command-line and logging specs say every `gband` subcommand starts logging and writes its log to a file series. The shell-completions capability added `completions` and `install-completions`, which start no logging and create no log file, so the main specs now contradict each other. The logging spec also names only `attach` and `kill-server` in the `client` series, though `list-sessions` and `kill-session` already log there.

## What Changes

- Scope the command-line requirement "Subcommands run the session" so that only the five subcommands it lists start logging and record a start event. The ban on log events on standard output and standard error still applies to every subcommand.
- Scope the logging requirement "Log to a file per role" to the subcommands that log, and list `list-sessions` and `kill-session` in the `client` series next to `attach` and `kill-server`.
- Scope the logging requirements "Log directory location", "Level from the environment" and "Panics are logged", which also read as applying to every subcommand or process, to the subcommands that log.
- Add scenarios that pin the `client` series for `list-sessions` and `kill-session`, and that the completion subcommands write no log.
- No behaviour changes. The code already logs this way; only the specs move to match it.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `command-line`: "Subcommands run the session" requires logging only of the subcommands it lists.
- `logging`: "Log to a file per role", "Log directory location", "Level from the environment" and "Panics are logged" apply to the subcommands that log, and the `client` series lists `list-sessions` and `kill-session`.

## Impact

- `openspec/specs/command-line/spec.md` and `openspec/specs/logging/spec.md`, through the delta specs at archive.
- No source, test or dependency changes.

## Coordination

### Author
- gmteixeira

### Depends On
- shell-completions

### Expected Files
- openspec/changes/scope-subcommand-logging/
