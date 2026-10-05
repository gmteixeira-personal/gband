## Context

`src/main.rs` maps each subcommand to a log role: `server` to `server`, and `attach`, `list-sessions`, `kill-session` and `kill-server` to `client`. `completions` and `install-completions` map to no role and never initialise logging, as the shell-completions capability requires. The command-line and logging specs predate shell-completions and say every subcommand logs, and the logging spec names only `attach` and `kill-server` in the `client` series. See proposal.md for why this matters.

## Goals / Non-Goals

**Goals:**
- Make the command-line and logging specs agree with shell-completions and with the code.

**Non-Goals:**
- Changing which subcommands log, or to which series.
- Extending "Events name their server" to `list-sessions` and `kill-session`. The code already attaches the socket path to their events, but the requirement's wording does not contradict shell-completions, so it stays for a separate change.
- Adding or changing tests.

## Decisions

- **Name the logging subcommands explicitly rather than excluding the completion ones.** "The five subcommands listed" stays true when another capability adds a subcommand that does not log, while "every subcommand except `completions` and `install-completions`" would break again. Each requirement defers other subcommands to the capability that defines them.
- **Keep "No subcommand SHALL write log events to standard output or standard error" universal.** It holds for the completion subcommands too, since they emit no log events at all.
- **Add only scenarios that existing tests already check.** The change is spec-only, so each new scenario maps to a test that passes today: `session_requests_without_a_server_fail_and_create_nothing` in `tests/subcommands.rs` for `list-sessions` and `kill-session`, and `no_log_file` in `tests/completions.rs` for `completions bash`.

## Risks / Trade-offs

- [`session_requests_without_a_server_fail_and_create_nothing` runs `list-sessions` and `kill-session` in one state directory and checks the `client` series once after both] → It cannot show which of the two wrote the start event. The code maps both to the `client` role in one match arm, so the gap is in test precision, not behaviour. A per-subcommand assertion belongs to a change that touches tests.
