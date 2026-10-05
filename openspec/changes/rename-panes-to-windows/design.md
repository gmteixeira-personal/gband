## Context

See proposal.md for the motivation. The relevant current state:

- "pane" appears about 3,300 times in 130 files outside `openspec/changes/`: every crate, `src/`, `tests/`, the sample plugins, the docs and 24 main specs.
- "window" already has a meaning. `gband.win` draws windows: a float over one client's ribbon, or a pane window, which shows its lines in a plugin pane of the shared layout. In Rust, the client and `gband-lua` call these `Window`, `WindowRequest` and the like, in `crates/client/src/windows.rs` and `crates/lua/src/windows.rs`. `gband.view().window` and the layout's `window` field name them, and so do the highlight groups `Window`, `WindowBorder`, `WindowTitle` and `WindowCursorLine`.
- Test files already use the word for plugin windows: `crates/client/tests/windows.rs`, `crates/lua/tests/windows.rs` and `tests/windows.rs`. `crates/server/tests/panes.rs` and `crates/server/tests/plugin_panes.rs` would collide with them once renamed.
- gband 0.1.0 shipped the pane names. A user configuration or plugin may use any of them.
- The wire encoding is postcard, which writes struct fields and enum variants by position and never by name. Client and server are one binary.
- Five open changes add or quote pane names: floating-windows (`toggle_pane_floating`, `move_pane_up`, `gband.pane.set_position`, `FloatingPane`, `PaneMoved`, the floating-panes spec), navigation-mode, center-column, loop-bands and sidebars-borders-steps. floating-windows is claimed and in progress.
- OpenSpec refuses to archive a MODIFIED requirement whose block lacks a scenario title that the main spec has, and it has no scenario rename. The precedent, rename-workspaces-to-bands, renamed scenario titles in the main specs in its first task.

## Goals / Non-Goals

**Goals:**
- After the change, "pane" appears only where gband talks about tmux and Zellij, in the error messages for the removed names, in archived changes, and in this change's own history.
- A bare "window" always means what niri calls a window. What `gband.win` draws is always a "plugin window".
- Behaviour, geometry, animation timing and wire bytes stay as they are. Every existing test keeps passing after its identifiers are renamed.

**Non-Goals:**
- Aliases for the old names. The user chose a hard break.
- Renaming `gband.win`. Its functions take and return plugin window numbers, and `win` already reads as short for it.
- Renaming the floats. A float is still a float, the floating kind of plugin window.
- Rewriting archived changes under `openspec/changes/archive/`.

## Decisions

### Run after every open change

`Depends On` lists floating-windows, navigation-mode, center-column, loop-bands and sidebars-borders-steps. The rename touches nearly every file those changes touch. Run beside them, every one of them would meet conflicts at `/ready`, and floating-windows is already half written. Run last, it meets the merged result once and renames the names they added in the same sweep.

Alternatives:
- Run first. Rejected. It would leave five changes, one in progress, to rename their own work while merging.

### Generate the spec deltas from the main specs

`rename_specs.py`, in this change's directory, holds the rename map and writes the delta specs from the current main specs. Every requirement whose text the map changes becomes a MODIFIED block, and a requirement whose name changes also gets a RENAMED entry. `added/<capability>.md` holds the one ADDED requirement. The deltas in this proposal were generated from the main specs as they stood when it was written. The five dependencies will change those specs first, so the first task runs the script again on the then-current main specs rather than patching about 130 blocks by hand.

The map runs in this order, so that no step renames the output of an earlier one:

1. Plugin window terms, in every spec: "pane window" becomes "tiled plugin window", "plugin pane" becomes "drawn window", and the kind `"pane"` becomes `"tiled"`.
2. In plugin-windows, lua-control and client-attach only, where "window" means a plugin window: bare "window" becomes "plugin window", `window` becomes `plugin_window`, and the `Window*` groups become `PluginWindow*`. "terminal emulator window" and the capability name "plugin-windows" are held out.
3. Identifiers, longest first: `gband.pane_state`, `gband.pane`, the `Pane*` events, the actions, `GBAND_PANE`, `pane_spec` and the open changes' names, such as `toggle_pane_floating`.
4. The words "Panes", "Pane", "panes" and "pane".

`rename_specs.py check` lists every main-spec line the map would still change, so the last task can prove that nothing was missed.

Alternatives:
- Write the deltas by hand. Rejected. About 130 blocks across 24 specs, and all of them stale once the five dependencies land.
- `skip_specs: true` and edit the main specs directly. Rejected. The change alters observable names, which is spec-level behaviour, and the archive should carry it.

