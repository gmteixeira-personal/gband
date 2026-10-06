## MODIFIED Requirements

### Requirement: Bars narrow the screen area
Bars SHALL narrow the client's reported size to the size of the ribbon area they leave, as the client-attach capability defines the reported size. While that client's report sets the session's screen area, as the session-server capability defines, the columns, the floating boxes and every window's terminal size SHALL therefore be measured against the space the bars leave, not the whole terminal. Every change of the ribbon area's size that the bars cause SHALL be reported to the server as a resize, as the client-attach capability's "Input to the server" defines, and SHALL be handled by the client's view and drawn state as the layout-view and animations capabilities handle a change of the client's terminal size. A change that moves the ribbon area and keeps its size SHALL be handled by the view and drawn state in the same way, and SHALL NOT be reported. Changing a bar's lines or `hl` SHALL NOT change the ribbon area. Several changes made by one callback or one load SHALL be handled as one change, and reported as at most one resize.

#### Scenario: Bar added by a key
- **WHEN** the client's terminal is 80×24, the client sets the screen area, the client has no bar, the only window sits in a column of width 1/2, and a binding function adds a left bar of size 20
- **THEN** the client reports the size 60×24, and the tile is 30 columns wide and drawn on screen columns 20 to 49
- **AND** once the server has settled, `tput cols` in the window prints `28`

#### Scenario: Camera follows the narrower ribbon
- **WHEN** the client's terminal is 80×24, the client sets the screen area, the client has no bar, the viewed band holds two columns of width 1/2 with the second focused, and a binding function adds a left bar of size 20
- **THEN** each tile is 30 columns wide, and the second column is still fully shown, on screen columns 50 to 79

#### Scenario: Full width beside a bar
- **WHEN** the client's terminal is 80×24, the client sets the screen area, its only bar is a left bar of size 20, and the only window sits in a column with full width on
- **THEN** the client reports the size 60×24, and the tile is 60 columns wide and drawn on screen columns 20 to 79 with its right border on column 79

#### Scenario: Full width follows the bar's size
- **WHEN** the client's terminal is 80×24, the client sets the screen area, its only bar is a left bar of size 20, the only window sits in a column with full width on, and a binding function calls `gband.bar.set_config(id, { size = 10 })` on it
- **THEN** the client reports the size 70×24, and the tile is 70 columns wide and drawn on screen columns 10 to 79

#### Scenario: Bar moved to the other side
- **WHEN** the client's terminal is 80×24, its only bar is a left bar of size 20, and a binding function calls `gband.bar.set_config(id, { side = "right" })` on it
- **THEN** the client reports no resize, and the ribbon area moves to columns 0 to 59

#### Scenario: Bar too wide to show
- **WHEN** the client's terminal is 10×24 and its only bar is a left bar of size 10
- **THEN** the bar is not shown and the client reports the size 10×24
