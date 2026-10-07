## 1. Deltas on the current specs

- [ ] 1.1 Re-copy the animations capability's "Motion" and "Turning animations off" from the main spec on `dev`, and re-apply this change's edits to them: the stiffness `800 × s²`, the cap `400 / s` ms, the speed change mid-flight, the option and the override, and their scenarios. Check that the configuration capability on `dev` has no requirement named "Animation options" and that its "Options and their values" table holds neither name. Verify with `openspec validate animation-options --strict`

## 2. Options

- [ ] 2.1 In `crates/lua/src/options.rs`, add `animations: bool` and `animation_speed: f64` to `Options`, with defaults `true` and `1.0`, to `OptionsPatch`, `merge`, `reset`, reading and the `BUILTIN` table with the types and descriptions the configuration delta gives. Reject a speed below 0.1, above 10, or not a number. Verify "Defaults", "Read back", "Speed out of range", "Wrong type" and "Client side only" in `crates/lua/tests/options.rs`, and add both names to the `gband.opt.list()` name test there
- [ ] 2.2 In `crates/lua/src/defaults.lua`, add `gband.opt.animations = true` and `gband.opt.animation_speed = 1` after `gband.opt.window_titles = true`. Verify "Set in the defaults file" and that "Defaults reproduce the built-in behaviour" still passes in `crates/lua/tests/`

## 3. Client

- [ ] 3.1 In `crates/client/src/animation.rs`, give `Spring` its own `omega` and `settle`, set from a speed, in place of `STIFFNESS` and `SETTLE`. Add a way to restart a moving spring at a new speed from its drawn value and velocity. Verify "Reaches the target", "Twice as fast", "Half as fast" and "Speed changed mid-flight" as unit tests in `crates/client/tests/animation.rs`, with the existing spring tests unchanged
- [ ] 3.2 In `crates/client/src/animation.rs`, make `parse_animations` return whether the environment allows animations, keeping its warning, and give `Presentation` setters for the option and the speed: turning animations off snaps on the next update, and a new speed restarts every moving spring. Verify "Turned off by a reload mid-flight" and a speed change on a moving camera in `crates/client/tests/animation.rs`
- [ ] 3.3 In `crates/client/src/lib.rs`, pass the environment's answer to `Display::new`, and `options.animations` and `options.animation_speed` to the presentation in `Display::configure`. Verify "Animations off by the option", "Environment overrides the option", "On by default" and "Unknown value" end to end in `tests/attach.rs`, next to `animations_off_opens_a_window_without_motion`
- [ ] 3.4 Run `cargo test` for the whole workspace and confirm every existing animation, mouse and harness test passes unchanged at speed 1 with `GBAND_ANIMATIONS=off` in the harness

## 4. Documentation

- [ ] 4.1 In `docs/plugins.md`, add `animations` and `animation_speed` to the client row of the `gband.opt` table, and a paragraph after the `window_titles` one: what each does, its default, the range of the speed, `GBAND_ANIMATIONS=off` overriding the option, and what a reload changes. Leave `gband.api_version` at 1. Verify the documentation tests in `crates/lua/tests/` pass
- [ ] 4.2 In `README.md`, extend the line on `GBAND_ANIMATIONS` to name `gband.opt.animations` and `gband.opt.animation_speed` and link to the options section of `docs/plugins.md`. Verify by reading the rendered section

## 5. Tutorials

- [ ] 5.1 In `docs/tutorial/03-options.md`, add a `lua` block setting `gband.opt.animation_speed = 2` and prose naming `gband.opt.animations = false` and `GBAND_ANIMATIONS=off` in inline code. Add the line to `examples/tutorial/03-options/user/init.lua` and to the `user/init.lua` of every later chapter directory, and a case to the chapter's spec that reads the option back. Verify the tutorial checks in `crates/lua/tests/tutorial.rs` and the `tutorial` test in `tests/lua_specs.rs` pass
- [ ] 5.2 In `docs/internals/11-default-configs.md`, quote the two new lines of `defaults/init.lua` with the other option assignments and say they set the animation options to their declared defaults. Verify the whole-file coverage check in `crates/lua/tests/tutorial.rs` passes
