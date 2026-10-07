## Why

gband has no license. It carries no LICENSE file and no `license` field in any `Cargo.toml`, so by default nobody may use, copy or change it. The project should be copyleft so that gband and its changes stay open. Plugin authors and users who copy the tutorial or the default configuration should not be bound by the GPL.

## What Changes

- **GPL for gband**: gband is licensed under GPL-3.0-or-later. `LICENSE` holds the verbatim GPL-3.0 text, preceded by one paragraph that names the license and points to the plugin exception and the MIT paths.
- **Plugin exception**: `LICENSE-EXCEPTION` holds an additional permission under GPLv3 section 7. Lua code that gband loads and that uses gband only through its Lua API, such as a plugin, an init file or a colorscheme, is not a work based on gband and may carry any license, proprietary included.
- **MIT for copied code**: `LICENSE-MIT` licenses the code users copy into their own configuration under MIT: `docs/tutorial/`, `examples/`, the default configurations `crates/lua/src/defaults.lua` and `crates/lua/src/defaults_server.lua`, and the runtime modules under `crates/lua/src/runtime/`.
- **README**: a `## License` section at the end of `README.md`, after `## Acknowledgements`, states the three parts in plain words and links the three files.
- **Cargo**: every package in the workspace declares `license-file = "LICENSE"`, inherited from `[workspace.package]`.

Out of scope:
- Third-party notices for the dependencies shipped in the binary.
- Credits for test vectors that the archived `input-decoder` change ported from Zellij and herdr.
- Per-file license headers.
- A contributor license agreement.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

None. No behaviour of gband changes: the change adds license files, a README section and package metadata. The change sets `skip_specs: true`.

## Impact

- New files: `LICENSE`, `LICENSE-EXCEPTION`, `LICENSE-MIT`.
- `README.md` gains a final `## License` section.
- `Cargo.toml` and every `crates/*/Cargo.toml` gain the license file field.
- Plugin authors may license plugins as they choose. Users may copy the tutorial, the examples and the default configuration under MIT.

## Coordination

### Author
- gmteixeira

### Depends On
- none

### Expected Files
- LICENSE
- LICENSE-EXCEPTION
- LICENSE-MIT
- README.md
- Cargo.toml
- crates/client/Cargo.toml
- crates/core/Cargo.toml
- crates/emulator/Cargo.toml
- crates/harness/Cargo.toml
- crates/lua/Cargo.toml
- crates/protocol/Cargo.toml
- crates/scratch/Cargo.toml
- crates/server/Cargo.toml
- crates/test-support/Cargo.toml
- openspec/changes/license/
