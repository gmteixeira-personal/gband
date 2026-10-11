## MODIFIED Requirements

### Requirement: Reading structure
`gband.sessions()` SHALL return the names of the sessions in ascending byte order. `gband.session(name)` SHALL return `{ name, bands, clients }` for the session `name`, or nil for an unknown session. `bands` SHALL list its bands from the top, each `{ band, columns, floating }`, each column `{ width, full_width, tabbed, windows }` from the left, where `tabbed` is a boolean, as the tabbed-columns capability defines it,, and each window `{ window }` from the top. `floating` SHALL list the band's floating windows in its floating list order, each `{ window, width, full_width, rows, col, row }`, where `width` and `full_width` are its box record's, `rows` is its box record's height, and `col` and `row` are its box's top-left cell as the floating-windows capability places it in the session's screen area. `clients` SHALL list the numbers of the clients attached to it in ascending order. Each call SHALL return new tables.

#### Scenario: Layout of a session
- **WHEN** the session `work` holds one band with two columns, holding windows 1 and 2, and one client is attached
- **THEN** `gband.session("work").bands[1].columns[2].windows[1].window` is 2
- **AND** `gband.session("work").clients` holds one number

#### Scenario: Floating window in the structure
- **WHEN** the screen area of `work` is 80×24, and band 1 holds a column with window 1 and floating window 2 with `col` 70, `row` 3, width 1/2, full width off and `rows` 24
- **THEN** `gband.session("work").bands[1].floating` is `{ { window = 2, width = 0.5, full_width = false, rows = 24, col = 40, row = 0 } }`

#### Scenario: Tabbed column in the structure
- **WHEN** band 1 of `work` holds one tabbed column with windows 1 and 2
- **THEN** `gband.session("work").bands[1].columns[1].tabbed` is `true`
