## 1. Focus after its layout

- [x] 1.1 In `crates/server/src/connection.rs`, add `deliver_reply`: for `Reply::Focus(window)` call `sync`, then send the focus only when `sent.state` holds the window in its layout, otherwise log at debug level and send nothing; deliver every other reply as today. Use it in the `replies` branch and in the `Barrier::Flush` branch, which syncs before draining replies and keeps its final sync. Verify with `cargo build -p gband-server` and `cargo test -p gband-server`
- [x] 1.2 Add a test to `crates/server/tests/programs.rs` for "Window closed before its focus": open a window running `true` with focus 30 times, pumping messages for 100 ms after each, and check that the layout ends holding only the first window. Verify it fails at least once in 20 runs against the old delivery, and passes 50 runs with the fix
- [x] 1.3 In `crates/server/tests/programs.rs`, make `argument_list_program_runs_without_a_shell` run the argument list `awk`, `BEGIN { printf "%s-%s", ARGV[1], ARGV[2]; getline line < "-" }`, `a`, `b`, so its window stays open after printing `a-b`

## 2. Checks

- [x] 2.1 Run the server's `programs` test binary 50 times with `--test-threads=16` and confirm `argument_list_program_runs_without_a_shell` never fails, then run `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, and check that every scenario of the session-server delta maps to a passing test
