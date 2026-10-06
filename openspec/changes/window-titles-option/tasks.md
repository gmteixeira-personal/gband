## 1. Deltas on the current specs

- [ ] 1.1 Re-copy configuration's "Options" and plugin-testing's "Case environment" from the main specs on `dev`. Re-apply this change's additions to each: the `window_titles` row and the boolean reading in "Options"; the `window_titles` field, `GBAND_WINDOW_TITLES=off` and the two scenarios in "Case environment". Check that window-names' "Border title" on `dev` still matches what "Titles turned off" refers to. Verify with `openspec validate window-titles-option --strict` and by diffing each requirement against the main spec, which must show only these additions

## 2. Option and environment

- [ ] 2.1 Declare the client option `window_titles` in `crates/lua/src/options.rs`, a boolean defaulting to `true`, read by `gband.opt` and set by `gband.set`. Verify "Window titles off" and "Window titles of the wrong type" in `crates/lua/tests/options.rs`
- [ ] 2.2 Read `GBAND_WINDOW_TITLES` once at client start in `crates/client/src/lib.rs`, following `crates/client/src/animation.rs`. In `crates/client/src/render.rs`, skip the border title when the variable is `off` or the option is `false`. Verify every "Titles turned off" scenario in `tests/window_names.rs`

## 3. Test environments

- [ ] 3.1 Add the `window_titles` field to `g.start` in `crates/harness/src/case.rs`, default `false`, and set `GBAND_WINDOW_TITLES=off` in the case environment unless it is `true`. Verify "No window titles by default" and "Window titles asked for" in `tests/lua/window_names_spec.lua`
- [ ] 3.2 Set `GBAND_WINDOW_TITLES=off` in the base environment of `crates/harness/src/env.rs`. Make the tests in `tests/window_names.rs` that read titles remove it through the terminal's `adjust` hook. Verify with `cargo test --test window_names`

## 4. References and assertions

- [ ] 4.1 Pass `window_titles = true` to every case of `tests/lua/window_names_spec.lua`. Regenerate the references under `tests/lua/screenshots/` and `examples/plugins/*/tests/screenshots/` with `GBAND_UPDATE_SCREENSHOTS`. Compare each with its content at the commit before window-names merged, `git show <commit>:<path>`. Each must match except the key list's `N` line and lines moved by it. Verify with `cargo test --test lua_specs`, which also runs the example plugins' cases
- [ ] 4.2 List the end-to-end tests window-names changed for titles with `git diff --no-ext-diff <window-names merge>^1 <window-names merge> --stat -- tests/`. Restore each assertion that reads a tile's top border to its form before window-names, leaving `tests/window_names.rs` alone. Verify with `cargo test`

## 5. Documentation and gates

- [ ] 5.1 Update `README.md` (the `window_titles` option, and `GBAND_WINDOW_TITLES` beside `GBAND_ANIMATIONS`), `docs/plugins.md` (`window_titles` in the options list) and `docs/testing.md` (`g.start`'s `window_titles` and the default without titles). Verify with `rg 'window_titles|GBAND_WINDOW_TITLES'` that every mention gives the right default
- [ ] 5.2 Run `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`, `cargo test` and `openspec validate window-titles-option --strict`, and verify all pass
