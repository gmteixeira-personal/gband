## Context

`src/main.rs` parses the command line with clap's derive API. `Cli` holds `command: Command` as a required subcommand, so clap rejects a bare `gband` with usage and status 2. The global options `-s`, `-S` and `-p` are declared `global = true` and already parse before or after the subcommand. clap generates `-h` and `--help` on its own; only `--help` is specified and tested today.

## Goals / Non-Goals

**Goals:**
- A bare `gband`, with or without global options, behaves exactly as `gband attach` with those options.
- `-h` is held by the spec and a test.

**Non-Goals:**
- Changing what `attach` does, its error messages, or any other subcommand.
- Treating unknown positional arguments as attach arguments: they stay invalid.

## Decisions

**Optional subcommand defaulted after parsing.** Declare `command: Option<Command>` and resolve it with `cli.command.unwrap_or(Command::Attach)` before any use. Everything downstream (`role`, `ignores_session`, dispatch) keeps matching on `Command` and needs no change. The alternative, clap's `args_conflicts_with_subcommands` plus a flattened `attach` argument group, fits subcommands that carry their own arguments; `attach` has none, so it would add structure without benefit.

**Mark the default in the help.** Change the `attach` about text to say it runs when no subcommand is given, so `--help` documents the behaviour. The usage line clap prints becomes `gband [OPTIONS] [COMMAND]`, which already signals that the command is optional.

**Keep the attach refusal text.** Without a terminal, a bare `gband` prints the existing `gband attach needs a terminal ...` line. Naming `attach` there is accurate, since that is the subcommand that ran.

## Risks / Trade-offs

- [A script relying on bare `gband` exiting 2 now attaches or, without a terminal, exits 1] → No such script exists in the repository; the change is marked breaking in the proposal.
- [A bare `gband` typed inside a gband pane hits the nested-attach refusal] → That is the same refusal `gband attach` gives there, and the intended outcome.
