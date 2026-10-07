## Context

See proposal.md for the motivation. This change starts after settings-interactive-on-new is archived, which closes the open chain of key-style, mouse, band-sidebar and settings-themes. The state it builds on:

- No window has a name anywhere: not in `crates/core/src/layout.rs`, the server's `Window` (`crates/server/src/window.rs`), the protocol or Lua.
- The emulator wraps `vt100::Parser<Callbacks>` (`crates/emulator/src/lib.rs`). vt100 0.16.2 calls `Callbacks::set_window_title` for OSC 0 and 2 only when the sequence has exactly two parameters. A title holding `;` reaches `unhandled_osc` instead. `CSI 22 t` and `CSI 23 t` reach `unhandled_csi`. gband's `Callbacks` overrides neither title hook today.
- The server spawns each program with portable-pty and keeps the master. `Window::foreground_group()` already calls `MasterPty::process_group_leader()` (tcgetpgrp), used today only to kill the group. Output arrives through `Window::process`. The session loop in `session.rs` is a `tokio::select!` with a settle timer.
- The client draws tiles and floating windows in `draw_tile` (`crates/client/src/render.rs`) through `draw_border`, with the border styles of `WindowBorder` and `WindowBorderFocused`. Floating plugin window titles are drawn in `draw_float` with the same cut rule this change uses.
- `gband.prompt` (`crates/lua/src/runtime/gband/prompt.lua`) holds the one-line editor privately: fitting, pastes, Backspace, Ctrl+U, Escape, Enter.
- After mouse, the protocol version is expected to be 9. This change takes the next one.
- The `gband test` runner builds each case's environment in `crates/harness/src/case.rs`, which already sets `GBAND_ANIMATIONS=off`. gband's own end-to-end tests build theirs in `crates/harness/src/env.rs`, which sets `SHELL=/bin/sh`. Each end-to-end terminal can change its command's environment through the `adjust` hook in `crates/harness/src/terminal.rs`.

## Goals / Non-Goals

**Goals:**
- One source of truth for a window's names: the server. Clients only number and draw.
- Behave like zellij where zellij defines behaviour: manual name, then the program's title, with the title stack honoured.
- Every requirement text this change modifies starts from the text on `dev` when implementation starts, so no sibling change's edit is reverted at archive.
- No existing screen reference gains a title. A reference shows one only when its case asks for titles.

**Non-Goals:**
- Process inspection outside Linux. Other systems fall back to the program's own command name.
- A shared one-line input module for other plugins. The rename prompt lives beside the Lua prompt in the same file.

## Decisions

### Title first, command second, as zellij does
zellij's `current_title` returns the pane's manual name, else the grid's OSC title, else the pane's fixed initial title. It never inspects the foreground process. gband keeps that order and replaces the fixed initial title with the live foreground command, which is strictly more informative.

A shell that sets `user@host:~` at every prompt therefore shows that title, also while a command runs, unless the shell sets the title to the command itself. fish does that by default, and zsh and bash can through a preexec hook.

Alternatives considered:
- *Command first, title second.* Shows `vim` instead of `VIM - notes.md` and `ssh` instead of the remote host's title, losing what titles exist for.
- *Prefer the command while a non-shell process is in the foreground.* It needs a list of shell names, and it overrides programs such as vim and ssh that set a deliberate title. Deferred until a user asks.

### The title stack stores absence
zellij pushes only a title that exists. Then vim, started under a shell that sets no title, pushes nothing, sets `VIM - …`, pops nothing, and leaves its title behind after it exits. gband pushes the absence of a title too, so the pop clears the title and the name falls back to the foreground command. The stack depth is 10, as xterm's is.

### The emulator owns the title; the server reads it after output
`Callbacks` gains `title: Option<String>`, the stack, and a changed flag:
- It implements `set_window_title` for the two-parameter case.
- `unhandled_osc` joins the parameters of a longer OSC 0 or 2.
- `unhandled_csi` handles `22` and `23` with `t`.
- `Grid` exposes `title()` and `take_title_changed()`.

The title is not part of a snapshot, because clients never read it from their own grids. Names travel in their own message.

### Foreground command read on output and on a one-second tick
A new `crates/server/src/process.rs` maps a process group id to a command name:
- It reads `/proc/<pgid>/cmdline` up to the first NUL, takes the base name, and strips a leading `-`.
- It falls back to `/proc/<pgid>/comm`, and then to the window program's command.

The session checks a window after each `Window::process` and on a `tokio::time::interval` of one second in its select loop, and caches the last process group id per window. A changed group or a changed title recomputes the automatic name. Only a changed name sends a message, so the tick costs one tcgetpgrp per window per second and a `/proc` read only when the group changes.

Alternatives considered:
- *Output only.* `sleep 100` writes nothing, so its name would never appear.
- *A faster tick.* 1 s matches the spec's bound. The output-driven check already makes interactive programs update at once.

