## MODIFIED Requirements

### Requirement: Jump mode tables
gband SHALL bundle the client module `gband.jump`. Its `setup(table)` SHALL declare the table `agents` a mode with the label `agents`, and the table `bands` a mode with the label `bands`, with `gband.keymap.mode`, SHALL make the bindings of "Agents mode" and "Bands mode", in their order, and SHALL bind `a` and `b` in the key table named `table`, or in `prefix` when `table` is nil, to enter agents mode and bands mode. Calling it again SHALL leave both modes as one call does. The modal and the direct presets SHALL call it, as the client-attach capability's "Key bindings" defines. A key bound in either mode SHALL run its binding, and any other key SHALL be discarded with the mode left active, as the client-attach capability defines for a mode. While either mode is active, the sidebar's mode letter SHALL be `A` or `B`, as the sidebar capability's "Mode letter" defines.

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

#### Scenario: Bands mode from direct navigation mode
- **WHEN** the direct key style is in use, the layout holds bands 1, 2 and 3, the client views band 1, and the user presses Ctrl+Space, Space, `b`, then `3`
- **THEN** the client views band 3 and bands mode stays active
- **AND** after Escape, `root` is active
