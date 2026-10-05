## 1. Default subcommand

- [x] 1.1 In `src/main.rs`, make `Cli::command` an `Option<Command>` and resolve it to `Command::Attach` when absent, before the `-s` check, the role lookup and the dispatch; verify `cargo build` succeeds
- [x] 1.2 Change the `attach` subcommand's about text to state that it runs when no subcommand is given; verify `gband --help` shows it

## 2. Tests

- [x] 2.1 In `tests/subcommands.rs`, replace `no_subcommand_prints_usage` with a test that runs `gband` with no arguments and no terminal and asserts the `needs a terminal` one-line failure with status 1 and a client log holding `client started`; verify it passes
- [x] 2.2 Add a test that runs `gband -s 'a/b'` with no subcommand and asserts status 2 with the session-name rule on standard error and no log file, proving global options reach the default `attach`; verify it passes
- [x] 2.3 Add a test that runs `gband --frobnicate` and asserts status 2, `--frobnicate` named on standard error and no log file; verify it passes
- [x] 2.4 Extend the help tests to run both `-h` and `--help`, assert identical subcommand listings, status 0, no log file, and that the `attach` line marks it as the default; verify they pass
- [x] 2.5 Run `cargo test` for the workspace and `cargo clippy --all-targets -- -D warnings`; verify both pass

## 3. Manual check

- [x] 3.1 Build, run a bare `gband` in a terminal, confirm it attaches to session `default`, detach, then run `gband -s work` and confirm it attaches to session `work`
