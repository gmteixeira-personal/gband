## Purpose

Defines the bytes gband writes to a pane's PTY for each key the user presses and each text the user pastes, given the input modes the program in that pane has set, so programs such as shells, nvim and Claude Code receive the input they expect.

## ADDED Requirements

### Requirement: Encoding inputs
gband SHALL encode a key from its key code and its Shift, Alt and Ctrl modifiers. gband SHALL encode a key or a paste from two pane input modes: application cursor keys (DECCKM, DEC private mode 1) and bracketed paste (DEC private mode 2004). The encoding SHALL depend on nothing else. In this spec, byte sequences use Rust byte-string escapes, so `\x1b` is ESC.

#### Scenario: Same input, same bytes
- **WHEN** gband encodes the same key with the same modes twice
- **THEN** both encodings produce the same bytes

### Requirement: Printable characters
A character key without Ctrl SHALL encode as the character's UTF-8 bytes. Shift SHALL add no bytes, because the character already carries its case.

#### Scenario: ASCII letter
- **WHEN** the user presses `a`
- **THEN** gband writes `a`

#### Scenario: Shifted letter
- **WHEN** the user presses Shift+`A`
- **THEN** gband writes `A`

#### Scenario: Non-ASCII character
- **WHEN** the user presses `é`
- **THEN** gband writes `\xc3\xa9`

### Requirement: Control characters
A character key with Ctrl SHALL encode as the C0 control code below. Ctrl with an ASCII letter SHALL ignore the letter's case and Shift. Ctrl with any other character SHALL encode as that character alone.

