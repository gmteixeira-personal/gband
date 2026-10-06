## Context

See proposal.md for the motivation. This change starts after navigation-mode, sidebars-borders-steps, clear-errors and lua-prompt are archived, and after key-list, which navigation-mode depends on. The state it builds on:

- `crates/lua/src/defaults.lua` holds navigation mode: the `prefix` mode declaration, every default binding, the setup of `gband.keylist`, `gband.prompt` and `gband.errors`, then `gband.statusline` and the segment plugins. `crates/lua/src/directory.rs` writes it to `defaults/init.lua`. The client uses it only when `user/init.lua` is missing, and a `user/init.lua` replaces it wholly.
- Bindings and modes can be made only while the configuration loads. A running client cannot change its bindings. It can only reload.
- `crates/lua/src/watch.rs` watches `user/` recursively in every process, client and server. A created, written, renamed or removed file whose name ends in `.lua` reloads that process's configuration after a 100 ms debounce. A reload closes every plugin window.
- `crates/lua/src/runtime.rs` puts the `user` directory first on `gband.runtimepath`. Module lookup looks only under `<entry>/lua/`. `source_plugins` skips the `user` entry. So a file directly in `user/`, other than `init.lua` and `server.lua`, is never evaluated.
- Lua cannot see the configuration directory. `gband.runtimepath[1]` happens to be `user/`, but the init file may change the list. `io` and `os` are available: the client's Lua state loads the safe standard library, and `gband/colorscheme.lua` already uses `io.open`.
- `crates/client/src/lib.rs` emits `Attached` once per client process, when the server's requirements arrive or on `attached()`. A reload emits `ConfigReloaded` and, when the table was not `root`, `KeyTableChanged`. It never emits `Attached`.
- `gband.win.open` is refused while the configuration loads. A chooser opened at start therefore needs a callback, such as an `Attached` handler.
- End-to-end tests (`tests/common/mod.rs`) and `gband test` cases (`crates/harness/src/case.rs`) run with a fresh configuration directory, and many use the default configuration.

## Goals / Non-Goals

**Goals:**
- Each style is one plain Lua file a user can read and copy. The defaults file shows the choice, not the bindings.
- A choice survives restarts and applies through the reload that already exists. No new reload API.
- The default configuration still evaluates alone to navigation mode, so the tests that compare it with the client-attach table keep their meaning.

**Non-Goals:**
- A Rust notion of key styles. Rust adds only `gband.config_dir` and writes the preset copies.
- Changing bindings in place without a reload.

## Decisions

### Presets are bundled modules, and `use` requires one
`crates/lua/src/runtime/gband/keystyle/modal.lua` and `direct.lua` join `MODULES` in `bundled.rs`, so `require("gband.keystyle.modal")` finds them. Each is a list of top-level calls, like a user's `init.lua`. Each first sets up `gband.keylist` and `gband.prompt`, whose actions it binds, so `gband.keystyle.use()` works in any `init.lua`. The defaults no longer set those plugins up. `gband.errors`, whose `errors.open` no preset binds, stays in the defaults with `gband.statusline` and the segment plugins. The defaults set it up right after `use()`, so the plugins keep the order lua-prompt left: key list, prompt, error list, status line, segments. The modal preset then declares the mode and binds. Function bindings are local functions in the preset, so `escape` and `enter` share one `interactive` function.

Because lookup searches the runtimepath first, a user's `user/lua/gband/keystyle/modal.lua` shadows the bundled preset. That needs no code and is documented.

Alternatives considered:
- *Presets as plugins with `setup`.* A plugin's errors become plugin errors that do not fail the load, so a broken preset would leave a half-bound configuration. Its callbacks would also belong to a plugin. A preset is configuration, not a plugin.
- *One module with a style argument.* It mixes both styles in one file, which no longer reads as an example of one style.
- *The defaults set up `gband.keylist` and `gband.prompt` before `use()`.* Then `use("direct")` alone in a user file fails to load, because `gband.action["keylist.open"]` is nil.

### `gband.keystyle` is a client API file
`crates/lua/src/runtime/gband/keystyle.lua` joins `bundled::API`, like `win.lua`, so `gband.keystyle` exists before the init file runs. It holds `use`, `saved` and `choose`, and keeps per-load state in locals: whether `use` ran. `keystyle` joins `CLIENT_ONLY` in `crates/lua/src/sides.rs`.

`use(style)` checks the phase with the same rule `gband.keymap.set` uses, refuses a second call, resolves nil through `saved()`, and calls `require("gband.keystyle." .. style)`. Errors raised in the preset propagate, so they fail the load at the preset's line.

### The choice is `user/keystyle.lua`, holding `return "<style>"`
The name ends in `.lua`, so saving it reloads every process through the existing watcher, and the reloaded configuration's `use()` applies it. The file is never evaluated as configuration, because of the runtimepath rules in Context. `saved()` reads it with `loadfile(path, "t", {})` and a protected call. Text mode rejects bytecode, and the empty environment leaves the chunk nothing to call. The instruction limit already stops a loop. Anything but `"modal"` or `"direct"` reads as nil, so a broken file falls back to modal and the next start offers the chooser, whose save repairs it.

