## MODIFIED Requirements

### Requirement: Agent windows
A window SHALL be an agent window while the leader of its terminal's foreground process group runs a known agent. This version SHALL know one agent:

| agent | name | leader |
|---|---|---|
| Claude Code | `claude` | its first argument's base name is `claude`; or its first argument's base name is `node` or `bun` and its second argument contains `@anthropic-ai/claude-code/` |

On Linux the server SHALL read the leader's arguments from `/proc/<pid>/cmdline`, and on macOS from the kernel's argument record of the process, as `ps -o args` shows it. When it cannot read them, the leader SHALL run Claude Code when the window's foreground command, as the window-names capability defines, is `claude`. The server SHALL check which agent a window runs at the times the window-names capability's "Foreground command" checks the foreground process group: after each output of the window's program, and at least once per second.

#### Scenario: Claude Code started from a shell
- **WHEN** a window runs bash and the user runs `claude`
- **THEN** within one second the window is an agent window running `claude`

#### Scenario: Claude Code run by node
- **WHEN** the foreground process group leader of a window's terminal has the arguments `node` and `/usr/lib/node_modules/@anthropic-ai/claude-code/cli.js`
- **THEN** the window is an agent window running `claude`

#### Scenario: Agent exits
- **WHEN** a window runs Claude Code from bash and Claude Code exits
- **THEN** within one second the window is not an agent window

#### Scenario: Other program
- **WHEN** a window runs `vim`
- **THEN** the window is not an agent window

#### Scenario: Claude Code run by node on macOS
- **WHEN** gband runs on macOS and the foreground process group leader of a window's terminal has the arguments `node` and `/opt/homebrew/lib/node_modules/@anthropic-ai/claude-code/cli.js`
- **THEN** the window is an agent window running `claude`
