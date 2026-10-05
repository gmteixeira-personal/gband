## Why

`gband` has five subcommands and three global options, and none of them complete in the shell. Users of fish, bash and zsh expect a binary to print its own completion script and, ideally, to put it where the shell loads it, so they do not have to look up each shell's completion directory by hand.

## What Changes

- New subcommand `gband completions <shell>` prints the completion script for `<shell>` to standard output and exits with status 0. `<shell>` is one of `fish`, `bash` or `zsh`.
- New subcommand `gband install-completions <shell>` writes the same script to the per-user directory that shell loads completions from, creating missing parent directories and replacing an existing file, then prints the path it wrote on one line.
  - fish: `$XDG_CONFIG_HOME/fish/completions/gband.fish`, `~/.config` when unset.
  - bash: `$BASH_COMPLETION_USER_DIR/completions/gband` when set, else `$XDG_DATA_HOME/bash-completion/completions/gband`, `~/.local/share` when unset.
  - zsh: `$XDG_DATA_HOME/zsh/site-functions/_gband`, `~/.local/share` when unset. zsh has no per-user directory it searches by default, so the subcommand also prints a line naming the `fpath` entry the user must add before `compinit`.
- Neither subcommand starts logging, contacts a server, or creates the runtime directory. Neither accepts `-s`, `-S` or `-p`: each is rejected as not applying, with exit status 2.
- An unsupported shell name is an invalid invocation, exit status 2.
- `gband --help` lists both subcommands.
- The completion scripts are static: they complete subcommands, options and shell names, not live session or server names.

## Capabilities

### New Capabilities

- `shell-completions`: the `completions` and `install-completions` subcommands, the shells they support, the install locations, and their exit statuses.

### Modified Capabilities

None. The `command-line` requirements list their own five subcommands and stay true as written; the new subcommands are specified in their own capability, so this change does not edit the requirements the open `default-attach` change also modifies.

## Impact

- `Cargo.toml`: adds `clap_complete` to the workspace and the `gband` package.
- `src/main.rs`: two new subcommands, dispatched before logging and socket resolution.
- `src/completions.rs`, `src/lib.rs`: script generation and install-path resolution.
- `tests/completions.rs`: new end-to-end tests.
- `README.md`: a section on installing completions.
- No protocol, server, client or core change.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- Cargo.toml
- Cargo.lock
- src/main.rs
- src/lib.rs
- src/completions.rs
- tests/completions.rs
- tests/subcommands.rs
- README.md
- openspec/changes/shell-completions/
