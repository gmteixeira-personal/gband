## MODIFIED Requirements

### Requirement: Snapshot on attach
When a client completes the handshake, the server SHALL send it the layout with the screen area, then one snapshot for each window in the layout. A window's snapshot SHALL reproduce that window's current screen: its size, every visible cell with its attributes, the cursor position and visibility, and the input modes the grid tracks, including application cursor keys and bracketed paste. The modifyOtherKeys level SHALL NOT be part of the snapshot: the server encodes keys with its own grid, which keeps the level across a detach and a later attach.

#### Scenario: Reattach restores the screen
- **WHEN** the user runs `ls` in a window, kills the client, and runs `gband attach` again
- **THEN** the new client shows the `ls` output and the prompt exactly as the old client showed them
- **AND** the cursor is at the same row and column of that window

#### Scenario: Reattach restores every window
- **WHEN** a session holds two windows, the user runs `echo one` in the first and `echo two` in the second, kills the client and attaches again
- **THEN** the new client receives a layout holding both windows and a snapshot of each
- **AND** the snapshots show `one` and `two` respectively

#### Scenario: Reattach restores input modes
- **WHEN** nvim is running in a window with application cursor keys enabled, the client is killed, and a new client attaches
- **THEN** pressing Up in the new client, with that window focused, sends `\x1bOA` to nvim

#### Scenario: Reattach keeps modifyOtherKeys
- **WHEN** the program in a window has written `\x1b[>4;1m`, the client is killed, and a new client attaches
- **THEN** pressing Ctrl+Enter in the new client, with that window focused, sends `\x1b[27;5;13~` to the program
