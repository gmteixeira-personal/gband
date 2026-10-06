## Context

See proposal.md for the motivation. This change starts after navigation-mode, and through it after key-list and center-column. The state it builds on:

- `prefix` is a mode labelled `navigation`. An unbound key in it is discarded, so typed text reaches a plugin window only once `root` is active. `gband.keymap.enter("root")` is allowed in any callback.
- `crates/lua/src/runtime/gband/win.lua` holds every plugin window in Lua. `Runtime::plugin_window_key` in `crates/lua/src/runtime.rs` calls its `key` hook with the key's name only. The hook runs a `keys` entry, then the default keys, which include `q` after key-list.
- `Controls::paste` in `crates/client/src/lib.rs` drops a paste while a plugin window is focused, before Lua sees it.
- The client hides the terminal cursor while a floating plugin window is focused (`render.rs`).
- Bundled modules in `bundled::MODULES` load through `require` and receive no host table. The modules in `bundled::API` receive it. `host.call(owner, label, fn, ...)` runs `fn` through `guard::isolated`: a fresh instruction budget, the given owner, and an error reported without failing the caller. With no label, a stop by the limit marks the owner failed, and no owner means no plugin is marked.
- `gband.keylist` and `gband.errors` open plugin windows that take typed keys, and each enters `root` itself when it opens.

The client-attach delta copies "Key bindings" as navigation-mode, key-list and center-column leave it together. It also writes `navigation keys` in the scenario "Open the key list", which navigation-mode's own key-list delta requires.

## Goals / Non-Goals

**Goals:**
- Any plugin window can take text, so the prompt is a plain user of `gband.win`.
- A line typed at the prompt can do whatever a binding in `user/init.lua` can, and nothing it does can disable the prompt.
- Every error from a line reads `prompt:1: <message>`.

**Non-Goals:**
- A text cursor drawn by Rust inside plugin windows.
- Any change to keys and pastes for a plugin window without `on_input`.

## Decisions

### Text input is a plugin window option, `on_input`
`win.lua` accepts `on_input` as a common field, checked like `on_close`. The `key` hook tries, in order: the `keys` entry, `on_input` for a key that types a character, then the default keys. `Runtime::plugin_window_key` computes the text in Rust and passes it as a third argument: the character of a character key with neither Ctrl nor Alt, and nil otherwise.

Alternatives:
- *Derive the text from the key name in Lua.* Rejected. It depends on how `key_name` spells keys, such as `space`.
- *A prompt-only path in Rust.* Rejected. The key list, the error list and later plugins would each need their own.

### Pastes reach Lua unchanged
While a plugin window is focused, `Controls::paste` runs a new `Runtime::plugin_window_paste` through `react`, as `press` runs keys, and applies its outcome. `win.lua`'s new `paste` hook calls `on_input` with the text as pasted, or drops it when the plugin window has none. Turning line breaks into spaces is the prompt's choice, not the plugin window's.

Alternative: *strip line breaks for every plugin window.* Rejected. A plugin that takes several lines would lose them.

### The prompt is a bundled plugin with the host table
The line must run as code of no plugin, with its own budget. The public API cannot do that, and it should not: a public way to run code as no plugin would let any plugin register names outside its namespace. So `install_searcher` takes the client's host table, and the bundled searcher passes it to a bundled module's chunk as its argument, as the API modules already get it. `gband/prompt.lua` keeps it with `local host = ...`. The other bundled modules ignore it. The server and test sides pass nil. The host table is never returned by `require`.

Alternatives:
- *Run the line directly in the Enter callback.* Rejected. It shares the prompt's budget and owner: an endless loop marks `prompt` failed until the next reload, and `gband.action.register("greet", fn)` registers `prompt.greet`.
- *A public `gband.eval(text)`.* Rejected for the namespace escape above.
- *An API module that puts the plugin in `package.preload`.* Rejected. `preload` is searched before the runtimepath, so `user/lua/gband/prompt.lua` could not override it as it can other bundled modules.

