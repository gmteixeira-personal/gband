# sidebar Specification

## Purpose
Defines the sidebar: the one-column bar that the bundled `gband.sidebar` plugin adds through the bar API to show the active mode, or the apps character that opens the floating key style's window list, the layout's bands with the viewed one emphasised, and whether the client reports an error.

## Requirements

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
On a shown sidebar of height `h`, row 0 SHALL hold the mode letter, or the apps character as "Apps character" defines, row 1 SHALL be blank, and row `h − 1` SHALL hold the error marker. When `h` is 6 or more, the sidebar SHALL be split in two halves at row `d = ⌊h / 2⌋`: row `d` SHALL hold the divider `─`, U+2500, in the group `SidebarDivider`, the band labels SHALL fill rows 2 to `d − 1` as "Band labels" defines, and the agent labels SHALL fill rows `d + 1` to `h − 2` as "Agent labels" defines. The divider SHALL be drawn whether or not an agent label is drawn. When `h` is 5 or less, no divider and no agent label SHALL be drawn, and the band labels SHALL fill rows 2 to `h − 2`. When `h` is 3 or less, no band label SHALL be drawn. When `h` is 1, the one row SHALL hold the error marker while one is drawn, and row 0's mode letter or apps character otherwise. A cell with no content SHALL be blank in the bar's `hl` group.

#### Scenario: Rows of a new session
- **WHEN** the client's terminal is 80×24, the default configuration is in use, the session holds one window in band 1, and no error is reported
- **THEN** column 0 shows `I` on row 0, nothing on row 1, `1` on row 2, `2` on row 3 and `─` on row 12
- **AND** rows 4 to 11 and 13 to 23 of column 0 are blank

#### Scenario: Rows with agents
- **WHEN** the client's terminal is 80×24, the default configuration is in use, band 1 holds windows 1 and 2, both running Claude Code, band 2 is empty, and no error is reported
- **THEN** column 0 shows `I` on row 0, nothing on row 1, `1` on row 2, `2` on row 3, `─` on row 12, `1` on row 13 and `2` on row 14
- **AND** rows 4 to 11 and 15 to 23 of column 0 are blank

#### Scenario: No divider on a short sidebar
- **WHEN** the client's terminal is 80×5, the default configuration is in use, the session holds one window in band 1, and no error is reported
- **THEN** column 0 shows `I` on row 0, `1` on row 2 and `2` on row 3, and no row shows `─`

#### Scenario: Short terminal
- **WHEN** the client's terminal is 80×3, the default configuration is in use and no error is reported
- **THEN** column 0 shows `I` on row 0 and nothing on rows 1 and 2

#### Scenario: Terminal background
- **WHEN** the default configuration and the `default` colorscheme are in use
- **THEN** no cell of column 0 has a background colour, and the blank cells have no foreground colour either

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

### Requirement: Band labels
The sidebar SHALL show one label for each band of the layout, in layout order from row 2 down, the empty last band included. The label of the band at position `p`, counted from 1, SHALL be the digit `p` for `p` from 1 to 9, and the letter at position `p − 9` of `abcdefghijklmnopqrstuvwxyz` for `p` from 10 to 35. A band at a position past 35 SHALL NOT be drawn, nor a band whose row would be the divider's row or below, as "Sidebar rows" defines, or `h − 1` or below on a sidebar with no divider. The labels SHALL change in the frame that follows a change of the bands.

#### Scenario: Window opened in the empty band
- **WHEN** the layout holds bands 1 and 2, band 2 is empty, and a window opens in band 2
- **THEN** the sidebar shows `1`, `2` and `3` on rows 2, 3 and 4

#### Scenario: Band removed
- **WHEN** the layout holds three bands, the window of band 2 closes, and band 2 is removed
- **THEN** the sidebar shows `1` and `2` on rows 2 and 3, and row 4 is blank

#### Scenario: Tenth band
- **WHEN** the client's terminal is 80×24 and the layout holds eleven bands
- **THEN** rows 2 to 10 show `1` to `9`, row 11 shows `a` and row 12 shows the divider
- **AND** the eleventh band is not drawn

#### Scenario: More bands than rows
- **WHEN** the client's terminal is 80×8 and the layout holds six bands
- **THEN** rows 2 and 3 show `1` and `2`, row 4 shows the divider, and bands 3 to 6 are not drawn

### Requirement: Viewed band emphasis
A band SHALL have an agent state when it holds at least one agent window, as the agent-state capability defines: the most urgent state among its agent windows, in the order `blocked`, `done`, `working`, `idle`, `unknown`. The label of a band with an agent state SHALL be drawn in the group "Agent colors" names for that state, with its `Active` group when the client views the band. The label of a band with no agent state SHALL be drawn in the group `SidebarBandActive` when the client views it, and in `SidebarBand` otherwise. The groups SHALL change in the frame that follows a change of the viewed band or of the state of an agent window.

