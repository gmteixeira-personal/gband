# shell-completions Specification

## Purpose

Lets users of fish, bash and zsh print `gband`'s shell completion script, or install it in the per-user directory their shell loads completions from.

## Requirements

### Requirement: Print a completion script
The `gband` binary SHALL accept the subcommand `completions <shell>`, where `<shell>` is `fish`, `bash` or `zsh`. It SHALL print to standard output a completion script for that shell, which completes the `gband` subcommands, their options and the `<shell>` values, print nothing to standard error, and exit with status 0. The script SHALL be the same on every run of the same build.

#### Scenario: Fish script
- **WHEN** the user runs `gband completions fish`
- **THEN** standard output holds a fish script made of `complete -c gband` commands that offers `server`, `attach`, `list-sessions`, `kill-session`, `kill-server`, `completions` and `install-completions`
- **AND** `fish --no-execute` accepts it
- **AND** the process exits with status 0

#### Scenario: Bash script
- **WHEN** the user runs `gband completions bash`
- **THEN** standard output holds a bash script that registers a completion function for `gband` with `complete` and offers the same seven subcommands
- **AND** `bash -n` accepts it
- **AND** the process exits with status 0

#### Scenario: Zsh script
- **WHEN** the user runs `gband completions zsh`
- **THEN** standard output starts with `#compdef gband` and offers the same seven subcommands
- **AND** `zsh -n` accepts it
- **AND** the process exits with status 0

#### Scenario: Shell values complete
- **WHEN** the user runs `gband completions fish`
- **THEN** the script offers `fish`, `bash` and `zsh` as values for both `completions` and `install-completions`

#### Scenario: Unsupported shell
- **WHEN** the user runs `gband completions powershell`
- **THEN** standard error names `powershell` as an invalid value and lists `fish`, `bash` and `zsh`
- **AND** the process exits with status 2

#### Scenario: Missing shell
- **WHEN** the user runs `gband completions`
- **THEN** standard error holds the usage
- **AND** the process exits with status 2

### Requirement: Install a completion script
The `gband` binary SHALL accept the subcommand `install-completions <shell>`, where `<shell>` is `fish`, `bash` or `zsh`. It SHALL write the script `gband completions <shell>` prints to the install path below, creating every missing parent directory and replacing a file already at that path. An environment variable counts only when it is set to an absolute path; otherwise its default applies.

| shell | install path |
|---|---|
| fish | `$XDG_CONFIG_HOME/fish/completions/gband.fish`, with `XDG_CONFIG_HOME` defaulting to `$HOME/.config` |
| bash | `$BASH_COMPLETION_USER_DIR/completions/gband` when `BASH_COMPLETION_USER_DIR` is set, else `$XDG_DATA_HOME/bash-completion/completions/gband`, with `XDG_DATA_HOME` defaulting to `$HOME/.local/share` |
| zsh | `$XDG_DATA_HOME/zsh/site-functions/_gband`, with `XDG_DATA_HOME` defaulting to `$HOME/.local/share` |

On success it SHALL print the absolute path it wrote as the first line of standard output and exit with status 0. For zsh, which searches no per-user directory by default, it SHALL print a second line stating that the directory holding the file must be added to `fpath` before `compinit` runs, naming that directory. It SHALL print nothing else to standard output and nothing to standard error.

When the path cannot be determined, because the variables it needs are unset or relative and `HOME` is unset, or when a directory or the file cannot be written, it SHALL print one line to standard error that names the cause, and the path when one was determined, and exit with status 1. An unsupported or missing `<shell>` SHALL be an invalid invocation, as for `completions`.

#### Scenario: Install for fish
- **WHEN** the user runs `gband install-completions fish` with `XDG_CONFIG_HOME=/tmp/c`
- **THEN** `/tmp/c/fish/completions/gband.fish` holds exactly what `gband completions fish` prints
- **AND** standard output is the line `/tmp/c/fish/completions/gband.fish`
- **AND** the process exits with status 0

