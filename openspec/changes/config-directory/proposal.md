## Why

lua-config evaluates a `defaults.lua` hidden inside the binary and then layers `~/.config/gband/init.lua` on top of it. A user cannot see the defaults without reading the source, and a configuration file only ever patches them. gband should instead lay out its configuration directory on first run: a `defaults/` folder holding the full default configuration as a readable file, and a `user/` folder where the user's own `init.lua` replaces the defaults outright.

## What Changes

- The configuration directory becomes `gband/` under `$XDG_CONFIG_HOME` (`~/.config/gband/` by default), holding `defaults/init.lua` and `user/init.lua`.
- When a process that loads the configuration starts, it creates `defaults/` and `user/` when missing. gband owns `defaults/init.lua`: it writes it when missing and rewrites it when its content differs from the default configuration built into this build. `user/` is created empty.
- **BREAKING** (against lua-config, not yet released): the user's `user/init.lua` no longer runs on top of the defaults. Loading evaluates `user/init.lua` alone. When it does not exist or fails to load at start, the default configuration applies alone.
- An evaluation starts from the option defaults and no key bindings, so a user configuration states every binding it wants, typically by copying `defaults/init.lua`.
- Live reload watches `user/init.lua`. A failed reload still keeps the configuration last loaded; deleting the file returns to the defaults.
- No new subcommand. Creation happens on its own; there is no command to create or reset the configuration.

## Capabilities

### New Capabilities
<!-- none -->

### Modified Capabilities
- `configuration`: the directory layout, first-run creation of `defaults/` and `user/`, the user file replacing rather than patching the defaults, the file watched for reload, and the paths in error reports.
- `client-attach`: the default bindings come from the default configuration, and the scenarios that assumed the user file patches the defaults now state the bindings they rely on.
- `actions`: the binding listing scenario names the default configuration instead of `defaults.lua`.

## Impact

- `crates/lua`: the loader evaluates one file instead of two, gains the directory preparation that writes `defaults/init.lua` atomically and creates `user/`, and the watcher observes `user/` instead of the directory of `init.lua`.
- `src/main.rs`: the client and the server prepare the directory before they load the configuration.
- `tests/common/mod.rs`, `tests/config.rs`: the scratch `XDG_CONFIG_HOME` gains the prepared layout, and the configuration tests write `user/init.lua`.
- `README.md`: the "Configuration" section describes the two folders.
- No migration: lua-config has not shipped, so no user holds a `gband/init.lua` to move.

## Non-goals

- A command to create, reset or inspect the configuration.
- Loading other files from `defaults/` or `user/`, or `require` between them.
- Keeping a user's edits to `defaults/init.lua`.

## Coordination

### Author
- gmteixeira

### Depends On
- lua-config

### Expected Files
- crates/lua/
- src/main.rs
- tests/common/mod.rs
- tests/config.rs
- README.md
- openspec/changes/config-directory/