Saving writes `return "<style>"` and a newline to a temporary file in `user/` whose name does not end in `.lua`, then `os.rename`s it over `user/keystyle.lua`. A failed write or rename removes the temporary file and raises an error naming `user/keystyle.lua` and the reason.

Alternatives considered:
- *Write `user/init.lua`.* Running the chooser again would overwrite the user's edits.
- *A plain-text `user/keystyle` file.* It would not end in `.lua`, so it needs a reload API or a change to the watcher's rule.
- *Rust reads and writes the choice.* It hides a short, example-worthy flow behind the host, and still needs the path in Rust.

### `gband.config_dir` is a public value
`runtime::install` sets `gband.config_dir` from `Locations::config`, on the client and the server, beside `gband.runtimepath`. It is nil without locations, which is the case for `gband_lua::defaults()`. The side guard must know the name, so a test file reading it errors even though the value is nil on both sides when no directory exists.

Alternatives considered:
- *A `host` field visible only to bundled API files.* `keystyle.lua` would then not read as plain Lua, and plugins that want the path could not get it.
- *`gband.runtimepath[1]`.* The init file may change the list.

### The offer is an `Attached` handler in the defaults
`defaults.lua` registers `gband.on("Attached", fn)`. `fn` calls `gband.keystyle.choose()` when `gband.config_dir` is set and `gband.keystyle.saved()` is nil. `Attached` comes once per client, so a reload never offers again, and each new attach offers until a style is saved. The `config_dir` check keeps `gband_lua::defaults()`, used by unit tests and by a start without a home directory, from opening a chooser that cannot save.

Alternative considered: *a flag in the Rust client that remembers the offer.* It is unnecessary, because the event already fires once.

### The chooser is a plain floating plugin window
`choose()` calls `gband.keymap.enter("root")`, then `gband.win.open` with a border, a title, `cursorline = true`, two lines, and `keys = { enter = pick }`. The prefix key is shown with the key form from key-list's `gband.keyform`. j, k, the arrow keys, Escape and `q` keep the plugin window defaults, so dismissing saves nothing. `pick` reads `gband.win.info(win).cursor`, closes the chooser, then saves. The reload that follows within a second applies the style. The module keeps the open chooser's number and focuses it when `choose()` runs again. The window belongs to the plugin of the code that called `choose()`, which is none for the defaults' handler.

### Copies of the presets in `defaults/keystyle/`
`directory.rs` writes the two preset texts to `defaults/keystyle/modal.lua` and `direct.lua` with the same write-if-different, rename-into-place rule as `defaults/init.lua`. A copied `defaults/init.lua` holds only `gband.keystyle.use()`. A user who wants to edit the bindings copies a preset's body instead. Nothing loads these copies.

### Tests save the modal style by default
`tests/common/mod.rs` writes `user/keystyle.lua` with `return "modal"` when it creates a `TestEnv`, and offers a constructor that writes none, for the first-run and offer tests. `crates/harness/src/case.rs` does the same from the `keystyle` start option, before it writes `files`. `runner.rs` parses the option. Existing cases and screenshots that use the default configuration then stay as they are.

### Order with other changes
The deltas are written against the specs as navigation-mode, sidebars-borders-steps, clear-errors and lua-prompt leave them. "Key bindings" and lua-prompt's "Default setup" are copied as lua-prompt leaves them. "Defaults use the public API" holds sidebars-borders-steps' `gband.errors` and `gband.statusline`, and lua-prompt's `:` scenario. "Side guard" holds sidebars-borders-steps' `bar` and `errors` and clear-errors' `clear_errors`. sidebars-borders-steps removes `statusline_position` without modifying plugin-testing, so the copy of "Case environment" restates the scenario "Configuration of the case" with a status line set up on the right.

## Risks / Trade-offs

- [A `user/init.lua` that does not call `gband.keystyle.use()` ignores the choice] → The chooser still saves and reloads, and nothing changes. README and `docs/plugins.md` say that the choice applies where the configuration calls `use()` with no argument.
- [Saving reloads the server's configuration too] → The server keeps every column's width on a reload. The cost is one extra evaluation of `server.lua`.
- [A reload closes every plugin window, including another plugin's] → The user just chose, so the reload is expected. The proposal accepts it.
- [Several clients share one configuration directory] → Each attach offers the chooser while nothing is saved. The first save reloads every client into the new style.
- [A lone Escape can merge with following bytes into an Alt sequence] → End-to-end tests that dismiss the chooser send Escape and wait for it to close before sending more, or use `q`.
- [The copies of "Key bindings", "Defaults use the public API", "Side guard", "Case environment" and lua-prompt's "Default setup" replace the requirements that earlier changes leave] → lua-prompt and clear-errors are declared dependencies, and through them navigation-mode and sidebars-borders-steps, so all four archive first and the copies hold their final text.

## Migration Plan

No data migration. A user with no `user/init.lua` sees the chooser on the next start. A user's own `user/init.lua` keeps working, and opts in with `gband.keystyle.use()`. Deleting `user/keystyle.lua` brings the offer back on the next start. A rollback is a revert of the change's merge. A `user/keystyle.lua` left behind is then never read.
