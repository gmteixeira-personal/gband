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

### Requirement: Graphics passthrough collection
An emulator SHALL collect the complete kitty graphics APC sequences that appear in the output it processes: each run `ESC _ G <control> ; <payload> ESC \` whose APC string begins with `G`. It SHALL end a sequence at the first `ESC \`, and SHALL hold a sequence whose bytes span several `process` calls until its end arrives. It SHALL report the collected sequences in the order they appeared and clear them when read. Collecting a sequence SHALL leave the screen, the cursor and the input modes unchanged. A test's emulator and a client's emulator MAY collect them too, and the collected sequences SHALL not appear in a snapshot or a diff.

An emulator SHALL ignore an APC string whose first byte is not `G`, and SHALL discard a graphics sequence that has not ended within 16 MiB of buffered bytes.

#### Scenario: A transmit sequence is collected
- **WHEN** an emulator processes `\x1b_Gf=24,s=1,v=1,a=T;AAAA\x1b\\`
- **THEN** it reports one collected graphics sequence holding those bytes
- **AND** its screen is empty

#### Scenario: A sequence split across calls
- **WHEN** an emulator processes `\x1b_Gf=24,s=1,v=1,` and then `a=T;AAAA\x1b\\`
- **THEN** it reports one collected graphics sequence holding the joined bytes

#### Scenario: Collected sequences are cleared when read
- **WHEN** an emulator processes one graphics sequence and its collected sequences are read twice
- **THEN** the first read reports the sequence and the second reports none

#### Scenario: A non-graphics APC string is ignored
- **WHEN** an emulator processes `\x1b_qMODE\x1b\\`
- **THEN** it reports no collected graphics sequence

### Requirement: Sixel passthrough collection
An emulator SHALL collect the complete sixel device control strings that appear in the output it processes: each run `ESC P <params> q <payload> ESC \` whose parameter bytes are only decimal digits and `;` and whose final byte before the payload is `q`. It SHALL end a sequence at the first `ESC \`, and SHALL hold a sequence whose bytes span several `process` calls until its end arrives. With each collected sequence it SHALL report the position of the cursor, 1-based from the top-left of the screen, as it stood when the sequence began. It SHALL report the collected sequences in the order they appeared and clear them when read. Collecting a sequence SHALL leave the screen, the cursor and the input modes unchanged, and the collected sequences SHALL not appear in a snapshot or a diff.

An emulator SHALL ignore a device control string that is not a sixel, including one holding an intermediate byte such as `$` before its final byte, and SHALL discard a sixel sequence that has not ended within 16 MiB of buffered bytes.

#### Scenario: A sixel sequence is collected with its position
- **WHEN** an emulator processes `\x1b[3;5H` and then `\x1bPq#0;2;0;0;0#0~~\x1b\\`
- **THEN** it reports one collected sixel sequence holding the sixel bytes, with the position row 3, column 5
- **AND** its screen shows nothing from the sixel

#### Scenario: A sequence split across calls
- **WHEN** an emulator processes `\x1bPq#0;2;0;0;0` and then `#0~~\x1b\\`
- **THEN** it reports one collected sixel sequence holding the joined bytes

#### Scenario: Collected sequences are cleared when read
- **WHEN** an emulator processes one sixel sequence and its collected sequences are read twice
- **THEN** the first read reports the sequence and the second reports none

#### Scenario: A non-sixel device control string is ignored
- **WHEN** an emulator processes `\x1bP$qm\x1b\\`
- **THEN** it reports no collected sixel sequence

### Requirement: Cell size replies
An emulator SHALL hold an optional cell size in pixels, unset when it is created. While it is set to a width and a height, the emulator SHALL answer `\e[16t` with `\e[6;<height>;<width>t`, and `\e[14t` with `\e[4;<rows times height>;<columns times width>t` for its own columns and rows. While it is unset, it SHALL answer neither. Answering SHALL leave the screen, the cursor and the input modes unchanged.

#### Scenario: Cell size set
- **WHEN** an emulator of 80 columns by 24 rows has the cell size 10 by 20 pixels and processes `\e[16t\e[14t`
- **THEN** it replies `\e[6;20;10t\e[4;480;800t`

#### Scenario: Cell size unset
- **WHEN** an emulator with no cell size processes `\e[16t\e[14t`
- **THEN** it replies nothing

### Requirement: Horizontal and vertical position
The emulator SHALL move the cursor on HVP, `\e[<row>;<col>f`, exactly as it does on CUP, `\e[<row>;<col>H`. Row and column are 1-based. An omitted or zero parameter SHALL default to 1, and the position SHALL be clamped to the screen. HVP and CUP are the same movement in ECMA-48, and a program SHALL be free to position every cell with either.

#### Scenario: HVP positions like CUP
- **WHEN** one emulator processes `\e[2;5fX` and a second of the same size processes `\e[2;5HX`
- **THEN** both show `X` at row 2, column 5, and their cursors are one column past it

#### Scenario: Omitted parameters default to 1
- **WHEN** an emulator at its home position processes `\e[10;10f` then `\e[fTOP`
- **THEN** `TOP` appears at row 1, starting at column 1

#### Scenario: A screen positioned only with HVP
- **WHEN** an emulator processes `\e[1;1fa\e[3;1fb\e[2;1fc`
- **THEN** row 1 column 1 shows `a`, row 2 column 1 shows `c`, and row 3 column 1 shows `b`
