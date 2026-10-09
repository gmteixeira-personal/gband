## MODIFIED Requirements

### Requirement: Mode letter
While the floating key style is in use, as "Apps character" defines, row 0 SHALL show the apps character in place of the mode letter. Otherwise, the mode letter SHALL be `I` while the active key table is `root` and multi mode is off, and `M` while the active key table is `root` and multi mode is on, as the multi-mode capability defines. While any other table is active, it SHALL be the first character of the label `gband.keymap.label` returns for that table, with an ASCII lowercase letter shown in uppercase. It SHALL be drawn in the group `SidebarMode`, and SHALL change in the frame that follows the change of the active table or of multi mode.

#### Scenario: Navigation mode
- **WHEN** the default configuration is in use and the user presses Ctrl+Space
- **THEN** row 0 of the sidebar shows `N`

#### Scenario: Back to interactive mode
- **WHEN** navigation mode is active and the user presses Escape
- **THEN** row 0 of the sidebar shows `I`

#### Scenario: Prefix without a mode
- **WHEN** the direct key style is in use, so `prefix` is not a mode, and the user presses Ctrl+Space
- **THEN** row 0 of the sidebar shows `P` until the key sequence ends

#### Scenario: User mode
- **WHEN** `user/init.lua` declares `resize` a mode with the label `resize`, and a binding enters `resize`
- **THEN** row 0 of the sidebar shows `R`


#### Scenario: Multi mode
- **WHEN** the default configuration is in use and the user presses Ctrl+Space then `m`
- **THEN** row 0 of the sidebar shows `M`
- **AND** after Ctrl+Space it shows `N`, after Escape `M` again, and after Ctrl+Space then `m` it shows `I`

#### Scenario: Multi mode with the floating style
- **WHEN** the floating key style is in use and multi mode is on
- **THEN** row 0 of the sidebar shows the apps character

### Requirement: Apps character
The floating key style SHALL be in use while `gband.keystyle.current()` returns `floating`, as the key-style capability defines. While it is in use, row 0 of a shown sidebar SHALL show the apps character `∷`, U+2237, in the group `SidebarMode`, whatever key table is active. A press of the left button on that cell SHALL dispatch the action `desktop.list`, which opens the window list centred in the ribbon area, as the floating-key-style capability's "Window list" defines, when `gband.action` holds `desktop.list`, and SHALL do nothing otherwise. The press SHALL NOT change the active key table. Row 0 SHALL show the mode letter again when a load in which the floating key style is not in use succeeds.

#### Scenario: Apps character with the floating style
- **WHEN** the client's terminal is 80×24, no `user/init.lua` exists, and `user/keystyle.lua` holds `return "floating"`
- **THEN** column 0 shows `∷` on row 0 in `SidebarMode`, nothing on row 1, and the band labels from row 2

#### Scenario: Apps character after the leader
- **WHEN** the floating style is in use with the default configuration and the user presses Ctrl+Space
- **THEN** row 0 of the sidebar shows `∷`

#### Scenario: Open the window list from the sidebar
- **WHEN** the floating style is in use with the default configuration, the viewed band holds floating windows named `notes` and `logs`, which this client focused in that order, and the user presses the left button on column 0, row 0
- **THEN** the window list is open and focused, centred in the ribbon area, with the rows `n New window`, `s Settings`, `m Multi mode`, a separator, `1 logs` and `2 notes` between its borders

#### Scenario: Floating preset required by name
- **WHEN** `user/init.lua` requires `gband.keystyle.floating` by name and sets up `gband.sidebar`
- **THEN** row 0 of the sidebar shows `I`
