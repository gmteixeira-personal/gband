# input-encoding Specification

## Purpose

Defines the bytes gband writes to a window's PTY for each key the user presses and each text the user pastes, given the input modes the program in that window has set, so programs such as shells, nvim and Claude Code receive the input they expect.

## Requirements

### Requirement: Encoding inputs
gband SHALL encode a key from its key code and its Shift, Alt and Ctrl modifiers. gband SHALL encode a key or a paste from three window input modes: application cursor keys (DECCKM, DEC private mode 1), bracketed paste (DEC private mode 2004) and the modifyOtherKeys level, as the terminal-emulator capability defines it. The encoding SHALL depend on nothing else. In this spec, byte sequences use Rust byte-string escapes, so `\x1b` is ESC.

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

### Requirement: Incomplete input from the terminal
gband SHALL read each key, mouse report, focus report and paste as soon as its last byte arrives, however the terminal's bytes are split across reads. When the bytes read so far end in an incomplete escape sequence, gband SHALL hold them until more bytes arrive. When no byte arrives for 25 ms, gband SHALL resolve what it holds, as zellij resolves it: a lone ESC SHALL become Escape, and ESC with `[` or `O` and bytes that complete no sequence SHALL become Alt with `[` or `O`, followed by the remaining bytes read as keys. gband SHALL read held bytes the same way, without waiting, once a byte arrives that no sequence can continue: a byte after `\x1b[` that starts no sequence, as "Keys from the terminal's bytes" lists the bytes that do, a byte after `\x1bO` that ends none of its keys, a byte after `\x1b[[` other than `A` to `E`, a byte that breaks an SGR mouse report's `<`, three decimal numbers separated by `;`, and `M` or `m`, or any other byte that is neither a parameter, an intermediate nor a final byte of the sequence. An incomplete UTF-8 character SHALL stay held until its remaining bytes arrive.

#### Scenario: Lone Escape
- **WHEN** the terminal sends `\x1b` and nothing more
- **THEN** gband reads Escape about 25 ms later

#### Scenario: Arrow split after its ESC
- **WHEN** the terminal sends `\x1b` and, 10 ms later, `[A`
- **THEN** gband reads Up and no Escape

#### Scenario: Mouse report split mid-way
- **WHEN** the terminal sends `\x1b[<35;10` and, 10 ms later, `;5M`
- **THEN** gband reads a motion with no button at column 9 and row 4, and no key

#### Scenario: Escape then a later key
- **WHEN** the terminal sends `\x1b` and, 100 ms later, `[`
- **THEN** gband reads Escape and then `[`

#### Scenario: Alt with a left bracket
- **WHEN** the terminal sends `\x1b[` and nothing more
- **THEN** gband reads Alt+`[` about 25 ms later

#### Scenario: Alt with a left bracket, then a letter
- **WHEN** the terminal sends `\x1b[` and, 10 ms later, `x`
- **THEN** gband reads Alt+`[` and then `x` as soon as `x` arrives

#### Scenario: Malformed mouse report
- **WHEN** the terminal sends `\x1b[<0;1M`
- **THEN** gband reads Alt+`[`, then `<`, `0`, `;`, `1` and Shift+`M`

#### Scenario: Unfinished sequence after its timeout
- **WHEN** the terminal sends `\x1b[11` and nothing more
- **THEN** gband reads Alt+`[`, then `1` and `1`, about 25 ms later

### Requirement: ESC before another input
When ESC is followed, without a 25 ms gap, by the bytes of a key, gband SHALL read one key: that key with Alt added. When ESC is followed, without a 25 ms gap, by a mouse report, a focus report or the start of a paste, gband SHALL read Escape and then that mouse event, focus change or paste. gband SHALL NOT drop an Escape in either case, and SHALL NOT read the bytes of an escape sequence as typed characters.

#### Scenario: Alt with a letter
- **WHEN** the terminal sends `\x1bx` in one write
- **THEN** gband reads Alt+`x`

#### Scenario: Doubled ESC before an arrow
- **WHEN** the terminal sends `\x1b\x1b[A` in one write
- **THEN** gband reads Alt+Up

#### Scenario: Alt with Escape
- **WHEN** the terminal sends `\x1b\x1b` in one write
- **THEN** gband reads Alt+Escape

#### Scenario: Escape merged with a mouse report
- **WHEN** the terminal sends `\x1b\x1b[<35;10;5M` in one write
- **THEN** gband reads Escape and then a motion with no button at column 9 and row 4

#### Scenario: Escape merged with a paste
- **WHEN** the terminal sends `\x1b\x1b[200~hi\x1b[201~` in one write
- **THEN** gband reads Escape and then a paste of `hi`

