## MODIFIED Requirements

### Requirement: Layout
Each shown component SHALL be placed in the region its `align` names. Within a region, components SHALL be placed in ascending `order`, and in the order they were added when their `order` is equal. Adjacent components in a region SHALL be separated by the value of the `statusline_separator` option. The width of a region SHALL be the width of its components and separators. Two adjacent non-empty regions SHALL be at least one cell apart. All widths SHALL be display widths, as `gband.ui.width` measures them.

When the regions with their gaps are wider than the status line, the shown component with the lowest `priority` SHALL be dropped, the one added last when several share it. Dropping SHALL repeat until the line fits or one component is left. When one component is left and it is still wider than the status line, it SHALL be cut to the status line's width as `gband.ui.truncate` cuts it. Dropping SHALL change only what is drawn, and SHALL NOT disable a component.

The left region SHALL start at the line's first column, and the right region SHALL end at its last column. The center region SHALL start at half of the line's width less half of its own width, rounded down, and SHALL then be moved the least distance that keeps it at least one cell from the left and right regions.

#### Scenario: Default segments
- **WHEN** the default configuration is in use, the client's terminal is 80×24, the viewed band is the first, the second of three columns is focused, and `root` is active
- **THEN** row 23 shows `band 1` from column 0 and `2/3` ending at column 79

#### Scenario: Mode shown
- **WHEN** the default configuration is in use and the user presses Ctrl+Space
- **THEN** the left region shows `band 1`, the separator, then `navigation`

#### Scenario: Lowest priority dropped first
- **WHEN** the line is 18 cells wide, the separator is ` │ `, and the left region holds A of priority 1 with output `aaaaaaaa` and B of priority 2 with output `bbbbbbbb`
- **THEN** only `bbbbbbbb` is drawn

#### Scenario: Last component cut
- **WHEN** the line is 6 cells wide and the only component returns `abcdefghij`
- **THEN** the line shows `abcde…`

#### Scenario: Wide characters
- **WHEN** the line is 5 cells wide and the only component returns `日本語`
- **THEN** the line shows `日本…`, which takes 5 cells

#### Scenario: Center region
- **WHEN** the line is 40 cells wide and the center region holds one component returning `mid`
- **THEN** `mid` starts at column 18

### Requirement: Bundled segment plugins
gband SHALL bundle these plugin modules, each set up with `gband.plugin` and each adding one component under its plugin's name with `gband.ui.statusline.add`:

| module | plugin | output | redraw on | align | priority | order | group |
|---|---|---|---|---|---|---|---|
| `gband.statusline.band` | `band` | `band ` and the viewed band's index | `BandChanged`, `LayoutChanged` | left | 20 | 10 | `StatusLineSegment` |
| `gband.statusline.mode` | `mode` | the active key table's label, as `gband.keymap.label` returns it; hidden while `root` is active | `KeyTableChanged` | left | 30 | 20 | `StatusLineAccent` |
| `gband.statusline.position` | `position` | the focused column's index, `/`, and the band's column count; hidden while the viewed band is empty | `FocusChanged`, `BandChanged`, `LayoutChanged` | right | 10 | 10 | `StatusLineMuted` |
| `gband.statusline.clock` | `clock` | the local time, formatted by `os.date` with `opts.format`, `"%H:%M"` by default | every `opts.interval` milliseconds, 1000 by default | right | 5 | 20 | `StatusLineMuted` |

Each SHALL take the options `align`, `priority`, `order` and `hl`, which replace the defaults in the table. An option of the wrong type or value SHALL make its `setup` raise an error. The default configuration SHALL set up `band`, `mode` and `position`, and SHALL NOT set up `clock`.

#### Scenario: Reorder a segment
- **WHEN** `user/init.lua` calls `gband.plugin("gband.statusline.mode", { align = "right", order = 1 })` and `gband.plugin("gband.statusline.position")`, and the user presses Ctrl+Space
- **THEN** the right region shows `prefix`, the separator, then the position

#### Scenario: Label of a mode
- **WHEN** `user/init.lua` declares `resize` a mode with the label `RESIZE`, sets up `gband.statusline.mode`, and a binding enters `resize`
- **THEN** the mode segment shows `RESIZE`

#### Scenario: Clock
- **WHEN** `user/init.lua` calls `gband.plugin("gband.statusline.clock", { format = "%H:%M:%S" })`
- **THEN** the right region shows the local time, and it changes every second

#### Scenario: User file without segments
- **WHEN** `user/init.lua` sets up no segment plugin and adds no component
- **THEN** the status line is drawn in `StatusLine` with no text
