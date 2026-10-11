# jump-modes Specification

## Purpose
Defines agents mode and bands mode: two key modes, entered after the prefix key, that reach an agent window or a band by its sidebar position with a digit, or step through them with `k`, `j`, Up and Down.

## Requirements

### Requirement: Jump mode tables
gband SHALL bundle the client module `gband.jump`. Its `setup()` SHALL declare the table `agents` a mode with the label `agents`, and the table `bands` a mode with the label `bands`, with `gband.keymap.mode`, and SHALL make the bindings of "Agents mode" and "Bands mode", in their order. The modal and the direct presets SHALL call it, as the client-attach capability's "Key bindings" defines. A key bound in either mode SHALL run its binding, and any other key SHALL be discarded with the mode left active, as the client-attach capability defines for a mode. While either mode is active, the sidebar's mode letter SHALL be `A` or `B`, as the sidebar capability's "Mode letter" defines.

#### Scenario: Labels
- **WHEN** the default configuration is in use
- **THEN** `gband.keymap.label("agents")` returns `agents` and `gband.keymap.label("bands")` returns `bands`

#### Scenario: Mode letter
- **WHEN** the default configuration is in use and the user presses Ctrl+Space then `a`
- **THEN** row 0 of the sidebar shows `A`
- **AND** after Escape it shows `I`

#### Scenario: Unbound key discarded
- **WHEN** agents mode is active and the user presses `x`
- **THEN** no window receives `x` and agents mode stays active

### Requirement: Agents mode
The `agents` table SHALL hold these bindings, each a Lua function with the description its row gives:

| key | Lua binding | does | description |
|---|---|---|---|
| `1` to `9` | `agents 1` to `agents 9` | focus the agent at that position, as `gband.agents.focus` does, then return to interactive mode; when there is no agent at that position, change nothing and keep agents mode active | `focus agent 1` to `focus agent 9` |
| `k` | `agents k` | focus the previous agent, as `gband.agents.step(-1)` does | `focus the previous agent` |
| Up | `agents up` | focus the previous agent, as `gband.agents.step(-1)` does | `focus the previous agent` |
| `j` | `agents j` | focus the next agent, as `gband.agents.step(1)` does | `focus the next agent` |
| Down | `agents down` | focus the next agent, as `gband.agents.step(1)` does | `focus the next agent` |
| `b` | `agents b` | enter bands mode | `bands mode` |
| Escape | `agents escape` | return to interactive mode | `interactive mode` |
| Enter | `agents enter` | return to interactive mode | `interactive mode` |

`k`, `j`, Up and Down SHALL keep agents mode active. The position of an agent SHALL be its position in `gband.agents.list()`, which is the label the sidebar shows for it.

#### Scenario: Jump to an agent
- **WHEN** `gband.agents.list()` holds entries for windows 3 and 2, the client focuses window 1, and the user presses Ctrl+Space, `a`, then `2`
- **THEN** window 2 is focused, `root` is the active table, and no window receives a key

#### Scenario: Jump with the direct style
- **WHEN** the direct key style is in use, `gband.agents.list()` holds entries for windows 3 and 2, and the user presses Ctrl+Space, `a`, then `1`
- **THEN** window 3 is focused and `root` is the active table

#### Scenario: Position past the list
- **WHEN** `gband.agents.list()` holds two entries and the user presses Ctrl+Space, `a`, then `5`
- **THEN** the focus is unchanged and agents mode stays active

#### Scenario: Step down twice
- **WHEN** `gband.agents.list()` holds entries for windows 3, 1 and 2, the client focuses window 3, and the user presses Ctrl+Space, `a`, Down, Down
- **THEN** window 2 is focused and agents mode stays active
- **AND** after Enter, `root` is the active table and window 2 stays focused

#### Scenario: Step with j and k
- **WHEN** `gband.agents.list()` holds entries for windows 3, 1 and 2, the client focuses window 3, and the user presses Ctrl+Space, `a`, `j`, `j`, then `k`
- **THEN** window 1 is focused and agents mode stays active

#### Scenario: Up with no agent focused
- **WHEN** `gband.agents.list()` holds entries for windows 3 and 2, the client focuses window 1, which runs bash, and the user presses Ctrl+Space, `a`, then Up
- **THEN** window 2 is focused

#### Scenario: Escape keeps the new focus
- **WHEN** `gband.agents.list()` holds entries for windows 3, 1 and 2, the client focuses window 3, and the user presses Ctrl+Space, `a`, Down, then Escape
- **THEN** window 1 stays focused and `root` is the active table

### Requirement: Bands mode
The `bands` table SHALL hold these bindings, each with the description its row gives. A row whose `does` names an action SHALL be bound to that action, and every other row SHALL be a Lua function:

| key | Lua binding | does | description |
|---|---|---|---|
| `1` to `9` | `bands 1` to `bands 9` | view the band at that position of `gband.layout().bands`, as `gband.band.view` does; when there is no band at that position, change nothing | `view band 1` to `view band 9` |
| `k` | `bands k` | the `focus_band_up` action | `view the band above` |
| Up | `bands up` | the `focus_band_up` action | `view the band above` |
| `j` | `bands j` | the `focus_band_down` action | `view the band below` |
| Down | `bands down` | the `focus_band_down` action | `view the band below` |
| `a` | `bands a` | enter agents mode | `agents mode` |
| Escape | `bands escape` | return to interactive mode | `interactive mode` |
| Enter | `bands enter` | return to interactive mode | `interactive mode` |

Only Escape and Enter SHALL leave bands mode. The digits, `k`, `j`, Up and Down SHALL keep it active, so the user can view band after band in one visit. `k`, `j`, Up and Down SHALL follow the `loop_bands` setting as those actions do. The position of a band SHALL be its label in the sidebar.

#### Scenario: Jump to a band
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 1, and the user presses Ctrl+Space, `b`, then `3`
- **THEN** the client views band 3 and bands mode stays active
- **AND** after `2` the client views band 2, and after Escape `root` is the active table and band 2 is still viewed

#### Scenario: Band past the layout
- **WHEN** the layout holds bands 1 and 2, the client views band 1, and the user presses Ctrl+Space, `b`, then `7`
- **THEN** the client still views band 1 and bands mode stays active

#### Scenario: Step through bands
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 1, and the user presses Ctrl+Space, `b`, Down, Down
- **THEN** the client views band 3 and bands mode stays active
- **AND** after Up it views band 2

#### Scenario: Switch between the modes
- **WHEN** agents mode is active and the user presses `b`
- **THEN** bands mode is active and the sidebar's mode letter is `B`
- **AND** after `a` agents mode is active and the mode letter is `A`

#### Scenario: Step bands with j and k
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 1, and the user presses Ctrl+Space, `b`, `j`, `j`, then `k`
- **THEN** the client views band 2 and bands mode stays active
