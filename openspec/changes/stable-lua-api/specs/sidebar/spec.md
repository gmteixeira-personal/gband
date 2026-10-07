## MODIFIED Requirements

### Requirement: Error marker
While the client reports an error, as the configuration capability defines, the sidebar's error marker row SHALL show `!` in the group `SidebarError`. Otherwise the row SHALL be blank. The sidebar SHALL redraw the row when the error list changes, from its handler of the lua-events capability's `ErrorsChanged`, and SHALL learn whether the client reports an error from `gband.errors()`. Each time it draws, the sidebar SHALL call `gband.bar.mark_errors` for its bar, as the bars capability's "Error marker bars" defines, with `true` exactly when `!` is in its row, so the client draws no error banner while the marker is drawn.

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

#### Scenario: Error reported by a binding
- **WHEN** the default configuration is in use on an 80×24 terminal and a binding function raises `boom`
- **THEN** row 23 of column 0 shows `!` once the binding returns, and no banner is drawn
