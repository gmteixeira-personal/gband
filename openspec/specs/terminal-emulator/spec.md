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
