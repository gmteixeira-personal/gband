# highlights Specification

## Purpose

Defines highlight groups: named styles that Lua code sets, links and reads, that colorschemes and plugins provide defaults for, and that the client draws with the colors its terminal supports.

## Requirements

### Requirement: Group names and styles
A highlight group SHALL have a name: a string that begins with an ASCII letter and holds only ASCII letters, digits, `_` and `.`. Group names SHALL NOT be namespaced by plugin: every plugin, colorscheme and configuration file addresses the same group by the same name.

A style spec SHALL be a table whose fields are all optional: `fg` and `bg`, each a color; `bold`, `italic`, `underline`, `reverse` and `dim`, each a boolean; and `link`, a group name. A color SHALL be one of:
- a string `#` followed by six hexadecimal digits of either case, giving a 24-bit color;
- an integer from 0 to 255, giving that index of the terminal's 256-color palette;
- one of the names `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white`, `bright_black`, `bright_red`, `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_cyan` and `bright_white`, giving the palette indexes 0 to 15 in that order.

A name that is not a valid group name, a field not listed, a field of the wrong type, a color in none of these forms, or a link to the group itself SHALL be an error at the line of the call that received it, and SHALL leave the group unchanged.

#### Scenario: Valid spec
- **WHEN** `user/init.lua` calls `gband.hl.set("Title", { fg = "#FFAA00", bg = 236, bold = true })`
- **THEN** `gband.hl.get("Title")` returns `{ fg = "#ffaa00", bg = 236, bold = true }`

#### Scenario: Named color
- **WHEN** `user/init.lua` calls `gband.hl.set("Warn", { fg = "bright_red" })`
- **THEN** the group's foreground is palette index 9

#### Scenario: Invalid color
- **WHEN** line 4 of `user/init.lua` calls `gband.hl.set("Title", { fg = "#ffaa0" })`
- **THEN** loading fails with an error at `user/init.lua` line 4 naming `fg`

#### Scenario: Unknown field
- **WHEN** line 2 of a plugin's `setup` calls `gband.hl.set("Title", { colour = 1 })`
- **THEN** a plugin error at that line names `colour`

### Requirement: Explicit and default settings
Each group SHALL hold up to two settings: an explicit setting and a default setting. `gband.hl.set(name, spec)` SHALL replace the group's explicit setting with `spec`. `gband.hl.default(name, spec)` SHALL replace the group's default setting with `spec`. A group's definition SHALL be its explicit setting when it has one, and its default setting otherwise. A group with neither SHALL be undefined. `gband.hl.set(name, {})` SHALL give the group an empty explicit setting, which hides its default. `gband.hl.set(name, nil)` SHALL remove the explicit setting, so the default shows again.

So a default never overrides a colorscheme or a user setting, whichever runs first. Both functions SHALL be callable while the configuration loads and in any callback.

#### Scenario: Default under an explicit setting
- **WHEN** `user/init.lua` calls `gband.hl.set("HelloSegment", { fg = 2 })`, and the plugin `hello` later calls `gband.hl.default("HelloSegment", { fg = 4, bold = true })`
- **THEN** `gband.hl.get("HelloSegment")` returns `{ fg = 2 }`

#### Scenario: Default alone
- **WHEN** only `gband.hl.default("HelloSegment", { fg = 4 })` has been called
- **THEN** `gband.hl.get("HelloSegment")` returns `{ fg = 4 }`

#### Scenario: Removing the explicit setting
- **WHEN** a group has the default `{ fg = 4 }` and the explicit setting `{ fg = 2 }`, and code calls `gband.hl.set` with that name and `nil`
- **THEN** `gband.hl.get` returns `{ fg = 4 }` for the group

### Requirement: Links
A group whose definition holds `link` SHALL inherit from the linked group. Its resolved style SHALL be the linked group's resolved style, with every field that the group's own definition sets replacing the inherited field. A link to an undefined group SHALL inherit nothing. The link SHALL be followed again each time the style is resolved, so a later change to the linked group reaches every group linked to it.

A call to `gband.hl.set` or `gband.hl.default` that would make the chain of definitions starting at its group return to that group SHALL be an error at the line of the call naming the groups of the cycle, and SHALL leave the group unchanged. When resolving a style meets a group already in the chain, which a change of another group's settings can cause, resolution SHALL stop at that link as if it were absent, and the process SHALL record a warning in its log naming the groups of the cycle.

