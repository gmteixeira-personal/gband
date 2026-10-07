## ADDED Requirements

### Requirement: Modify other keys level
Every grid SHALL report its program's modifyOtherKeys level: 0, 1 or 2, and 0 at the start. A program SHALL set the level by writing `\e[>4;n m` with `n` 0, 1 or 2. `\e[>4;n m` with any other `n` SHALL leave the level unchanged. `\e[>4m` and `\e[>m` SHALL make it 0. A full reset, `\ec`, SHALL make it 0. Other `\e[>` sequences ending in `m` SHALL leave it unchanged. None of these sequences SHALL change the grid's cells, attributes or cursor, and none SHALL produce bytes to write back. The level SHALL NOT be part of a snapshot, so a grid reproduced from a snapshot SHALL report 0.

#### Scenario: Fish turns it on
- **WHEN** a program writes `\e[>4;1m`
- **THEN** its grid reports modifyOtherKeys level 1

#### Scenario: Level 2
- **WHEN** a program writes `\e[>4;2m`
- **THEN** its grid reports modifyOtherKeys level 2

#### Scenario: Turned off
- **WHEN** a program writes `\e[>4;1m` and then `\e[>4;0m`
- **THEN** its grid reports modifyOtherKeys level 0

#### Scenario: Reset without a value
- **WHEN** a program writes `\e[>4;2m` and then `\e[>4m`
- **THEN** its grid reports modifyOtherKeys level 0

#### Scenario: Full reset
- **WHEN** a program writes `\e[>4;1m` and then `\ec`
- **THEN** its grid reports modifyOtherKeys level 0

#### Scenario: Unknown level is ignored
- **WHEN** a program writes `\e[>4;1m` and then `\e[>4;7m`
- **THEN** its grid reports modifyOtherKeys level 1

#### Scenario: Attributes untouched
- **WHEN** a program writes `\e[1m`, `\e[>4;1m` and then `x`
- **THEN** `x` is drawn bold and the grid reports modifyOtherKeys level 1
