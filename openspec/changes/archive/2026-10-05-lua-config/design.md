## Context

The defaults this change makes configurable are Rust constants today:

- `crates/client/src/bindings.rs` holds `PREFIX` and the `BINDINGS` table. `Leader` runs a one-key prefix state machine over them, and `matches` ignores Shift on characters.
- `crates/core/src/layout.rs` holds `WIDTH_PRESETS` and `DEFAULT_WIDTH`. `Column::new` and `Column::cycle_width` read them.
- `crates/core/src/view.rs` runs the only camera policy in `View::follow`, with an unsigned camera. The animation layer in `crates/client/src/animation.rs` already draws a signed `i64` camera.
- `SessionAction::OpenPane { workspace, after }` carries no program. The server starts every pane with `ServerConfig::program`, the user's shell.

`mlua` (`lua54`, `vendored`, `serialize`, `async`, `send`) and `notify` are already workspace dependencies, but no code uses them. One server process hosts every session of its socket. Every client resolves bindings locally, as `summary.txt` plans: behaviour Lua in the server, presentation Lua and key bindings in the client.

## Goals / Non-Goals

**Goals:**
- One Lua API crate shared by the client and the server, so both evaluate the configuration identically.
- Typed validation: every option reaches Rust through `serde`, and a Lua table never travels further than the loader.
- `defaults.lua` is the only source of the default options and bindings. The test of API completeness is that it reproduces today's tables.

**Non-Goals:**
- Running Lua on the server after loading. The server evaluates the file only to read options.
- A runaway-script guard, or an instruction-count hook. A binding function that loops forever hangs its own client only.

## Decisions

### A new `gband-lua` crate owns Lua
`crates/lua` holds the `gband` table, the option structs, key-name parsing, `defaults.lua` (`include_str!`), the loader and the file watcher. The client and the server depend on it. Neither depends on `mlua` or `notify` directly, so their entries leave `crates/client/Cargo.toml` and `crates/server/Cargo.toml`. Core types gain no Lua dependency: `gband-core` stays free of I/O and of `mlua`.

*Alternative:* putting the loader in the client and sending options to the server. Two clients could then disagree about a session's presets, and the server would need a protocol message for options. Each process reading the same file is simpler and matches the planned split.

### Options are serde structs, merged per call
`gband.set` deserializes its table with `lua.from_value::<OptionsPatch>()`. `OptionsPatch` has `#[serde(deny_unknown_fields)]` and one `Option<_>` field per option, so an unknown name and a wrong type both surface as serde errors that name the field. Each call merges the patch into the accumulated `Options`. Value types:

- `Width`: deserializes from a Lua number. Rejects NaN, infinity, values at or below 0 and above 10000, then converts to `Proportion` by the best rational approximation with a denominator of at most 100, found by continued fractions. Floating `1/3` becomes exactly 1/3.
- `CenterFocusedColumn`: an enum with `#[serde(rename_all = "kebab-case")]`, defined in `gband-core::view` because `View` consumes it.
- `KeySpec`: deserializes from a string through the key-name parser.

`Options` splits into the parts each process owns: `LayoutOptions { default_width, presets }`, defined in `gband-core::layout`, for the server; the prefix and `CenterFocusedColumn` for the client. The presets are sorted and de-duplicated when the patch is merged.

### Errors carry the configuration file and line
The loader names the `init.lua` chunk `@<absolute path>`, so Lua's own syntax and runtime errors already read `path:line: message`. `defaults.lua` is named `@defaults.lua`. A validation error raised from Rust inside `gband.set` or `gband.bind` takes its location from `lua.inspect_stack(1)`, the calling Lua frame, and formats the same prefix. Each binding records its call site, so the end-of-evaluation check, a direct binding of the prefix key, can report the binding's line. A `ConfigError { location: Option<(PathBuf, u32)>, message }` normalises `mlua::Error` variants, including the `CallbackError` wrapper, into one `Display` form.

### Actions are callable userdata; calls queue rather than run
`gband.action.<name>` is a `LuaAction(Action)` userdata with a `__call` metamethod. `gband.bind` accepts a `LuaAction` or a `mlua::Function` and stores `Binding::Action(Action)` or `Binding::Function(RegistryKey)`.

