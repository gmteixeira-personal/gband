## ADDED Requirements

### Requirement: Animation options
The client SHALL have two more built-in options, alongside those "Options and their values" lists:

| option | value | default | description |
|---|---|---|---|
| `animations` | a boolean | `true` | `whether the client animates changes of the layout and the view` |
| `animation_speed` | a number at least 0.1 and at most 10 | `1` | `how fast animations run, 2 being twice as fast as 1` |

Their effect SHALL be as the animations capability defines. A value of the wrong type or out of range SHALL be rejected as "Options and their values" defines. Reading `gband.opt.animations` SHALL return a boolean, and reading `gband.opt.animation_speed` SHALL return a number equal to the value set. `gband.opt.list()` SHALL hold an entry for each, with the type `"boolean"` or `"number"`, and the default and description above. The default configuration SHALL set both options to their defaults, with the other client options.

#### Scenario: Defaults
- **WHEN** the default configuration is evaluated alone and reads `gband.opt.animations` and `gband.opt.animation_speed`
- **THEN** they read `true` and `1`

#### Scenario: Set in the defaults file
- **WHEN** `defaults/init.lua` is read
- **THEN** it holds `gband.opt.animations = true` and `gband.opt.animation_speed = 1`

#### Scenario: Read back
- **WHEN** `user/init.lua` sets `gband.opt.animation_speed = 1.5` and `gband.set({ animations = false })`, then reads both options
- **THEN** `gband.opt.animation_speed` reads `1.5`
- **AND** `gband.opt.animations` reads `false`

#### Scenario: Speed out of range
- **WHEN** line 2 of `user/init.lua` sets `gband.opt.animation_speed = 0`
- **THEN** a configuration error at `user/init.lua` line 2 naming `animation_speed` is reported
- **AND** `gband.opt.animation_speed` reads `1`

#### Scenario: Wrong type
- **WHEN** line 3 of `user/init.lua` sets `gband.opt.animations = "no"`
- **THEN** a configuration error at `user/init.lua` line 3 naming `animations` is reported
- **AND** `gband.opt.animations` reads `true`

#### Scenario: Client side only
- **WHEN** line 1 of `user/server.lua` sets `gband.opt.animations = false`
- **THEN** the error reported at `user/server.lua` line 1 names the client side
