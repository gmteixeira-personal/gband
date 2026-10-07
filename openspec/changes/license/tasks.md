## 1. License files

- [x] 1.1 Create `LICENSE` with the opening paragraph from design.md, a blank line, and the GPL-3.0 text downloaded from `https://www.gnu.org/licenses/gpl-3.0.txt`. Verify the text below the paragraph is byte-identical to the download with `diff <(tail -n +6 LICENSE) gpl-3.0.txt`.
- [x] 1.2 Create `LICENSE-EXCEPTION` with the exception text from design.md, verbatim. Verify with a diff against the design's block.
- [x] 1.3 Create `LICENSE-MIT` with the MIT License text, `Copyright (c) 2026 The gband authors`, followed by the covered paths: `docs/tutorial/`, `examples/`, `crates/lua/src/defaults.lua`, `crates/lua/src/defaults_server.lua` and `crates/lua/src/runtime/`. Verify every listed path exists with `ls -d`.

## 2. Package metadata

- [x] 2.1 Add `license-file = "LICENSE"` to `[workspace.package]` in `Cargo.toml`, and `license-file.workspace = true` to the root `[package]` and to the `[package]` of every `crates/*/Cargo.toml`. Verify with `cargo metadata --format-version 1 --no-deps | jq -r '.packages[] | "\(.name) \(.license_file)"'` that every package names the workspace `LICENSE`.
- [x] 2.2 Run `cargo build --workspace` and verify it succeeds with no manifest warning.

## 3. README

- [x] 3.1 Append the `## License` section from design.md to `README.md`, after `## Acknowledgements`. Verify it is the last `##` heading with `rg -n '^## ' README.md | tail -1`, and that its three links resolve to the new files.

## 4. Checks

- [x] 4.1 Run `cargo test --workspace` and verify it passes, including the tutorial checks in `crates/lua/tests/tutorial.rs`.
- [x] 4.2 Verify no source file gained a license header with `git diff --stat --no-ext-diff origin/dev -- crates/ docs/ examples/ src/`, which lists only the `Cargo.toml` files.