Before running a binding function, the client installs an empty dispatch queue as Lua app data. Calling an action value or `gband.spawn` pushes onto the queue, and fails when no queue is installed, which is how "only inside a binding function" is enforced. After the function returns or fails, the client takes the queue and dispatches each entry in order through the existing `dispatch`. Queuing avoids a re-entrant borrow of `Display` from inside Lua. Sequential dispatch is equivalent to dispatching during the call, because each entry resolves against the view the previous one left.

*Alternative:* passing `&mut Display` into the Lua scope with `Lua::scope`. That works, but it ties `Display` to Lua lifetimes and makes the error path harder to keep partial.

### Spawn widens open pane, not `Action`
`Action`, `SessionCommand` and `ClientAction` stay `Copy`. A queue entry is `Dispatch::Action(Action)` or `Dispatch::Spawn(Option<Program>)`. The client resolves a spawn like open pane and sends `SessionAction::OpenPane { workspace, after, program }`. `Program` is `CommandLine(String)` or `Argv(Vec<String>)`, defined in `gband-core::layout`. `SessionAction` loses `Copy` and keeps `Clone`. The server turns `CommandLine` into `[shell, "-c", line]`, using its own `user_shell()` so the pane-program rule stays the server's.

`ClientAction::SendKey(PREFIX)` becomes `ClientAction::SendPrefix`, resolved against the current keymap at dispatch, because `send_prefix` is bound before the prefix option may change. `SendKey` stays for pass-through keys.

### The keymap replaces the table
`bindings.rs` keeps `Leader`, `Command` and `matches`. `Leader::handle` takes a `&Keymap { prefix, direct, prefixed }` built from the loaded configuration and returns `Command::Run(&Binding)`. The prefix sequence starts only when `prefixed` is non-empty. A reload replaces the keymap and resets the leader.

### Each process watches the configuration directory
A `notify` watcher observes the directory that holds `init.lua`, non-recursively, and filters events by file name. Watching the directory catches the write-to-temporary-then-rename that editors use. When the directory does not exist, the watcher observes its parent until the directory appears. Events within 100 ms collapse into one reload. A reload evaluates in a new `Lua` on a blocking thread and sends the result to the owner over a channel:

- The client's main loop receives `Result<ClientConfig, ConfigError>`. On success it swaps the keymap, the camera policy and the `Lua` that owns the binding functions, and clears the banner. On failure it keeps all three and sets the banner.
- The server publishes `LayoutOptions` through a `tokio::sync::watch` channel. Each session reads the current value when it applies an action. A failure is logged with `tracing::warn!`.

`Layout::apply` and `Layout::open` take the options, or the width they need, as a parameter, so `gband-core` stays free of global state and its tests stay pure.

### The camera policy lives in the view
`View` holds the `CenterFocusedColumn` policy, set at construction and on reload, and a signed camera of `i64`. `follow` receives the previously focused column's index when focus moved within the workspace, which `on-overflow` needs to pick the neighbouring column S. Rendering and `shown` already subtract the camera from strip positions, so the signed camera only removes the `u32` conversion in the animation targets.

### The banner is drawn last
`Display` holds `Option<String>` for the latest error. `render` draws it after the ribbon on the bottom row, cut to the width, in a reversed error style. The screen area sent to the server is unchanged, so no pane resizes when an error appears.

### Tests isolate the configuration
Every integration harness that starts gband sets `XDG_CONFIG_HOME` to a temporary directory, so a developer's own `init.lua` never reaches a test. Configuration tests write `init.lua` into that directory.

## Risks / Trade-offs

- [Two processes evaluate the same file and could see different versions during an edit] → Each reload is atomic within its process, and both settle on the last saved version within the debounce window.
- [The server and the client may run with different environments, so `os.getenv` in `init.lua` can differ between them] → Accepted. Options the server owns are read only by the server.
- [A binding function that never returns freezes its client] → Out of scope. The Lua API has no blocking call, so only a deliberate loop can do this.
- [Rational approximation could surprise a user who writes `0.333`] → It reads as 1/3, which is the intended value. The scenario in the spec pins `0.35` to 7/20.
- [Dropping `Copy` from `SessionAction` touches every match site in the server] → The compiler finds each one, and they are mechanical.
- [Watching a directory that does not exist yet] → The parent-directory fallback covers creating `~/.config/gband/` after start. A missing `~/.config` is treated as no configuration, with no watch.

## Migration Plan

No user data migrates. With no `init.lua`, behaviour is unchanged. Client and server come from one build, and the existing handshake refuses a mismatched server, so the wider `OpenPane` message needs no compatibility path.
