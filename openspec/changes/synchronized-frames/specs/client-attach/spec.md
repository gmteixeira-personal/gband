## ADDED Requirements

### Requirement: Whole frames
Every time the client draws the terminal, it SHALL write the frame as one synchronized update: `\e[?2026h` first, `\e[?2026l` last, and every byte of the frame between them. The first bytes after `\e[?2026h` SHALL hide the cursor with `\e[?25l`, before any cell is written or the cursor is moved. The cursor SHALL then stay hidden until the frame's last cell is written, and SHALL be shown, when the ribbon presentation shows it, only from the frame's final cursor position.

When writing a frame fails after `\e[?2026h` was written, the client SHALL still try to write `\e[?2026l` before it reports the failure.

A terminal that does not support mode 2026 ignores both markers and still receives the frame and the cursor hidden while cells are written.

#### Scenario: Frame between the markers
- **WHEN** the client draws a frame in which a window's output changed three cells and the focused window's cursor is shown
- **THEN** the bytes written for that frame start with `\e[?2026h\e[?25l` and end with `\e[?2026l`
- **AND** no `\e[?2026l` appears before the frame's last byte

#### Scenario: Cursor hidden while cells are written
- **WHEN** the client draws a frame that changes cells on rows 2 and 10 and places the cursor on row 5
- **THEN** no `\e[?25h` appears among the frame's bytes before the last changed cell's bytes
- **AND** the terminal's cursor ends shown on row 5

#### Scenario: Cursor stays hidden
- **WHEN** the client draws a frame while a floating plugin window is focused
- **THEN** the frame's bytes hold `\e[?25l` and no `\e[?25h`

#### Scenario: Failed frame still ends
- **WHEN** writing a frame fails after the begin marker and some cells were written
- **THEN** the client writes `\e[?2026l` before it returns the error

## MODIFIED Requirements

### Requirement: Leaving the client
Whenever the client exits after taking the terminal, it SHALL first end any synchronized update with `\e[?2026l`, then leave the alternate screen, disable raw mode and bracketed paste, and show the cursor. It SHALL then print one line to standard output, followed by the note for a server from a different build when one applies, and exit as follows:

| cause | line printed | exit status |
|---|---|---|
| the user detached | `[detached]` | 0 |
| the session ended | `[exited]` | 0 |
| the connection closed without the server saying why | `[lost server]` | 1 |

#### Scenario: Detach and reattach
- **WHEN** the user runs `sleep 100`, detaches with Ctrl+Space then `D`, and runs `gband attach` again
- **THEN** the first client printed `[detached]` and exited with status 0
- **AND** the new client shows `sleep 100` still running

#### Scenario: Shell exits
- **WHEN** the user runs `exit` in the only window
- **THEN** the client restores the terminal, prints `[exited]` and exits with status 0

#### Scenario: Server killed
- **WHEN** the server is killed with SIGKILL while a client is attached
- **THEN** the client restores the terminal, prints `[lost server]` and exits with status 1

#### Scenario: Exit ends a synchronized update
- **WHEN** the client exits while a frame's `\e[?2026h` has no matching `\e[?2026l`
- **THEN** the client writes `\e[?2026l` before it leaves the alternate screen
