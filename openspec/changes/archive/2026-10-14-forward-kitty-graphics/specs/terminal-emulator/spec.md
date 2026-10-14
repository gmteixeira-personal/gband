## ADDED Requirements

### Requirement: Graphics passthrough collection
An emulator SHALL collect the complete kitty graphics APC sequences that appear in the output it processes: each run `ESC _ G <control> ; <payload> ESC \` whose APC string begins with `G`. It SHALL end a sequence at the first `ESC \`, and SHALL hold a sequence whose bytes span several `process` calls until its end arrives. It SHALL report the collected sequences in the order they appeared and clear them when read. Collecting a sequence SHALL leave the screen, the cursor and the input modes unchanged. A test's emulator and a client's emulator MAY collect them too, and the collected sequences SHALL not appear in a snapshot or a diff.

An emulator SHALL ignore an APC string whose first byte is not `G`, and SHALL discard a graphics sequence that has not ended within 16 MiB of buffered bytes.

#### Scenario: A transmit sequence is collected
- **WHEN** an emulator processes `\x1b_Gf=24,s=1,v=1,a=T;AAAA\x1b\\`
- **THEN** it reports one collected graphics sequence holding those bytes
- **AND** its screen is empty

#### Scenario: A sequence split across calls
- **WHEN** an emulator processes `\x1b_Gf=24,s=1,v=1,` and then `a=T;AAAA\x1b\\`
- **THEN** it reports one collected graphics sequence holding the joined bytes

#### Scenario: Collected sequences are cleared when read
- **WHEN** an emulator processes one graphics sequence and its collected sequences are read twice
- **THEN** the first read reports the sequence and the second reports none

#### Scenario: A non-graphics APC string is ignored
- **WHEN** an emulator processes `\x1b_qMODE\x1b\\`
- **THEN** it reports no collected graphics sequence
