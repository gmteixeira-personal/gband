## 1. Private bind

- [x] 1.1 In `crates/server/src/lib.rs`, replace the umask step in `bind`: create a `0700` `.gband-bind-<pid>-<n>` directory beside the socket path, retrying the counter on `EEXIST`; open it with `O_PATH | O_DIRECTORY`; bind to `/proc/self/fd/<fd>/s`; set `s` to `0600` with `fchmodat`; `renameat` it to the socket path; remove the directory; on any failure after the `mkdir`, remove `s` and the directory before returning the error. Remove `PRIVATE_SOCKET_MASK` and the `Mode` import. Verify with `cargo build -p gband-server` and `rg -n umask crates/server/src` finding nothing
- [x] 1.2 Add unit tests beside `bind` for "Servers started together in one process": eight binds released by one barrier on an eight-worker runtime each give mode `0600`, and the `Umask:` line of `/proc/self/status` is unchanged; and for "Long explicit socket path": a 107-byte socket path binds with mode `0600`. Both check that no `.gband-bind-` directory remains. Verify with `cargo test -p gband-server --lib`
- [x] 1.3 Check that the race is gone: run the lifecycle test binary 200 times with `--test-threads=16` and confirm no `socket_is_private` failure, where the old bind failed about once in 60 runs

## 2. Checks

- [x] 2.1 Run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, and check that every scenario of the session-server delta maps to a passing test
