## Purpose

Defines how the server finds the coding agents that run in windows and decides each one's state from its title and screen, how it publishes that state to every client, and the client module that lists and focuses agent windows.

## ADDED Requirements

### Requirement: Agent windows
A window SHALL be an agent window while the leader of its terminal's foreground process group runs a known agent. This version SHALL know one agent:

| agent | name | leader |
|---|---|---|
| Claude Code | `claude` | its first argument's base name is `claude`; or its first argument's base name is `node` or `bun` and its second argument contains `@anthropic-ai/claude-code/` |

On Linux the server SHALL read the leader's arguments from `/proc/<pid>/cmdline`. When it cannot read them, the leader SHALL run Claude Code when the window's foreground command, as the window-names capability defines, is `claude`. The server SHALL check which agent a window runs at the times the window-names capability's "Foreground command" checks the foreground process group: after each output of the window's program, and at least once per second.

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

### Requirement: Agent states
Every agent window SHALL have one of these states:

| state | meaning |
|---|---|
| `blocked` | the agent shows a form that waits for the user's answer |
| `working` | the agent runs a turn |
| `done` | the agent became idle after working or blocked, and no client has looked at the window since, as "Done until seen" defines |
| `idle` | the agent shows its prompt |
| `unknown` | the agent runs and no rule of "Claude Code rules" matches |

The rules of the window's agent SHALL give one of `blocked`, `working`, `idle` and `unknown` from the window's title and screen, called the screen state. The window's state SHALL be its screen state, except that `done` replaces `idle` as "Done until seen" defines. A rule SHALL read the window's title, as the window-names capability's "Window title" defines, and the bottom non-empty lines of the screen the window shows: counted up from the screen's last row, a row holding only spaces left out. Text SHALL be compared with ASCII letters matched without regard to case, and every other character matched exactly.

#### Scenario: State of a new agent window
- **WHEN** a window starts Claude Code and its screen matches none of the rules
- **THEN** the window's state is `unknown`

#### Scenario: Case of the text
- **WHEN** an agent window running Claude Code shows `ESC TO INTERRUPT` on its last row, and its title begins with no spinner character
- **THEN** the window's state is `working`

### Requirement: Claude Code rules
The screen state of a window running Claude Code SHALL be the state of the first rule of this table that matches, in order:

| order | screen state | the rule matches when |
|---|---|---|
| 1 | `working` | the title begins with a character from U+2800 to U+28FF or from U+25D0 to U+25D3, followed by a space |
| 2 | `blocked` | one of the bottom 20 non-empty lines contains `esc to cancel`, and one of them contains `do you want to proceed?`, `enter to confirm` or `enter to select` |
| 3 | `working` | one of the bottom 12 non-empty lines contains `esc to interrupt`, or begins, after any spaces, with one of `·`, `✢`, `✳`, `✶`, `✻` and `✽` followed by a space, and contains `…`, U+2026 |
| 4 | `idle` | one of the bottom 12 non-empty lines begins with `❯`, U+276F, after any spaces |
| 5 | `unknown` | no rule above matches |

#### Scenario: Spinner in the title
- **WHEN** Claude Code sets its window's title to `⠋ Fix the parser`
- **THEN** the window's screen state is `working`, whatever its screen shows

#### Scenario: Permission form
- **WHEN** the title of a window running Claude Code begins with `✳`, and its bottom lines hold `Do you want to proceed?`, `❯ 1. Yes`, `2. No` and `Esc to cancel`
- **THEN** the window's screen state is `blocked`

#### Scenario: Turn without a spinner title
- **WHEN** the title of a window running Claude Code is `claude`, and its bottom lines hold `✻ Whirlpooling… (14s · ↓ 2.8k tokens · thinking)` above a prompt line `❯ `
- **THEN** the window's screen state is `working`

#### Scenario: Finished turn line
- **WHEN** the title of a window running Claude Code is `✳ Claude Code`, and its bottom lines hold `✻ Crunched for 15s` above a prompt line `❯ `
- **THEN** the window's screen state is `idle`

#### Scenario: Prompt shown
- **WHEN** the title of a window running Claude Code is `✳ Claude Code`, and one of its bottom lines is `❯ `
- **THEN** the window's screen state is `idle`

#### Scenario: Form above the bottom lines
- **WHEN** a window running Claude Code shows `Do you want to proceed?` and `Esc to cancel` 25 non-empty lines above its last row, and `❯ ` on its last non-empty line
- **THEN** the window's screen state is `idle`

### Requirement: Done until seen
When an agent window's screen state becomes `idle` and its state before was `working`, `blocked` or `done`, its state SHALL become `done`, unless a client of the session focuses the window at that moment. A client SHALL focus a window when the last focus message it sent named that window, as the client-attach capability's "Report the focused window" defines. A window in `done` SHALL change to `idle` when a client of its session focuses it, or sends it a key, a paste or a mouse event. A screen state other than `idle` SHALL replace `done` at once. A screen state of `idle` reached from `unknown` SHALL give `idle`. `done` SHALL last while no client is attached.

#### Scenario: Agent finishes in the background
- **WHEN** the only client focuses window 1, and Claude Code in window 2 finishes its turn and shows its prompt
- **THEN** window 2's state is `done`

#### Scenario: Focus clears done
- **WHEN** window 2's state is `done` and the client focuses window 2
- **THEN** window 2's state is `idle`

#### Scenario: Agent finishes in the focused window
- **WHEN** a client focuses window 2, and Claude Code in window 2 finishes its turn and shows its prompt
- **THEN** window 2's state is `idle`

