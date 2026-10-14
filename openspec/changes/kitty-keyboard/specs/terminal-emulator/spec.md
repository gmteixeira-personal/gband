## ADDED Requirements

### Requirement: Kitty keyboard flags
Every grid SHALL report its program's kitty keyboard flags, and 0 at the start. The main screen and the alternate screen SHALL each keep a base value and a stack of at most 16 pushed values, and every sequence below SHALL act on the screen in use. A screen's flags SHALL be its top pushed value, or its base value when nothing is pushed. gband supports only the disambiguate bit, 1, so every value a program sets or pushes SHALL be kept with its other bits cleared.

- `\e[>f u` SHALL push `f`, or 0 when `f` is omitted. A push onto a full stack SHALL drop the oldest pushed value.
- `\e[<n u` SHALL remove the top `n` pushed values, or one when `n` is omitted or 0. A pop that leaves nothing pushed SHALL make the base value 0.
- `\e[=f;m u` SHALL change the top pushed value, or the base value when nothing is pushed: to `f` when `m` is 1 or omitted, to its bits or `f` when `m` is 2, and to its bits without `f` when `m` is 3. Any other `m` SHALL change nothing.
- `\e[?u` SHALL produce `\e[?` and the screen's flags in decimal and `u` as bytes to write back.

Entering the alternate screen SHALL make the alternate screen's base value 0 and empty its stack. A full reset, `\ec`, SHALL do the same for both screens. `\e[u` without a `>`, `<`, `=` or `?` SHALL leave the flags unchanged. None of these sequences SHALL change the grid's cells, attributes or cursor, and only `\e[?u` SHALL produce bytes to write back. The flags SHALL NOT be part of a snapshot, so a grid reproduced from a snapshot SHALL report 0.

#### Scenario: Query before any push
- **WHEN** a program writes `\e[?u`
- **THEN** the emulator has `\e[?0u` to write back

#### Scenario: Claude Code pushes the disambiguate flag
- **WHEN** a program writes `\e[>1u` and then `\e[?u`
- **THEN** its grid reports kitty keyboard flags 1 and the emulator has `\e[?1u` to write back

#### Scenario: Unsupported flags cleared
- **WHEN** a program writes `\e[>31u` and then `\e[?u`
- **THEN** its grid reports kitty keyboard flags 1 and the emulator has `\e[?1u` to write back

#### Scenario: Pop restores the value below
- **WHEN** a program writes `\e[>1u`, `\e[>0u` and then `\e[<u`
- **THEN** its grid reports kitty keyboard flags 1

#### Scenario: Popping everything resets
- **WHEN** a program writes `\e[=1u`, `\e[>1u` and then `\e[<5u`
- **THEN** its grid reports kitty keyboard flags 0

#### Scenario: Set and clear bits
- **WHEN** a program writes `\e[=1;2u` and then `\e[=1;3u`
- **THEN** its grid reports kitty keyboard flags 0

#### Scenario: Full stack drops the oldest value
- **WHEN** a program writes `\e[>1u`, then `\e[>0u` sixteen times, then `\e[<u` fifteen times
- **THEN** its grid reports kitty keyboard flags 0

#### Scenario: Screens keep their own flags
- **WHEN** a program writes `\e[>1u` and then `\e[?1049h`
- **THEN** its grid reports kitty keyboard flags 0, and 1 again after the program writes `\e[?1049l`

#### Scenario: A new alternate screen starts clear
- **WHEN** a program writes `\e[?1049h\e[>1u\e[?1049l` and then `\e[?1049h`
- **THEN** its grid reports kitty keyboard flags 0

#### Scenario: Full reset clears the flags
- **WHEN** a program writes `\e[>1u` and then `\ec`
- **THEN** its grid reports kitty keyboard flags 0

#### Scenario: Flags left out of a snapshot
- **WHEN** a program writes `\e[>1u` and a grid is reproduced from its grid's snapshot
- **THEN** the reproduced grid reports kitty keyboard flags 0

#### Scenario: Restore cursor leaves the flags
- **WHEN** a program writes `\e[>1u` and then `\e[u`
- **THEN** its grid reports kitty keyboard flags 1 and the emulator has no bytes to write back