#### Scenario: Fish default directory
- **WHEN** the user runs `gband install-completions fish` with `HOME=/tmp/h` and `XDG_CONFIG_HOME` unset
- **THEN** the script is written to `/tmp/h/.config/fish/completions/gband.fish`

#### Scenario: Install for bash
- **WHEN** the user runs `gband install-completions bash` with `XDG_DATA_HOME=/tmp/d` and `BASH_COMPLETION_USER_DIR` unset
- **THEN** `/tmp/d/bash-completion/completions/gband` holds exactly what `gband completions bash` prints
- **AND** standard output is the line `/tmp/d/bash-completion/completions/gband`

#### Scenario: Bash user directory override
- **WHEN** the user runs `gband install-completions bash` with `BASH_COMPLETION_USER_DIR=/tmp/b` and `XDG_DATA_HOME=/tmp/d`
- **THEN** the script is written to `/tmp/b/completions/gband`

#### Scenario: Install for zsh
- **WHEN** the user runs `gband install-completions zsh` with `XDG_DATA_HOME=/tmp/d`
- **THEN** `/tmp/d/zsh/site-functions/_gband` holds exactly what `gband completions zsh` prints
- **AND** the first line of standard output is `/tmp/d/zsh/site-functions/_gband`
- **AND** the second line names `/tmp/d/zsh/site-functions`, `fpath` and `compinit`

#### Scenario: Relative variable ignored
- **WHEN** the user runs `gband install-completions fish` with `XDG_CONFIG_HOME=relative` and `HOME=/tmp/h`
- **THEN** the script is written to `/tmp/h/.config/fish/completions/gband.fish`

#### Scenario: Replace an existing file
- **WHEN** `/tmp/c/fish/completions/gband.fish` already holds other text and the user runs `gband install-completions fish` with `XDG_CONFIG_HOME=/tmp/c`
- **THEN** the file holds exactly what `gband completions fish` prints
- **AND** the process exits with status 0

#### Scenario: No home
- **WHEN** the user runs `gband install-completions fish` with `HOME` and `XDG_CONFIG_HOME` unset
- **THEN** standard error holds one line saying the install directory cannot be determined
- **AND** the process exits with status 1

#### Scenario: Unwritable directory
- **WHEN** the user runs `gband install-completions fish` with `XDG_CONFIG_HOME` naming a directory the user cannot write
- **THEN** standard error holds one line naming the path that could not be written
- **AND** the process exits with status 1
- **AND** no file is created

### Requirement: Completion subcommands stand alone
`gband completions` and `gband install-completions` SHALL NOT start logging, create or write a log file, create the gband runtime directory, connect to a server, or start one. When `-s`, `-S` or `-p` is given to either, before or after the subcommand, `gband` SHALL print one line to standard error saying that the option does not apply to that subcommand and exit with status 2, writing no file.

#### Scenario: No log file
- **WHEN** the user runs `gband completions bash` with `XDG_STATE_HOME` naming an empty directory
- **THEN** the directory stays empty
- **AND** no `gband` directory is created in `XDG_RUNTIME_DIR`

#### Scenario: Session option rejected
- **WHEN** the user runs `gband completions fish -s work`
- **THEN** standard error says that `-s` does not apply to `completions`
- **AND** the process exits with status 2

#### Scenario: Server option rejected
- **WHEN** the user runs `gband -S feature install-completions zsh`
- **THEN** standard error says that `-S` does not apply to `install-completions`
- **AND** the process exits with status 2
- **AND** no completion file is written

### Requirement: Help lists the completion subcommands
`gband --help` SHALL list the `completions` and `install-completions` subcommands alongside the subcommands the command-line capability lists.

#### Scenario: Help
- **WHEN** the user runs `gband --help`
- **THEN** standard output lists `completions` and `install-completions`
- **AND** the process exits with status 0
