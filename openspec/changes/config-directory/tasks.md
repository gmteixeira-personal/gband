## 1. Paths and preparation

- [ ] 1.1 Replace the configuration path in `crates/lua` with `config_dir()`, returning `gband` under an absolute `XDG_CONFIG_HOME`, else `~/.config/gband`, else `None`, plus `user_file()` and `defaults_file()` derived from it. Verify with unit tests for the absolute, relative and unset `XDG_CONFIG_HOME` cases and the derived `user/init.lua` and `defaults/init.lua` paths
- [ ] 1.2 Implement `prepare(dir)` in `crates/lua`: create `dir`, `defaults` and `user` with `create_dir_all`, and write the built-in default text to `defaults/init.lua` through a temporary file in `defaults/` renamed over it, only when the file is missing or differs. Verify with unit tests in a temporary directory for "First run", "Missing user directory", "Edited defaults are restored", "User files are kept", no write when the content already matches (unchanged modification time), and an error returned for a read-only directory

## 2. Loading and reload

- [ ] 2.1 Change `load` to evaluate `user/init.lua` alone when it exists, otherwise the built-in default text named `@<dir>/defaults/init.lua`, each starting from `Options::default()` and no bindings. Verify with unit tests for "User file replaces the defaults", "Copied defaults load unchanged", "Rebind a key", "Unbind a key" and "Prefix changed after binding", and that an error in the default text would name `defaults/init.lua`
- [ ] 2.2 Make the watcher observe `user/` for `init.lua`, falling back to the configuration directory until `user/` exists, and never observe `defaults/`. Verify with tests that writing, renaming over and deleting `user/init.lua` each deliver a reload within one second, and that writing `defaults/init.lua` delivers none within one second

## 3. Processes

- [ ] 3.1 Call `prepare` before `load` for the client and the server in `src/main.rs`, logging a failure with `tracing::warn!` and continuing, and skip preparation and the watcher when `config_dir()` is `None`. Verify that `cargo build` succeeds and that a server started with a fresh scratch `XDG_CONFIG_HOME` leaves `gband/defaults/init.lua` and an empty `gband/user` behind

## 4. End to end

- [ ] 4.1 Update `tests/common/mod.rs` and `tests/config.rs` so configuration tests write `gband/user/init.lua`, and adjust the harness check to expect the prepared layout with no `user/init.lua` unless the test writes one. Verify with `cargo test --test config`
- [ ] 4.2 Add `tests/config.rs` tests for "First run", "Read-only configuration directory", "User file replaces the defaults", "Syntax error" naming `gband/user/init.lua`, "Another prefix key" with a copied defaults file, and "Defaults file edited while running". Verify with `cargo test --test config`
- [ ] 4.3 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`, and check that no comment breaks the rules in `CLAUDE.md`. Verify that both commands pass

## 5. Documentation

- [ ] 5.1 Rewrite the location part of the README "Configuration" section: the `gband/` directory, `defaults/init.lua` written by gband and overwritten when it differs, `user/init.lua` replacing the defaults, and copying `defaults/init.lua` as the way to start. Verify by reading the rendered section