### Name the parts of a plugin window

A float keeps its name. A pane window becomes a **tiled plugin window**, and its kind value becomes `"tiled"`, which says where it sits rather than what it sits in. The window in the layout that shows a tiled plugin window, called a plugin pane today, becomes a **drawn window**: a window with no program, whose screen its owning client draws. The server knows nothing about plugin windows, so its specs speak of drawn windows only.

Alternatives:
- "plugin-drawn window". Rejected. It sits too close to "plugin window" for two different things.
- Keep `kind = "pane"`. Rejected. It would be the one pane left in the API.

### Free the plugin window names first

Before the domain rename, the plugin window names move out of the way, the way rename-workspaces-to-bands moved the client's `Band` to `DrawnBand` first:

- Rust: `Window` becomes `PluginWindow`, `WindowRequest` becomes `PluginWindowRequest`, and so on, in `crates/client` and `crates/lua`. `crates/client/src/windows.rs` and `crates/lua/src/windows.rs` become `plugin_windows.rs`, and so do the three `windows.rs` test files.
- Lua: `view().window` and the layout's `window` become `plugin_window`, the kind `"pane"` becomes `"tiled"`, and the groups take the `PluginWindow` prefix.

Done afterwards, a blind `Pane` to `Window` replacement would produce two `Window` types in one module, and every compile error after it would be ambiguous.

### Mechanical rename, by case-preserving forms

The domain rename maps `Panes`, `panes`, `Pane` and `pane` to `Windows`, `windows`, `Window` and `window`, in identifiers, strings, test names, Lua and Markdown, longest compound first: `PaneId` to `WindowId`, `allocate_pane` to `allocate_window`, `SessionAction::OpenPane` to `OpenWindow`, `pane.rs` to `window.rs`. `crates/server/tests/panes.rs` becomes `windows.rs`, and `plugin_panes.rs` becomes `drawn_windows.rs`. Run `sd` per file over the files `rg -l -i pane` lists, crate by crate, and let `cargo build` and the `LSP` tool find any shadowing the rename created, such as a local `window` beside a `window` parameter.

The words that stay: "pane" in the README's comparison with tmux and Zellij, and in the removed-name error messages.

Alternatives:
- `ast-grep` per pattern. Rejected for the sweep. The rename crosses Rust, Lua, strings and Markdown, and one case-preserving text map covers them all. `ast-grep` and the `LSP` tool still check the result.
- Keep the Rust names. Rejected by the user. Code that says pane while every spec and doc says window would need its own glossary.

### Removed names fail with the new name

The user chose no aliases. A bare removal would surface as "attempt to call a nil value" or "unknown event", which says nothing about the rename. Instead, one table maps each removed Lua name to its replacement, and every place that resolves a name looks there before failing: the `gband` table's index, `gband.action`'s index, event name parsing in `gband.on` and `redraw_on`, the action target field check, `gband.win.open`'s `kind` check, and the hints segment's `labels`. The error message reads "`close_pane` is now `close_window`". The table and its lookups are the only code that keeps the old names.

Fields that gband fills in, such as `view().pane`, are plain tables. Reading an absent field gives nil, as for any other unknown field, and no metatable is added to catch it.

### No protocol version change

Postcard encodes by position, so renaming fields and variants changes no byte. The wire-protocol tests keep their expected bytes. Event names never cross the wire as strings.

## Risks / Trade-offs

- [A dependency adds a pane name the map does not know] → Task 1.1 lists every remaining `pane` in the merged main specs with `rename_specs.py check` before generating, and adds any new compound name to the map.
- [The generated deltas read awkwardly in places] → Task 1.1 reads the plugin-windows, lua-control, client-attach and session-server deltas in full before committing them. These are the specs where the two meanings of window meet.
- [A blind replacement hits a "pane" that must stay] → The README's tmux and Zellij sentences and the removed-name table are restored by hand after each sweep, and the final check lists every remaining match.
- [0.1.0 configurations and plugins stop loading] → Accepted by the user. Every removed name fails with an error that names its replacement, and the next release notes list the renames.
- [Scripts reading `GBAND_PANE` get an empty value] → Accepted. The release notes list it.
- [Snapshot and screenshot files whose names say pane] → They are renamed with `git mv` and their contents change only where they show text that the rename changes, such as the sample plugin's `window 1`.

## Migration Plan

Users replace the old names as the errors direct, or with the table in the release notes. A rollback is a revert of the change's merge.
