## Purpose

Defines the sidebar: the one-column bar that the bundled `gband.sidebar` plugin adds through the bar API to show the active mode, the layout's bands with the viewed one emphasised, and whether the client reports an error.

## ADDED Requirements

### Requirement: Sidebar plugin
gband SHALL bundle the client plugin `gband.sidebar`, named `sidebar`. Its `setup` SHALL add one bar through the bars capability's `gband.bar.add`, with no `id`, `size` 1, the default `hl`, and the `side` and `order` its options give. The bar's full id SHALL therefore be `sidebar`. The plugin SHALL take these options:

| option | value | default |
|---|---|---|
| `side` | `"left"` or `"right"` | `"left"` |
| `order` | a number | `0` |

An option of the wrong type or value, or an unknown option, SHALL make its `setup` raise an error naming the option, and SHALL add no bar. Placing the bar, the ribbon area it leaves, reloads and the removal of the bar when the plugin is marked failed SHALL follow the bars capability.

#### Scenario: Default sidebar
- **WHEN** the client's terminal is 80×24 and `user/init.lua` calls `gband.plugin("gband.sidebar")`
- **THEN** `gband.bar.info("sidebar")` holds `side = "left"`, `size = 1`, `col = 0`, `width = 1` and `height = 24`
- **AND** the ribbon area spans columns 1 to 79

#### Scenario: Sidebar on the right
- **WHEN** the client's terminal is 80×24 and `user/init.lua` calls `gband.plugin("gband.sidebar", { side = "right" })`
- **THEN** the sidebar is drawn on column 79 and the ribbon area spans columns 0 to 78

#### Scenario: Unknown option
- **WHEN** `user/init.lua` calls `gband.plugin("gband.sidebar", { width = 3 })`
- **THEN** the setup raises an error naming `width`, and `gband.bar.list()` holds no bar `sidebar`

#### Scenario: Wrong side
- **WHEN** `user/init.lua` calls `gband.plugin("gband.sidebar", { side = "top" })`
- **THEN** the setup raises an error naming `side`, and no bar is added

### Requirement: Sidebar rows
On a shown sidebar of height `h`, row 0 SHALL hold the mode letter, row 1 SHALL be blank, rows 2 to `h − 2` SHALL hold the band labels, and row `h − 1` SHALL hold the error marker. When `h` is 3 or less, no band label SHALL be drawn. When `h` is 1, the one row SHALL hold the error marker while one is drawn, and the mode letter otherwise. A cell with no content SHALL be blank in the bar's `hl` group.

#### Scenario: Rows of a new session
- **WHEN** the client's terminal is 80×24, the default configuration is in use, the session holds one window in band 1, and no error is reported
- **THEN** column 0 shows `I` on row 0, nothing on row 1, `1` on row 2 and `2` on row 3
- **AND** rows 4 to 23 of column 0 are blank

#### Scenario: Short terminal
- **WHEN** the client's terminal is 80×3, the default configuration is in use and no error is reported
- **THEN** column 0 shows `I` on row 0 and nothing on rows 1 and 2

#### Scenario: Terminal background
- **WHEN** the default configuration and the `default` colorscheme are in use
- **THEN** no cell of column 0 has a background colour, and the blank cells have no foreground colour either

### Requirement: Mode letter
The mode letter SHALL be `I` while the active key table is `root`. While any other table is active, it SHALL be the first character of the label `gband.keymap.label` returns for that table, with an ASCII lowercase letter shown in uppercase. It SHALL be drawn in the group `SidebarMode`, and SHALL change in the frame that follows the change of the active table.

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

### Requirement: Band labels
The sidebar SHALL show one label for each band of the layout, in layout order from row 2 down, the empty last band included. The label of the band at position `p`, counted from 1, SHALL be the digit `p` for `p` from 1 to 9, and the letter at position `p − 9` of `abcdefghijklmnopqrstuvwxyz` for `p` from 10 to 35. A band at a position past 35, or whose row would be `h − 1` or below, SHALL NOT be drawn. The labels SHALL change in the frame that follows a change of the bands.

#### Scenario: Window opened in the empty band
- **WHEN** the layout holds bands 1 and 2, band 2 is empty, and a window opens in band 2
- **THEN** the sidebar shows `1`, `2` and `3` on rows 2, 3 and 4

