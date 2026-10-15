## ADDED Requirements

### Requirement: Horizontal and vertical position
The emulator SHALL move the cursor on HVP, `\e[<row>;<col>f`, exactly as it does on CUP, `\e[<row>;<col>H`. Row and column are 1-based. An omitted or zero parameter SHALL default to 1, and the position SHALL be clamped to the screen. HVP and CUP are the same movement in ECMA-48, and a program SHALL be free to position every cell with either.

#### Scenario: HVP positions like CUP
- **WHEN** one emulator processes `\e[2;5fX` and a second of the same size processes `\e[2;5HX`
- **THEN** both show `X` at row 2, column 5, and their cursors are one column past it

#### Scenario: Omitted parameters default to 1
- **WHEN** an emulator at its home position processes `\e[10;10f` then `\e[fTOP`
- **THEN** `TOP` appears at row 1, starting at column 1

#### Scenario: A screen positioned only with HVP
- **WHEN** an emulator processes `\e[1;1fa\e[3;1fb\e[2;1fc`
- **THEN** row 1 column 1 shows `a`, row 2 column 1 shows `c`, and row 3 column 1 shows `b`
