## Context

See proposal.md for the motivation. The relevant current state:

- `gband-core` names the domain type `Workspace` with `WorkspaceId`, reached through `Layout::workspaces()`, `Layout::workspace(id)` and `Layout::workspace_index(id)`. `Location` has a `workspace` field. `ViewAction` has `WorkspaceDown` and `WorkspaceUp`. `View` keeps a `WorkspaceView` per workspace. `LayoutEvent` has `WorkspaceAdded` and `WorkspaceRemoved`, and every other variant carries a `workspace` field. `SessionAction::OpenPane` names a `workspace`.
- `gband-client`'s `animation.rs` already uses `Band` for one workspace's frame in a switch: `Band { workspace, top, camera }`, collected in `Drawn::bands`. `Targets::band` and `Shown::band` hold the frame height, the client terminal's rows. `render.rs` iterates `drawn.bands` and looks each one up with `layout.workspace(band.workspace)`.
- `gband-lua`'s `api.rs` maps `focus_workspace_down` and `focus_workspace_up` to the two view actions. The built-in default configuration binds them to `prefix u` and `prefix i`.
- The wire encoding is postcard, which writes struct fields and enum variants by position and never by name.
- config-directory and ctrl-space-prefix, which this change depends on, rewrite the default configuration file, the loader and the "Kinds of action" and "Key bindings" requirements. The delta specs here are built on config-directory's version of those two requirements. ctrl-space-prefix was proposed after them, so task 1.1 folds in its wording.
- Cargo uses "workspace" for its own concept: `[workspace]`, `*.workspace = true` and `cargo ... --workspace`. The terminal-emulator spec's "workspace's manifests" scenario means Cargo's workspace.
- OpenSpec 1.12 refuses to archive a MODIFIED requirement whose block lacks a scenario title that the main spec has, and it has no scenario rename. Thirteen scenario titles contain "workspace".

## Goals / Non-Goals

**Goals:**
- After the change, "workspace" appears only in Cargo's sense, in archived changes, and in this change's own history.
- Behaviour, geometry, animation timing and wire bytes stay as they are. Every existing test keeps passing after its identifiers are renamed.

**Non-Goals:**
- Aliases for `focus_workspace_down` and `focus_workspace_up`. The names never shipped.
- Rewriting archived changes under `openspec/changes/archive/`.
- Renaming Cargo's workspace or the gband crates.

## Decisions

### Free the name `Band` in the client first

The client's `Band` becomes `DrawnBand`, which pairs with `DrawnTile` in the same `Drawn` value. Its `workspace` field becomes `band`. `Targets::band` and `Shown::band` become `band_height`. In `render.rs` the loop variable becomes `drawn`, so `layout.band(drawn.band)` reads plainly.

This rename runs before the domain rename. Done afterwards, a blind `Workspace` to `Band` replacement would produce two `Band` types in `animation.rs`, and every later compile error would be ambiguous.

Alternatives:
- `Frame` or `Slice`. Rejected. "Frame" already means one drawn screen in the animations spec, and `DrawnBand` says what the value is beside `DrawnTile`.

### Mechanical rename, by case-preserving forms

The domain rename maps `Workspaces` to `Bands`, `workspaces` to `bands`, `Workspace` to `Band` and `workspace` to `band`, in identifiers, strings, test names and doc text. Compound names follow: `WorkspaceId` to `BandId`, `workspace_index` to `band_index`, `next_workspace` to `next_band`, `WorkspaceView` to `BandView`, `ViewAction::WorkspaceDown` to `ViewAction::BandDown`. The Lua table entries become `focus_band_down` and `focus_band_up`. Test environment names such as `TestEnv::new("workspaces")` become `"bands"`.

Run `sd` per file, longest form first, over the files `rg -l -i workspace` lists outside `openspec/changes/archive/`. Leave every `Cargo.toml`, every `cargo ... --workspace` command line and the terminal-emulator spec alone. Then let `cargo build` and the `LSP` tool find any shadowing that the rename created, for example a local `band` beside a `band` parameter.

Alternatives:
- `ast-grep` per pattern. Rejected for this change. The rename crosses identifiers, string literals, Lua source and Markdown, and a single case-preserving text map covers them all. `ast-grep` still suits checking the result.

### Scenario titles are renamed in the main specs on the branch

The delta specs rename every requirement body and the three requirement names. They keep the thirteen scenario titles exactly as the main specs have them, so `openspec validate --strict` passes now. The first implementation task renames those thirteen titles in both the main specs and the delta specs, in the same commit. The archive then finds every main-spec title in the matching delta block.

This is the only edit the change makes to `openspec/specs/` outside the archive. It renames titles and changes no requirement text, so the archive's merge still owns every requirement. The Purpose sections of the animations, layout and layout-view specs are edited the same way, because a delta cannot change a Purpose.

Alternatives:
- REMOVED plus ADDED for each affected requirement. Rejected. ADDED and REMOVED cannot share a name, so seven requirements such as "Layout events" would need new names for no other reason.
- Keep the old titles. Rejected. The specs would contradict the term they define.

### No protocol version change

Postcard encodes by position, so renaming `workspace` fields and the `WorkspaceAdded` variant changes no byte. Client and server are one binary, so a mixed-version pair is not a concern either. The wire-protocol tests that round-trip open pane and the layout snapshot keep their expected bytes.

## Risks / Trade-offs

- [config-directory or ctrl-space-prefix changes the wording of a shared requirement after this proposal was drafted, for example the Ctrl+A in "Kinds of action"] → The first task diffs each MODIFIED block against the then-current main spec with `workspace` mapped to `band`, and folds in any other difference before renaming the titles.
- [A blind replacement hits a Cargo use of "workspace"] → The replacement skips every `Cargo.toml` and every `--workspace` flag. The final check lists every remaining match, and each one must be Cargo's.
- [A user config written against lua-config's names stops loading] → No release shipped those names. The README lists the new names.
- [Log lines show `BandAdded` instead of `WorkspaceAdded`] → Accepted. Logs are for debugging and have no stable format.

## Migration Plan

None. Nothing has been released. A rollback is a revert of the change's merge.
