## ADDED Requirements

### Requirement: Keyboard protocols of the user's terminal
When the client attaches, it SHALL write `\x1b[>4;1m` and `\x1b[>5u` to its terminal after entering the alternate screen and before its first frame, asking for modifyOtherKeys level 1 and for the kitty keyboard protocol's disambiguate and alternate-key flags. When the client detaches, it SHALL write `\x1b[<1u` and `\x1b[>4m` before leaving the alternate screen. The client SHALL send no query for either protocol: a terminal that supports neither ignores both requests, and gband reads the keys it sends as before.

#### Scenario: Requests on attach
- **WHEN** the client attaches in a terminal that implements both protocols
- **THEN** that terminal's alternate screen holds kitty keyboard flags with the disambiguate bit set and its modifyOtherKeys level is 1 before the first frame

#### Scenario: Requests undone on detach
- **WHEN** the client detaches from that terminal
- **THEN** the terminal's modifyOtherKeys level is 0 and its main screen's kitty keyboard flags are what they were before the client attached

### Requirement: Kitty keyboard encoding
While a window's kitty keyboard flags include the disambiguate bit, 1, as the terminal-emulator capability defines them, gband SHALL encode its keys as below, in place of the encodings that "Control characters", "Editing keys", "Alt as an ESC prefix" and "Tilde and function keys" give, and whatever the window's modifyOtherKeys level is. `m` is the modifier parameter that "Modifier parameter" defines.

- A character key without Ctrl and without Alt SHALL encode as "Printable characters" defines.
- A character key with Ctrl or Alt SHALL encode as `\x1b[`, the key's code, `;`, `m` and `u`. The code SHALL be the decimal code point of the character, or of its lowercase form when the character is an uppercase letter with a single lowercase form.
- Enter, Tab and Backspace without modifiers SHALL encode as `\r`, `\t` and `\x7f`. Enter and Backspace with any modifier, and Tab with Ctrl or Alt, SHALL encode as `\x1b[13;mu`, `\x1b[9;mu` and `\x1b[127;mu`. Shift+Tab without Ctrl or Alt SHALL encode as `\x1b[Z`.
- Escape SHALL encode as `\x1b[27u` without modifiers and as `\x1b[27;mu` with any.
- F13 to F35 SHALL encode as `\x1b[`, 57376 plus the key's number minus 13, and `u`, with `;` and `m` before `u` when the key has a modifier.
- Cursor keys, Insert, Delete, Page Up, Page Down and F1 to F12 SHALL encode as "Cursor keys follow application cursor mode" and "Tilde and function keys" define.

#### Scenario: Shift+Enter reaches Claude Code
- **WHEN** the program in the focused window has written `\x1b[>1u` and the user presses Shift+Enter
- **THEN** gband writes `\x1b[13;2u`

#### Scenario: Ctrl+Enter
- **WHEN** the window's kitty keyboard flags are 1 and the user presses Ctrl+Enter
- **THEN** gband writes `\x1b[13;5u`

#### Scenario: Plain Enter stays legacy
- **WHEN** the window's kitty keyboard flags are 1 and the user presses Enter
- **THEN** gband writes `\r`

#### Scenario: Ctrl with a letter
- **WHEN** the window's kitty keyboard flags are 1 and the user presses Ctrl+`c`
- **THEN** gband writes `\x1b[99;5u`

#### Scenario: Alt with Shift and a letter
- **WHEN** the window's kitty keyboard flags are 1 and the user presses Alt+Shift+`A`
- **THEN** gband writes `\x1b[97;4u`

#### Scenario: Alt with a symbol
- **WHEN** the window's kitty keyboard flags are 1 and the user presses Alt+`!`
- **THEN** gband writes `\x1b[33;3u`

#### Scenario: Escape
- **WHEN** the window's kitty keyboard flags are 1 and the user presses Escape
- **THEN** gband writes `\x1b[27u`

#### Scenario: Shift+Tab stays legacy
- **WHEN** the window's kitty keyboard flags are 1 and the user presses Shift+Tab
- **THEN** gband writes `\x1b[Z`

#### Scenario: Ctrl+Tab
- **WHEN** the window's kitty keyboard flags are 1 and the user presses Ctrl+Tab
- **THEN** gband writes `\x1b[9;5u`

#### Scenario: Shifted letter is text
- **WHEN** the window's kitty keyboard flags are 1 and the user presses Shift+`A`
- **THEN** gband writes `A`

#### Scenario: F13
- **WHEN** the window's kitty keyboard flags are 1 and the user presses F13
- **THEN** gband writes `\x1b[57376u`

