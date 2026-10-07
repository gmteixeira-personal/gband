## Why

The client's animations can only be turned off by setting `GBAND_ANIMATIONS=off` before attaching, and their speed is fixed in the executable. A configuration cannot turn them on or off or change their speed, so a user who wants faster motion, or no motion over a slow link, has to change the environment of every attach instead of writing it once in `user/init.lua`.

## What Changes

- **Two client options**: `animations`, a boolean, `true` by default, turns animations on and off. `animation_speed`, a number from 0.1 to 10, `1` by default, scales how fast every animation runs. At speed `s`, every animated quantity moves as a critically damped spring of stiffness `800 × s²` and reaches its target within `400 / s` ms. Both are read with `gband.opt.<name>`, set with `gband.opt.<name> = value` or `gband.set`, and listed by `gband.opt.list()`.
- **Defaults in Lua**: the default configuration, `crates/lua/src/defaults.lua`, written as `defaults/init.lua`, sets `gband.opt.animations = true` and `gband.opt.animation_speed = 1` with the other client options.
- **Reload applies them**: a reload that turns `animations` off snaps every running animation on the next frame. A reload that changes `animation_speed` carries every moving quantity on from where it is drawn, at the new speed, without a jump.
- **`GBAND_ANIMATIONS` becomes an override**: `off` turns animations off whatever `animations` holds. Unset or `on` leaves the option in charge. Any other value is logged as a warning and read as unset. This is how `GBAND_WINDOW_TITLES` already works, and the test harness keeps setting `GBAND_ANIMATIONS=off`.
- **Docs**: `docs/plugins.md` lists both options in the `gband.opt` table and describes them and the override. `README.md` names the options next to `GBAND_ANIMATIONS`.
- **Tutorials**: the scripting tutorial's options chapter sets `animation_speed` and shows how to turn animations off. The internals tutorial's chapter on the default configurations quotes the two new lines of `defaults/init.lua`, which its whole-file coverage check requires.
- The change only adds to the documented API, so `gband.api_version` stays `1`.

Out of scope:
- A settings window line for animations.
- An action or a function that changes animations outside a configuration load.
- Per-kind control, such as turning off only the band switch.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `animations`: the motion's stiffness and settle time depend on `animation_speed`, and animations are turned off by the `animations` option as well as by `GBAND_ANIMATIONS`, which becomes an override.
- `configuration`: the built-in client options gain `animations` and `animation_speed`.

## Impact

- `crates/lua/src/options.rs`: the two options, their validation, defaults, descriptions, reading and reset.
- `crates/lua/src/defaults.lua`: the two default assignments.
- `crates/client/src/animation.rs` and `crates/client/src/lib.rs`: the speed in the spring, the option and the override combined, and reload handling.
- `crates/lua/tests/options.rs`, `crates/client/tests/animation.rs`: tests of the options and of the motion at another speed.
- `docs/plugins.md`, `README.md`: the options and the override.
- `docs/tutorial/03-options.md` and `examples/tutorial/`: the options chapter and the chapter snapshots that carry its `user/init.lua`.
- `docs/internals/11-default-configs.md`: quotes of the new `defaults/init.lua` lines.
- No protocol, server or harness change, and no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- scripting-tutorial
- lua-internals-tutorial

### Expected Files
- crates/lua/src/options.rs
- crates/lua/src/defaults.lua
- crates/lua/tests/options.rs
- crates/lua/tests/config.rs
- crates/client/src/animation.rs
- crates/client/src/lib.rs
- crates/client/tests/animation.rs
- tests/attach.rs
- docs/plugins.md
- README.md
- docs/tutorial/03-options.md
- examples/tutorial/
- docs/internals/11-default-configs.md
- openspec/changes/animation-options/