#### Scenario: Inheritance with an override
- **WHEN** `Base` is `{ fg = 1, bg = 2, bold = true }` and `Child` is `{ link = "Base", fg = 3 }`
- **THEN** `Child` resolves to `{ fg = 3, bg = 2, bold = true }`

#### Scenario: Chain
- **WHEN** `A` links to `B`, `B` links to `C`, and `C` is `{ fg = 5 }`
- **THEN** `A` resolves to `{ fg = 5 }`

#### Scenario: Later change of the target
- **WHEN** `Child` links to `Base`, `Base` is `{ fg = 1 }`, and a callback sets `Base` to `{ fg = 6 }`
- **THEN** `Child` resolves to `{ fg = 6 }`

#### Scenario: Cycle refused
- **WHEN** `A` links to `B`, `B` links to `C`, and line 7 of `user/init.lua` sets `C` to `{ link = "A" }`
- **THEN** loading fails with an error at `user/init.lua` line 7 naming `A`, `B` and `C`

#### Scenario: Cycle made across layers
- **WHEN** `A` has the default `{ link = "B" }`, `B` has the explicit setting `{ fg = 1 }` and the default `{ link = "A" }`, and a colorscheme switch clears the explicit setting of `B`
- **THEN** `A` resolves to an empty style
- **AND** the log records a warning naming `A` and `B`

### Requirement: Reading groups
`gband.hl.get(name)` SHALL return a copy of the group's definition, with colors given as `set` accepts them: hex colors in lowercase, and named colors as their names. It SHALL return nil for an undefined group. `gband.hl.get(name, { resolve = true })` SHALL return the group's resolved style instead, with no `link` field, or an empty table for an undefined group. Changing a returned table SHALL NOT change the group.

#### Scenario: Definition and resolved style
- **WHEN** `Base` is `{ fg = 1 }` and `Child` is `{ link = "Base", bold = true }`
- **THEN** `gband.hl.get("Child")` returns `{ link = "Base", bold = true }`
- **AND** `gband.hl.get("Child", { resolve = true })` returns `{ fg = 1, bold = true }`

#### Scenario: Undefined group
- **WHEN** no setting names `Absent`
- **THEN** `gband.hl.get("Absent")` returns nil and `gband.hl.get("Absent", { resolve = true })` returns an empty table

### Requirement: Highlight change events
After the configuration has loaded, each call to `gband.hl.set` or `gband.hl.default` that changes a group's settings SHALL emit `HighlightChanged` with the group's name, as the lua-events capability defines. Calls while the configuration loads, and settings a colorscheme makes as it loads, SHALL NOT emit it.

#### Scenario: Change in a callback
- **WHEN** a binding function calls `gband.hl.set("StatusLineAccent", { fg = 3 })`
- **THEN** `HighlightChanged` runs once with `group` `StatusLineAccent`

#### Scenario: No event during the load
- **WHEN** `user/init.lua` registers a `HighlightChanged` handler and then calls `gband.hl.set("Title", { fg = 1 })`
- **THEN** the handler does not run

### Requirement: Drawn colors
The client SHALL draw a resolved style with each set field applied: `fg` and `bg` as the foreground and background color, and each true boolean as that attribute. Fields the style does not set SHALL leave the terminal's default.

The client SHALL support 24-bit color when its own environment's `COLORTERM` is `truecolor` or `24bit`, and SHALL NOT support it otherwise. When the client supports 24-bit color, it SHALL draw a hex color as that 24-bit color. When it does not, it SHALL draw a hex color as the palette index from 16 to 255 whose standard xterm color is nearest to it by squared distance in RGB, the lower index winning a tie. The standard xterm colors SHALL be the 6×6×6 cube at indexes 16 to 231, with the channel levels 0, 95, 135, 175, 215 and 255, and the gray ramp at indexes 232 to 255, with the level `8 + 10 × (index − 232)` on every channel. Index and named colors SHALL be drawn as their palette index in both cases.

#### Scenario: Truecolor terminal
- **WHEN** the client's `COLORTERM` is `truecolor` and a drawn style's foreground is `#ff8800`
- **THEN** the client draws the 24-bit color `#ff8800`

#### Scenario: Downgrade without truecolor
- **WHEN** `COLORTERM` is unset in the client's environment and a drawn style's foreground is `#ff8800`
- **THEN** the client draws palette index 208

#### Scenario: Gray downgrade
- **WHEN** the client does not support 24-bit color and a drawn style's background is `#303030`
- **THEN** the client draws palette index 236

#### Scenario: Index unchanged
- **WHEN** the client does not support 24-bit color and a drawn style's foreground is `4`
- **THEN** the client draws palette index 4
