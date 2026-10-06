## 1. Scratch guard

- [ ] 1.1 Create `crates/scratch` (package `gband-scratch`, depending only on `rustix`) with `Scratch::new(kind, name)`, `Deref<Target = Path>`, `AsRef<Path>` and the drop behaviour of design.md: remove on pass and sweep dead `gband-<kind>-<name>-<digits>` siblings, keep and print `kept <path>` while panicking, and retry removal once after setting directories to 0700. Add it to `[workspace.dependencies]` in `Cargo.toml`, and verify `cargo build -p gband-scratch` succeeds
- [ ] 1.2 Add tests in `crates/scratch/tests/` for removal on drop, keeping on panic (drop inside `catch_unwind`), sweeping a sibling named with a dead process ID, sparing one named with a live process ID and one whose suffix is not all digits, and removing a tree with a read-only directory. Verify `cargo test -p gband-scratch` passes

## 2. Lint

- [ ] 2.1 Add `clippy.toml` at the workspace root disallowing `std::env::temp_dir` with a reason naming `gband_scratch::Scratch`, and allow it on the guard's own call. Verify `cargo clippy --all-targets -- -D warnings` lists every remaining call site, which sections 3 to 5 then remove
- [ ] 2.2 Allow `clippy::disallowed_methods` on `scratch` in `crates/harness/src/case.rs` and on the two leftover scans in `tests/plugin_testing.rs`, leaving the `gband test` runner's naming and removal unchanged. Verify `cargo test --test plugin_testing` passes

## 3. End-to-end and harness tests

- [ ] 3.1 Make the end-to-end `TestEnv` in `tests/common/mod.rs` own a `Scratch` of kind `e2e` as its last field, drop `scratch_root` and the FNV hash, and add `gband-scratch` to the root crate's dev-dependencies. Verify `cargo test --tests` passes with no `/tmp/gband-e2e-*` left afterwards
- [ ] 3.2 Move `tests/subcommands.rs` (kind `subcommands`) and the `Project` in `tests/plugin_testing.rs` (kind `pt`) to `Scratch`, keeping one guard per test for the runtime home that `runtime_home` returns today. Verify `cargo test --test subcommands --test plugin_testing` passes with no `/tmp/gband-subcommands-*` or `/tmp/gband-pt-*` left
- [ ] 3.3 Move `crates/harness/tests/terminal.rs` (kind `harness`) and the tests in `crates/harness/src/screenshot.rs` (kind `screenshot`) to `Scratch`, with `gband-scratch` as a dev-dependency of `gband-harness`. Verify `cargo test -p gband-harness` passes with no `/tmp/gband-harness-*` or `/tmp/gband-screenshot-*` left

## 4. Server and client tests

- [ ] 4.1 Make `runtime_dir` in `crates/test-support/src/lib.rs` return a `Scratch` of kind `srv`, drop its FNV hash, make `TestServer` own the guard after its server handle, and change `start_in` and `start_with` to take a `Scratch`. Update the callers in `crates/server/tests/` and `crates/client/tests/connect.rs`. Verify `cargo test -p gband-server` passes with no `/tmp/gband-srv-*` left
- [ ] 4.2 Replace the `Scratch` types and the `Client` root in `crates/client/tests/` (events, bridge, bars, actions, statusline, plugin_windows, local_actions, including `scratch` for `gband-client-opener-*`) and the two directories in `crates/client/src/bindings.rs` with `gband_scratch::Scratch`, with `gband-scratch` as a dev-dependency of `gband-client`. Verify `cargo test -p gband-client` passes with no `/tmp/gband-client-*` left

## 5. Lua and binary unit tests

- [ ] 5.1 Replace the `Scratch` types in `crates/lua/tests/common/mod.rs`, `crates/lua/src/watch.rs` and `crates/lua/src/directory.rs` with `gband_scratch::Scratch`, with `gband-scratch` as a dev-dependency of `gband-lua`. Verify `cargo test -p gband-lua` passes with no `/tmp/gband-lua-*` left
- [ ] 5.2 Move the `scratch` helpers in `src/paths.rs` (kind `paths`) and `src/completions.rs` (kind `completions`) to `Scratch` and drop their trailing `remove_dir_all` calls. Verify `cargo test --bin gband` passes with no `/tmp/gband-paths-*` or `/tmp/gband-completions-*` left

## 6. Verification

- [ ] 6.1 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass with no `std::env::temp_dir` call outside the allowed sites
- [ ] 6.2 Count `/tmp/gband-*` entries before and after a full `cargo test --workspace` run with no other gband tests running, and verify the run adds none
- [ ] 6.3 Make one end-to-end test fail on purpose, run it, and verify `kept <path>` appears in its output and the directory exists. Restore the test, run it again, and verify the directory is gone
