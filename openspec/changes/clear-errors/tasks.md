## 0. Specs

- [x] 0.1 For each MODIFIED requirement in this change's delta specs, compare the block with the current `openspec/specs/<capability>/spec.md` block of the same requirement, as sidebars-borders-steps and the changes archived before this one left it, and fold in every requirement text and scenario this change did not write. Verify that the only differences left are this change's own, and that `openspec validate clear-errors --strict` passes with no line about `error-list`

## 1. Clearing in the client

- [x] 1.1 Add `gband.clear_errors()` to `crates/lua/src/api.rs`. Raise `outside_callback` while loading. Empty `error` and `errors` in the stored state through a new function in `crates/lua/src/ui.rs` that also marks the state changed, and queue `Dispatch::ClearErrors`. Make `ui::set_state` run `on_state` after that mark even when the pushed state is equal. Verify "Clear while loading", "Clearing keeps a failed plugin failed" and the first step of "Cleared by hand", the empty list read in the same function, in `crates/lua/tests/config.rs`.
- [x] 1.2 Add `clear_errors` to `CLIENT_ONLY` in `crates/lua/src/sides.rs`. Verify "Clearing errors in the server" in `crates/lua/tests/plugins.rs`.
- [x] 1.3 Apply `Dispatch::ClearErrors` in `crates/client/src/lib.rs`: empty the error list and the latest error, in order with the callback's other dispatches and before the callback's errors are reported. Verify in `crates/client/tests/statusline.rs`: the configuration scenarios "Cleared by hand", "Banner cleared by hand", "Error after a clear" and "Error in the clearing callback", and the status-line scenarios "Cleared by hand" and "Disabled component stays disabled".
- [x] 1.4 Add `tests/errors.rs` for "Server error cleared in one client": two clients attach to a server whose `user/server.lua` holds a syntax error on line 2, the first clears through a binding, the second still shows the error, and a third client that attaches shows it. Verify with `cargo test --test errors`.

## 2. Error list plugin

- [x] 2.1 In `crates/lua/src/runtime/gband/errors.lua`, add a local `clear()` that calls `gband.clear_errors()` and, when the error list is open, empties its held texts, shows `no errors` and sets a floating window's title to `errors`. Call it from a `c` entry in the plugin window's `keys` and from the action and command `errors.clear`. Title a floating window `errors  c clear` while it holds texts. Verify in `crates/lua/tests/errors.rs`: "Clear action and command registered", "Floating plugin window of the defaults", "No errors", "Clear a tiled error list", "Clear command with the list open" and "Clear action without the list".
- [x] 2.2 Add `tests/lua/errors_spec.lua`, with references under `tests/lua/screenshots/errors_spec/`, and a test in `tests/lua_specs.rs` that runs it, as the gband-test skill describes. Cover "Clear with c" with a screenshot before and after `c`, "Cleared list wrapped again" with `g.resize`, and "Prefix c is not the list's c". Read each reference before committing it, and verify with `cargo test --test lua_specs`.

## 3. Docs

- [x] 3.1 Update `README.md` and `docs/plugins.md`:
  - `gband.clear_errors()` beside `gband.errors()`, where it may be called, and what it leaves alone: no reload, disabled components and failed plugins, the logs, and other clients;
  - the action and command `errors.clear`, with a sample binding;
  - `c` in the error list, in both kinds, and the title hint.

  Verify that `rg -n 'clear_errors|errors\.clear' README.md docs/plugins.md` prints lines from both files.

## 4. Gate

- [x] 4.1 Run `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`, and verify all pass.
