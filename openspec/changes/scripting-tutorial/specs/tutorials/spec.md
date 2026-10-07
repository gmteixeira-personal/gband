## Purpose

Defines gband's tutorials: Markdown guides that teach the Lua API by building working code, and the checks in gband's test suite that keep each tutorial's code, screens, API version and coverage of the API equal to the build that ships it.

## ADDED Requirements

### Requirement: Scripting tutorial
gband SHALL include a scripting tutorial in `docs/tutorial/`. It SHALL consist of an index, `docs/tutorial/README.md`, and chapters named `NN-<slug>.md`, where `NN` is two digits counting from `00`. A reader reads the chapters in ascending order of `NN`. The chapters SHALL be:

| chapter | teaches |
|---|---|
| `00-setup.md` | the configuration directory, writing `user/init.lua` from an empty file, reloading, the Lua prompt, `print` and the client log, and the error list |
| `01-keys.md` | key bindings, key tables and modes, key styles, and mouse names |
| `02-actions.md` | actions, action targets, commands, and starting programs in windows |
| `03-options.md` | options and the settings window |
| `04-events.md` | events, event groups and `User` events |
| `05-layout.md` | reading the layout and the view, and acting on windows and bands |
| `06-colors.md` | highlight groups, colorschemes, themes and the terminal palette |
| `07-plugin-windows.md` | plugin windows |
| `08-bars.md` | side bars and the sidebar |
| `09-plugin.md` | moving the reader's code into a plugin: the manifest, the side guard, `gband.plugin`, options of a plugin, and ownership |
| `10-server.md` | the plugin's server half: server events, window state, server commands, `gband.emit` from the server, and `gband.rpc` |
| `11-testing.md` | writing the plugin's tests with `gband test`, and screenshots |

The chapters SHALL start from an empty `user/init.lua` and end with a plugin that has a client half, a server half and tests. A chapter SHALL link to the section of `docs/plugins.md` or `docs/testing.md` that documents each part of the API it introduces. The index SHALL link to every chapter, in order. The Scripting section of `README.md` and the introduction of `docs/plugins.md` SHALL link to the index.

#### Scenario: Index lists the chapters
- **WHEN** a reader opens `docs/tutorial/README.md`
- **THEN** it links to `00-setup.md` first and `11-testing.md` last, and to every other chapter between them in ascending order

#### Scenario: Reached from the README
- **WHEN** a reader opens the Scripting section of `README.md`
- **THEN** it links to `docs/tutorial/README.md`

### Requirement: Chapter directories
Each chapter `docs/tutorial/NN-<slug>.md` SHALL have a directory `examples/tutorial/NN-<slug>/`, and each directory under `examples/tutorial/` SHALL have a chapter. The directory SHALL hold every file the reader has written by the end of the chapter, laid out as the reader installs it:

| path in the chapter directory | installed at |
|---|---|
| `user/` | `user/` of the configuration directory |
| `plugins/<name>/` | `<name>/` of the plugins directory |

The directory SHALL also hold `tests/`, with at least one `_spec.lua` file and the screenshot references its cases compare. `gband test`, run in the chapter directory, SHALL pass every case. Each case SHALL start gband with the chapter's files: the text of `user/init.lua` and `user/server.lua` as the `config` and `server_config` of `g.start`, any other file under `user/` through `files`, and every plugin under `plugins/` through `plugins`. A plugin directory under `plugins/` MAY hold `tests/` of its own, the plugin's tests that the chapter has the reader write; `gband test` run in that plugin directory SHALL pass every case. gband's test suite SHALL run `gband test` in every chapter directory and in every plugin directory of a chapter that holds `tests/`, and SHALL fail when a case in one of them fails.

#### Scenario: Chapter tests run
- **WHEN** gband's test suite runs
- **THEN** `gband test` runs in `examples/tutorial/00-setup/` and in every other chapter directory, and every case passes

#### Scenario: Plugin tests run
- **WHEN** `examples/tutorial/11-testing/plugins/marks/tests/marks_spec.lua` exists and gband's test suite runs
- **THEN** `gband test` also runs in `examples/tutorial/11-testing/plugins/marks/`, with `marks` as the plugin under test, and every case passes

