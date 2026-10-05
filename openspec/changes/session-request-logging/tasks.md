## 1. Per-subcommand client series

- [ ] 1.1 Split `session_requests_without_a_server_fail_and_create_nothing` in `tests/subcommands.rs` so each of `list-sessions`, `kill-session` and `-s work kill-session` runs in its own state directory, and each case asserts the one-line stderr failure, a `client started` event in its own `client` log, no `server` file and no runtime directory; verify with `cargo test --test subcommands session_requests_without_a_server`

## 2. Session request events name their server

- [ ] 2.1 Add a test to `tests/subcommands.rs` that runs `gband -S a list-sessions` and `gband -S a kill-session` with no server running, each in its own state directory, and asserts that the `client` log holds at least one line besides `client started` and that every such line contains `socket=` followed by the path to `a.sock`; verify with `cargo test --test subcommands`
- [ ] 2.2 Confirm the new test fails when the `socket` span is removed from `src/main.rs`, then restore the span, so the test is shown to check the span and not the message text

## 3. Gate

- [ ] 3.1 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings` and verify both pass
- [ ] 3.2 Run `openspec validate session-request-logging --strict` and verify it passes
