## MODIFIED Requirements

### Requirement: Spawn a program
`gband.spawn` SHALL take a table whose optional `cmd` field is a string or a list of strings, and whose optional `band` and `after` fields name a band and a pane as the `open_pane` target does in the lua-control capability. Called inside a binding function, it SHALL dispatch open pane naming the program to run: a string as a command line the user's shell runs, a list as the program and its arguments, and no `cmd` as the user's shell. With neither `band` nor `after`, open pane SHALL be resolved as the actions capability defines. With either, it SHALL name the band and the pane to open after as that target does. A `cmd` of any other type, an empty list, a `band` or `after` that the target would refuse, or any other field SHALL be an error.

#### Scenario: Spawn a command line
- **WHEN** `init.lua` binds `alt+n` to `function() gband.spawn({ cmd = "fish" }) end` and the user presses Alt+N
- **THEN** a new pane opens right of the focused pane's column, running `fish`, and is focused

#### Scenario: Spawn an argument list
- **WHEN** a binding function calls `gband.spawn({ cmd = { "htop", "-d", "10" } })`
- **THEN** a new pane runs `htop` with the arguments `-d` and `10`

#### Scenario: Spawn after a named pane
- **WHEN** band 1 holds columns with panes 1 and 2, pane 2 is focused, and a binding function calls `gband.spawn({ cmd = "fish", after = 1 })`
- **THEN** band 1 holds pane 1's column, a new column running `fish`, and pane 2's column, in that order

#### Scenario: Unknown field
- **WHEN** a binding function calls `gband.spawn({ cmd = "fish", width = 1/2 })`
- **THEN** the call raises an error naming `width`