#### Scenario: Another client looks
- **WHEN** two clients are attached, window 2's state is `done`, and the second client focuses window 2
- **THEN** window 2's state is `idle` for both clients

#### Scenario: Done across a detach
- **WHEN** the only client detaches, Claude Code in window 2 finishes its turn, and a client attaches and focuses window 1
- **THEN** window 2's state is `done`

#### Scenario: Typed into
- **WHEN** window 2's state is `done`, and a binding sends window 2 a paste while the client focuses window 1
- **THEN** window 2's state is `idle`

### Requirement: Detection timing
The server SHALL apply the rules again after each change of an agent window's screen or title, so that its state follows the screen within one second. A change from `working` or `blocked` to `idle` or `done` SHALL wait until the screen state has been `idle` for 500 ms without a break. Every other change of state SHALL follow the screen state without waiting.

#### Scenario: Spinner pauses
- **WHEN** a window running Claude Code is `working`, and its title loses its spinner character and shows its prompt for 200 ms before the spinner comes back
- **THEN** the window's state stays `working`

#### Scenario: Turn ends
- **WHEN** a window running Claude Code is `working`, and its title loses its spinner character and its prompt stays shown
- **THEN** the window's state becomes `done` or `idle` within one second, and not within 500 ms

#### Scenario: Form appears
- **WHEN** a window running Claude Code is `working` and shows a permission form
- **THEN** the window's state becomes `blocked` within one second

### Requirement: Agent state key
The server SHALL publish an agent window's agent and state in the window state key `agent_state`, as the server-runtime capability's "Shared window state" defines, as a table holding `agent`, the agent's name, and `state`, the window's state. It SHALL set the key when the window becomes an agent window, change it on each change of the state, and remove it within one second after the window stops being an agent window. Only the server's agent detection SHALL set the key. The key SHALL reach every client of the session as any window state does, and each change SHALL emit `WindowStateChanged` with `key` `agent_state` in each client, as the plugin-bridge capability defines.

#### Scenario: Key while working
- **WHEN** window 2 runs Claude Code and its state is `working`
- **THEN** `gband.window_state(2).agent_state` in a client is `{ agent = "claude", state = "working" }`

#### Scenario: Change event
- **WHEN** window 2 runs Claude Code, its state changes from `working` to `blocked`, and a client is attached
- **THEN** the client's `WindowStateChanged` runs with `window` 2, `key` `agent_state`, `value` `{ agent = "claude", state = "blocked" }` and `previous` `{ agent = "claude", state = "working" }`

#### Scenario: Key removed
- **WHEN** Claude Code exits in window 2
- **THEN** within one second `gband.window_state(2).agent_state` in a client is nil

#### Scenario: State at attach
- **WHEN** window 2's state is `blocked` and a client attaches
- **THEN** `gband.window_state(2).agent_state.state` reads `blocked` in the client's `Attached` handler

### Requirement: Agent list in the client
gband SHALL bundle the client module `gband.agents`. Its functions SHALL read the client's layout and the window states it received:

| function | result |
|---|---|
| `list()` | an array with one entry for each window of the client's layout whose state holds `agent_state`, each a table holding `window`, `band`, the band's id, `agent` and `state` |
| `focus(n)` | focuses the window of entry `n` of `list()`, as `gband.window.focus` does, and returns `true`; returns `false` and focuses nothing when there is no entry `n` |
| `step(delta)` | with `delta` 1, focuses the window of the entry after the entry of the focused window, and with `delta` −1 the entry before it, wrapping from the last entry to the first and from the first to the last; returns the focused window, or nil when the list is empty |

`list()` SHALL order its entries by layout: bands in layout order, and in each band its tiled windows column by column from the left, each column from the top, then its floating windows in the order `gband.layout()` gives them. When the focused window has no entry, `step(1)` SHALL focus the first entry and `step(-1)` the last. A `focus` argument that is not a positive integer, or a `step` argument other than 1 and −1, SHALL be an error at the line of the call, and SHALL focus nothing.

#### Scenario: Agents in layout order
- **WHEN** band 1 holds columns with windows 3 and 1, both running Claude Code, and band 2 holds window 2 running Claude Code
- **THEN** `gband.agents.list()` holds entries for windows 3, 1 and 2, in that order

#### Scenario: Plain windows left out
- **WHEN** band 1 holds window 1 running bash and window 2 running Claude Code
- **THEN** `gband.agents.list()` holds one entry, for window 2

#### Scenario: Focus by position
- **WHEN** `gband.agents.list()` holds entries for windows 3 and 2, window 2 is in band 2, and the client views band 1
- **THEN** `gband.agents.focus(2)` returns `true`, and the client views band 2 with window 2 focused

#### Scenario: No such position
- **WHEN** `gband.agents.list()` holds two entries
- **THEN** `gband.agents.focus(5)` returns `false` and the focus is unchanged

#### Scenario: Step wraps
- **WHEN** `gband.agents.list()` holds entries for windows 3, 1 and 2, and the client focuses window 2
- **THEN** `gband.agents.step(1)` focuses window 3 and returns 3

#### Scenario: Step from a plain window
- **WHEN** `gband.agents.list()` holds entries for windows 3 and 2, and the client focuses window 1, which runs bash
- **THEN** `gband.agents.step(-1)` focuses window 2

#### Scenario: Bad step
- **WHEN** a binding function calls `gband.agents.step(2)` on line 4 of `user/init.lua`
- **THEN** an error at that line is reported and the focus is unchanged
