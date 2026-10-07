## MODIFIED Requirements

### Requirement: Motion
Every animated quantity SHALL move from its drawn value toward its target as a critically damped spring with a stiffness of `800 × s²` and a damping ratio of 1, where `s` is the value of the client option `animation_speed`, as the configuration capability defines. At the default speed of 1 the stiffness is 800, the default niri uses. A quantity at rest SHALL move toward its target without passing it. The drawn value SHALL be rounded to the nearest whole cell. A quantity SHALL reach its target exactly no later than `400 / s` ms after its animation starts, after it was last retargeted, or after the speed last changed, and SHALL then be at rest. When a target changes while the quantity is moving, the animation SHALL continue from the quantity's current drawn value and velocity, without a jump. When a reload changes `animation_speed` while the quantity is moving, the animation SHALL continue from the quantity's current drawn value and velocity at the new speed, without a jump.

#### Scenario: Reaches the target
- **WHEN** `animation_speed` is 1 and a quantity at rest at 0 is given the target 40
- **THEN** it is drawn at 0 at the instant the animation starts
- **AND** it is drawn at a value strictly between 0 and 40 50 ms later
- **AND** it is drawn at 40 and is at rest 400 ms later

#### Scenario: Never passes the target from rest
- **WHEN** a quantity at rest at 0 is given the target 40
- **THEN** every drawn value until it is at rest lies between 0 and 40, and no value is smaller than one drawn before it

#### Scenario: Retarget mid-flight
- **WHEN** a quantity moving from 0 toward 40 is drawn at 20, and its target changes to 0
- **THEN** the next frame draws it within one cell of where it would have been drawn without the change
- **AND** it later reaches 0

#### Scenario: Twice as fast
- **WHEN** `animation_speed` is 2 and a quantity at rest at 0 is given the target 40
- **THEN** it is drawn at a value strictly between 0 and 40 25 ms later
- **AND** it is drawn at 40 and is at rest 200 ms later

#### Scenario: Half as fast
- **WHEN** `animation_speed` is 0.5 and a quantity at rest at 0 is given the target 40
- **THEN** it is not at rest 200 ms later
- **AND** it is drawn at 40 and is at rest 800 ms later

#### Scenario: Speed changed mid-flight
- **WHEN** a quantity moving from 0 toward 40 at speed 1 is drawn at 20, and a reload sets `animation_speed` to 2
- **THEN** at that instant it is drawn at the same value, with the same velocity
- **AND** it is drawn at 40 and is at rest no later than 200 ms after the reload

### Requirement: Turning animations off
The client SHALL animate only while the client option `animations` is `true` and the environment variable `GBAND_ANIMATIONS` does not turn animations off. While animations are off, every change SHALL snap as if every case snapped, and the client SHALL draw exactly as the client-attach capability defines without animations.

The client SHALL read `GBAND_ANIMATIONS` when it starts. When its value is `off`, animations SHALL be off whatever `animations` holds. When it is unset or `on`, the option SHALL decide. Any other value SHALL be logged as a warning in the client's log and read as unset.

A reload that turns animations off SHALL draw every animated quantity at its target from the next frame the client draws. A reload that turns them on SHALL animate the next change, and SHALL NOT move anything that is already at its target.

#### Scenario: Animations off
- **WHEN** a client starts with `GBAND_ANIMATIONS=off` and focus moves to a column that needs the camera to scroll
- **THEN** the next frame draws the camera at its new position

#### Scenario: Environment overrides the option
- **WHEN** a client starts with `GBAND_ANIMATIONS=off`, `user/init.lua` sets `gband.opt.animations = true`, and focus moves to a column that needs the camera to scroll
- **THEN** the next frame draws the camera at its new position

#### Scenario: Animations off by the option
- **WHEN** a client starts with `GBAND_ANIMATIONS` unset, `user/init.lua` sets `gband.opt.animations = false`, and focus moves to a column that needs the camera to scroll
- **THEN** the next frame draws the camera at its new position

#### Scenario: On by default
- **WHEN** a client starts with `GBAND_ANIMATIONS` unset and the default configuration, and focus moves to a column that needs the camera to scroll
- **THEN** the camera scrolls over several frames

#### Scenario: Unknown value
- **WHEN** a client starts with `GBAND_ANIMATIONS=fast` and `user/init.lua` sets `gband.opt.animations = false`
- **THEN** the client's log holds a warning naming the value
- **AND** animations are off

#### Scenario: Turned off by a reload mid-flight
- **WHEN** the camera is scrolling and a reload sets `gband.opt.animations = false`
- **THEN** the next frame the client draws shows the camera at its target

### Requirement: Animated presentation
The client SHALL separate what it draws from what the layout and its view decide. The layout, the tile geometry, the focused window, the viewed band and each camera SHALL change at once, as the layout, layout-view and client-attach capabilities define. Only the drawn camera, the drawn band and each tile's drawn position and size SHALL move toward those targets over time. Key input, pastes and session actions SHALL go to the newly focused window from the moment focus changes, whether or not an animation is running. When no animation is running, the drawn state SHALL equal the target state.

#### Scenario: Typing during a scroll
- **WHEN** the client focuses the column to the right, the camera starts to scroll, and the user types `echo right` and Enter before the scroll ends
- **THEN** `right` appears in the newly focused window only

#### Scenario: At rest
- **WHEN** no layout or view change has happened for `400 / s` ms, where `s` is the value of `animation_speed`
- **THEN** the client draws exactly what it would draw with animations off

### Requirement: Band switch
When the viewed band changes to another band that is still in the layout, the client SHALL animate a vertical position from the old band to the new one. Band `i` in the layout SHALL be drawn in a region as tall as the client terminal, starting at row `i` times the terminal's height less the drawn vertical position. Each band's tiles SHALL be cut at the edges of its region. The band being left SHALL be drawn with the camera it was drawn with when the switch started. A switch made while another is running SHALL continue from the drawn vertical position. When bands are added or removed above the viewed one during a switch, the drawn vertical position SHALL shift with them, so that no band jumps on screen.

#### Scenario: Switch down
- **WHEN** the terminal is 80×24, the client views B1 at rest and views the band below, B2
- **THEN** the first frame shows B1 only
- **AND** a later frame shows the bottom part of B1 above the top part of B2
- **AND** within `400 / s` ms, where `s` is the value of `animation_speed`, only B2 is shown

#### Scenario: Two switches in a row
- **WHEN** the client views B1 of B1, B2 and B3, views the band below, and views the band below again before the first slide ends
- **THEN** the slide continues downward without a jump and ends showing B3

#### Scenario: Old band removed
- **WHEN** the client switches from B1 to B2 and the last window of B1 exits before the slide ends
- **THEN** the client shows B2 at rest from the next frame