#### Scenario: Arrow unchanged
- **WHEN** the window's kitty keyboard flags are 1 and the user presses Ctrl+Left
- **THEN** gband writes `\x1b[1;5D`

#### Scenario: Kitty over modifyOtherKeys
- **WHEN** the program in the window has written `\x1b[>4;1m` and `\x1b[>1u` and the user presses Ctrl+Enter
- **THEN** gband writes `\x1b[13;5u`

#### Scenario: Popped flags return to legacy
- **WHEN** the program in the window has written `\x1b[>1u` and then `\x1b[<u` and the user presses Shift+Enter
- **THEN** gband writes `\r`

## MODIFIED Requirements

### Requirement: Encoding inputs
gband SHALL encode a key from its key code and its Shift, Alt and Ctrl modifiers. gband SHALL encode a key or a paste from four window input modes: application cursor keys (DECCKM, DEC private mode 1), bracketed paste (DEC private mode 2004), the modifyOtherKeys level and the kitty keyboard flags, as the terminal-emulator capability defines them. The encoding SHALL depend on nothing else. In this spec, byte sequences use Rust byte-string escapes, so `\x1b` is ESC.

#### Scenario: Same input, same bytes
- **WHEN** gband encodes the same key with the same modes twice
- **THEN** both encodings produce the same bytes

### Requirement: Alt as an ESC prefix
Alt on a character key or an editing key SHALL prefix that key's encoding without Alt with `\x1b`, except where a key encodes in the modifyOtherKeys form or in the kitty form that "Kitty keyboard encoding" defines, which carry Alt in their modifier parameter instead.

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

#### Scenario: Alt with a letter and kitty flags
- **WHEN** the user presses Alt+`x` in a window whose kitty keyboard flags are 1
- **THEN** gband writes `\x1b[120;3u`

### Requirement: Modifier parameter
Where this spec encodes a key with a modifier parameter `m`, `m` SHALL be 1 plus 1 for Shift, plus 2 for Alt, plus 4 for Ctrl. Cursor, tilde and function keys, editing keys in the modifyOtherKeys form, and keys in the kitty form SHALL carry Alt in this parameter, not as an ESC prefix.

#### Scenario: All three modifiers
- **WHEN** the user presses Shift+Alt+Ctrl+Up
- **THEN** gband writes `\x1b[1;8A`

#### Scenario: All three modifiers in the kitty form
- **WHEN** the user presses Shift+Alt+Ctrl+Enter in a window whose kitty keyboard flags are 1
- **THEN** gband writes `\x1b[13;8u`

### Requirement: Tilde and function keys
Insert, Delete, Page Up and Page Down SHALL encode as `\x1b[2~`, `\x1b[3~`, `\x1b[5~` and `\x1b[6~`. F1 to F4 SHALL encode as `\x1bOP` to `\x1bOS`. F5 to F12 SHALL encode as `\x1b[` followed by `15`, `17`, `18`, `19`, `20`, `21`, `23` and `24` respectively and `~`. With any modifier, a tilde key SHALL encode as `\x1b[`, its number, `;`, the modifier parameter and `~`, and F1 to F4 SHALL encode as `\x1b[1;`, the modifier parameter and `P` to `S`. Function keys above F12 SHALL encode as no bytes, except in a window whose kitty keyboard flags include the disambiguate bit, where "Kitty keyboard encoding" defines them.

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
- **WHEN** the user presses F13 in a window whose kitty keyboard flags are 0
- **THEN** gband writes nothing

### Requirement: Keys from the terminal's bytes
gband SHALL read keys from the bytes the user's terminal sends, as legacy xterm-style terminals send them and as terminals send them with modifyOtherKeys or the kitty keyboard protocol's disambiguate and alternate-key flags in effect, and SHALL turn each into a key with its key code and its Shift, Alt and Ctrl modifiers:

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
| `\x1b[P`, `\x1b[Q` and `\x1b[S` | F1, F2 and F4 |
| `\x1b[[A` to `\x1b[[E` | F1 to F5 |
| `\x1b[n~` | Home for 1 and 7, Insert for 2, Delete for 3, End for 4 and 8, PageUp for 5, PageDown for 6, F1 to F5 for 11 to 15, F6 to F10 for 17 to 21, F11 for 23 and F12 for 24 |
| `\x1b[27;m;c~`, `\x1b[c;mu` and `\x1b[cu`, where `c` may carry alternate keys as `c:s`, `c:s:b` or `c::b` | the key with code `c`, as the code table below names it |

