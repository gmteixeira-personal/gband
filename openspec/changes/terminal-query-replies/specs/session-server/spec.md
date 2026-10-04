## ADDED Requirements

### Requirement: Terminal query replies
The server SHALL answer the terminal queries in the table below when the pane's program writes them to its PTY. It SHALL write each reply to that PTY after the grid has processed every byte the program wrote before the query. It SHALL answer whether or not a client is attached, and SHALL NOT send replies to clients. Replies and client input SHALL reach the PTY as whole sequences, never interleaved within one another. Positions are 1-based, counted from the top-left cell of the server's own screen of the pane. `<version>` is the package version of the running server.

| query | written by the program | reply |
|---|---|---|
| Primary Device Attributes | `\e[c` or `\e[0c` | `\e[?62;22c` |
| operating status | `\e[5n` | `\e[0n` |
| cursor position | `\e[6n` | `\e[<row>;<column>R`, the cursor's position |
| XTVERSION | `\e[>q` or `\e[>0q` | `\eP>\|gband <version>\e\` |
| DEC private mode report | `\e[?<mode>$p` | `\e[?<mode>;<state>$y` |
| ANSI mode report | `\e[<mode>$p` | `\e[<mode>;0$y` |

In a DEC private mode report, `<state>` SHALL be `1` when the mode is set and `2` when it is reset, for the modes 1, 9, 25, 47, 1000, 1002, 1003, 1005, 1006, 1049 and 2004, read from the server's screen. For any other mode, `<state>` SHALL be `0`, meaning not recognised.

A query the server answers SHALL NOT be recorded in the log as an unhandled sequence. Every other query SHALL stay unanswered.

#### Scenario: fish starts without waiting
- **WHEN** the pane's program is fish 4, which waits for a Primary Device Attributes reply before drawing its prompt
- **THEN** the prompt appears within 1 second of the server starting
- **AND** fish prints no warning about the Primary Device Attributes query

#### Scenario: Device attributes with no client attached
- **WHEN** no client is attached and the pane's program writes `\e[0c`
- **THEN** the program reads `\e[?62;22c` from its terminal

#### Scenario: Cursor position
- **WHEN** the pane's program moves the cursor to row 5, column 10 with `\e[5;10H` and writes `\e[6n`
- **THEN** the program reads `\e[5;10R` from its terminal

#### Scenario: Operating status
- **WHEN** the pane's program writes `\e[5n`
- **THEN** the program reads `\e[0n` from its terminal

#### Scenario: Version
- **WHEN** the pane's program writes `\e[>q`
- **THEN** the program reads `\eP>|gband ` followed by the package version and `\e\` from its terminal

#### Scenario: Mode set
- **WHEN** the pane's program enables bracketed paste with `\e[?2004h` and writes `\e[?2004$p`
- **THEN** the program reads `\e[?2004;1$y` from its terminal

#### Scenario: Mode reset
- **WHEN** the pane's program writes `\e[?1$p` without having enabled application cursor keys
- **THEN** the program reads `\e[?1;2$y` from its terminal

#### Scenario: Mode not recognised
- **WHEN** the pane's program writes `\e[?7727$p`
- **THEN** the program reads `\e[?7727;0$y` from its terminal

#### Scenario: Unsupported query stays unanswered
- **WHEN** the pane's program writes the kitty keyboard query `\e[?u` followed by `\e[c`
- **THEN** the only bytes the program reads from its terminal are `\e[?62;22c`
