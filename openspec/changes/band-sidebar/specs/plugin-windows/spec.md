## ADDED Requirements

### Requirement: Display width helpers
`gband.ui.width(text)` SHALL return the number of terminal cells `text` takes, with control characters removed: two for each wide character, zero for each zero-width character, and one for each other character. `gband.ui.truncate(text, width)` SHALL return `text`, with its control characters removed, when it takes at most `width` cells. Otherwise it SHALL return the longest prefix that takes at most `width − 1` cells, followed by `…`. When `width` is 0, it SHALL return the empty string.

#### Scenario: Width
- **WHEN** code calls `gband.ui.width("a日b")`
- **THEN** it returns 4

#### Scenario: Truncate at a wide character
- **WHEN** code calls `gband.ui.truncate("日本語", 4)`
- **THEN** it returns `日…`

## MODIFIED Requirements

### Requirement: Plugin window contents
A plugin window's lines SHALL be a list, in which each line is a string or a list of spans. A span SHALL be a string or a table `{ text = <string>, hl = <group name> }`. A string line SHALL be one span. A span without `hl`, or a string span, SHALL use the group `PluginWindow`. Control characters, the code points U+0000 to U+001F, U+007F and U+0080 to U+009F, SHALL be removed from every span's text. `gband.win.set_lines(win, lines)` SHALL replace the plugin window's lines.

A plugin window's **content area** SHALL be the cells it shows its lines in: a floating plugin window's box inside its border, or the whole box when it has no border, or a drawn window's screen. Row `r` of the content area, counted from 0, SHALL show line `top + r`, where `top` is the plugin window's first shown line. The row SHALL show the spans of that line from left to right at their display widths, as "Display width helpers" measures them. The line SHALL be cut at the content area's right edge. A wide character that does not fit wholly SHALL leave its cells blank. Each cell a span covers SHALL have the span group's resolved style applied over `PluginWindow`'s resolved style: a field the span's style sets replaces that field, and the other fields of `PluginWindow` stay. Cells that no span covers SHALL be blank in `PluginWindow`'s resolved style.

#### Scenario: Styled line
- **WHEN** `PluginWindow` resolves to `{ fg = 7 }`, `Key` resolves to `{ fg = 3, bold = true }`, and a 20-column floating plugin window shows the line `{ { text = "C-h", hl = "Key" }, " left" }`
- **THEN** the content row's first three cells read `C-h` with foreground 3 and bold
- **AND** the next five read ` left` with foreground 7, and the rest are blank

#### Scenario: Long line is cut
- **WHEN** a floating plugin window's content area is 5 columns wide and shows the line `abcdefgh`
- **THEN** the row reads `abcde`

#### Scenario: Control characters removed
- **WHEN** a plugin window shows the line `"a\tb"`
- **THEN** the row reads `ab`