| code `c` | key |
|---|---|
| 13, 9, 127, 27 | Enter, Tab, Backspace, Escape |
| 57376 to 57398 | F13 to F35 |
| 57399 to 57408 | `0` to `9` |
| 57409, 57410, 57411, 57412, 57413, 57415, 57416 | `.`, `/`, `*`, `-`, `+`, `=`, `,` |
| 57414 | Enter |
| 57417, 57418, 57419, 57420 | Left, Right, Up, Down |
| 57421, 57422, 57423, 57424, 57425, 57426 | PageUp, PageDown, Home, End, Insert, Delete |
| any other code from 57344 to 63743 | no key |
| any other code that is a printable character | that character |

A cursor key, F1 to F4 as `\x1b[1;mP` to `\x1b[1;mS`, a tilde key as `\x1b[n;m~`, or a key with code `c` SHALL carry its modifiers in the parameter `m`, as "Modifier parameter" defines. A key with code `c` and no `m` SHALL carry no modifier. Tab with Shift as its only modifier SHALL be read as the same Shift+Tab that `\x1b[Z` is read as. Higher modifier bits, such as Super, Meta, Caps Lock and Num Lock, SHALL be dropped. When `m` holds Shift and the code carries a shifted key `s` that is a printable character, the key SHALL be the character `s`, and it SHALL keep Shift only when `s` is an uppercase letter. When `m` holds Shift, the code carries no shifted key and `c` is a lowercase ASCII letter, the key SHALL be the uppercase letter with Shift. The base-layout key `b` SHALL be ignored. A complete escape sequence that starts as one of these keys, a mouse report, a focus report or a paste does, by `\x1b[` and a digit, `?`, `<`, `[`, `M` or one of the letters `A`, `B`, `C`, `D`, `E`, `F`, `H`, `I`, `O`, `P`, `Q`, `S` and `Z`, and then names none of them SHALL become no key, and its bytes SHALL NOT reach any window. A key with code `c` whose `c` names no key in the code table, or whose parameters hold anything other than decimal numbers separated by `;` apart from the alternate keys in its first parameter, names none of them.

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
- **WHEN** the terminal sends `\x1b[13;5:1u` and then `a`
- **THEN** gband reads `a` and nothing else

#### Scenario: Shifted key names the character
- **WHEN** the terminal sends `\x1b[49:33;4u`
- **THEN** gband reads Alt+`!`, the same key as `\x1b!`

#### Scenario: Shifted letter keeps Shift
- **WHEN** the terminal sends `\x1b[97:65;6u`
- **THEN** gband reads Ctrl+Shift+`A`

#### Scenario: Shifted letter without an alternate key
- **WHEN** the terminal sends `\x1b[97;4u`
- **THEN** gband reads Alt+Shift+`A`, the same key as `\x1bA`

#### Scenario: Base-layout key ignored
- **WHEN** the terminal sends `\x1b[1089::99;5u`
- **THEN** gband reads Ctrl+`с`

#### Scenario: Kitty Ctrl+I stays apart from Tab
- **WHEN** the terminal sends `\x1b[105;5u`
- **THEN** gband reads Ctrl+`i`, and no binding on Tab matches it

#### Scenario: Kitty Escape
- **WHEN** the terminal sends `\x1b[27u`
- **THEN** gband reads Escape at once, without waiting 25 ms

#### Scenario: Keypad Enter
- **WHEN** the terminal sends `\x1b[57414u`
- **THEN** gband reads Enter

#### Scenario: Kitty F13
- **WHEN** the terminal sends `\x1b[57376;2u`
- **THEN** gband reads Shift+F13

#### Scenario: Modifier key alone is no key
- **WHEN** the terminal sends `\x1b[57441;2u` and then `a`
- **THEN** gband reads `a` and nothing else

#### Scenario: Caps Lock dropped
- **WHEN** the terminal sends `\x1b[99;69u`
- **THEN** gband reads Ctrl+`c`

#### Scenario: Kitty F1
- **WHEN** the terminal sends `\x1b[P`
- **THEN** gband reads F1

#### Scenario: Ctrl+Enter reaches a program that asked for modifyOtherKeys
- **WHEN** the program in the focused window has written `\x1b[>4;1m` and the terminal sends `\x1b[27;5;13~`
- **THEN** the program reads `\x1b[27;5;13~` from its terminal

#### Scenario: Shift+Enter reaches a program that pushed kitty flags
- **WHEN** the program in the focused window has written `\x1b[>1u` and the terminal sends `\x1b[13;2u`
- **THEN** the program reads `\x1b[13;2u` from its terminal
