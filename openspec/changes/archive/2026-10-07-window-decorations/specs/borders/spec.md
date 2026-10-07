## MODIFIED Requirements

### Requirement: Drawing a border
A border SHALL take the outermost cell on every side of its box, whichever sides are drawn. The box's interior SHALL be the box less one column on the left and on the right, and one row at the top and at the bottom, whatever the sides are.

- A drawn side SHALL fill its row or column with its side character, except at its corners.
- A corner SHALL be drawn with its corner character when both sides that meet there are drawn. When only one of them is drawn, the corner SHALL be drawn with that side's character, so the side reaches the box's edge. When neither is drawn, the corner SHALL be blank.
- Every cell of a side that is not drawn SHALL be blank.

Border cells SHALL be drawn in the border's style, blank cells included.

Text that another capability draws on a border after the border, such as a window's title as the window-names capability defines it, a window's decorations as the window-decorations capability defines them, or a floating plugin window's title as the plugin-windows capability defines it, SHALL replace the cells it covers. Every other border cell SHALL stay as this requirement defines it.

#### Scenario: Only the left side
- **WHEN** a 10×5 box is drawn with the sides `left` and the `plain` set
- **THEN** column 0 shows `│` on rows 0 to 4
- **AND** row 0, row 4 and column 9 are blank, and the interior is columns 1 to 8 and rows 1 to 3

#### Scenario: Top and left sides
- **WHEN** a 10×5 box is drawn with the sides `top` and `left` and the `rounded` set
- **THEN** cell (0, 0) shows `╭`, row 0 shows `─` from column 1 to column 9, and column 0 shows `│` from row 1 to row 4

#### Scenario: No sides
- **WHEN** a box is drawn with no sides
- **THEN** its outermost cells are blank and its interior keeps its size

#### Scenario: Decorations keep the corner cells
- **WHEN** a tile 20 columns wide is drawn with all four sides and the `rounded` set, and its decorations are the span `[X]`
- **THEN** its top row shows `╭` on column 0, `[X]` from column 15 to column 17, `─` on column 18 and `╮` on column 19
