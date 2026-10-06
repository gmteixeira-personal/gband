## MODIFIED Requirements

### Requirement: Camera scroll
When the camera of the viewed band changes, the client SHALL animate the drawn camera from its drawn position to the new camera position. Tiles SHALL be drawn at their strip position less the drawn camera. While the band's strip loops, as the layout-view capability defines, the drawn camera SHALL scroll in the direction and over the distance that the camera moved before it was held within the strip, and each tile SHALL be drawn at its drawn copy. Holding the camera within the strip SHALL NOT make the drawn camera jump or scroll the other way.

#### Scenario: Scroll right
- **WHEN** the terminal is 80 columns wide, a band holds three columns of 40 cells, the camera is at 0 at rest, and focus moves from the second column to the third
- **THEN** the first frame draws the camera at 0
- **AND** later frames draw it between 0 and 40 until it reaches 40

#### Scenario: Scroll reversed before it ends
- **WHEN** the camera is scrolling from 0 toward 40 and focus moves back to the first column
- **THEN** the drawn camera turns back without a jump and reaches 0

#### Scenario: Scroll right across the seam
- **WHEN** `loop_bands` is on, the terminal is 80 columns wide, a band holds three columns of 40 cells, the first is focused with the camera at 80 at rest, and focus moves to the column to the right
- **THEN** the camera moves to 120 and is held at 0
- **AND** every frame shows the strip moving left, with the second column entering from the right edge, until the first column is in cells 0 to 39 and the second in cells 40 to 79

#### Scenario: Scroll left across the seam
- **WHEN** `loop_bands` is on, the terminal is 80 columns wide, a band holds three columns of 40 cells, the first is focused with the camera at 0 at rest, and focus moves to the column to the left
- **THEN** every frame shows the strip moving right, with the third column entering from the left edge, until the third column is in cells 0 to 39 and the first in cells 40 to 79