### Names on the wire: automatic and manual, unsuffixed
The `window name` server message carries the automatic name and the manual name separately, so a client can show the manual name in the rename prompt and Lua can read `manual_name`. The `rename` client message carries the new manual name or nothing. Neither is a session action, because neither changes the layout, and the session-server capability's "Session actions" defines the layout's only mutations.

Alternatives considered:
- *The window state key/value channel.* It is the server Lua's per-window store and is visible to plugins as `WindowStateChanged`. Reserving a key there mixes a host fact into plugin data.
- *Numbered names from the server.* The client recomputes numbers anyway on every layout, and a move would then need a name message for every renumbered window.

### Numbering is a pure function in core
`crates/core/src/names.rs` takes a `Band` and a map from window id to name, and returns the shown names in layout order. The client calls it per band when a layout or a window name message arrives, and caches the result for drawing and for `gband.layout()`. Unit tests cover the scenarios of "Shown name" without a server.

### Border title drawn after the border
`draw_tile` draws the shown name on the top row after `draw_border`, skipping it when the top side is not drawn, with the border's resolved style. It uses the same cut-to-`width - 2` helper as `draw_float`'s plugin title, shared rather than copied.

### Rename prompt beside the Lua prompt
`prompt.lua` registers `prompt.rename` in the same `setup`. The editor state becomes one table per prompt kind: title, leading marker (`:` or none), line, target window and submit function. `open` closes the other kind first. Enter for the rename kind calls `gband.window.rename(target, line)` after checking the target is still in `gband.layout()`.

The presets already set up `gband.prompt`, so binding `prefix N` needs no new plugin. That keeps the key-style capability's list of set-up plugins as it is.

Alternatives considered:
- *A new bundled plugin `gband.rename`.* It would add a plugin to both presets and a requirement change to key-style, for about 20 lines of Lua that share the editor.

### Titles off in tests: an option for users, an environment variable for tests
Users turn titles off with `gband.set({ window_titles = false })`, which reloads like every other option. Tests cannot use the option. Most cases run the default configuration with no `user/init.lua`, and writing one would replace the defaults under test. An environment variable reaches the client in every case with no file. `GBAND_ANIMATIONS` already sets the pattern: one read at start, `off` or `on`, and a warning otherwise. `off` wins over the option, so a case whose configuration sets `window_titles = true` still draws no title unless the case asks for titles.

The runner sets the variable unless `g.start{ window_titles = true }`. A named field matches `keystyle` and `theme` and keeps the variable an implementation detail of the runner. `TestEnv` sets it in gband's own base environment, beside `SHELL=/bin/sh`, and `tests/window_names.rs` removes it through `adjust`.

Shipping titles off in tests together with titles themselves means no screen reference is rewritten with titles and then rewritten back.

Alternatives considered:
- *Regenerate every reference with titles.* Every later change to naming or title drawing would then touch every reference, and plugin authors' `gband test` references would carry titles their plugins never draw.
- *Mask titles when comparing references.* It would hide a regression in the border under a title.
- *A saved file under `user/`, as `keystyle` and `theme` are.* Those are settings-window preferences. Titles are not a settings row.

### `gband.window.rename` validated in the client
`crates/lua/src/control.rs` checks the window number against the client's layout, rejects drawn windows and control characters, trims, and dispatches a rename item in the callback's ordered action list, as `set_position` is dispatched.

### Protocol version
The delta writes 10, on the expectation that mouse takes 9. The first implementation task reads `PROTOCOL_VERSION` on `dev` and uses the next number, correcting the Handshake delta and its scenario when it differs.

## Risks / Trade-offs

- [Deltas written before their siblings archive] → client-attach's "Key bindings" was copied from settings-interactive-on-new's delta, and wire-protocol's "Client messages" from mouse's delta. plugin-testing's "Case environment" was copied from the main spec, and band-sidebar and settings-themes both modify it first. Task 1.1 re-copies each modified requirement from the main spec on `dev`, re-applies this change's additions, and runs `openspec validate --strict`, before any code.
- [Prompt titles hide the command for most shells] → Chosen to match zellij, and documented in the README with the fish default and a zsh/bash preexec example. The manual name covers the rest.
- [`/proc` reads race with exiting processes] → Every read failure falls back, and the next tick corrects it.
- [Key list references shift] → The `N` line is added to the key list, so its first-page and last-page references are regenerated. Each diff is checked to hold only the new line and what moves below it.
- [Most screen tests never draw a title] → `window_names_spec.lua` covers titles on tiles, floating windows, numbering, the cut and both styles. `tests/window_names.rs` covers titles end to end.

## Migration Plan

A client and server of different builds already refuse each other at the handshake. `gband kill-server` and a restart apply the new version, as for every protocol bump.
