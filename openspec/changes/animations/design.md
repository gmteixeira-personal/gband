## Context

The client owns presentation. `Display` in `crates/client/src/lib.rs` holds the latest `Layout`, the screen area, the client terminal's size, every pane's `Grid` and the client's `View`. The attach loop draws once after every server message batch and every input event, and otherwise sleeps in `tokio::select!`. `render::render` draws the viewed workspace's `tiles` at `tile.x - view.camera()` and puts the cursor in the focused tile. `View` in `gband-core` decides focus and each workspace's camera and changes them at once. Nothing in the client measures time.

resize-semantics, which this change depends on, makes the server publish layout changes at once and resize PTYs only after a 100 ms quiet period, and only for panes some client shows. Until then the client draws a grid whose size differs from its tile, cut or blank-padded. It also has the client report its shown panes, computed from the view's camera. This change only draws between the tile sizes the client receives, on top of that drawing rule.

## Goals / Non-Goals

**Goals:**
- All animation state and math in one client module, testable with explicit instants and no terminal.
- No change to `gband-core`, `gband-protocol` or the server.
- One retarget path for every cause of motion, so view actions, server focus, layout messages and their interleavings behave alike.

**Non-Goals:**
- Animating panes of workspaces that are not drawn.
- Frame pacing tied to the outer terminal's refresh, or synchronised output.
- Interpolating grid contents. A morphing tile shows the current grid, cut or padded.

## Decisions

### Targets are derived, drawn values are springs

A new `crates/client/src/animation.rs` holds a `Presentation`: the drawn camera, a drawn vertical position in rows, and per-pane drawn `x`, `y`, `width` and `height` in strip coordinates for the drawn workspace. Each is a `Spring`. Before each frame the client computes the **targets** from the layout, the view and the terminal: the viewed workspace's camera, `index × terminal rows` for the vertical position, and each tile from `tiles`. `Presentation::update(now, targets)` retargets every spring whose target changed, starting from its value and velocity at `now`. A pane new to the targets gets a spring at rest on its target. A pane absent from the targets is dropped.

Retargeting lazily at frame time, rather than inside `View::apply` or `Display::apply`, keeps `dispatch`, `Display::apply` and `View` free of time. The loop draws right after each change, so the start of an animation is off by at most one loop iteration.

*Alternative considered:* recording the drawn state inside each mutating call. It needs `now` threaded through `dispatch`, `Display::apply` and `View`, and every new cause of motion would need its own hook.

Screen positions are `drawn x − drawn camera`, so a tile that moves while the camera scrolls stays continuous: at the first frame both springs sit at their old values, which reproduces the old screen position exactly.

### The spring

`Spring` is the closed-form critically damped spring, `ω = √800`, matching niri's default `damping-ratio=1.0, stiffness=800`. With displacement `d₀ = from − target` and initial velocity `v₀`:

- `value(t) = target + (d₀ + (v₀ + ω·d₀)·t)·e^(−ω·t)`
- `velocity(t) = (v₀ − ω·(v₀ + ω·d₀)·t)·e^(−ω·t)`

A spring is at rest once `t ≥ 400 ms`, or once `|value − target| < 0.5` and `|velocity|` is below 10 cells per second. It then reports exactly `target`. Drawn values round to the nearest cell. From rest the closed form never overshoots, which the spec relies on. For a 40-cell move from rest the drawn value is 17 at 50 ms and 37 at 150 ms, and the spring is at rest by about 240 ms. Tests pin these values.

*Alternatives considered:* a fixed-duration ease-out cubic keeps no velocity across a retarget, so a reversed scroll visibly stops and restarts. A numerically integrated spring depends on the frame rate and is harder to test.

### Workspace switch as a vertical camera

Workspaces stack in bands of the client terminal's height, and the vertical spring is a camera over that stack. The render draws every workspace whose band intersects the screen: the drawn one with the drawn camera and its animated tiles, and the one being left with the camera it was drawn with when the switch started. `Presentation` keeps that `(WorkspaceId, camera)` pair and drops it at rest. Tiles of the workspace being left are drawn at their layout positions, without animation.

When the viewed workspace keeps its id but its index changes because a workspace above it was added or removed, `update` shifts the vertical spring's value and target by the same number of rows, so nothing jumps. When the viewed workspace changes and the old one is gone from the layout, `update` snaps the vertical spring.

### Snapping

`Display` sets a snap flag on the first view, on a layout whose `(cols, rows)` differ from the previous area, and on a terminal resize. The next `update` puts every spring at rest on its target and clears the flag. With animations off, every `update` snaps, so the rendered output equals today's.

### Rendering

`Ribbon` gains the drawn values: a list of bands, each a workspace, a top row and a camera, plus each tile's drawn rectangle. `render` draws each band's tiles cut to the band and the terminal. The focused tile is drawn last, the others in layout order. `draw_tile` renders the border at the drawn size and the `PseudoTerminal` into the drawn inner area. tui-term reads only cells inside that area, so a smaller area cuts the grid, and cells beyond the grid's size render blank.

The cursor is placed only when `Presentation` reports that the drawn camera, the vertical position and the focused pane's springs are all at rest.

### Frame scheduling

The attach loop's `tokio::select!` gains a branch on `tokio::time::sleep_until(next_frame)`, armed only while `Presentation::is_animating()`. After each draw, `next_frame` is `now + 16 ms`. With nothing animating, the branch is disabled and the loop sleeps as it does today.

### `GBAND_ANIMATIONS`

`gband_client::run` reads the variable once, before attaching, through a pure parser over `Option<&str>`: `None` or `on` enables, `off` disables, anything else logs a warning naming the value and enables. Reading it in the client crate, not `src/main.rs`, leaves `ClientConfig` and its four construction sites unchanged. A later Lua configuration replaces the variable, so it gets no command-line flag.

## Risks / Trade-offs

- [A frame every 16 ms over a slow link redraws a lot] → ratatui diffs buffers, so only changed cells are written. A reduced mode over SSH is left for when an SSH transport exists.
- [The end-to-end tests may sample the screen mid-animation] → they poll with `wait_for` until a predicate holds, and every animation settles within 400 ms, well inside the 10 s timeout. A test that asserts an intermediate screen once would flake. Such assertions belong in render snapshot tests with explicit drawn values instead.
- [Lazy retargeting starts motion one loop iteration late] → the delay is the time to drain one message batch, far below a frame.
- [Overlapping tiles during an open look busy] → the focused tile, usually the new one, is drawn on top, and the slide lasts about a quarter second. An open effect is out of scope.
- [resize-semantics may change when grid sizes reach the client] → the morph uses only drawn tile sizes and the grid as it is, so it is independent of when the grid resizes.
- [Shown panes follow the target camera, not the drawn one] → a pane scrolling into view is reported shown at the start of the scroll, so its PTY settles while it slides in. A pane scrolling out is reported hidden at once and keeps its size, which the drawing does not depend on.
