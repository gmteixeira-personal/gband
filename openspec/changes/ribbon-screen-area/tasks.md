## 1. Reported size follows the ribbon

- [x] 1.1 In `crates/client/src/lib.rs`, make `Display::reported_size()` return the ribbon area's size. Make `set_size` and `set_bars` return the new reported size only when it differs from the last one reported. Verify with a test in `crates/client/tests/bars.rs`: on an 80×24 terminal, adding a left bar of size 20 by a binding reports exactly one `Resize` of 60×24.
- [x] 1.2 Push a `resize_step` from the bar refresh in `Controls::react` whenever `set_bars` returns a size, so a single callback or load reports at most once. Verify with `bars.rs` tests: one callback that adds two bars reports one `Resize`, and `set_config(id, { side = "right" })` on a left bar of size 20 reports none.
- [x] 1.3 Place the bars before the handshake. Build the `Display` and `Controls` in `run` from the terminal's size and the loaded configuration, before `connect`, and pass `display.reported_size()` into `connect`/`handshake` in place of `crossterm::terminal::size()` in `crates/client/src/connect.rs`. Make `attach` take the prepared display and controls. Verify with an end-to-end test: attaching to a new session in an 80×24 terminal with the default configuration gives the first window `tput cols` `37` with no resize after it.

- [x] 1.4 Add `--size <cols>x<rows>` to `gband server` in `src/main.rs`, rejecting a value not of that form with exit status 2 and no log, and start the server's session at it through `ServerConfig::area` in `crates/server/src/lib.rs`. Make `Transport::open` take the reported size and have `UnixTransport` pass it to the server it starts. Verify with `tests/attach.rs` `started_server_takes_the_first_report` (no server running, 80×24, default configuration: `tput cols` prints `37` and the server log shows no PTY resize), `tests/sessions.rs` `server_starts_its_session_at_the_given_size` (`stty size` prints `28 48` with no client) and `tests/subcommands.rs` `invalid_server_size_is_rejected_without_a_log`.

## 2. Spec scenarios as tests

- [x] 2.1 Update `crates/client/tests/bars.rs` to the bars delta's scenarios: "Bar added by a key", "Camera follows the narrower ribbon", "Full width beside a bar", "Full width follows the bar's size", "Bar moved to the other side" and "Bar too wide to show". Verify with `cargo test -p gband-client --test bars`.
- [x] 2.2 Update `crates/client/tests/sidebar.rs` so that the default sidebar reports 79×24 and "Turning the sidebar off" reports 80×24 once. Verify with `cargo test -p gband-client --test sidebar`.
- [x] 2.3 Add an end-to-end test for "Full width beside the sidebar": toggling full width on the only column, beside the default sidebar on an 80×24 terminal, draws a 79-column tile with its right border on column 79, and `tput cols` prints `77`. Verify with `cargo test --test attach`.
- [x] 2.4 Restate "Resize reaches the program" in `tests/attach.rs` as two tests, one with no bar at 70 columns and one beside the sidebar at 71 columns, both printing `33`. Verify with `cargo test --test attach`.

## 3. Tests that measured the old area

- [x] 3.1 Run `cargo test --workspace` and list every failure that comes from tiles measured beside the default sidebar. Candidates are `tests/attach.rs`, `tests/navigation.rs`, `tests/mouse.rs`, `tests/floating.rs`, `tests/sidebar.rs` and `tests/config.rs`. For each one, restate the numbers for the 79-column area, or give it a configuration with no bar where the test is not about bars. Verify that `cargo test --workspace` passes.
- [x] 3.2 Regenerate the Lua screenshots with `gband test --update` for `tests/lua/` and for each example plugin under `examples/plugins/*/tests/`. Check with `git diff` that each changed reference only moves tile borders by one column. Verify that `gband test` passes with no `.new` files left.

- [x] 3.3 In `crates/core/src/geometry.rs`, make `loop_width` loop when the strip's width less its widest column is at least the viewport less one, and add the layout-view Camera delta with the scenario "Strip one cell short of the terminal loops". Verify with `crates/core/tests/geometry.rs`, a `crates/core/tests/view.rs` test for the scenario, and `tests/lua/loop_bands_spec.lua` passing unchanged beside the default sidebar.

## 4. Key bindings scenarios

- [ ] 4.1 After merging the latest `origin/dev`, copy the client-attach requirement "Key bindings", whole, from `openspec/specs/client-attach/spec.md` on `origin/dev` into this change's client-attach delta under `## MODIFIED Requirements`. Add "the client has no bar" to the scenarios "Repeated resize" and "Grow the column", and give their tests in `tests/navigation.rs` and `tests/attach.rs` a configuration with no bar. If an open change that also modifies "Key bindings" has archived since, take its version from `origin/dev`. Verify with `openspec validate ribbon-screen-area --strict` and `cargo test --test navigation --test attach`.

## 5. Documentation

- [ ] 5.1 In `docs/plugins.md`, replace "Bars never change a window's size" with the new rule: the client reports the space its bars leave, so adding, resizing or removing a bar resizes the windows, and moving a bar to the other side does not. Verify by reading the bars section against the bars delta.
