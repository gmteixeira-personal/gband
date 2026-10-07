## MODIFIED Requirements

### Requirement: Encoding inputs
gband SHALL encode a key from its key code and its Shift, Alt and Ctrl modifiers. gband SHALL encode a key or a paste from three window input modes: application cursor keys (DECCKM, DEC private mode 1), bracketed paste (DEC private mode 2004) and the modifyOtherKeys level, as the terminal-emulator capability defines it. The encoding SHALL depend on nothing else. In this spec, byte sequences use Rust byte-string escapes, so `\x1b` is ESC.

#### Scenario: Same input, same bytes
- **WHEN** gband encodes the same key with the same modes twice
- **THEN** both encodings produce the same bytes

### Requirement: Editing keys
Enter SHALL encode as `\r`, Tab as `\t`, Shift+Tab as `\x1b[Z`, Backspace as `\x7f`, Ctrl+Backspace as `\x08`, and Escape as `\x1b`. While the window's modifyOtherKeys level is 0, Shift and Ctrl on Enter, Tab and Escape SHALL add no bytes, and Shift on Backspace SHALL add no bytes.

While the window's modifyOtherKeys level is 1 or 2, Enter or Backspace with Shift or Ctrl, and Tab with Ctrl, SHALL encode as `\x1b[27;`, the modifier parameter, `;`, the key's code and `~`. The code SHALL be 13 for Enter, 9 for Tab and 127 for Backspace. Tab with Shift and without Ctrl, and every editing key without Shift or Ctrl, SHALL encode as at level 0. Escape SHALL encode as at level 0 whatever its modifiers.

#### Scenario: Enter
- **WHEN** the user presses Enter
- **THEN** gband writes `\r`

#### Scenario: Shift+Enter matches Enter
- **WHEN** the user presses Shift+Enter in a window whose modifyOtherKeys level is 0
- **THEN** gband writes `\r`

#### Scenario: Ctrl+Enter without modifyOtherKeys
- **WHEN** the user presses Ctrl+Enter in a window whose modifyOtherKeys level is 0
- **THEN** gband writes `\r`

#### Scenario: Ctrl+Enter with modifyOtherKeys
- **WHEN** the user presses Ctrl+Enter in a window whose modifyOtherKeys level is 1
- **THEN** gband writes `\x1b[27;5;13~`

#### Scenario: Shift+Enter with modifyOtherKeys
- **WHEN** the user presses Shift+Enter in a window whose modifyOtherKeys level is 2
- **THEN** gband writes `\x1b[27;2;13~`

#### Scenario: Plain Enter with modifyOtherKeys
- **WHEN** the user presses Enter in a window whose modifyOtherKeys level is 1
- **THEN** gband writes `\r`

#### Scenario: Ctrl+Tab with modifyOtherKeys
- **WHEN** the user presses Ctrl+Tab in a window whose modifyOtherKeys level is 1
- **THEN** gband writes `\x1b[27;5;9~`

#### Scenario: Shift+Tab with modifyOtherKeys
- **WHEN** the user presses Shift+Tab in a window whose modifyOtherKeys level is 1
- **THEN** gband writes `\x1b[Z`

#### Scenario: Ctrl+Backspace with modifyOtherKeys
- **WHEN** the user presses Ctrl+Backspace in a window whose modifyOtherKeys level is 1
- **THEN** gband writes `\x1b[27;5;127~`

#### Scenario: Shift+Tab
- **WHEN** the user presses Shift+Tab
- **THEN** gband writes `\x1b[Z`

#### Scenario: Backspace
- **WHEN** the user presses Backspace
- **THEN** gband writes `\x7f`

#### Scenario: Ctrl+Backspace
- **WHEN** the user presses Ctrl+Backspace in a window whose modifyOtherKeys level is 0
- **THEN** gband writes `\x08`

### Requirement: Alt as an ESC prefix
Alt on a character key or an editing key SHALL prefix that key's encoding without Alt with `\x1b`, except where an editing key encodes in the modifyOtherKeys form, which carries Alt in its modifier parameter instead.

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

#### Scenario: Alt with Enter and modifyOtherKeys
- **WHEN** the user presses Alt+Enter in a window whose modifyOtherKeys level is 1
- **THEN** gband writes `\x1b\r`

#### Scenario: Alt with Ctrl+Enter and modifyOtherKeys
- **WHEN** the user presses Alt+Ctrl+Enter in a window whose modifyOtherKeys level is 1
- **THEN** gband writes `\x1b[27;7;13~`

### Requirement: Modifier parameter
Where this spec encodes a key with a modifier parameter `m`, `m` SHALL be 1 plus 1 for Shift, plus 2 for Alt, plus 4 for Ctrl. Cursor, tilde and function keys, and editing keys in the modifyOtherKeys form, SHALL carry Alt in this parameter, not as an ESC prefix.

#### Scenario: All three modifiers
- **WHEN** the user presses Shift+Alt+Ctrl+Up
- **THEN** gband writes `\x1b[1;8A`

