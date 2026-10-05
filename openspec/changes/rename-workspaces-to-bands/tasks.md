## 1. Specs

- [ ] 1.1 For each MODIFIED requirement in this change's delta specs, compare the block with the current `openspec/specs/<capability>/spec.md` block of the same requirement (the old name for the three RENAMED ones), mapping `workspace` to `band` and `W<n>` to `B<n>`. Fold in any other difference, such as wording config-directory or ctrl-space-prefix changed after this proposal. Verify that the mapped main-spec block and the delta block now differ in no line
- [ ] 1.2 Rename the thirteen scenario titles that contain "workspace" to their band form, in both `openspec/specs/` and this change's delta specs: "Open in the empty workspace", "Last pane of a middle workspace closes", "Down to the empty workspace", "Top workspace", "Viewed workspace removed", "Other workspaces", "Old workspace removed", "Open pane on the empty workspace", "Pane opened in the empty workspace", "Middle workspace emptied", "Open in an empty workspace", "Empty workspace" and "Another workspace". Change nothing else in `openspec/specs/` in this step. Verify that `rg -n -i '^#### Scenario:.*workspace' openspec/specs openspec/changes/rename-workspaces-to-bands` prints nothing and that `openspec validate rename-workspaces-to-bands --strict` passes
- [ ] 1.3 Rewrite the Purpose sections of `openspec/specs/animations/spec.md`, `openspec/specs/layout/spec.md` and `openspec/specs/layout-view/spec.md` to say "band", and update the project context line in `openspec/config.yaml`. Verify that `rg -n -i workspace openspec/config.yaml` prints nothing and that `openspec validate --specs --strict` passes

## 2. Free the client's Band name

- [ ] 2.1 In `crates/client/src/animation.rs` and `crates/client/src/render.rs`, rename `Band` to `DrawnBand`, rename its `workspace` field to `band`, rename `Targets::band` and `Shown::band` to `band_height`, and name the render loop variable `drawn`. Update `crates/client/tests/animation.rs` and `crates/client/tests/render.rs` to match. Verify that `cargo test -p gband-client` passes

## 3. Rename the domain term

- [ ] 3.1 Rename the domain term in `crates/core/src` (`layout.rs`, `view.rs`, `event.rs`, `geometry.rs`) and `crates/core/tests`, mapping `Workspaces`, `workspaces`, `Workspace` and `workspace` to `Bands`, `bands`, `Band` and `band`, including `WorkspaceId`, `WorkspaceView`, `ViewAction::WorkspaceDown` and `WorkspaceUp`, `LayoutEvent::WorkspaceAdded` and `WorkspaceRemoved`, `workspace_index`, `next_workspace` and every test function name. Verify that `cargo test -p gband-core` passes
- [ ] 3.2 Apply the same rename to `crates/client`, `crates/server`, `crates/protocol/tests` and `crates/test-support`, and fix any local name that the rename made clash, such as a `band` binding beside a `band` parameter. Rename `crates/client/tests/snapshots/render__empty_workspace_is_blank_without_a_cursor.snap` and `render__workspace_switch_mid_slide.snap` with `git mv` to match their renamed tests, leaving their contents unchanged. Verify that `cargo test -p gband-client -p gband-server -p gband-protocol` passes and that `fd -e new . crates/client/tests/snapshots` prints nothing afterwards
- [ ] 3.3 In `crates/lua`, rename the Lua actions to `focus_band_down` and `focus_band_up` in the action table and in the built-in default configuration, and update `crates/lua/tests`. Keep no alias for the old names. Verify that `cargo test -p gband-lua` passes and that a configuration calling `gband.action.focus_workspace_down` fails to load with an error naming its file and line
- [ ] 3.4 Rename `leader_u_and_i_switch_workspaces`, its `TestEnv` name and its echoed markers in `tests/attach.rs`, and any other match under `tests/`. Verify with `cargo test --test attach`

## 4. Documentation

- [ ] 4.1 Update `README.md` and `summary.txt` to say "band", including the README action table's `focus_band_down` and `focus_band_up`. Verify by reading the rendered README sections

## 5. Check

- [ ] 5.1 Run `rg -n -i workspace --glob '!openspec/changes/archive/**' --glob '!openspec/changes/rename-workspaces-to-bands/**'` and verify that every remaining match is Cargo's workspace: a `Cargo.toml` key, a `cargo ... --workspace` command line, or the terminal-emulator spec's manifests scenario
- [ ] 5.2 Run `cargo test --workspace` and `cargo clippy --workspace --all-targets -- -D warnings`, and check that no comment breaks the rules in `CLAUDE.md`. Verify that both commands pass
- [ ] 5.3 Attach to a fresh server, open a pane, press `prefix u` and `prefix i`, and verify that the view slides to the empty band below and back with no change in behaviour