#### Scenario: Viewed band
- **WHEN** the layout holds bands 1, 2 and 3, no window runs an agent, and the client views band 1
- **THEN** the label `1` is drawn in `SidebarBandActive`, and `2` and `3` in `SidebarBand`

#### Scenario: View moves down
- **WHEN** the client views band 1 of three, no window runs an agent, and the user focuses the band below
- **THEN** the label `2` is drawn in `SidebarBandActive`, and `1` and `3` in `SidebarBand`

#### Scenario: Band of a working agent
- **WHEN** the layout holds bands 1 and 2, band 2 holds one agent window whose state is `working`, and the client views band 1
- **THEN** the band label `2` is drawn in `SidebarAgentWorking`, and `1` in `SidebarBandActive`

#### Scenario: Most urgent agent wins
- **WHEN** band 1 holds agent windows whose states are `idle`, `blocked` and `working`, and the client views band 1
- **THEN** the band label `1` is drawn in `SidebarAgentBlockedActive`

#### Scenario: Band follows its agent
- **WHEN** band 2 holds one agent window whose state is `working`, the client views band 1, and the agent's state becomes `done`
- **THEN** the band label `2` is drawn in `SidebarAgentDone` in the next frame

### Requirement: Band click
A press of the left button on a cell of a shown sidebar that holds a band label SHALL view the band that label stands for, as `gband.band.view` does, whatever key table is active. A press of the left button on a cell that holds an agent label SHALL act as "Agent click" defines. A press of the left button on the apps character SHALL act as "Apps character" defines. A press of the left or the right button on the drawn error marker SHALL act as "Error marker press" defines. A press of another button, or on any other cell of the sidebar, SHALL do nothing. A press SHALL NOT change the active key table, except as "Error marker press" defines.

#### Scenario: Click a band label
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 1, and the user presses the left button on column 0, row 4
- **THEN** the client views band 3, and the label `3` is drawn in `SidebarBandActive`

#### Scenario: Click on the right sidebar
- **WHEN** the client's terminal is 80×24, the sidebar is on the right, the layout holds bands 1 and 2, the client views band 1, and the user presses the left button on column 79, row 3
- **THEN** the client views band 2

#### Scenario: Click beside the labels
- **WHEN** the client views band 1 and the user presses the left button on the sidebar's mode letter, its blank row 1, the divider, or a row below the last label
- **THEN** the client still views band 1

#### Scenario: Click the mode letter's row with the modal style
- **WHEN** the modal key style is in use, the client views band 1, and the user presses the left button on column 0, row 0
- **THEN** the client still views band 1 and no floating plugin window opens

#### Scenario: Right click on a label
- **WHEN** the layout holds bands 1 and 2, the client views band 1, and the user presses the right button on the label `2`
- **THEN** the client still views band 1

#### Scenario: Click the blank error marker row
- **WHEN** the client's terminal is 80×24, the default configuration is in use, the client reports no error, and the user presses the left button on column 0, row 23
- **THEN** no plugin window opens

### Requirement: Band wheel
A wheel step down on any cell of a shown sidebar, with no Ctrl, Alt or Shift held, SHALL view the band below the viewed band, as `focus_band_down` does. A wheel step up there SHALL view the band above, as `focus_band_up` does. Each step SHALL move one band from the band viewed when the step is handled, whatever row is under the pointer and whatever key table is active. A step past the first or the last band SHALL change nothing. A step left or right, or a step while the sidebar is not shown, SHALL do nothing. The sidebar SHALL take no action on a step with Ctrl, Alt or Shift held. A key binding on that step SHALL act as it does elsewhere. With the default keys, an Alt step over the sidebar therefore views one band, as a step without Alt does. A step SHALL NOT change the active key table.

#### Scenario: Wheel down over the labels
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 1, and the user turns the wheel one step down over column 0, row 2
- **THEN** the client views band 2, and the label `2` is drawn in `SidebarBandActive`

#### Scenario: Steps move one band each
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 1, and the user turns the wheel two steps down over the sidebar
- **THEN** the client views band 3

#### Scenario: Wheel up
- **WHEN** the layout holds bands 1, 2 and 3, the client views band 3, and the user turns the wheel one step up over the sidebar
- **THEN** the client views band 2

#### Scenario: Any row of the sidebar
- **WHEN** the layout holds bands 1 and 2, the client views band 1, and the user turns the wheel one step down over the sidebar's mode letter, its blank row 1, or its last row
- **THEN** the client views band 2

