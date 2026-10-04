## 1. Crate

- [ ] 1.1 Run `cargo init --name gband` in the repository root and verify that `Cargo.toml` declares edition 2024 and that `src/main.rs` exists
- [ ] 1.2 Reduce `src/main.rs` to an empty `fn main() {}` with no comments, and verify that `cargo run` exits with status 0 and prints nothing
- [ ] 1.3 Add `/target` to `.gitignore` and verify that `git status --porcelain` lists nothing under `target/` after a build

## 2. Dependencies

- [ ] 2.1 Run the `cargo add` commands from the request, in order, and verify that `Cargo.toml` lists all 20 dependencies with the features in the design's Context table
- [ ] 2.2 Set `rust-version` in `Cargo.toml` to the highest `rust-version` among the direct dependencies, and verify that `cargo build` succeeds
- [ ] 2.3 Run `cargo tree -d -e normal` and verify that `crossterm`, `ratatui` and `vt100` each appear in only one version
- [ ] 2.4 Run `cargo clippy --all-targets -- -D warnings` and verify that it exits with status 0

## 3. Documentation

- [ ] 3.1 Add the C compiler requirement for the vendored Lua to the README's Building section, and verify that `cargo build --release` produces `target/release/gband` as the README says
