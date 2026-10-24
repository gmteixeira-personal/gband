## Context

See proposal.md, Why. The sidebar's drawing code already picks an `Active` group in two places: `sidebar.lua` draws the viewed band's label in `SidebarBandActive` or its most urgent agent state's `Active` group, and draws the focused agent's label in its state's `Active` group. So the look of both band labels and agent labels comes entirely from the six `Active` groups. Their defaults are set in `sidebar.lua` with `gband.hl.default`.

Two places override `SidebarBandActive` with an explicit setting: `theme.apply` in `theme.lua`, which 14 bundled theme files call, and `colors/terminal.lua`. A colorscheme switch removes every explicit setting and keeps the defaults (colorschemes spec, "Switch colorscheme"). No bundled theme sets an agent group.

The colorschemes spec requires every theme other than `default` to give `SidebarBandActive` an explicit setting, and `crates/lua/tests/themes.rs` checks that through `THEME_GROUPS`.

## Goals / Non-Goals

**Goals:**
- Every viewed band label and focused agent label differs from its unfocused peer by `underline = true` alone, under the `default` colorscheme and every bundled theme.
- A user or theme that restyles `SidebarBand` or a `SidebarAgent<State>` group restyles its `Active` peer too.

**Non-Goals:**
- A new option on the `gband.sidebar` plugin. The groups stay the one way to style the labels.
- Changing which group the drawing code picks for a label.
- Changing `SidebarMode`, `SidebarError` or `SidebarDivider`.

## Decisions

**Each `Active` group links to its unfocused peer and adds `underline = true`.** `SidebarBandActive` becomes `{ link = "SidebarBand", underline = true }`, and each `SidebarAgent<State>Active` becomes `{ link = "SidebarAgent<State>", underline = true }`. A link is followed at every resolve (highlights spec, "Links"), so a theme's `SidebarBand` colour reaches the viewed band with no second setting. Alternative: literal specs such as `{ fg = "red", underline = true }`. Rejected because they repeat the peer's colour, and a theme that recolours the peer leaves the `Active` group behind.

**`SidebarAgentUnknownActive` keeps `{ link = "SidebarBandActive" }`.** It resolves to `SidebarBand` plus the underline, the same as linking to `SidebarAgentUnknown`, which itself links to `SidebarBand`. Keeping it avoids a change that alters no resolved style. A user who restyles `SidebarBandActive` still restyles the unknown-state label too, as today.

**Themes keep an explicit `SidebarBandActive`, set to the same link.** `theme.apply` and `terminal.lua` set `{ link = "SidebarBand", underline = true }`. The colorschemes spec's theme-group contract and `THEME_GROUPS` stay as they are. Alternative: drop the line so the default shows through. Rejected because it breaks the "Every theme sets every theme group" requirement and its test for no change in what is drawn.

**The example colorscheme `examples/plugins/window/colors/dusk.lua` follows the bundled themes.** Its `SidebarBandActive` line becomes the same link, so the example teaches the current default.

**`sidebar.lua`'s drawing code is left as it is.** Only five `gband.hl.default` lines change. The `bright_*` palette names stay in `hl.lua`, `palette.lua` and the themes, where other code uses them.

## Risks / Trade-offs

- [An underline on one dim cell is less visible than bright bold text.] The user chose underline only. A user wanting more can still set `SidebarBandActive` to any style, as the "Theme the viewed band" scenario shows.
- [A terminal that does not draw SGR 4 shows no mark on the viewed band.] Every terminal gband targets draws underline. The client already sends it through `Modifier::UNDERLINED` in `crates/client/src/color.rs`.
- [About 55 screenshot references change at the sidebar's active cells.] They are regenerated with `gband test --update` and read before committing, so a change outside column 0 shows up as an unexpected diff.

## Migration Plan

A user who set `SidebarBandActive` or an agent `Active` group in `user/init.lua` keeps that setting, because an explicit setting hides the default. No saved file changes format. Rollback restores the five defaults and the two theme lines.
