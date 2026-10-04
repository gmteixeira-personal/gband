## 1. Wire protocol version 3

- [ ] 1.1 Add `SessionName` to `crates/protocol/src/` with `FromStr` enforcing 1 to 64 bytes of `[A-Za-z0-9_-]` and a `Deserialize` that applies the same check, and add `SessionSummary`. Verify with tests in `crates/protocol/tests/messages.rs` that `default`, `work-2` and a 64-byte name parse, and that an empty name, a 65-byte name, `work.2`, `a/b` and `a b` are refused both by `FromStr` and when decoding
- [ ] 1.2 Set `PROTOCOL_VERSION = 3` and add `Attach`, `ListSessions` and `KillSession` to `ClientMessage`, and `Sessions`, `Killed` and `NoSuchSession` to `ServerMessage`. Verify with round-trip tests of every new message, including an attach naming `work` with `/tmp` and a sessions message of `default` (1, 0) and `work` (2, 1), and confirm the hello pinned-bytes test passes unchanged

## 2. Sessions in the server

- [ ] 2.1 Add the registry task to `crates/server/src/` that owns the map of session handles, spawns a session task per name, removes a session on `SessionEnded`, sets the server-wide `ended` flag when the map empties, and forwards SIGTERM to every session. Create the first session from the server config's session name before binding the socket. Verify that the existing lifecycle tests in `crates/server/tests/lifecycle.rs` pass, including the `/bin/true` exit
- [ ] 2.2 Give each session task its own working directory and initial screen area, run it in a `tracing` span carrying its name, and set `GBAND_SESSION` in its panes' environment. Verify with a test in `crates/server/tests/sessions.rs` that `echo $GBAND_SESSION` prints the session's name and that `pwd` in a session created with `/tmp` prints `/tmp`
- [ ] 2.3 Make the connection task wait for one request after `Info`, close on anything else, and on an attach ask the registry for the session and follow that session's state. Update the harness in `crates/server/tests/common/mod.rs` to send an attach request. Verify with tests that a key before a request closes the connection, that two clients attaching to a missing `work` at once share one session, and that the sync tests still pass
- [ ] 2.4 Answer `ListSessions` from each session's latest state and client counter, and `KillSession` by closing every pane of the session and replying `Killed` once it has ended, or `NoSuchSession`. Verify with tests that a server hosting `default` (1 pane, 0 clients) and `work` (2 panes, 1 client) lists both in name order, that killing `work` sends its client `Exited` and leaves `default` running, that killing the last session ends the server and removes the socket, and that killing `nope` answers `NoSuchSession`
- [ ] 2.5 Verify with tests in `crates/server/tests/sessions.rs` that opening a pane in `work` sends no layout to a client of `play`, that a key sent in `work` reaches no pane of `play`, that a 100×30 attach to `work` leaves the screen area of `default` at 120×40, that `exit` in the only pane of `work` tells only its client the session ended, and that SIGTERM with a client in each of two sessions sends both `Exited` before the server exits

## 3. Client and command line

- [ ] 3.1 Add the global `-s`/`--session` option to `src/main.rs` with `SessionName` as its value parser, resolve its absence to `default`, raise a clap `ArgumentConflict` error when it is given to `kill-server` or `list-sessions`, and pass the name to the server config and the client config. Verify with tests in `tests/subcommands.rs` that `gband attach -s 'a/b'` and `gband kill-server -s work` exit with status 2 without a log file, and that `gband --help` lists the five subcommands and `-s`
- [ ] 3.2 Make `gband attach` send an attach request with the session name and its working directory after reading `Info`, and spawn `<current_exe> server -s <name>` when no server is running. Update `crates/client/tests/connect.rs` to the new handshake. Verify with `cargo test -p gband-client`
- [ ] 3.3 Add a no-spawn mode to the client's connect path and the `list-sessions` and `kill-session` subcommands on top of it, printing the tab-separated list or waiting up to 5 s for `Killed`, and reporting a missing server, an unknown session and a protocol mismatch as the specs define. Verify with tests in `tests/` that both commands exit with status 1 naming the socket when no server runs and that neither creates the runtime directory

## 4. End-to-end

- [ ] 4.1 Add `tests/sessions.rs` with tests that `gband attach -s work` with no server starts one hosting only `work`, that a second `gband attach -s play` in another directory adds `play` whose `pwd` is that directory, that `gband list-sessions` prints both with their pane and client counts, that `gband -s work attach` after a detach shows `sleep 100` still running, and that `gband kill-session -s play` makes its client print `[exited]` while `work` keeps running. Verify that they pass
- [ ] 4.2 Add a test that `gband kill-server` with two sessions stops the shells of both. Verify that `cargo test --test sessions --test kill_server --test subcommands --test attach` passes

## 5. Finish

- [ ] 5.1 Run `.claude/flow-gate` and verify that formatting, clippy with `-D warnings` and the workspace tests all pass
- [ ] 5.2 Check every new or changed Rust file against the repository's coding rules, which forbid narrative comments, and verify that no comment remains except one explaining an upstream API restriction
- [ ] 5.3 In a real terminal, run `gband attach -s work`, open a second pane, detach, run `gband attach -s play` in another directory, then run `gband list-sessions` from a terminal outside gband and verify the counts. Reattach to `work` and verify both panes are restored
