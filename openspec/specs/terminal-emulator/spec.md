# terminal-emulator Specification

## Purpose
Defines the contract that every terminal grid in gband meets, on the server, in the client and in tests, so that the emulation library behind the grids can be replaced without changing any other behaviour.

## Requirements

### Requirement: One emulator behind every grid
Every terminal grid in gband SHALL be an instance of a single emulator interface. This covers the server's authoritative grid for each window, the client's grid for each window, and the grids tests use to read a window or a client's terminal. Only the emulator's implementation SHALL depend on the emulation library. Replacing the library SHALL change only that implementation.

#### Scenario: Library confined to the emulator
- **WHEN** the workspace's manifests are searched for the emulation library
- **THEN** only the emulator's own manifest names it

### Requirement: Reproduction from a snapshot
An emulator SHALL produce a snapshot of its screen as terminal output. Fed into a new emulator of the same size, the snapshot SHALL reproduce the visible cells with their contents and attributes, the cursor position and visibility, and the input modes. Those modes are application cursor keys and bracketed paste.

#### Scenario: Coloured text
- **WHEN** an emulator of 80×24 processes `printf '\e[1;31mred\e[0m plain\n'`, and its snapshot is fed into a new 80×24 emulator
- **THEN** both show `red` bold in red followed by ` plain` with default attributes, on the same row, with the cursor at the start of the next row

#### Scenario: Wide characters
- **WHEN** an emulator processes `printf '漢字x'` and its snapshot is fed into a new emulator of the same size
- **THEN** both show the two wide characters in columns 0 to 3 and `x` in column 4

#### Scenario: Alternate screen and modes
- **WHEN** an emulator processes `\e[?1049h\e[?1h\e[?2004h` followed by text, and its snapshot is fed into a new emulator of the same size
- **THEN** both show the text, and both report application cursor keys and bracketed paste as on

### Requirement: Diff from a checkpoint
An emulator SHALL take a checkpoint of its current state. It SHALL then produce a diff from that checkpoint as terminal output. Fed into an emulator that reproduces the checkpoint, the diff SHALL make that emulator reproduce the current screen. A diff SHALL be produced only when the checkpoint has the emulator's current size. Otherwise the caller SHALL send a snapshot. A diff from a checkpoint of the current state SHALL be empty.

#### Scenario: Cursor movement after a checkpoint
- **WHEN** an emulator takes a checkpoint, then processes `printf 'abc\e[2;5Hxy\e[1K'`, and the diff is fed into a copy built from the checkpoint's snapshot
- **THEN** the copy's cells and cursor equal the emulator's

#### Scenario: Nothing changed
- **WHEN** an emulator takes a checkpoint and processes nothing
- **THEN** its diff from that checkpoint is empty

### Requirement: Resize
Resizing an emulator SHALL change its size to the given columns and rows and keep the cells that still fit. An emulator SHALL report its current size.

#### Scenario: Shrink
- **WHEN** an 80×24 emulator showing `hello` on its first row is resized to 40×10
- **THEN** it reports 40×10 and still shows `hello` on its first row

### Requirement: Output for the program
An emulator SHALL collect the bytes that processing a program's output requires it to write back to that program, such as replies to terminal queries, in the order the queries arrived. The server SHALL write a window emulator's collected bytes to that window's PTY, in order with the keys and pastes that clients send. A client's emulator and a test's emulator SHALL discard them. An emulator SHALL NOT produce write-back bytes for output that asks no question.

#### Scenario: Plain output asks nothing
- **WHEN** an emulator processes `printf 'hello\n'`
- **THEN** it has no bytes to write back

### Requirement: Mouse modes
Every grid SHALL report its program's mouse tracking mode and mouse encoding. The tracking mode SHALL be none, press (mode 9), press and release (mode 1000), button motion (mode 1002) or any motion (mode 1003), and SHALL be none at the start. The encoding SHALL be default, UTF-8 (mode 1005) or SGR (mode 1006), and SHALL be default at the start. Setting a tracking mode SHALL replace the one before, and resetting the mode in effect SHALL make it none. Setting an encoding SHALL replace the one before, and resetting the encoding in effect SHALL make it default. A full reset SHALL return both to their start values. A grid reproduced from another's snapshot SHALL report the same mode and encoding.

#### Scenario: Program turns on SGR mouse reporting
- **WHEN** a program writes `\x1b[?1002h\x1b[?1006h`
- **THEN** its grid reports button motion tracking with SGR encoding

#### Scenario: Reset the tracking mode
- **WHEN** a program writes `\x1b[?1000h` and then `\x1b[?1000l`
- **THEN** its grid reports no mouse tracking

#### Scenario: Mode in a snapshot
- **WHEN** a window's program enabled modes 1003 and 1006 and a client attaches
- **THEN** the client's grid of that window reports any motion tracking with SGR encoding

### Requirement: Modify other keys level
Every grid SHALL report its program's modifyOtherKeys level: 0, 1 or 2, and 0 at the start. A program SHALL set the level by writing `\e[>4;n m` with `n` 0, 1 or 2. `\e[>4;n m` with any other `n` SHALL leave the level unchanged. `\e[>4m`, `\e[>m` and `\e[>0m` SHALL make it 0. A full reset, `\ec`, SHALL make it 0. Other `\e[>` sequences ending in `m` SHALL leave it unchanged. None of these sequences SHALL change the grid's cells, attributes or cursor, and none SHALL produce bytes to write back. The level SHALL NOT be part of a snapshot, so a grid reproduced from a snapshot SHALL report 0.

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

#### Scenario: Reset without a resource
- **WHEN** a program writes `\e[>4;1m` and then `\e[>m`
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
