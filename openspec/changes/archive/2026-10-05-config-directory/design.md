## Context

lua-config, which this change builds on, adds the `gband-lua` crate in `crates/lua`. Its loader evaluates an embedded `defaults.lua`, then `$XDG_CONFIG_HOME/gband/init.lua` when it exists, in one `Lua`. Its watcher observes the directory holding `init.lua`, or that directory's parent until it appears. The client and the server both load and watch from `src/main.rs`, and the integration harness in `tests/common/mod.rs` points `XDG_CONFIG_HOME` at a scratch directory.

This change keeps the `gband` API, the option types, error formatting, the banner and the last-good rule. It changes where files live, which files are evaluated, and who writes them.

## Goals / Non-Goals

**Goals:**
- A user sees the full default configuration as a file on disk, and starts their own by copying it.
- The user's `user/init.lua` replaces the defaults instead of patching them.
- The directory lays itself out on first run, with no command to run.

**Non-Goals:**
- Create, reset or show-config subcommands.
- Preserving edits to `defaults/init.lua`; gband owns that file.
- Migrating a lua-config `gband/init.lua`, which no release ever read.

## Decisions

### The built-in text stays the source; the disk copy mirrors it
The default configuration remains `include_str!` in `gband-lua`. Preparing the directory writes that text to `defaults/init.lua` when the file is missing or differs, and the loader evaluates the built-in text, named `@<configuration directory>/defaults/init.lua` so its errors and stack frames name the file the user can open. Evaluating the text in memory rather than the file means a read-only or half-written disk copy can never leave a process without bindings, and the result is identical because preparation makes the file match.

*Alternative:* evaluating `defaults/init.lua` from disk and falling back to the built-in text on failure. Two code paths produce the same configuration, and an edit to the disk copy would take effect until the next start, contradicting the rule that gband owns the file.

### One file is evaluated
`load` evaluates `user/init.lua` when it exists, otherwise the default configuration, never both. Evaluation starts from `Options::default()` and an empty binding map. `Options::default()` holds the "Options" table defaults, built on `LayoutOptions::default()`, and the "Defaults reproduce the built-in behaviour" test pins that the default configuration sets the same values. A failed load at start falls back to the default configuration through the existing last-good path, whose initial value becomes the default configuration's result.

### Preparation is a separate step before loading
`gband_lua::prepare(dir) -> Result<()>` creates `dir`, `dir/defaults` and `dir/user` with `create_dir_all`, then compares `defaults/init.lua` with the built-in text and, when it differs or is missing, writes a temporary file in `defaults/` and renames it over `init.lua`. The rename makes the replacement atomic, so a client and a server starting together each write identical bytes and neither reads a partial file. `src/main.rs` calls it for the client and the server before `load`, and logs an error with `tracing::warn!` without stopping. Keeping it out of `load` keeps reloads free of writes.

### The watcher observes `user/`
The watcher observes `user/` non-recursively and filters for `init.lua`. Preparation runs first, so `user/` normally exists; when it cannot be created, the watcher keeps lua-config's parent fallback, observing the configuration directory until `user/` appears. `defaults/` is never watched, so the rewrite at start and any hand edit cause no reload.

### Paths come from one function
`config_dir()` returns `gband` under an absolute `XDG_CONFIG_HOME`, else `~/.config/gband`, and `None` without a home directory. `user_file()` and `defaults_file()` derive from it. With `None`, a process skips preparation and the watcher and runs the default configuration, as lua-config treats a missing `~/.config`.

## Risks / Trade-offs

- [A user expects `user/init.lua` to patch the defaults and loses every binding by writing one line] → The README states that `user/init.lua` replaces the defaults and to start from a copy of `defaults/init.lua`. The "User file replaces the defaults" scenario pins the behaviour.
- [A user's copy of the defaults goes stale after an upgrade adds a binding] → Accepted; that is the cost of replacing instead of patching. `defaults/init.lua` always shows the current defaults to compare against.
- [Writing to the user's config directory on every start] → Only when the content differs, so a steady state writes nothing.
- [Two processes racing to create the directories] → `create_dir_all` tolerates an existing directory, and the temporary-file rename tolerates a concurrent writer of the same bytes.

## Migration Plan

None. lua-config has not shipped, so no user holds a `gband/init.lua`. A file left at that path from a development build is ignored.