| character with Ctrl | byte |
|---|---|
| `a` to `z` | `\x01` to `\x1a` |
| space, `@`, `2` | `\x00` |
| `[`, `3` | `\x1b` |
| `\`, `4` | `\x1c` |
| `]`, `5` | `\x1d` |
| `^`, `6` | `\x1e` |
| `_`, `/`, `7` | `\x1f` |
| `?`, `8` | `\x7f` |

#### Scenario: Ctrl with a letter
- **WHEN** the user presses Ctrl+`a`
- **THEN** gband writes `\x01`

#### Scenario: Ctrl+H stays distinct from Backspace
- **WHEN** the user presses Ctrl+`h`
- **THEN** gband writes `\x08`

#### Scenario: Ctrl with a space
- **WHEN** the user presses Ctrl+space
- **THEN** gband writes `\x00`

#### Scenario: Ctrl with a digit that has a control code
- **WHEN** the user presses Ctrl+`4`
- **THEN** gband writes `\x1c`

#### Scenario: Ctrl with a character that has no control code
- **WHEN** the user presses Ctrl+`1`
- **THEN** gband writes `1`

#### Scenario: Ctrl+I is the same byte as Tab
- **WHEN** the user presses Ctrl+`i`
- **THEN** gband writes `\x09`, the same byte as Tab

### Requirement: Editing keys
Enter SHALL encode as `\r`, Tab as `\t`, Shift+Tab as `\x1b[Z`, Backspace as `\x7f`, Ctrl+Backspace as `\x08`, and Escape as `\x1b`. Shift and Ctrl on Enter, Tab and Escape SHALL add no bytes, and Shift on Backspace SHALL add no bytes.

#### Scenario: Enter
- **WHEN** the user presses Enter
- **THEN** gband writes `\r`

#### Scenario: Shift+Enter matches Enter
- **WHEN** the user presses Shift+Enter
- **THEN** gband writes `\r`

#### Scenario: Shift+Tab
- **WHEN** the user presses Shift+Tab
- **THEN** gband writes `\x1b[Z`

#### Scenario: Backspace
- **WHEN** the user presses Backspace
- **THEN** gband writes `\x7f`

#### Scenario: Ctrl+Backspace
- **WHEN** the user presses Ctrl+Backspace
- **THEN** gband writes `\x08`

### Requirement: Alt as an ESC prefix
Alt on a character key or an editing key SHALL prefix that key's encoding without Alt with `\x1b`.

#### Scenario: Alt with a letter
- **WHEN** the user presses Alt+`x`
- **THEN** gband writes `\x1bx`

#### Scenario: Alt with Ctrl and a letter
- **WHEN** the user presses Alt+Ctrl+`a`
- **THEN** gband writes `\x1b\x01`

#### Scenario: Alt with Backspace
- **WHEN** the user presses Alt+Backspace
- **THEN** gband writes `\x1b\x7f`

#### Scenario: Alt with Escape
- **WHEN** the user presses Alt+Escape
- **THEN** gband writes `\x1b\x1b`

### Requirement: Modifier parameter
Where this spec encodes a key with a modifier parameter `m`, `m` SHALL be 1 plus 1 for Shift, plus 2 for Alt, plus 4 for Ctrl. Cursor, tilde and function keys SHALL carry Alt in this parameter, not as an ESC prefix.

#### Scenario: All three modifiers
- **WHEN** the user presses Shift+Alt+Ctrl+Up
- **THEN** gband writes `\x1b[1;8A`

### Requirement: Cursor keys follow application cursor mode
Up, Down, Right, Left, Home and End without modifiers SHALL encode as `\x1b[` followed by `A`, `B`, `C`, `D`, `H` and `F` respectively while application cursor keys mode is off. While the mode is on, they SHALL encode as `\x1bO` followed by the same letter. With any modifier, they SHALL encode as `\x1b[1;` followed by the modifier parameter and the letter, whatever the mode.

#### Scenario: Up in normal mode
- **WHEN** application cursor keys mode is off and the user presses Up
- **THEN** gband writes `\x1b[A`

#### Scenario: Up in application mode
- **WHEN** application cursor keys mode is on and the user presses Up
- **THEN** gband writes `\x1bOA`

#### Scenario: Home in application mode
- **WHEN** application cursor keys mode is on and the user presses Home
- **THEN** gband writes `\x1bOH`

#### Scenario: Modified arrow ignores the mode
- **WHEN** application cursor keys mode is on and the user presses Shift+Up
- **THEN** gband writes `\x1b[1;2A`

#### Scenario: Ctrl+Left
- **WHEN** the user presses Ctrl+Left
- **THEN** gband writes `\x1b[1;5D`

#### Scenario: Alt+Up
- **WHEN** the user presses Alt+Up
- **THEN** gband writes `\x1b[1;3A`

### Requirement: Tilde and function keys
Insert, Delete, Page Up and Page Down SHALL encode as `\x1b[2~`, `\x1b[3~`, `\x1b[5~` and `\x1b[6~`. F1 to F4 SHALL encode as `\x1bOP` to `\x1bOS`. F5 to F12 SHALL encode as `\x1b[` followed by `15`, `17`, `18`, `19`, `20`, `21`, `23` and `24` respectively and `~`. With any modifier, a tilde key SHALL encode as `\x1b[`, its number, `;`, the modifier parameter and `~`, and F1 to F4 SHALL encode as `\x1b[1;`, the modifier parameter and `P` to `S`. Function keys above F12 SHALL encode as no bytes.

#### Scenario: Delete
- **WHEN** the user presses Delete
- **THEN** gband writes `\x1b[3~`

#### Scenario: Ctrl+Delete
- **WHEN** the user presses Ctrl+Delete
- **THEN** gband writes `\x1b[3;5~`

#### Scenario: F1
- **WHEN** the user presses F1
- **THEN** gband writes `\x1bOP`

#### Scenario: Shift+F1
- **WHEN** the user presses Shift+F1
- **THEN** gband writes `\x1b[1;2P`

#### Scenario: F12
- **WHEN** the user presses F12
- **THEN** gband writes `\x1b[24~`

#### Scenario: F13
- **WHEN** the user presses F13
- **THEN** gband writes nothing

### Requirement: Paste
Before encoding pasted text, gband SHALL remove every C0 control character other than tab, line feed and carriage return, and every C1 control character. While bracketed paste mode is on, gband SHALL write `\x1b[200~`, the filtered text and `\x1b[201~`. While the mode is off, gband SHALL write the filtered text with each `\r\n` pair and each remaining `\n` replaced by `\r`.

#### Scenario: Bracketed paste
- **WHEN** bracketed paste mode is on and the user pastes `ls\n`
- **THEN** gband writes `\x1b[200~ls\n\x1b[201~`

#### Scenario: Plain paste converts line endings
- **WHEN** bracketed paste mode is off and the user pastes `a\nb\r\nc`
- **THEN** gband writes `a\rb\rc`

#### Scenario: Pasted text cannot end the bracket early
- **WHEN** bracketed paste mode is on and the user pastes `x\x1b[201~rm`
- **THEN** gband writes `\x1b[200~x[201~rm\x1b[201~`

#### Scenario: Tab survives the filter
- **WHEN** bracketed paste mode is off and the user pastes `a\tb`
- **THEN** gband writes `a\tb`

### Requirement: Key events from the terminal
A key press or key repeat reported by the user's terminal SHALL become a key with its key code and its Shift, Alt and Ctrl modifiers. A key release SHALL become no key. Super, Hyper and Meta SHALL be dropped from the modifiers. A key outside the codes this spec encodes, such as Caps Lock, a media key or a bare modifier key, SHALL become no key.

#### Scenario: Press becomes a key
- **WHEN** the terminal reports a press of Ctrl+`c`
- **THEN** gband encodes Ctrl+`c` and writes `\x03`

#### Scenario: Release is dropped
- **WHEN** the terminal reports a release of `a`
- **THEN** gband writes nothing

#### Scenario: Super is dropped
- **WHEN** the terminal reports a press of Super+`a`
- **THEN** gband writes `a`

#### Scenario: Caps Lock is dropped
- **WHEN** the terminal reports a press of Caps Lock
- **THEN** gband writes nothing
