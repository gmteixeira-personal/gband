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
- **THEN** the next frame draws it within one cell of where it would have been drawn without the change
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
