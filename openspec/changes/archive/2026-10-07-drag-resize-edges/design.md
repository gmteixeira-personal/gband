## Context

`drag_resize_window` is a client action, `ClientAction::DragResize` in `crates/core/src/action.rs`. The Lua value `gband.action.drag_resize_window` is a `LuaAction::Builtin`. Called with an argument, it goes through `control::targeted` in `crates/lua/src/control.rs`, which today accepts a target only for session actions and `send_prefix`, and returns "`drag_resize_window` takes no target" for every other action.

In the client, `Controls::run_action` in `crates/client/src/lib.rs` starts a gesture for the three drag actions only while `self.pressing` holds the press being handled. `Controls::start_gesture` in `crates/client/src/mouse.rs` always calls `pick_edges`, the thirds rule, and passes the resulting `Edges` to `Display::window_motion` or to the plugin window's `Motion::Resize`. Every later motion reads `edges` from the gesture. So the picked `Edges` is the only thing that has to change; the motion code already moves any combination of edges.

`Edges` is defined in `crates/client/src/mouse.rs` and used by `crates/client/tests/actions.rs` through `gband_client::mouse::Edges`. `Action` never crosses the wire: the client sends `SessionAction`s only.

## Goals / Non-Goals

**Goals:**
- Let Lua name the edges, and reuse the existing motion code for every kind of box unchanged.
- Keep a binding to the bare action value, and every preset, exactly as it is.

**Non-Goals:**
- Corner margins, title bars or any other MS Windows rule. The `floating-key-style` change builds those in Lua on top of this target.
- Targets for `drag_window` or `drag_band`.
- Telling Lua which edge a press lies on. A binding function reads the press from the `MousePressed` payload and the box from `gband.layout()`.

## Decisions

### The action carries the edges
`Edges { left, right, top, bottom }` moves from `crates/client/src/mouse.rs` to `crates/core/src/action.rs`, with `Serialize` and `Deserialize` as `ClientAction` needs. `ClientAction::DragResize` becomes `DragResize(Option<Edges>)`. The built-in action is `DragResize(None)`, and a target produces `Dispatch::Action(Action::Client(ClientAction::DragResize(Some(edges))))`. `crates/client/src/mouse.rs` re-exports `Edges`, so `gband_client::mouse::Edges` keeps its path.

`start_gesture` uses the carried edges when they are `Some`, and calls `pick_edges` otherwise. `window_motion` and `run_action` match `DragResize(_)`.

- *Alternative*: a new `Dispatch::DragResize(Edges)` handled beside `Dispatch::Action`. Rejected. It would need its own path into `start_gesture` and its own check of `pressing`, and the gesture would be started from two places.
- *Alternative*: keep `ClientAction` unchanged and stash the edges in client state before dispatch. Rejected. Dispatch order would then decide which edges a gesture uses, and a second call in the same callback would overwrite the first.

### Reading the target
`control::targeted` accepts a target for `Action::Client(ClientAction::DragResize(_))` and reads it in a new `edges_target`:

1. `fields(target, &["edges"], name)` rejects every other field, as other targets do.
2. A missing `edges` is an error.
3. `edges` must be a list of strings, read as `api::list_of_strings` reads one. A table with other keys, or a value that is not a string, is an error.
4. The list must not be empty. Each name must be one of the four edge names, and no name may come twice.
5. Both `left` and `right`, or both `top` and `bottom`, is an error naming both.

Every error goes back as a `String`, and `LuaAction`'s call raises it with `ConfigError::raise`, so it reports the line of the call and nothing is queued.

A target on a view action, `detach`, `drag_window` or `drag_band` keeps the existing "takes no target" error. The spec now names these client actions, since its old wording named only `detach`.

### Tile edges defer to the existing rules
A named `top` or `bottom` on a tile sets the window's height exactly as a picked one does: the tile height at the press, less or plus `dy`, limited as the layout's "Set a window's height" defines. On a column's first window, `top` therefore changes its height while its tile stays at the column's top, which is what the thirds rule already does when it picks `top` there. A named `left` on a tile moves the camera by the change in width, as a picked one does. No new layout rule is needed.

### `gband.api_version` does not rise
The API version rule raises the version when a build removes a documented part, or changes the documented arguments, return values, errors or effect of one. It keeps the version when a build only adds to the documented API. Under this change:

- Every call that worked before works the same. A binding to `gband.action.drag_resize_window`, and a call with no target, still pick the edges by thirds.
- Every documented error rule still holds. A field the action does not take is still an error; `edges` is now a field it takes. A target on a view action, `detach`, `drag_window` or `drag_band` is still an error.
- The change adds one optional argument and the errors that check it.

This is an addition. The archived `stable-lua-api` design states the same reading: generalising a part of the API later is an addition, and additions do not raise `gband.api_version`. The "API and stability" table in `docs/plugins.md` gains no row.

### Documentation
`docs/plugins.md` is the reference for targets, so it gets the full rule, the errors and an example. `README.md` lists every action and what a target names, so its `drag_resize_window` row and that sentence change. `docs/tutorial/02-actions.md` teaches targets and links to the reference, so it gains one sentence and no code: a code block would have to appear in `examples/tutorial/02-actions/user/init.lua` and in every later chapter's copy. `docs/internals/` needs no change. Its only mention of the drag actions is the key style presets quoted in `08-key-styles.md`, and the presets do not change.

## Risks / Trade-offs

- [`ClientAction` grows a payload, so every match on `ClientAction::DragResize` must change] → The matches are few and the compiler finds each one: `crates/client/src/lib.rs`, `crates/client/src/mouse.rs`, `crates/client/src/bindings.rs`, `crates/lua/src/actions.rs` and the tests that name the action.
- [A user expects the named edges to land exactly under the pointer for a tile's `top` on a column's first window] → The spec says the height changes and the tile stays at the column's top. The floating key style, the reason for this change, only resizes floating boxes, where every edge moves.
- [A Lua list with holes, such as `{ "left", nil, "top" }`, has an unclear length] → `list_of_strings` rejects a table whose pair count differs from its length, so such a list is an error.

## Migration Plan

No migration. Existing configurations and presets keep their behaviour, no protocol message changes, and this change does not raise `gband.api_version`. Rollback is a revert of the change's commits.
