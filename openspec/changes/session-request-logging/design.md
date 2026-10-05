## Context

`src/main.rs` logs `client started` before it resolves the socket path, then enters a `gband` span carrying `socket` for the rest of the run. `list-sessions` and `kill-session` run inside that span, so an error such as `no server is running` is logged as `gband{socket=<path>}: ...`. The start event carries no socket, which the requirement allows: it names only events written after the path is resolved.

`tests/subcommands.rs` drives each run with its own `XDG_STATE_HOME` and `XDG_RUNTIME_DIR` and has `log_text` and `log_files` helpers per role. `session_requests_without_a_server_fail_and_create_nothing` loops over `list-sessions`, `kill-session` and `-s work kill-session` in one state directory and checks the logs once after the loop.

## Goals / Non-Goals

**Goals:**
- Each of `list-sessions` and `kill-session` is shown to start the `client` series and write no `server` file on its own.
- Their post-start events are shown to carry the socket path.

**Non-Goals:**
- Changing what or where either subcommand logs.
- Testing the successful paths for socket naming. A successful `list-sessions` or `kill-session` writes no event after start, so it has nothing to check.

## Decisions

- **Give each subcommand its own state directory inside the existing test.** Loop over the argument lists and call `state_home` with a distinct name per case, asserting the client start event, the empty `server` series and the missing runtime directory per case. This keeps the stderr checks the test already makes and needs no new helper.
- **Test socket naming with no server running.** It is the one path where both subcommands log an event after start, and it needs no running server, so the test stays in `tests/subcommands.rs` beside the split test instead of `tests/servers.rs`, which starts real servers.
- **Match the span field, not the path text.** The `no server is running` message itself contains the socket path, so a check for the path alone would pass with no span. The test checks that each post-start line contains `socket=` followed by the path to `a.sock`.

## Risks / Trade-offs

- [The span's text form `gband{socket=...}` is `tracing-subscriber`'s formatting, not a spec guarantee] → The test checks only `socket=<path>`, the field-and-value pair, which stays stable across the formatter's layout options.