#### Scenario: Band removed
- **WHEN** the layout holds three bands, the window of band 2 closes, and band 2 is removed
- **THEN** the sidebar shows `1` and `2` on rows 2 and 3, and row 4 is blank

#### Scenario: Tenth band
- **WHEN** the client's terminal is 80×24 and the layout holds eleven bands
- **THEN** rows 2 to 10 show `1` to `9`, row 11 shows `a` and row 12 shows `b`

#### Scenario: More bands than rows
- **WHEN** the client's terminal is 80×8 and the layout holds six bands
- **THEN** rows 2 to 6 show `1` to `5` and the sixth band is not drawn

### Requirement: Viewed band emphasis
The label of the band the client views SHALL be drawn in the group `SidebarBandActive`. Every other label SHALL be drawn in the group `SidebarBand`. The groups SHALL change in the frame that follows a change of the viewed band.

#### Scenario: Viewed band
- **WHEN** the layout holds bands 1, 2 and 3 and the client views band 1
- **THEN** the label `1` is drawn in `SidebarBandActive`, and `2` and `3` in `SidebarBand`

#### Scenario: View moves down
- **WHEN** the client views band 1 of three and the user focuses the band below
- **THEN** the label `2` is drawn in `SidebarBandActive`, and `1` and `3` in `SidebarBand`

### Requirement: Band click
A press of the left button on a cell of a shown sidebar that holds a band label SHALL view the band that label stands for, as `gband.band.view` does, whatever key table is active. A press of another button, or on a cell of the sidebar that holds no band label, SHALL do nothing. A press SHALL NOT change the active key table.

#### Scenario: Click a band label
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 1, and the user presses the left button on column 0, row 4
- **THEN** the client views band 3, and the label `3` is drawn in `SidebarBandActive`

#### Scenario: Click on the right sidebar
- **WHEN** the client's terminal is 80×24, the sidebar is on the right, the layout holds bands 1 and 2, the client views band 1, and the user presses the left button on column 79, row 3
- **THEN** the client views band 2

#### Scenario: Click beside the labels
- **WHEN** the client views band 1 and the user presses the left button on the sidebar's mode letter, its blank row 1, or a row below the last band label
- **THEN** the client still views band 1

#### Scenario: Right click on a label
- **WHEN** the layout holds bands 1 and 2, the client views band 1, and the user presses the right button on the label `2`
- **THEN** the client still views band 1

### Requirement: Error marker
While the client reports an error, as the configuration capability defines, the sidebar's error marker row SHALL show `!` in the group `SidebarError`. Otherwise the row SHALL be blank. The marker SHALL count as drawn only while the sidebar is shown and `!` is in its row. While it is drawn, the client SHALL NOT draw the error banner the configuration capability defines.

#### Scenario: Error reported
- **WHEN** the client's terminal is 80×24, the default configuration is in use, and the user file holds a syntax error
- **THEN** row 23 of column 0 shows `!` in `SidebarError`
- **AND** no banner is drawn over the ribbon area

#### Scenario: Error cleared
- **WHEN** the sidebar shows its error marker and a binding calls `gband.clear_errors()`
- **THEN** row 23 of column 0 is blank

#### Scenario: Sidebar hidden
- **WHEN** an error is reported and the terminal is 1 column wide, so the sidebar is not shown
- **THEN** the error is drawn as the banner on the ribbon area's bottom row

### Requirement: Sidebar groups
The plugin SHALL define these groups as defaults, in the defaults layer the highlights capability defines: `SidebarMode` as `{ bold = true }`, `SidebarBand` as `{ dim = true }`, `SidebarBandActive` as `{ bold = true }`, and `SidebarError` as `{ fg = 1, bold = true }`. A change of a group's resolved style SHALL redraw the sidebar, as the bars capability defines.

#### Scenario: Defaults without a colorscheme
- **WHEN** `user/init.lua` sets up `gband.sidebar` and sets no group
- **THEN** `SidebarBand` resolves to `{ dim = true }` and `SidebarError` to `{ fg = 1, bold = true }`

#### Scenario: Theme the viewed band
- **WHEN** `user/init.lua` calls `gband.hl.set("SidebarBandActive", { fg = 2 })`
- **THEN** the viewed band's label is drawn with foreground 2