### The line runs through `host.call(nil, nil, ...)` as chunk `@prompt`
On Enter, the prompt closes its plugin window and then calls `host.call(nil, nil, run, line)`. `run` compiles with `load(line, "@prompt", "t")` and raises a compile error with `error(message, 0)`, so syntax errors take the same report path as runtime errors. No owner means the line belongs to no plugin and a stop marks no plugin failed. No label means the message is not prefixed with a plugin name.

The `@` makes `prompt` the chunk's file name. Lua's messages then read `prompt:1: ...`, and the guard's limit hook, which takes locations only from `@` sources, reports `prompt:1: instruction limit exceeded`. Mode `"t"` refuses binary chunks. The line runs inside the Enter callback, so its dispatches join that callback's queue.

Alternatives:
- *A label, as colorschemes use `colors/<name>`.* Rejected. With chunk `@prompt`, every message would read `prompt: prompt:1: ...`, and with chunk `=prompt` the limit stop would have no location.
- *Chunk name `=prompt`.* Rejected for the missing location on a stop.

### Enter closes before it runs
A line that raises an error or is stopped leaves no prompt behind. A floating plugin window that the line opens takes focus, as any new one does. A focus change the line makes needs no focus hold, because the prompt is already closed.

### `prompt.open` enters `root` itself, and `prefix :` binds the action
This follows `keylist.open` and `errors.open`. Any binding of `prompt.open`, from any mode, then works without repeating `gband.keymap.enter("root")`. The default binding is an action binding, so its description is the action's, `run Lua`, which the hints segment and the key list show. Enter on the key list's `:` line opens the prompt over the list.

Alternative: *a function binding in the defaults that enters `root` and calls the action.* Rejected. It gives the same result for `:` only, and every other binding of `prompt.open` would have to repeat it.

### Bottom rows, full width, cursor as a span
The box sits where Neovim's command line sits. `prompt.open` reads `gband.view().cols` and `rows`, opens with `width = cols`, `height = 3`, `row = rows` and `col = 0`. Placement cuts the row so the box ends on the ribbon area's last row, and cuts the box to the ribbon area each time it is drawn, so a smaller ribbon still shows it whole. The prompt stores the content area's width from `gband.win.info` after opening and from `on_resize`, and fits the line from it.

The line is `{ ":" .. shown, { text = " ", hl = "PromptCursor" } }`. `shown` drops leading characters, measured with `gband.ui.width`, until the line fits. `PromptCursor` gets `gband.hl.default("PromptCursor", { reverse = true })` when the module loads, as the key list does with its groups.

### Return values are dropped
`gband.notify` asks the terminal for a desktop notification, and `print` writes to the log. Neither is a place to show what `return 1 + 2` returns. A user who wants a value calls `gband.notify(tostring(x))` in the line.

## Risks / Trade-offs

- [With `on_input`, `j`, `k` and `q` stop scrolling and closing] → Only plugin windows that set `on_input` change. The docs list the order of `keys`, `on_input` and the defaults.
- [Joining pasted lines with spaces can change a paste's meaning: a `--` comment swallows the rest] → Multi-line input is out of scope. The docs say a paste becomes one line.
- [Bundled modules get the host table] → It is gband's own code. A module that overrides a bundled one from the runtimepath loads through the runtimepath searcher and gets no host.
- [A line that calls a plugin's registered action that loops forever marks that plugin failed] → This is the plugins capability's rule for that plugin's own callback. The prompt is unaffected.
- [The prompt does not follow a ribbon area that grows while it is open] → It is open for one line. Opening it again places it anew.
- [sidebars-borders-steps also modifies "Open a plugin window" and "Defaults use the public API", and its order with this change is not fixed] → Task 0.1 folds in whatever an earlier archive left. sidebars-borders-steps' own fold-in does the same if it archives later.
- [An error from a line stays shown until a reload or a newer error] → This is how every callback error behaves today.

## Migration Plan

No data migration. The default configuration gains the prompt. A user file that copies the defaults from before this change has no prompt until it adds `gband.plugin("gband.prompt")` and binds `prompt.open`, which the README shows. A rollback is a revert of the change's merge.
