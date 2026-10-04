## 1. Reply generation

- [ ] 1.1 Rename `LoggingCallbacks` to `PaneCallbacks` in `crates/server/src/callbacks.rs` and give it a `replies: Vec<u8>` buffer. Answer DA1, DSR 5 and 6, XTVERSION and both DECRQM forms in `unhandled_csi`, as the spec's table defines. Read mode states from the `Screen` accessors the design lists. Log only the sequences it does not answer. Verify with unit tests that feed each query through a `vt100::Parser<PaneCallbacks>` and compare the buffer with the exact reply bytes, including `\e[5;10H\e[6n\e[1;1H` in one `process` call answering `\e[5;10R`
- [ ] 1.2 Verify with a unit test that `\e[?u`, `\e]11;?\a` and `\e[999z` leave the reply buffer empty

## 2. Writing replies to the PTY

- [ ] 2.1 Add `Input::Reply(Vec<u8>)` in `crates/server/src/pane.rs`, written raw by the input thread. Create the input channel before the reader thread starts, and have `Pane::process` take the reply buffer under the lock and send it when it is not empty. Verify with `cargo build -p gband-server`
- [ ] 2.2 Add `crates/server/tests/queries.rs`. Each test runs a server whose program is `sh -c` with `stty raw -echo`, `printf` of one query, and `head -c <reply length>` into a file. Verify that the file holds the exact reply for DA1, DSR 5, cursor position after `\e[5;10H`, XTVERSION with the package version, DECRQM for 2004 set, 1 reset and 7727 unrecognised, and that all of this happens with no client attached
- [ ] 2.3 Verify with a test in `crates/server/tests/queries.rs` that a program writing `\e[?u\e[c` reads exactly `\e[?62;22c`

## 3. Finish

- [ ] 3.1 Run `.claude/flow-gate` and verify that formatting, clippy with `-D warnings` and the workspace tests all pass
- [ ] 3.2 Check every new or changed Rust file against the repository's coding rules, which forbid narrative comments, and verify that no comment remains except one explaining an upstream API restriction
- [ ] 3.3 With fish 4 as the pane's shell, run `cargo build`, `gband kill-server` and `gband attach` in a real terminal, and verify that the fish prompt appears at once, with no warning about the Primary Device Attribute query
