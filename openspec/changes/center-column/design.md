## Context

Each client keeps a `View` (`crates/core/src/view.rs`) with one `BandView { focus, camera }` per band. `View::apply` runs a `ViewAction`, then `settle`, which ends in `follow`: it places the camera by the `center_focused_column` policy, using `centred(span, viewport)` for `"always"` and `revealed(span, viewport, camera)` otherwise. The client dispatches view actions locally in `crates/client/src/lib.rs` (`view_action`), and the animation layer already scrolls the drawn camera toward any new camera target. Built-in actions are listed once in `ACTIONS` (`crates/lua/src/actions.rs`), which feeds `gband.action`, `gband.action.list()` and the server-side guard in `crates/lua/src/sides.rs`.

## Goals / Non-Goals

**Goals:**
- One new `ViewAction` variant, reusing `centred`, with no new state in `View`.
- No change to the camera policies, the protocol or the animation layer.

**Non-Goals:**
- A sticky "centred" mode that survives later focus changes. niri's `center-column` is a one-shot camera move, and so is this.

## Decisions

### `ViewAction::CenterColumn` sets the camera, then settles as usual
`apply` gains a `CenterColumn` arm. It locates the focused window's column in the viewed band, takes its span from `column_spans(band, scene.area)`, and sets the band's camera to `centred(span, viewport)`. The usual `settle` runs after it. Under `"never"`, and under `"on-overflow"` when focus did not change column, `follow` falls to `revealed`, which keeps a camera whose column lies wholly inside the view. A centred column always does, or the camera already sits at the column's start when the column is at least as wide as the terminal. Under `"always"`, `follow` computes the same value. So `settle` never undoes the move, and every later change returns to the policy with no extra state.

- *Alternative*: a flag in `BandView` that skips `follow` once. Rejected. It adds state that the spans already make unnecessary.
- *Alternative*: a fourth policy value. Rejected. The policy decides what happens on every change; this is a one-shot command.

### A client-side view action, not a session action
The camera is per client, so the action never reaches the server, as the actions capability defines for view actions. `is_client_action` already treats every non-session action as the client's, so the server's `gband.action` reports `center_column` as a client action with no change to `sides.rs`.

### Name, description and key
The Lua name `center_column` follows niri's `center-column`, and the US spelling matches the existing `center_focused_column` option. The description is `center the focused column`, the short hint label is `center`, and the default key is `prefix c`, which is unbound today.

## Risks / Trade-offs

- [Open changes edit the same tables: `floating-windows`, `key-list`, `navigation-mode` and `sidebars-borders-steps` all touch `ACTIONS`, `defaults.lua` and the actions and default-binding tables in the specs] → Each addition is one row, so a merge conflict is a two-sided row insertion that keeps both rows. The `MODIFIED` requirement deltas here must be re-read against `<base>` at `/ready` so that archiving keeps the other changes' rows.
- [After `floating-windows` lands, the focused window may be floating] → `follow` there reads the tiled focus. `center_column` should do the same, and centre the tiled focus's column. Whichever change lands second reconciles the arm.
- [After `sidebars-borders-steps` lands, the viewport is the ribbon width rather than the terminal width] → `CenterColumn` uses `scene.viewport`, the same width `follow` uses, so it follows that change without edits.