### Requirement: Keys from the terminal's bytes
gband SHALL read keys from the bytes the user's terminal sends, as legacy xterm-style terminals send them, and SHALL turn each into a key with its key code and its Shift, Alt and Ctrl modifiers:

| bytes | key |
|---|---|
| a printable character in UTF-8 | that character, with Shift when it is an uppercase letter |
| `\r` | Enter |
| `\t` | Tab |
| `\x7f` | Backspace |
| `\x00` | Ctrl+Space |
| `\x01` to `\x1a`, other than `\t` and `\r` | Ctrl with `a` to `z`, so `\x08` is Ctrl+`h` and `\n` is Ctrl+`j` |
| `\x1c` to `\x1f` | Ctrl with `4` to `7` |
| `\x1b[Z` | Shift+Tab |
| `\x1b[A`, `\x1b[B`, `\x1b[C`, `\x1b[D`, `\x1b[H`, `\x1b[F`, and the same final letters after `\x1bO` | Up, Down, Right, Left, Home, End |
| `\x1bOP` to `\x1bOS` | F1 to F4 |
| `\x1b[[A` to `\x1b[[E` | F1 to F5 |
| `\x1b[n~` | Home for 1 and 7, Insert for 2, Delete for 3, End for 4 and 8, PageUp for 5, PageDown for 6, F1 to F5 for 11 to 15, F6 to F10 for 17 to 21, F11 for 23 and F12 for 24 |
| `\x1b[27;m;c~`, `\x1b[c;mu` and `\x1b[cu` | the key with code `c`: Enter for 13, Tab for 9, Backspace for 127, Escape for 27, and for any other `c` that is a printable character, that character |

A cursor key, F1 to F4 as `\x1b[1;mP` to `\x1b[1;mS`, a tilde key as `\x1b[n;m~`, or a key with code `c` SHALL carry its modifiers in the parameter `m`, as "Modifier parameter" defines. A key with code `c` and no `m` SHALL carry no modifier. Tab with Shift as its only modifier SHALL be read as the same Shift+Tab that `\x1b[Z` is read as. Higher modifier bits, such as Meta, SHALL be dropped. A complete escape sequence that starts as one of these keys, a mouse report, a focus report or a paste does, by `\x1b[` and a digit, `?`, `<`, `[`, `M` or one of the letters `A`, `B`, `C`, `D`, `F`, `H`, `I`, `O` and `Z`, and then names none of them SHALL become no key, and its bytes SHALL NOT reach any window. A key with code `c` whose `c` is neither 9, 13, 27 nor 127 and not a printable character, or whose parameters hold anything other than decimal numbers separated by `;`, names none of them. A valid SGR mouse report of a button other than those "Mouse events from the terminal" names SHALL become no event in the same way.

#### Scenario: Control byte becomes a key
- **WHEN** the terminal sends `\x03`
- **THEN** gband reads Ctrl+`c`

#### Scenario: Backspace byte is Ctrl+H
- **WHEN** the terminal sends `\x08`
- **THEN** gband reads Ctrl+`h`

#### Scenario: Modified cursor key
- **WHEN** the terminal sends `\x1b[1;5A`
- **THEN** gband reads Ctrl+Up

#### Scenario: rxvt Home
- **WHEN** the terminal sends `\x1b[7~`
- **THEN** gband reads Home

#### Scenario: Linux console F1
- **WHEN** the terminal sends `\x1b[[A`
- **THEN** gband reads F1

#### Scenario: Unknown sequence is dropped
- **WHEN** the terminal sends `\x1b[99~` and then `a`
- **THEN** gband reads `a` and nothing else

#### Scenario: Ghostty's Ctrl+Enter
- **WHEN** the terminal sends `\x1b[27;5;13~`
- **THEN** gband reads Ctrl+Enter

#### Scenario: CSI u Ctrl+Enter
- **WHEN** the terminal sends `\x1b[13;5u`
- **THEN** gband reads Ctrl+Enter

#### Scenario: Modified character
- **WHEN** the terminal sends `\x1b[27;5;49~`
- **THEN** gband reads Ctrl+`1`

#### Scenario: Shift+Tab in the modifyOtherKeys form
- **WHEN** the terminal sends `\x1b[27;2;9~`
- **THEN** gband reads Shift+Tab, the same key as `\x1b[Z`

#### Scenario: Control code is no key
- **WHEN** the terminal sends `\x1b[27;5;1~` and then `a`
- **THEN** gband reads `a` and nothing else

#### Scenario: Kitty sub-parameters are no key
- **WHEN** the terminal sends `\x1b[13:13;5u` and then `a`
- **THEN** gband reads `a` and nothing else

#### Scenario: Ctrl+Enter reaches a program that asked for modifyOtherKeys
- **WHEN** the program in the focused window has written `\x1b[>4;1m` and the terminal sends `\x1b[27;5;13~`
- **THEN** the program reads `\x1b[27;5;13~` from its terminal
