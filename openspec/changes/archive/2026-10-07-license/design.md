## Context

gband has no LICENSE file and no license field in any `Cargo.toml`. Every dependency in the shipped binary is under MIT, Apache-2.0, ISC, Zlib, Unicode or MPL-2.0 terms, alone or as one choice of several. `option-ext`, pulled in by `directories`, is MPL-2.0 without the "Incompatible With Secondary Licenses" notice. All of these combine with GPLv3. Apache-2.0 does not combine with GPL-2.0-only.

gband loads user Lua into the same process as its own code. The FSF reads code linked through an interpreter's bindings as a work based on the interpreter. Without an exception, a plugin that calls `gband.*` could be bound by the GPL.

Users copy three kinds of gband code into their own configuration: the tutorial chapters and their directories, the plugin samples under `examples/plugins/`, and the default configuration gband writes under `defaults/` of the configuration directory. gband writes the defaults from `crates/lua/src/defaults.lua`, `crates/lua/src/defaults_server.lua` and the runtime modules under `crates/lua/src/runtime/`.

## Goals / Non-Goals

**Goals:**
- gband and its modifications stay open when distributed.
- A plugin's author chooses the plugin's license.
- Code copied from the tutorial, the examples or the defaults binds the copier to MIT terms only.

**Non-Goals:**
- Network copyleft. gband runs on the user's machines, so AGPL's network clause adds nothing.
- Dual licensing or a CLA. The copyright holder can still add either later.

## Decisions

### GPL-3.0-or-later
The strongest copyleft that suits a local terminal tool, and every shipped dependency combines with it. "Or later" lets gband follow a future GPL without asking every contributor. *Alternatives:* AGPL-3.0, whose network clause has nothing to act on here. GPL-2.0-only, which Apache-2.0 code cannot join. MPL-2.0, whose per-file copyleft lets a fork keep its new files closed.

### The plugin exception, as a GPLv3 section 7 additional permission
Section 7 is the GPL's own mechanism for extra permissions, the one GCC's runtime exception and Autoconf's exception use. `LICENSE-EXCEPTION` holds this text, verbatim:

```
gband Plugin Exception
Additional permission under GNU GPL version 3 section 7

As a special exception, the copyright holders of gband give you
permission to write, license and distribute a Plugin under terms of
your choice, including proprietary terms. A Plugin is not a work based
on gband, even when gband loads and runs it.

"Plugin" means Lua code that gband loads at run time, such as a plugin
directory, an init file, a colorscheme or a module one of these
requires, and that interacts with gband only through the Lua API. A
Plugin may require the API modules gband provides, such as
require("gband.hl"), but contains no code copied from gband apart from
code gband licenses under other terms.

"Lua API" means the gband table and the gband.* modules that gband
provides to Lua code in the client and in the server, as documented in
docs/plugins.md.

If you modify gband, you may extend this exception to your version,
but you are not obliged to do so. If you do not wish to do so, delete
this exception statement from your version.
```

The boundary is the Lua API. A plugin that calls `gband.*` or requires an API module is free. One that pastes gband's GPL code is not. "Code gband licenses under other terms" lets a plugin start from the MIT files. Programs that run in windows need no mention: they talk to gband over a PTY, which the GPL never reaches. *Alternative:* the LGPL, which covers linked libraries and does not fit Lua loaded by an executable.

### MIT by path, not by header
`LICENSE-MIT` holds the MIT text, followed by the list of paths it covers: `docs/tutorial/`, `examples/`, `crates/lua/src/defaults.lua`, `crates/lua/src/defaults_server.lua` and `crates/lua/src/runtime/`. No file gains a header. The project forbids comments that do not explain an upstream restriction or a bug mitigation. The tutorial's checks compare chapter blocks with their files. The defaults are written out verbatim, and the internals tutorial quotes them whole. A header would add noise to every one of them. *Alternative:* a LICENSE file in each directory, which cannot cover the three defaults sources without also covering the Rust beside them.

### `LICENSE` opens with one paragraph
`LICENSE` starts with this paragraph, then a blank line, then the verbatim GPL-3.0 text from `https://www.gnu.org/licenses/gpl-3.0.txt`:

```
gband is licensed under the GNU General Public License, version 3 or
any later version, with the additional permission in LICENSE-EXCEPTION.
The files listed in LICENSE-MIT are licensed under the MIT License.
The text of the GNU General Public License, version 3, follows.
```

A reader opening `LICENSE` alone, or cargo's `license-file`, then learns of the other two files. *Alternative:* the verbatim text alone, which hides the exception from anyone who reads only `LICENSE`.

### `license-file`, not `license`
SPDX lists no identifier for this exception, so no `license` expression can state it. Every package takes `license-file = "LICENSE"` from `[workspace.package]` with `license-file.workspace = true`. No package is published to crates.io, which would reject a non-SPDX `license`.

### The README section
`## License` goes after `## Acknowledgements`, as the last section, in the README's one-sentence-per-line style:

```markdown
## License

gband is licensed under the [GNU General Public License, version 3](LICENSE) or any later version.
A [plugin exception](LICENSE-EXCEPTION) lets plugins, init files and colorschemes carry any license, proprietary included, as long as they use gband only through its Lua API.
The tutorial, the examples and the default configuration are licensed under the [MIT License](LICENSE-MIT), so you can copy them into your own configuration freely.
```

## Risks / Trade-offs

- [GitHub's license detection may not read `LICENSE` as GPL-3.0 because of the opening paragraph] → Check the repository page after the push. If detection fails, move the paragraph to the README and leave `LICENSE` verbatim.
- [Contributions arrive under the GPL, so relicensing later needs every contributor's consent] → Accepted. A CLA can be added before the first outside contribution.
- [The exception's wording is not reviewed by a lawyer] → Accepted for an early project. Review it before a commercial release.

## Open Questions

- The copyright holder's name for the MIT notice. The default is `Copyright (c) 2026 The gband authors`.
