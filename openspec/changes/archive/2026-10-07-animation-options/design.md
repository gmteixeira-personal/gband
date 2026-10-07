## Context

The client reads `GBAND_ANIMATIONS` once, in `crates/client/src/lib.rs`, and hands `Animations::On` or `Animations::Off` to `Presentation::new`. The spring in `crates/client/src/animation.rs` uses two constants, `STIFFNESS = 800.0` and `SETTLE = 400 ms`. Nothing in the client's options reaches the presentation.

Client options live in `crates/lua/src/options.rs`: the `Options` struct with its Rust defaults, `OptionsPatch`, `reset`, reading, and the `BUILTIN` table of names, types and descriptions that `gband.opt.list()` returns. The default configuration, `crates/lua/src/defaults.lua`, assigns every client option its default, and the configuration spec's "Defaults reproduce the built-in behaviour" scenario checks that those values equal the declared defaults. `Display::configure` copies the options into the client after each load.

`window_titles` with `GBAND_WINDOW_TITLES` is the existing pattern for an option that the environment can override. The test harness sets `GBAND_ANIMATIONS=off` for every case, so the override must keep winning.

Both tutorials, `scripting-tutorial` and `lua-internals-tutorial`, are archived before this change starts, so `docs/tutorial/`, `docs/internals/` and their checks exist.

## Goals / Non-Goals

**Goals:**
- Turn animations on and off, and set their speed, from a configuration, with the defaults written in the default configuration.
- Keep every existing animation scenario true at speed 1.
- Keep the harness's `GBAND_ANIMATIONS=off` effective for every case.

**Non-Goals:**
- Changing the shape of the motion: the spring stays critically damped.
- Exposing stiffness or settle time as separate options.

## Decisions

### Speed scales time, so the stiffness is `800 × s²` and the settle cap `400 / s` ms

Running a critically damped spring `s` times faster is the same spring with its natural frequency `ω = √800` multiplied by `s`, so its stiffness multiplied by `s²`. Each `Spring` carries its own `omega` and `settle`, set from the speed when it starts, is retargeted, or is given a new speed. The closed-form value and velocity already take the real velocity and `ω`, so a restart at a new speed continues from the drawn value and velocity with no conversion. The rest thresholds, 0.5 cells and 10 cells per second, stay as they are.

Alternatives:
- A duration option, such as `animation_duration = 400`. Rejected by the user in favour of a multiplier, which reads as "speed".
- Exposing `stiffness` directly. It is not a unit a user thinks in, and it would not move the 400 ms cap with it.

### `animation_speed` ranges from 0.1 to 10

At 10 every motion settles within 40 ms, two or three frames of 16 ms; faster is indistinguishable from off, which `animations = false` already gives. At 0.1 a motion takes up to 4 s, slow enough to watch a band switch. Zero and negative values have no meaning. The value is held as an `f64`, not read as a fraction as widths and steps are, because it is not a proportion of the screen.

### A speed change carries moving quantities on at once

`Presentation::set_speed` restarts every moving spring at its drawn value and velocity with the new `omega` and `settle`, as `retarget` does for a new target. A spring at rest only records the speed. The alternative, letting running springs finish at the old speed, needs each spring to remember a speed that no longer applies, and the user sees no difference after 400 ms either way.

### The environment overrides the option, as `GBAND_WINDOW_TITLES` does

`parse_animations` keeps its warning for an unknown value and returns whether the environment allows animations. The presentation animates while the environment allows it and the option is `true`. `Presentation::new` takes the environment's answer, and `Display::configure` passes the option's value and speed after each load. Turning animations off through `configure` sets the presentation to snap, so the next frame draws every target.

Alternative: dropping `GBAND_ANIMATIONS`. Rejected: the harness, `docs/testing.md` and the `plugin-testing` spec rely on it, and over a slow link a user may want animations off for one attach without editing a file.

### Defaults are written in both `defaults.lua` and `Options::default`

`defaults.lua` assigns `animations = true` and `animation_speed = 1` next to `window_titles`, as the user asked. `Options::default` keeps the same values, because a `user/init.lua` that never names the options still needs them, and `reset` restores them after an invalid assignment. The existing defaults test compares the two, so they cannot drift.

### Tutorials

The scripting tutorial's options chapter adds `gband.opt.animation_speed = 2` to its `user/init.lua` in a fenced `lua` block, and names `gband.opt.animations = false` and `GBAND_ANIMATIONS=off` in prose with inline code, which the block check does not read. Every later chapter directory is a complete snapshot, so the line is copied into each later `user/init.lua`. The chapter's spec reads the option back through the test channel; the harness keeps animations off, so the spec checks the value, not motion.

The internals tutorial's whole-file coverage requires every non-blank line of `defaults/init.lua` to be quoted once, so `11-default-configs.md` gains a quote of the two new lines, next to the quote of the other option assignments.

## Risks / Trade-offs

- [The option name `animations` is plural and boolean, unlike `loop_bands`] → It matches the environment variable it shadows, and its description says what it does.
- [Every later tutorial snapshot repeats the new line] → Accepted by the scripting tutorial's own design; its block check finds a snapshot that misses it.
- [A reload with a new speed during a long motion at 0.1 restarts its 4 s cap] → The cap is a bound, not a duration, and the restart starts from the drawn value and velocity, so nothing jumps.