#### Scenario: Past the last band
- **WHEN** the layout holds bands 1 and 2, the client views band 2, and the user turns the wheel one step down over the sidebar
- **THEN** the client still views band 2

#### Scenario: Wheel on the right sidebar
- **WHEN** the client's terminal is 80×24, the sidebar is on the right, the layout holds bands 1 and 2, the client views band 1, and the user turns the wheel one step down over column 79
- **THEN** the client views band 2

#### Scenario: Wheel with Alt held
- **WHEN** the default configuration is in use, the layout holds bands 1, 2 and 3, the client views band 1, and the user turns the wheel one step down over the sidebar with Alt held
- **THEN** the client views band 2

#### Scenario: Wheel in navigation mode
- **WHEN** the default configuration is in use, the layout holds bands 1 and 2, the client views band 1, navigation mode is active, and the user turns the wheel one step down over the sidebar
- **THEN** the client views band 2 and row 0 of the sidebar still shows `N`

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
The plugin SHALL define these groups as defaults, in the defaults layer the highlights capability defines:

| group | default |
|---|---|
| `SidebarMode` | `{ bold = true }` |
| `SidebarBand` | `{ dim = true }` |
| `SidebarBandActive` | `{ bold = true }` |
| `SidebarError` | `{ fg = 1, bold = true }` |
| `SidebarAgentBlocked` | `{ fg = "red" }` |
| `SidebarAgentBlockedActive` | `{ fg = "bright_red", bold = true }` |
| `SidebarAgentDone` | `{ fg = "green" }` |
| `SidebarAgentDoneActive` | `{ fg = "bright_green", bold = true }` |
| `SidebarAgentWorking` | `{ fg = "yellow" }` |
| `SidebarAgentWorkingActive` | `{ fg = "bright_yellow", bold = true }` |
| `SidebarAgentIdle` | `{ fg = "blue" }` |
| `SidebarAgentIdleActive` | `{ fg = "bright_blue", bold = true }` |
| `SidebarAgentUnknown` | `{ link = "SidebarBand" }` |
| `SidebarAgentUnknownActive` | `{ link = "SidebarBandActive" }` |
| `SidebarDivider` | `{ link = "SidebarBand" }` |

A change of a group's resolved style SHALL redraw the sidebar, as the bars capability defines.

#### Scenario: Defaults without a colorscheme
- **WHEN** `user/init.lua` sets up `gband.sidebar` and sets no group
- **THEN** `SidebarBand` resolves to `{ dim = true }` and `SidebarError` to `{ fg = 1, bold = true }`
- **AND** `SidebarAgentWorking` resolves to `{ fg = "yellow" }`, and `SidebarAgentUnknown` and `SidebarDivider` to `{ dim = true }`

#### Scenario: Theme the viewed band
- **WHEN** `user/init.lua` calls `gband.hl.set("SidebarBandActive", { fg = 2 })`
- **THEN** the viewed band's label is drawn with foreground 2

#### Scenario: State colors follow the palette
- **WHEN** the agent at position 1 is `blocked`, the client focuses the agent at position 2, which is `blocked` too, and no group is set
- **THEN** the label `1` is drawn with palette index 1, and the label `2` with palette index 9 in bold

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

### Requirement: Agent labels
The sidebar SHALL show one agent label for each entry of `gband.agents.list()`, as the agent-state capability defines, in that order. The label of the agent at position `p`, counted from 1, SHALL be the digit `p` for `p` from 1 to 9, and the letter at position `p − 9` of `abcdefghijklmnopqrstuvwxyz` for `p` from 10 to 35. With the divider on row `d`, as "Sidebar rows" defines, the first agent label SHALL be on row `d + 1`. An agent at a position past 35, or whose row would be `h − 1` or below, SHALL NOT be drawn. A sidebar with no divider SHALL draw no agent label. The labels SHALL change in the frame that follows a change of the list.

#### Scenario: Agent starts
- **WHEN** the client's terminal is 80×24, the layout holds bands 1 and 2, band 2 is empty, and the user runs Claude Code in band 1's only window
- **THEN** within one second the sidebar shows `1` on row 13, below the divider on row 12

#### Scenario: Agent exits
- **WHEN** the sidebar shows one agent label on row 13 and that agent exits
- **THEN** within one second row 13 is blank and row 12 still shows the divider

#### Scenario: Bands and agents share the sidebar
- **WHEN** the client's terminal is 80×8, the layout holds five bands, and one window runs Claude Code
- **THEN** rows 2 and 3 show the band labels `1` and `2`, row 4 shows the divider, and row 5 shows the agent label `1`

