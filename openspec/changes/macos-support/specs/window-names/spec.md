## MODIFIED Requirements

### Requirement: Foreground command
A window's foreground command SHALL be the base name of the first argument of the foreground process group leader of the window's terminal, with a leading `-` removed, as a login shell's first argument begins with one. On Linux the server SHALL read the argument from `/proc/<pid>/cmdline`, and SHALL use `/proc/<pid>/comm` when the argument is empty. On macOS the server SHALL read the argument from the kernel's argument record of the process, and SHALL use the process's command name, as `ps -o comm` shows it, when the argument is empty. When neither can be read, the foreground command SHALL be the base name of the command that started the window's program.

The server SHALL check every window's foreground process group after each output of the window's program and at least once per second, so that the automatic name follows the foreground command within one second.

#### Scenario: Command in the foreground
- **WHEN** a window runs bash and the user runs `/usr/bin/sleep 100`
- **THEN** within one second the window's automatic name is `sleep`
- **AND** once `sleep` exits, within one second the automatic name is `bash`

#### Scenario: Login shell
- **WHEN** a window's program is a login shell whose first argument is `-zsh`
- **THEN** the window's automatic name is `zsh`

#### Scenario: Command that writes nothing
- **WHEN** a window runs bash and the user runs `sleep 100`, which writes no output
- **THEN** within one second the window's automatic name is `sleep`

#### Scenario: Command in the foreground on macOS
- **WHEN** gband runs on macOS, a window runs zsh and the user runs `/bin/sleep 100`
- **THEN** within one second the window's automatic name is `sleep`
- **AND** once `sleep` exits, within one second the automatic name is `zsh`
