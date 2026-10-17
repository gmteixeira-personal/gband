## ADDED Requirements

### Requirement: Record the layout entering the floating style

When the settings window's `keys` line switches a style other than `floating` to `floating`, the settings SHALL record the client's layout, as `gband.layout()` gives it, before the reload the switch causes floats the windows. The record SHALL hold only windows that run a program, as the floating-key-style capability's "Windows float" defines. It SHALL hold, for each band, its tiled columns left to right with each column's `width`, `full_width` and `tabbed` state and each column's windows top to bottom with each window's height, and its floating windows with each window's box, `col`, `row`, `width`, `full_width` and `rows`. A column that holds no window that runs a program SHALL NOT be recorded.

The record SHALL persist across the reload the switch causes and across the settings window closing and reopening. It SHALL be replaced when the `keys` line switches a non-floating style to `floating` again, and discarded when the `keys` line switches `floating` to another style, as "Restore the layout leaving the floating style" defines. The `keys` line SHALL still save the key style as "Settings window keys" defines.

#### Scenario: A record is kept entering floating
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds columns with windows 1 and 2, the settings window is open on its third line, and the user presses `h`
- **THEN** `user/keystyle.lua` holds `return "floating"`
- **AND** after the reload both windows float

### Requirement: Restore the layout leaving the floating style

When the settings window's `keys` line switches `floating` to `modal` or `direct` and a record exists, as "Record the layout entering the floating style" defines, the settings SHALL restore that layout, then discard the record. Each recorded window still in the client's layout SHALL return to the band, and tiled or floating state, the record holds for it:

- A window the record holds tiled SHALL tile in its recorded band, in a column at the recorded position from the left, at the recorded position from the top within that column, so a column's windows keep their order and stacking. Its column SHALL take the recorded `width`, `full_width` and `tabbed` state, and the window SHALL take its recorded height.
- A window the record holds floating SHALL float with its recorded box, `col`, `row`, `width`, `full_width` and `rows`.
- A floating window that runs a program and is not in the record, such as one opened while `floating` was in use, SHALL tile after the restored tiled windows of its band.

A recorded window no longer floating in the client's layout SHALL be skipped. When the `keys` line switches `floating` to another style and no record exists, every floating window that runs a program SHALL tile. A floating window that runs no program, such as the settings window, SHALL stay floating. The `keys` line SHALL still save the key style as "Settings window keys" defines.

#### Scenario: Columns restored across a round trip
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds columns with windows 1 and 2, the settings window is open on its third line, the user presses `h`, and after the reload presses `l`
- **THEN** `user/keystyle.lua` holds `return "modal"`
- **AND** after the reload the band holds columns with windows 1 and 2, in that order, and no window floats

#### Scenario: A stacked column restored
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds one column with windows 1 and 2 stacked, the settings window is open on its third line, the user presses `h`, and after the reload presses `l`
- **THEN** after the reload the band holds one column with windows 1 and 2 stacked, in that order

#### Scenario: A column width restored
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds a column of width 1/3 with window 1 and a column with window 2, the settings window is open on its third line, the user presses `h`, and after the reload presses `l`
- **THEN** after the reload window 1's column has width 1/3

#### Scenario: A window opened while floating tiles after the restored windows
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds one column with window 1, the settings window is open on its third line, the user presses `h`, after the reload a floating window 2 opens, and the user presses `l`
- **THEN** after the reload the band holds columns with windows 1 and 2, window 1 before window 2, and no window floats

#### Scenario: A previously floating window keeps its box
- **WHEN** no `user/init.lua` exists, the modal style is saved, the viewed band holds a column with window 1 and a floating window 3 with `col` 5, `row` 3, width 1/3 and `rows` 10, the settings window is open on its third line, the user presses `h`, and after the reload presses `l`
- **THEN** after the reload window 1 tiles and window 3 floats with `col` 5, `row` 3, width 1/3 and `rows` 10

#### Scenario: No record tiles every floating window
- **WHEN** `user/keystyle.lua` holds `return "floating"`, no `user/init.lua` exists, the viewed band holds floating windows 1 and 2, the settings window is open on its third line, and the user presses `l`
- **THEN** `user/keystyle.lua` holds `return "modal"`
- **AND** after the reload windows 1 and 2 tile and no window floats