#### Scenario: No room for agents
- **WHEN** the client's terminal is 80×5, the layout holds bands 1 and 2, and one window runs Claude Code
- **THEN** rows 2 and 3 show the band labels `1` and `2` and no agent label is drawn

#### Scenario: Agents cut at the last row
- **WHEN** the client's terminal is 80×10, the layout holds bands 1 and 2, and four windows run Claude Code
- **THEN** rows 6 to 8 show the agent labels `1` to `3`
- **AND** the fourth agent is not drawn on row 9

### Requirement: Agent colors
An agent label SHALL be drawn in the group of its agent's state: `SidebarAgentBlocked` for `blocked`, `SidebarAgentDone` for `done`, `SidebarAgentWorking` for `working`, `SidebarAgentIdle` for `idle` and `SidebarAgentUnknown` for `unknown`. The label of the agent whose window this client focuses SHALL be drawn in that state's `Active` group, the group's name followed by `Active`. The groups SHALL change in the frame that follows a change of an agent's state or of the focused window. The label SHALL be the same character in every state.

#### Scenario: Blocked agent
- **WHEN** the agent at position 1 is `blocked` and the client focuses another window
- **THEN** its label `1` is drawn in `SidebarAgentBlocked`

#### Scenario: Focused agent is brighter
- **WHEN** the agent at position 2 is `working` and the client focuses its window
- **THEN** its label `2` is drawn in `SidebarAgentWorkingActive`

#### Scenario: State change recolors
- **WHEN** the agent at position 1 is `working` and its state becomes `blocked`
- **THEN** its label `1` is drawn in `SidebarAgentBlocked` in the next frame, and is still the character `1`

### Requirement: Agent click
A press of the left button on a cell of a shown sidebar that holds an agent label SHALL focus that agent's window, as `gband.window.focus` does, so that the client views the agent's band, whatever key table is active. The press SHALL NOT change the active key table.

#### Scenario: Click an agent in another band
- **WHEN** the client's terminal is 80×24, the layout holds bands 1, 2 and 3, `gband.agents.list()` holds windows 1 and 4, window 4 is in band 2, the client views band 1, and the user presses the left button on column 0, row 14
- **THEN** the client views band 2 with window 4 focused
- **AND** the agent label `2` is drawn in its state's `Active` group

#### Scenario: Click in navigation mode
- **WHEN** navigation mode is active and the user presses the left button on an agent label
- **THEN** that agent's window is focused and row 0 of the sidebar still shows `N`

### Requirement: Error marker press
While the error marker is drawn, as "Error marker" defines, a press of the left button on its cell SHALL dispatch the action `errors.open` when `gband.action` holds it, as the error-list capability defines, whatever key table is active. Opening the error list SHALL make `root` the active table, as the error-list capability defines. A press of the right button on that cell SHALL dispatch the action `errors.clear` when `gband.action` holds it, whatever key table is active, and SHALL NOT change the active key table. Either press SHALL do nothing when `gband.action` does not hold its action. While the sidebar is 1 row high, its one row SHALL be the error marker's cell while the marker is drawn, and a press there SHALL NOT act as "Apps character" defines.

#### Scenario: Left click opens the error list
- **WHEN** the client's terminal is 80×24, the default configuration is in use, the user file holds a syntax error, and the user presses the left button on column 0, row 23
- **THEN** a focused floating plugin window titled `errors  c clear` opens and lists the error
- **AND** `root` is active

#### Scenario: Left click in navigation mode
- **WHEN** the default configuration is in use with the modal key style, the sidebar shows its error marker, navigation mode is active, and the user presses the left button on the marker
- **THEN** the error list opens and `root` is active

#### Scenario: Right click clears the errors
- **WHEN** the client's terminal is 80×24, the default configuration is in use, the sidebar shows its error marker, and the user presses the right button on column 0, row 23
- **THEN** `gband.errors()` returns an empty list and row 23 of column 0 is blank
- **AND** no plugin window opens and the active key table does not change

#### Scenario: Right click with the error list open
- **WHEN** the error list is open as a floating plugin window holding one error, and the user presses the right button on the sidebar's error marker
- **THEN** the error list shows `no errors` and is titled `errors`

#### Scenario: Error list plugin not set up
- **WHEN** `user/init.lua` calls `gband.keystyle.use()`, sets up `gband.sidebar` and not `gband.errors`, a binding function raises an error, and the user presses the left or the right button on the error marker
- **THEN** no plugin window opens and `gband.errors()` still returns the error

#### Scenario: One-row sidebar with the floating style
- **WHEN** the floating key style is in use, the sidebar is 1 row high, the client reports an error, and the user presses the left button on that row
- **THEN** the error list opens and the window list does not
