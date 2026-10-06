## MODIFIED Requirements

### Requirement: Hints segment plugin
gband SHALL bundle the plugin module `gband.statusline.hints`, whose plugin is named `hints`. Its `setup` SHALL add one component under the plugin's name with `gband.ui.statusline.add`, with these settings:

| field | value |
|---|---|
| `align` | `"top"` |
| `priority` | `0` |
| `order` | `30` |
| `hl` | `"KeyHintLabel"` |
| `redraw_on` | `KeyTableChanged` |
| `fill` | true |

`setup` SHALL take the options `align`, `priority` and `order`, which replace the defaults in the table, `labels`, a table from action names to strings or `false`, and `root`, a boolean that is true by default. An option of the wrong type or value SHALL make `setup` raise an error.

#### Scenario: Component entry
- **WHEN** `user/init.lua` calls `gband.plugin("gband.statusline.hints")` and reads `gband.ui.statusline.list()`
- **THEN** it holds an entry with `id` `hints`, `align` `top`, `priority` 0, `order` 30, `hl` `KeyHintLabel` and `plugin` `hints`

#### Scenario: Invalid option
- **WHEN** `user/init.lua` calls `gband.plugin("gband.statusline.hints", { labels = 3 })`
- **THEN** `gband.plugin` returns `false` and a plugin error names `hints`

### Requirement: Hints of the active table
The segment SHALL show one hint for each binding of the active key table, in the order `gband.keymap.list` returns them. While `root` is active, the hints SHALL start with one hint for the prefix key, labelled with `gband.keymap.label("prefix")`, shown only when the `prefix` table holds a binding, followed by the bindings of `root`. While `root` is active and the `root` option is false, the segment SHALL be hidden. A binding left out by "Hint labels" SHALL take no hint. When no hint is left, the segment SHALL be hidden.

Each hint SHALL be the key, as "Key form" shows it, in the group `KeyHintKey`, one space, and the label in the group `KeyHintLabel`. Hints SHALL be laid out in lines, as "Fit to the available width" defines.

#### Scenario: Root with the defaults
- **WHEN** the default configuration is in use and `root` is active
- **THEN** row 0 of the status line shows `band 1` and row 1 shows `C-space navigation`

#### Scenario: Prefix table that is not a mode
- **WHEN** `user/init.lua` binds `prefix h` to `gband.action.focus_column_left`, declares no mode, sets up the hints segment, and `root` is active
- **THEN** the segment shows `C-space prefix`

#### Scenario: Prefix table with the defaults
- **WHEN** the default configuration is in use, the client's terminal is 80×40, and the user presses Ctrl+Space
- **THEN** the segment's first lines are `h left  l right`, `j down  k up`, `u band down` and `i band up  c center`

#### Scenario: Hints stay in navigation mode
- **WHEN** navigation mode's hints are shown and the user presses `h`
- **THEN** the segment still shows navigation mode's hints

#### Scenario: Back to root
- **WHEN** navigation mode's hints are shown and the user presses Escape
- **THEN** the segment shows `C-space navigation` again

#### Scenario: Named table
- **WHEN** `user/init.lua` binds `h` with description `west` and `l` with description `east` in the table `move`, and a binding enters `move`
- **THEN** the segment shows `h west  l east`

#### Scenario: No prefix bindings
- **WHEN** `user/init.lua` binds only `alt+h` in `root` to `gband.action.focus_column_left` and sets up the hints segment
- **THEN** the segment shows `A-h left`

#### Scenario: Root hints turned off
- **WHEN** the hints segment is set up with `{ root = false }` and `root` is active
- **THEN** the segment is hidden
- **AND** it shows navigation mode's hints after Ctrl+Space

#### Scenario: Groups
- **WHEN** the segment shows `C-space navigation`
- **THEN** `C-space` is drawn in `KeyHintKey` and ` navigation` in `KeyHintLabel`

### Requirement: Fit to the available width
The segment SHALL lay its hints out in lines no wider than its context's `width`, in no more lines than its context's `height`. Hints SHALL be placed in order, separated on a line by two spaces in `KeyHintLabel`. A hint that does not fit after the hints already on a line SHALL start the next line. Layout SHALL stop at the first hint that does not fit on an empty line, or that would need a line beyond `height`. When hints are left out, the last line SHALL end with a space and `…` in `KeyHintLabel`, and SHALL still fit in `width`, leaving out further hints from its end when needed. When no hint fits, the segment SHALL be hidden. Widths SHALL be display widths, as `gband.ui.width` measures them.

#### Scenario: Some hints left out
- **WHEN** the prefix table's default hints are shown and the segment's context has `width` 20 and `height` 2
- **THEN** the segment shows the lines `h left  l right` and `j down  k up …`

#### Scenario: All hints fit
- **WHEN** the active table `move` has the hints `h west` and `l east` and the segment's context has `width` 14 and `height` 1
- **THEN** the segment shows the line `h west  l east`

#### Scenario: Nothing fits
- **WHEN** the segment's context has `width` 3 and the first hint is `C-space prefix`
- **THEN** the segment is hidden

#### Scenario: No rows left
- **WHEN** the segment's context has `height` 0
- **THEN** the segment is hidden

### Requirement: Default setup
The default configuration SHALL set up `gband.statusline.hints` with `gband.plugin` and no options, after `gband.statusline.mode` and before `gband.statusline.position`.

#### Scenario: Hints in the default line
- **WHEN** no `user/init.lua` exists and a client attaches with an 80×24 terminal
- **THEN** row 0 shows `band 1`, row 1 shows `C-space navigation`, and row 23 shows the position, each from column 0
