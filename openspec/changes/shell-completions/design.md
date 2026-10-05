## Context

`src/main.rs` parses a clap derive `Cli` with five subcommands, then always starts logging, resolves the socket and dispatches. Two post-parse checks already reject option combinations clap cannot express: `-S` with `-p`, and `-s` with a subcommand that selects no session (`Command::ignores_session`). `src/paths.rs` resolves the runtime directory with pure functions that take environment values as arguments, so unit tests need no process environment. The requirements are in `specs/shell-completions/spec.md`.

## Goals / Non-Goals

**Goals:**
- Scripts generated from the same `Cli` definition the binary parses, so they cannot drift from the real command line.
- Install-path resolution unit-testable without touching the process environment.
- The open `default-attach` change also edits `src/main.rs`; keep this change's footprint there small so the later merge is a small conflict.

**Non-Goals:**
- Dynamic completion of live session or server names. It needs the binary to answer completion queries at runtime (clap_complete's unstable dynamic engine) and a running server; a later change can add it.
- Shells other than fish, bash and zsh.
- Uninstalling. Removing the one file the install printed is trivial by hand.
- Editing the user's shell rc files, for example adding the zsh `fpath` entry. Writing to a user's dotfiles is intrusive; printing the line to add is enough.

## Decisions

**Generate with `clap_complete` from `Cli::command()` at runtime.** `clap_complete::generate(shell, &mut Cli::command(), "gband", &mut stdout)`. The binary name is fixed to `gband` rather than taken from `argv[0]`, so a script printed by `target/debug/gband` still completes `gband`. Alternative: a build script writing the scripts into the binary. Rejected: it adds a `build.rs` and a second copy of the CLI definition for no gain, as generation is microseconds.

**Own `Shell` value enum with three variants.** `#[derive(ValueEnum)] enum Shell { Fish, Bash, Zsh }`, mapped to `clap_complete::Shell`. Using `clap_complete::Shell` directly would also accept `powershell` and `elvish`, which the spec makes invalid. clap's `ValueEnum` gives the "invalid value ... possible values" error and exit status 2 for free.

**Dispatch before logging.** In `main`, after the existing conflict checks and before `logging::init`, match `Command::Completions` and `Command::InstallCompletions` and return. `Command::role` then has no role for them; it returns `Option<Role>` or the early return makes those arms unreachable, whichever reads cleaner.

**Reject `-s`, `-S` and `-p` with the existing post-parse pattern.** Add a `Command::is_standalone() -> Option<&'static str>` returning the subcommand name, and, when any of the three options is set, emit `ErrorKind::ArgumentConflict` with "the option '<flag>' does not apply to '<subcommand>', which addresses no server". The flag named is the first set of `-s`, `-S`, `-p`. This mirrors `ignores_session` so both checks read alike. Alternative: clap `conflicts_with` on the globals. Rejected: global arguments conflicting with specific subcommands is not expressible cleanly in derive.

**Path resolution as a pure function in `src/completions.rs`.** `install_path(shell, env: impl Fn(&str) -> Option<OsString>) -> Result<PathBuf>`; `main` passes `std::env::var_os`. A value counts only when non-empty and absolute, the XDG Base Directory rule. Unit tests cover every row of the spec's table and the no-home error. `directories::BaseDirs` was considered, but it has no notion of `BASH_COMPLETION_USER_DIR`, maps the config dir differently on macOS, and cannot be fed a fake environment.

**Atomic replace.** Create the parent directories, write to `<file>.<pid>.tmp` in the same directory, then `rename` over the target. A failed write leaves no partial or truncated script, which is what "no file is created" in the unwritable-directory scenario asks. On error after the temporary file exists, remove it.

**`src/completions.rs` owns generation, path resolution and install; `main.rs` only dispatches.** Exposed from `src/lib.rs` like `paths` and `logging`, so unit tests live next to the code.

## Risks / Trade-offs

- [zsh does not search `~/.local/share/zsh/site-functions`] → The install prints the `fpath` line. Users of a framework such as oh-my-zsh can instead redirect `gband completions zsh` wherever their framework expects.
- [Overlap with `default-attach` on `src/main.rs`] → Logic lives in `src/completions.rs` and tests in a new `tests/completions.rs`; `main.rs` gains two enum variants, one check and two dispatch arms. Whichever change merges second resolves a small conflict, and if `default-attach` lands first the `Option<Command>` it introduces needs the same early dispatch.
- [The unwritable-directory test fails when run as root, where permissions do not bind] → Skip that test when the effective UID is 0.
- [`fish`, `bash` or `zsh` missing on the test machine] → The syntax-check tests skip the shell when its binary is not on `PATH`; the content assertions still run. zsh is absent on the current development machine, so its syntax check is verified once by hand or in CI.
- [`clap_complete` output changes across versions] → Tests assert on stable markers (`complete -c gband`, `#compdef gband`, subcommand names) and on install output equalling print output, never on whole-script snapshots.
