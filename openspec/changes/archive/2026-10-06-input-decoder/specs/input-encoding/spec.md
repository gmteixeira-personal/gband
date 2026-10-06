## ADDED Requirements

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

A cursor key, F1 to F4 as `\x1b[1;mP` to `\x1b[1;mS`, or a tilde key as `\x1b[n;m~` SHALL carry its modifiers in the parameter `m`, as "Modifier parameter" defines. Higher modifier bits, such as Meta, SHALL be dropped. A complete escape sequence that starts as one of these keys, a mouse report, a focus report or a paste does, by `\x1b[` and a digit, `?`, `<`, `[`, `M` or one of the letters `A`, `B`, `C`, `D`, `F`, `H`, `I`, `O` and `Z`, and then names none of them SHALL become no key, and its bytes SHALL NOT reach any window. A valid SGR mouse report of a button other than those "Mouse events from the terminal" names SHALL become no event in the same way.

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

## REMOVED Requirements

### Requirement: Key events from the terminal
**Reason**: gband now reads keys from the bytes the terminal sends, as "Keys from the terminal's bytes" defines. Legacy terminal input cannot report a key release, Super, Hyper, Meta, Caps Lock, a media key or a bare modifier key, so none of them reach gband.
**Migration**: None. gband never enabled the keyboard protocol that reports them, so no terminal sent them to gband before.
