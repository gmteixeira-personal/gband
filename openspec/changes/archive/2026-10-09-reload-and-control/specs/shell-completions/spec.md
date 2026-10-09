## MODIFIED Requirements

### Requirement: Print a completion script
The `gband` binary SHALL accept the subcommand `completions <shell>`, where `<shell>` is `fish`, `bash` or `zsh`. It SHALL print to standard output a completion script for that shell, which completes the `gband` subcommands, their options and the `<shell>` values, print nothing to standard error, and exit with status 0. The script SHALL be the same on every run of the same build.

#### Scenario: Fish script
- **WHEN** the user runs `gband completions fish`
- **THEN** standard output holds a fish script made of `complete -c gband` commands that offers `server`, `attach`, `list-sessions`, `kill-session`, `kill-server`, `reload`, `errors`, `eval`, `cmd`, `completions` and `install-completions`
- **AND** `fish --no-execute` accepts it
- **AND** the process exits with status 0

#### Scenario: Bash script
- **WHEN** the user runs `gband completions bash`
- **THEN** standard output holds a bash script that registers a completion function for `gband` with `complete` and offers the same eleven subcommands
- **AND** `bash -n` accepts it
- **AND** the process exits with status 0

#### Scenario: Zsh script
- **WHEN** the user runs `gband completions zsh`
- **THEN** standard output starts with `#compdef gband` and offers the same eleven subcommands
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

