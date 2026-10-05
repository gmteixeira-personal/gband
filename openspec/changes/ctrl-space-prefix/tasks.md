## 1. Precondition

- [ ] 1.1 Confirm config-directory has an archive record on `origin/<base>` and that `openspec validate ctrl-space-prefix --strict` passes against the specs it left. Verify that the command exits with status 0

## 2. Default prefix

- [ ] 2.1 Set `prefix = "ctrl+space"` in the built-in default configuration in `crates/lua/src/`, and make the `prefix` of `Options::default()` in `crates/lua/src/options.rs` Ctrl with `KeyCode::Char(' ')`. Verify with `cargo test -p gband-lua` after updating the tests that pin the default prefix, including `crates/lua/tests/config.rs`
- [ ] 2.2 Update the `crates/lua` tests for "Default prefix", "Rebind a key", "Unbind a key", "Prefix changed after binding" (Ctrl+Space reaching the pane) and "Direct binding of the prefix key" binding `ctrl+space` at line 4. Verify with `cargo test -p gband-lua`

## 3. Client

- [ ] 3.1 In `crates/client/src/bindings.rs` and `crates/client/tests/actions.rs`, press `ctrl+space` wherever a test uses the default keymap's prefix, expect `\x00` for the literal prefix, and keep Ctrl+A where a test pins it as an ordinary key. Verify with `cargo test -p gband-client`

## 4. End to end

- [ ] 4.1 Replace the prefix byte `\x01` with `\x00` in `tests/attach.rs`, `tests/config.rs` and `tests/sessions.rs`, leaving tests that set their own prefix unchanged. Verify with `cargo test --test attach --test config --test sessions`
- [ ] 4.2 Split `prefix_key_passes_ctrl_a_and_discards_unbound_keys` in `tests/attach.rs` into a "Literal Ctrl+Space" test that runs `cat -v`, presses Ctrl+Space twice and Enter and expects one `^@` line, and a "Literal Ctrl+A" test that presses Ctrl+A once at a bash prompt with `abc` typed and checks the cursor moved to the line start. Keep the unbound-key discard check. Verify with `cargo test --test attach`
- [ ] 4.3 Add a `tests/config.rs` test for "Default prefix": a `user/init.lua` that binds `prefix q` to `gband.action.close_pane` without setting `prefix` closes the focused pane on Ctrl+Space then `q`. Verify with `cargo test --test config`
- [ ] 4.4 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`, and check that no comment breaks the rules in `CLAUDE.md`. Verify that both commands pass

## 5. Documentation

- [ ] 5.1 Change the `prefix` default in the README "Configuration" options table to `"ctrl+space"`, and add one sentence on setting `prefix = "ctrl+a"` in `user/init.lua` to get the old prefix back. Verify by reading the rendered section
