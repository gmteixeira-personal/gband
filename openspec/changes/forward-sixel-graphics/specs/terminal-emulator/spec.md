## ADDED Requirements

### Requirement: Sixel passthrough collection
An emulator SHALL collect the complete sixel device control strings that appear in the output it processes: each run `ESC P <params> q <payload> ESC \` whose parameter bytes are only decimal digits and `;` and whose final byte before the payload is `q`. It SHALL end a sequence at the first `ESC \`, and SHALL hold a sequence whose bytes span several `process` calls until its end arrives. With each collected sequence it SHALL report the position of the cursor, 1-based from the top-left of the screen, as it stood when the sequence began. It SHALL report the collected sequences in the order they appeared and clear them when read. Collecting a sequence SHALL leave the screen, the cursor and the input modes unchanged, and the collected sequences SHALL not appear in a snapshot or a diff.

An emulator SHALL ignore a device control string that is not a sixel, including one holding an intermediate byte such as `$` before its final byte, and SHALL discard a sixel sequence that has not ended within 16 MiB of buffered bytes.

#### Scenario: A sixel sequence is collected with its position
- **WHEN** an emulator processes `\x1b[3;5H` and then `\x1bPq#0;2;0;0;0#0~~\x1b\\`
- **THEN** it reports one collected sixel sequence holding the sixel bytes, with the position row 3, column 5
- **AND** its screen shows nothing from the sixel

#### Scenario: A sequence split across calls
- **WHEN** an emulator processes `\x1bPq#0;2;0;0;0` and then `#0~~\x1b\\`
- **THEN** it reports one collected sixel sequence holding the joined bytes

#### Scenario: Collected sequences are cleared when read
- **WHEN** an emulator processes one sixel sequence and its collected sequences are read twice
- **THEN** the first read reports the sequence and the second reports none

#### Scenario: A non-sixel device control string is ignored
- **WHEN** an emulator processes `\x1bP$qm\x1b\\`
- **THEN** it reports no collected sixel sequence
