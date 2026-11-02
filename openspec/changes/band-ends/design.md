## Context

`View` in `crates/core/src/view.rs` holds `loop_bands: bool`. It is read in two places: `neighbour_column` wraps focus when it is on, and the loop test at `loop_width(spans, viewport).filter(|_| self.loop_bands)` decides whether a strip loops. The client stores the option in `Client::loop_bands`, passes it through `View::set_loop_bands` in `configure` and `sync`, and re-syncs the view when it changes. `CenterFocusedColumn` is the existing model for a string-valued client option: a serde `kebab-case` enum in `gband_core::view`, read and written through `lua.to_value` and the `gband.set` patch in `crates/lua/src/options.rs`.

## Goals / Non-Goals

**Goals:**
- Make bounce reuse the existing non-looping camera path, so its scroll back needs no new camera or animation code.

**Non-Goals:**
- No looping or bouncing between bands.
- No alias that keeps `loop_bands` working.

## Decisions

**One enum splits the two reads.** `BandEnds { Loop, Bounce, Stop }`, serde `kebab-case`, `Loop` the default, lives beside `CenterFocusedColumn`. `neighbour_column` wraps unless `Stop`. The loop test passes only under `Loop`. Bounce is therefore today's short-strip case applied to every strip. Focus wraps, the strip is drawn once, and under `"never"` the first column starts left of the view, so the camera moves to its start. The camera spring already animates any camera change, so the scroll back is drawn with no new code. Alternative considered: a boolean `stop_at_band_end` beside `loop_bands`. Rejected by the user because it would do nothing under the default `loop_bands = true`.

**`loop_bands` is removed outright.** It becomes an unknown option name. An assignment to it is reported at its line, and the load still succeeds, as the configuration spec defines for any undeclared name. Alternative considered: accept `true` and `false` as aliases for `"loop"` and `"stop"`. Rejected: it keeps two names for one setting, and the error names the line to fix.

**`"on-overflow"` under bounce needs no new rule.** The policy takes S as the column next to C on the side focus came from. Moving right from the last column to the first, focus came from the last column, which lies right of C, so S is the second column. The non-looping branch already handles that.

## Risks / Trade-offs

- [Existing user configs set `loop_bands`] → The load succeeds with the default and an error names the line. The README and `docs/plugins.md` name the replacement.
- [The test file `tests/lua/loop_bands_spec.lua` keeps an old name] → It is renamed to `tests/lua/band_ends_spec.lua`, and its registration in `tests/lua_specs.rs` is renamed with it.
