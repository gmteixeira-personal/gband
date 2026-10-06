## ADDED Requirements

### Requirement: Move a window to a place
Moving a tiled window to a place SHALL name a reference window and one of four places: a new column left of the reference window's column, a new column right of it, above the reference window in its column, or below it. When the moved window is not tiled, or the reference window is the moved window or is not a tiled window of the moved window's band, nothing SHALL change.

The window SHALL leave its column as closing removes a window from a column, and the column rule SHALL then apply. It SHALL then be placed:

- For a new column: alone in a new column inserted directly left or right of the reference window's column. The new column SHALL take the width and full-width flag of the column the window left, and the window an automatic height of weight 1.
- Above or below: in the reference window's column, directly above or below the reference window. A window that came from another column SHALL take an automatic height of weight 1. A window that stays in its column SHALL keep its height.

When the result equals the layout before the move, nothing SHALL change.

#### Scenario: Into a new column right of another
- **WHEN** a band holds columns A of width 1/3, B and C, each with one window, and A's window is moved to a new column right of B's window's column
- **THEN** the band holds B, a column of width 1/3 with A's window, and C, in that order

#### Scenario: From a stack into a new column
- **WHEN** column A holds P1 with a fixed height of 16 rows above P2, column B holds P3, and P1 is moved to a new column left of P3's column
- **THEN** column A holds P2 alone with an automatic height of weight 1, and a new column holding P1 with an automatic height of weight 1 sits between A and B with A's width

#### Scenario: Into another column
- **WHEN** column A holds P1 alone and column B holds P2 with a fixed height of 8 rows above P3, and P1 is moved below P2
- **THEN** column A is removed and column B holds P2, P1 with an automatic height of weight 1, and P3, in that order

#### Scenario: Within its column
- **WHEN** a column holds P1 with a fixed height of 16 rows above P2, and P1 is moved below P2
- **THEN** the column holds P2 above P1, and P1 keeps its fixed height of 16 rows

#### Scenario: Back to its own place
- **WHEN** a band holds columns A and B, each with one window, and A's window is moved to a new column left of B's window's column
- **THEN** the layout is unchanged

#### Scenario: Floating window
- **WHEN** a floating window is moved above a tiled window
- **THEN** the layout is unchanged
