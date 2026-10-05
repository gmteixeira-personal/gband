## Why

Attaching is what a user does almost every time they run `gband`, yet a bare `gband` prints usage and exits with status 2. Making attach the default subcommand, as `tmux` does with `new-session`, removes a word from the most common invocation.

## What Changes

- **BREAKING** `gband` run with no subcommand runs `gband attach` instead of printing usage and exiting with status 2. The global options still apply, so `gband -s work` attaches to the session `work` and `gband -S feature` attaches to the server named `feature`.
- The help marks `attach` as the subcommand run when none is given.
- `gband -h` is specified alongside `gband --help`: both print the help to standard output and exit with status 0. clap already accepts `-h`; the spec and tests now hold it.
- An unrecognised subcommand or option is still an invalid invocation that prints usage to standard error and exits with status 2.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `command-line`: a missing subcommand runs `attach` rather than being an invalid invocation, and `-h` is specified as a help flag next to `--help`.

## Impact

- `src/main.rs`: the subcommand becomes optional and defaults to `attach`; the `attach` help text names it as the default.
- `tests/subcommands.rs`: the no-subcommand test asserts the attach behaviour; the help test covers `-h`.
- No protocol, server or client library change.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- src/main.rs
- tests/subcommands.rs
- openspec/changes/default-attach/