#### Scenario: Broken chapter
- **WHEN** a change renames `gband.keymap.set` and leaves `examples/tutorial/01-keys/user/init.lua` calling it
- **THEN** gband's test suite fails and names `examples/tutorial/01-keys`

#### Scenario: Chapter without a directory
- **WHEN** `docs/tutorial/12-more.md` exists and `examples/tutorial/12-more/` does not
- **THEN** gband's test suite fails and names `docs/tutorial/12-more.md`

### Requirement: Code blocks match the chapter directory
The text of every fenced code block whose info string's first word is `lua`, in a chapter, SHALL appear verbatim in a file under that chapter's directory, including its indentation. A code block whose info string's first word is not `lua` is not checked by this requirement. gband's test suite SHALL fail when a `lua` block appears in no file of its chapter's directory, and SHALL name the chapter and the first line of the block.

#### Scenario: Block found
- **WHEN** `docs/tutorial/01-keys.md` holds a `lua` block whose text appears in `examples/tutorial/01-keys/user/init.lua`
- **THEN** the check passes for that block

#### Scenario: Prompt line found in a spec
- **WHEN** `docs/tutorial/00-setup.md` holds a `lua` block with one line that the reader types at the Lua prompt, and `examples/tutorial/00-setup/tests/setup_spec.lua` types the same line
- **THEN** the check passes for that block

#### Scenario: Stale block
- **WHEN** `docs/tutorial/04-events.md` holds a `lua` block that no file under `examples/tutorial/04-events/` contains
- **THEN** gband's test suite fails and names `docs/tutorial/04-events.md` and the block's first line

### Requirement: Screens match screenshot references
A chapter SHALL show what the reader sees in fenced code blocks whose info string is `screen` followed by a path, relative to the chapter's directory, to a screenshot reference that the chapter's tests compare. The text of such a block SHALL equal the screen rows of that reference, in order, each without its `N|` prefix. The size line, the cursor and the cell attributes are not shown. gband's test suite SHALL fail when a `screen` block names a reference that does not exist, or when its text differs from the reference's rows, and SHALL name the chapter and the reference.

#### Scenario: Screen shown
- **WHEN** `docs/tutorial/08-bars.md` holds a block opened by `` ```screen tests/screenshots/bars_spec/shows-the-bar--two-windows.txt `` and the block's text equals that reference's rows without their prefixes
- **THEN** the check passes for that block

#### Scenario: Screen changed
- **WHEN** a change redraws the sidebar, `gband test --update` rewrites `examples/tutorial/08-bars/tests/screenshots/bars_spec/shows-the-bar--two-windows.txt`, and `docs/tutorial/08-bars.md` still shows the old rows
- **THEN** gband's test suite fails and names `docs/tutorial/08-bars.md` and the reference

### Requirement: Tutorial API version
At least one Lua file under `examples/tutorial/` SHALL set a plugin's `api` field, as the plugins capability defines it. Every `api` field that a Lua file under `examples/tutorial/` sets SHALL equal the `gband.api_version` of the build. gband's test suite SHALL fail when one differs, and SHALL name the file.

#### Scenario: Version raised
- **WHEN** a change raises `gband.api_version` to 2 and `examples/tutorial/09-plugin/plugins/marks/lua/marks/init.lua` still sets `api = 1`
- **THEN** gband's test suite fails and names that file

### Requirement: Every namespace taught
For every top-level field of the client's `gband` table and of the server's `gband` table, collected with only the API installed and before any configuration file runs, the text `gband.<field>` SHALL appear in at least one chapter. The field `core` is exempt. gband's test suite SHALL fail when a field appears in no chapter, and SHALL name the field.

#### Scenario: Every field named
- **WHEN** a test collects the top-level fields of the client's and the server's `gband` tables
- **THEN** each one other than `core`, such as `keymap`, `window_state` and `sessions`, appears as `gband.<field>` in some chapter

#### Scenario: New namespace without a chapter
- **WHEN** a change adds a top-level field `gband.menu` to the client and no chapter names `gband.menu`
- **THEN** gband's test suite fails and names `gband.menu`
