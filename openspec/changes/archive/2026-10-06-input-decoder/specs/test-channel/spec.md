## ADDED Requirements

### Requirement: Input read
The runner SHALL be able to ask the client to answer once it has read a given number of bytes from its terminal's input since it started, and holds no incomplete escape sequence, as the input-encoding capability's "Incomplete input from the terminal" defines. The runner SHALL count every byte written to the client's terminal input, the replies of its own terminal emulator included, and settle's first step SHALL complete through this answer.

#### Scenario: Escape is read before the answer
- **WHEN** the runner writes `\x1b` to the client's terminal and asks the client to answer once it has read every byte written so far
- **THEN** the answer comes after the client has read Escape, about 25 ms later

#### Scenario: Escape then settle
- **WHEN** a runner writes `\x1b` and then settles, 100 times in a row
- **THEN** every settle completes
