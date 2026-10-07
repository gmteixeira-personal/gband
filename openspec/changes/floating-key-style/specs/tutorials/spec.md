## MODIFIED Requirements

### Requirement: Internals tutorial
gband SHALL include an internals tutorial in `docs/internals/`. It SHALL explain gband's bundled Lua as the files are, in the order the prelude loads the API. It SHALL consist of an index, `docs/internals/README.md`, and chapters named `NN-<slug>.md`, where `NN` is two digits counting from `00`. A reader reads the chapters in ascending order of `NN`. The chapters SHALL be:

| chapter | teaches |
|---|---|
| `00-boundary.md` | what the executable provides and what Lua builds: `gband.core`, the providers, the prelude, `require` and the runtimepath, the copies under `defaults/`, and replacing a bundled file by copying it under `user/` |
| `01-themes.md` | colorscheme files, through the `gruvbox` theme |
| `02-highlights.md` | the modules `gband.palette` and `gband.hl`, and the styles provider |
| `03-colorschemes.md` | the modules `gband.colorscheme` and `gband.theme` |
| `04-bars.md` | the module `gband.bar` and the bars provider |
| `05-plugin-windows.md` | the state, drawing and API of the module `gband.win` |
| `06-window-provider.md` | the windows provider of `gband.win`: its input, layout and drawing functions |
| `07-settings.md` | the module `gband.settings` and the settings provider |
| `08-key-styles.md` | the module `gband.keystyle`, the modal, direct and floating presets, and the bundled plugin `gband.desktop` that the floating preset sets up |
| `09-sidebar-and-errors.md` | the bundled plugins `gband.sidebar` and `gband.errors` |
| `10-keylist-and-prompt.md` | the bundled plugins `gband.keylist` and `gband.prompt`, and the module `gband.keyform` |
| `11-default-configs.md` | the default client and server configurations |
| `12-own-prelude.md` | writing the reader's own `gband.prelude` |

The index SHALL link to every chapter, in order. The Scripting section of `README.md`, the section "The prelude and the API modules" of `docs/plugins.md`, and the scripting tutorial's index SHALL link to the internals tutorial's index.

#### Scenario: Index lists the chapters
- **WHEN** a reader opens `docs/internals/README.md`
- **THEN** it links to `00-boundary.md` first and `12-own-prelude.md` last, and to every other chapter between them in ascending order

#### Scenario: Reached from the plugin guide
- **WHEN** a reader opens the section "The prelude and the API modules" of `docs/plugins.md`
- **THEN** it links to `docs/internals/README.md`

#### Scenario: Reached from the scripting tutorial
- **WHEN** a reader opens `docs/tutorial/README.md`
- **THEN** it links to `docs/internals/README.md`

#### Scenario: Floating style explained
- **WHEN** a reader opens `docs/internals/08-key-styles.md`
- **THEN** it quotes `defaults/keystyle/floating.lua` and `defaults/lua/gband/desktop.lua`, and names both paths outside fenced code blocks
