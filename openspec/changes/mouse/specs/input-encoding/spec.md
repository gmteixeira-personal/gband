## ADDED Requirements

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
