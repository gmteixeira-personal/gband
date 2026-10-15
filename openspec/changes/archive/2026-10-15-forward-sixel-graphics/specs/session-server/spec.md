## MODIFIED Requirements

### Requirement: Terminal query replies
The server SHALL answer the terminal queries in the table below when the window's program writes them to its PTY. It SHALL write each reply to that PTY after the grid has processed every byte the program wrote before the query. It SHALL answer the queries in the table whether or not a client is attached, and SHALL NOT send replies to clients. Replies and client input SHALL reach the PTY as whole sequences, never interleaved within one another. Positions are 1-based, counted from the top-left cell of the server's own screen of the window. `<version>` is the package version of the running server.

| query | written by the program | reply |
|---|---|---|
| Primary Device Attributes | `\e[c` or `\e[0c` | `\e[?62;22c`, or `\e[?62;4;22c` when an attached client supports sixel |
| operating status | `\e[5n` | `\e[0n` |
| cursor position | `\e[6n` | `\e[<row>;<column>R`, the cursor's position |
| XTVERSION | `\e[>q` or `\e[>0q` | `\eP>\|gband <version>\e\` |
| DEC private mode report | `\e[?<mode>$p` | `\e[?<mode>;<state>$y` |
| ANSI mode report | `\e[<mode>$p` | `\e[<mode>;0$y` |
| cell size | `\e[16t` | `\e[6;<height>;<width>t`, the session's cell size in pixels, only while it is known |
| text area size | `\e[14t` | `\e[4;<height>;<width>t`, the window's PTY pixel size, only while the session's cell size is known |
| kitty keyboard flags | `\e[?u` | `\e[?<flags>u`, the kitty keyboard flags of the screen in use, as the terminal-emulator capability's "Kitty keyboard flags" defines |

In a DEC private mode report, `<state>` SHALL be `1` when the mode is set and `2` when it is reset, for the modes 1, 9, 25, 47, 1000, 1002, 1003, 1005, 1006, 1049 and 2004, read from the server's screen. For any other mode, `<state>` SHALL be `0`, meaning not recognised.

The server SHALL add attribute `4` to the Primary Device Attributes reply, making it `\e[?62;4;22c`, only when at least one client attached to the session reported its terminal supports sixel, as the window-sixel capability's "Offering sixel to the program" defines; otherwise the reply SHALL be `\e[?62;22c`.

The server SHALL answer the cell size and text area size queries only while the session's cell size is known, as the window-sixel capability's "Cell size in pixels" defines; otherwise both SHALL stay unanswered.

The server SHALL also answer a kitty graphics support query, a graphics sequence whose control holds `a=q`, only when an attached client supports kitty graphics, as the window-graphics capability's "Answering the support query" defines. A query the server answers SHALL NOT be recorded in the log as an unhandled sequence. Every other query SHALL stay unanswered.

#### Scenario: fish starts without waiting
- **WHEN** the window's program is fish 4, which waits for a Primary Device Attributes reply before drawing its prompt
- **THEN** the prompt appears within 1 second of the server starting
- **AND** fish prints no warning about the Primary Device Attributes query

#### Scenario: Device attributes with no client attached
- **WHEN** no client is attached and the window's program writes `\e[0c`
- **THEN** the program reads `\e[?62;22c` from its terminal

#### Scenario: Device attributes with a sixel client
- **WHEN** a client whose terminal supports sixel is attached and the window's program writes `\e[c`
- **THEN** the program reads `\e[?62;4;22c` from its terminal

#### Scenario: Cell size with a sixel client
- **WHEN** a client whose terminal supports sixel attached reporting the cell size 10 by 20 pixels, window 1's PTY is 80 columns by 24 rows, and the window's program writes `\e[16t` and then `\e[14t`
- **THEN** the program reads `\e[6;20;10t` and then `\e[4;480;800t` from its terminal

#### Scenario: Cursor position
- **WHEN** the window's program moves the cursor to row 5, column 10 with `\e[5;10H` and writes `\e[6n`
- **THEN** the program reads `\e[5;10R` from its terminal

#### Scenario: Operating status
- **WHEN** the window's program writes `\e[5n`
- **THEN** the program reads `\e[0n` from its terminal

#### Scenario: Version
- **WHEN** the window's program writes `\e[>q`
- **THEN** the program reads `\eP>|gband ` followed by the package version and `\e\` from its terminal

#### Scenario: Mode set
- **WHEN** the window's program enables bracketed paste with `\e[?2004h` and writes `\e[?2004$p`
- **THEN** the program reads `\e[?2004;1$y` from its terminal

#### Scenario: Mode reset
- **WHEN** the window's program writes `\e[?1$p` without having enabled application cursor keys
- **THEN** the program reads `\e[?1;2$y` from its terminal

#### Scenario: Mode not recognised
- **WHEN** the window's program writes `\e[?7727$p`
- **THEN** the program reads `\e[?7727;0$y` from its terminal

#### Scenario: Kitty keyboard query
- **WHEN** the window's program writes `\e[>1u` and then `\e[?u`
- **THEN** the program reads `\e[?1u` from its terminal

#### Scenario: Unsupported query stays unanswered
- **WHEN** the window's program writes the background color query `\e]11;?\e\` followed by `\e[c`
- **THEN** the only bytes the program reads from its terminal are `\e[?62;22c`
