## ADDED Requirements

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
