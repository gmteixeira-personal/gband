## Why

The logging requirement "Events name their server" lists only `server`, `attach` and `kill-server`. `list-sessions` and `kill-session` also resolve a socket path and already tag their events with it, but no requirement or test holds them to it. Separately, `session_requests_without_a_server_fail_and_create_nothing` runs `list-sessions` and `kill-session` in one state directory and checks the `client` log once, so it cannot show that each of them writes its own start event, as scope-subcommand-logging's scenarios require.

## What Changes

- Extend "Events name their server" to `list-sessions` and `kill-session`, with a scenario for each.
- Split `session_requests_without_a_server_fail_and_create_nothing` so each subcommand runs in its own state directory and is checked for its own `client` start event and for writing no `server` file.
- Test that the events `list-sessions` and `kill-session` write after start carry the socket path they resolved.
- No behaviour changes. The code already logs this way.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `logging`: "Events name their server" covers `list-sessions` and `kill-session`.

## Impact

- `tests/subcommands.rs`: the split test and the new socket-path test.
- `openspec/specs/logging/spec.md`, through the delta spec at archive.
- No source or dependency changes.

## Coordination

### Author
- gmteixeira

### Depends On
- scope-subcommand-logging

### Expected Files
- tests/subcommands.rs
- openspec/changes/session-request-logging/