### Requirement: Focus reports and pastes from the terminal
gband SHALL read `\x1b[I` as the terminal gaining focus and `\x1b[O` as it losing focus. gband SHALL read `\x1b[200~`, the bytes that follow and `\x1b[201~` as one paste of the text between the two markers, and SHALL read no key from that text, including any ESC it holds.

#### Scenario: Focus gained
- **WHEN** the terminal sends `\x1b[I`
- **THEN** gband reads a focus gain and no key

#### Scenario: Paste holding an escape
- **WHEN** the terminal sends `\x1b[200~a\x1b[Ab\x1b[201~`
- **THEN** gband reads one paste of `a\x1b[Ab` and no key

### Requirement: Mouse events from the terminal
A mouse report from the user's terminal SHALL become a mouse event: a press, a release or a motion, with the left, middle or right button or, for a motion, no button, or a wheel step up, down, left or right. The event SHALL hold the terminal cell it reports, counted from 0, and its Shift, Alt and Ctrl modifiers. Other modifiers SHALL be dropped. A report of any other button SHALL become no event.

#### Scenario: SGR press becomes an event
- **WHEN** the terminal reports `\x1b[<16;13;4M`
- **THEN** gband reads a press of the left button with Ctrl at column 12 and row 3

#### Scenario: Wheel becomes a step
- **WHEN** the terminal reports `\x1b[<65;1;1M`
- **THEN** gband reads a wheel step down at column 0 and row 0

### Requirement: Mouse reports
gband SHALL encode a mouse event for a window from the event and two of the window's input modes: its mouse tracking mode and its mouse encoding, as the terminal-emulator capability defines them. The event's cell SHALL be its content cell, and the reported column and row SHALL each be that cell's plus 1.

The tracking mode SHALL decide which events are reported. Every other event SHALL produce no bytes:

| tracking mode | events reported |
|---|---|
| none | none |
| press, mode 9 | presses of the left, middle and right buttons, without modifiers |
| press and release, mode 1000 | presses, releases and wheel steps |
| button motion, mode 1002 | as mode 1000, and motions while a button is held |
| any motion, mode 1003 | as mode 1002, and motions with no button |

The button code SHALL be 0 for the left button, 1 for the middle and 2 for the right, 3 for a release outside the SGR encoding and for a motion with no button, and 64, 65, 66 and 67 for a wheel step up, down, left and right. A motion SHALL add 32. Shift SHALL add 4, Alt 8 and Ctrl 16.

The encoding SHALL decide the bytes:
- **SGR**: `\x1b[<`, the button code, `;`, the column, `;`, the row, all in decimal, then `M`, or `m` for a release. A release SHALL carry the released button's code.
- **Default**: `\x1b[M`, then three bytes: 32 plus the button code, 32 plus the column, and 32 plus the row. When the column or the row is above 223, the event SHALL produce no bytes.
- **UTF-8**: as the default encoding, with each of the three values written as the UTF-8 encoding of that code point. When the column or the row is above 2015, the event SHALL produce no bytes.

#### Scenario: SGR press and release
- **WHEN** the window's mouse tracking mode is 1000 with SGR encoding and the user presses and releases the right button at content column 0 and row 0
- **THEN** gband writes `\x1b[<2;1;1M` and then `\x1b[<2;1;1m`

#### Scenario: Default encoding press
- **WHEN** the window's mouse tracking mode is 1000 with the default encoding and the user presses the left button at content column 9 and row 4
- **THEN** gband writes `\x1b[M *%`

#### Scenario: Default encoding release
- **WHEN** the window's mouse tracking mode is 1000 with the default encoding and the user releases the left button at content column 9 and row 4
- **THEN** gband writes `\x1b[M#*%`

#### Scenario: Drag with button motion
- **WHEN** the window's mouse tracking mode is 1002 with SGR encoding and the user moves the pointer to content column 2 and row 1 with the left button held
- **THEN** gband writes `\x1b[<32;3;2M`

#### Scenario: Motion not reported in mode 1000
- **WHEN** the window's mouse tracking mode is 1000 and the user moves the pointer with the left button held
- **THEN** gband writes nothing

#### Scenario: Wheel with Ctrl
- **WHEN** the window's mouse tracking mode is 1000 with SGR encoding and the user turns the wheel up with Ctrl at content column 0 and row 0
- **THEN** gband writes `\x1b[<80;1;1M`

#### Scenario: Column beyond the default encoding
- **WHEN** the window's mouse tracking mode is 1000 with the default encoding and the user presses at content column 230
- **THEN** gband writes nothing
