## Why

gband's scripting documentation is a reference: `docs/plugins.md` and `docs/testing.md` describe every field, ordered by API area, but nothing leads a new user from an empty `user/init.lua` to a working plugin. A tutorial written by hand drifts from the code with every change. gband has no CI, and a release is `dev` merged into `main`, so no release step exists to catch that drift. The tutorial must therefore be checked by `cargo test`, which every gated merge already runs.

## What Changes

- **Scripting tutorial**: a new `docs/tutorial/` holds an index, `docs/tutorial/README.md`, and twelve chapters, `00-setup.md` to `11-testing.md`. They lead from an empty `user/init.lua` through key bindings, actions and commands, options and settings, events, the layout, colors, plugin windows, bars, a plugin, its server half, and its tests. Each chapter teaches by example and links to `docs/plugins.md` and `docs/testing.md` for the full reference.
- **Chapter directories**: each chapter has a runnable directory, `examples/tutorial/NN-<slug>/`. It holds the reader's files at the end of the chapter, laid out as they are installed: `user/` for the configuration directory's `user/`, and `plugins/<name>/` for a plugin. Its `tests/` holds `_spec.lua` files and screenshot references. A reader runs `gband test` in the directory to check their own copy.
- **Checks in `cargo test`**:
  - every chapter directory's tests pass under `gband test`;
  - every fenced `lua` block in a chapter appears verbatim in a file of that chapter's directory;
  - every `screen` block in a chapter equals the text of a committed screenshot reference of that chapter;
  - every `api` field in the chapter directories equals `gband.api_version`, so raising the API version fails the suite until the tutorial is revised;
  - every top-level field of the client's and the server's `gband` table, except `core`, is named in at least one chapter.
- **Links**: the README's Scripting section and the introduction of `docs/plugins.md` link to the tutorial's index.

Out of scope:
- A tutorial that walks through gband's bundled Lua and the `gband.core` primitives. A later change, `lua-internals-tutorial`, adds it to the `tutorials` capability and reuses this change's checks.
- Copies of the tutorial written under `defaults/` of the configuration directory.
- A `gband test` mode that runs gband's own Lua spec suite against a user's copies of the bundled files.
- New fields in `gband.test` or new `g.start` options. The chapter tests use the harness as it is.

## Capabilities

### New Capabilities

- `tutorials`: the scripting tutorial, its chapter directories, and the checks that keep its code, its screens, its API version and its coverage of the API current.

### Modified Capabilities

None.

## Impact

- `docs/tutorial/`: new, the index and twelve chapters.
- `examples/tutorial/`: new, twelve chapter directories with `user/`, `plugins/` where a chapter has a plugin, and `tests/` with specs and screenshot references.
- `crates/lua/tests/tutorial.rs`: new, the checks on code blocks, screens, the API version and namespace coverage.
- `crates/lua/tests/common/docs.rs`: a constructor from text, so the coverage check can search every chapter.
- `tests/lua_specs.rs`: a test that runs `gband test` in every chapter directory.
- `README.md` and `docs/plugins.md`: links to the tutorial.
- No change to the executable, the Lua API, the protocol or the harness, and no new dependency.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- docs/tutorial/
- examples/tutorial/
- crates/lua/tests/tutorial.rs
- crates/lua/tests/common/docs.rs
- tests/lua_specs.rs
- README.md
- docs/plugins.md
- openspec/changes/scripting-tutorial/
