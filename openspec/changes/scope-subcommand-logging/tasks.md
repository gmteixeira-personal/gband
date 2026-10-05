## 1. Confirm the specs match the code

- [x] 1.1 Check that `Command::role` in `src/main.rs` maps `server` to the `server` role, `attach`, `list-sessions`, `kill-session` and `kill-server` to the `client` role, and the completion subcommands to none, as the delta specs state
- [x] 1.2 Run `cargo test --test subcommands --test completions` and verify that `session_requests_without_a_server_fail_and_create_nothing`, `kill_server_without_a_server_logs_to_the_client_series` and `no_log_file` pass

## 2. Validate the change

- [x] 2.1 Run `openspec validate scope-subcommand-logging --strict` and verify it passes
- [x] 2.2 Run `rg -n -i 'every .{0,10}subcommand|each subcommand|any subcommand|a subcommand' openspec/specs/command-line openspec/specs/logging` and verify that every match outside a requirement this change modifies holds for `completions` and `install-completions` too
