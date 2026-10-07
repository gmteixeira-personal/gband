## ADDED Requirements

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
| `08-key-styles.md` | the module `gband.keystyle` and the modal and direct presets |
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

### Requirement: Quotes of bundled files
A fenced code block in an internals chapter whose info string is `lua` followed by a second word is a quote. The second word SHALL be the path, relative to the configuration directory, of a file that gband writes under `defaults/`, such as `defaults/lua/gband/bar.lua` or `defaults/colors/gruvbox.lua`. The quote's lines SHALL equal a run of consecutive lines of that file's text, verbatim, indentation and blank lines included. The run SHALL occur at exactly one position in the file. gband's test suite SHALL fail when a quote names a path that gband does not write, matches no run of the file, or matches more than one, and SHALL name the chapter, the path and the quote's first line.

#### Scenario: Quote found
- **WHEN** `docs/internals/04-bars.md` holds a block opened by `` ```lua defaults/lua/gband/bar.lua `` whose lines equal lines 1 to 12 of the text gband writes at `defaults/lua/gband/bar.lua`
- **THEN** the check passes for that quote

#### Scenario: Stale quote
- **WHEN** a change edits a line of the bundled `gband.sidebar` module and `docs/internals/09-sidebar-and-errors.md` still quotes the old line
- **THEN** gband's test suite fails and names `docs/internals/09-sidebar-and-errors.md`, `defaults/lua/gband/sidebar.lua` and the quote's first line

#### Scenario: Ambiguous quote
- **WHEN** a quote of `defaults/lua/gband/win.lua` holds only the line `  end`
- **THEN** gband's test suite fails, because the line occurs at more than one position, and names the chapter, the path and `  end`

#### Scenario: Unknown path
- **WHEN** a chapter holds a block opened by `` ```lua defaults/lua/gband/menu.lua `` and gband writes no such file
- **THEN** gband's test suite fails and names the chapter and `defaults/lua/gband/menu.lua`

### Requirement: Every bundled file covered
The covered files SHALL be every file that gband writes under `defaults/`, except the files under `defaults/colors/` other than `defaults/colors/gruvbox.lua`, and except `defaults/lua/gband/theme/catppuccin.lua`. Every non-blank line of every covered file SHALL be in exactly one quote across the internals chapters, and no line of any file SHALL be in more than one quote. Quotes MAY appear in any order and in any chapter. Every covered file SHALL be named by its path, such as `defaults/colors/gruvbox.lua`, outside fenced code blocks in at least one internals chapter. gband's test suite SHALL fail when a line of a covered file is quoted in no chapter, naming the file, the line's number and its text; when a line is quoted twice, naming the file, the line's number and both chapters; and when a covered file is named in no chapter, naming the file.

#### Scenario: Whole file quoted
- **WHEN** gband's test suite runs on the tutorial as written
- **THEN** every non-blank line of `defaults/lua/gband/win.lua`, `defaults/keystyle/modal.lua`, `defaults/init.lua`, `defaults/colors/gruvbox.lua` and every other covered file is in exactly one quote

#### Scenario: New function not explained
- **WHEN** a change adds a function to the bundled `gband.bar` module and no chapter quotes it
- **THEN** gband's test suite fails and names `defaults/lua/gband/bar.lua`, the number of the function's first line and that line's text

#### Scenario: Line quoted twice
- **WHEN** `05-plugin-windows.md` and `06-window-provider.md` both quote the line of `defaults/lua/gband/win.lua` that registers the windows provider
- **THEN** gband's test suite fails and names the file, the line's number and both chapters

#### Scenario: Covered theme changed
- **WHEN** a change adds a field to the `ui` table of the bundled `gruvbox` theme and no chapter quotes the new line
- **THEN** gband's test suite fails and names `defaults/colors/gruvbox.lua`, the new line's number and its text

#### Scenario: Other theme data not covered
- **WHEN** a change edits a color of the bundled `nord` theme and a color in `defaults/lua/gband/theme/catppuccin.lua`, and adds a bundled colorscheme `defaults/colors/ocean.lua`, and no chapter quotes or names any of them
- **THEN** the coverage and naming checks pass

#### Scenario: File never named
- **WHEN** a change adds a bundled module whose copy is `defaults/lua/gband/notes.lua`, every line of it is quoted, and no chapter names `defaults/lua/gband/notes.lua` outside fenced code blocks
- **THEN** gband's test suite fails and names `defaults/lua/gband/notes.lua`

### Requirement: Every primitive taught
For every field of the client's `gband.core` table, at every depth, collected with only the API installed and before any configuration file runs, the text `gband.core.<path>`, such as `gband.core.provide` or `gband.core.palette.set`, SHALL appear outside fenced code blocks in at least one internals chapter. gband's test suite SHALL fail when a field appears in no chapter, and SHALL name the field.

#### Scenario: Primitive explained
- **WHEN** `docs/internals/00-boundary.md` explains `gband.core.provide` in its text
- **THEN** the check passes for `gband.core.provide`

#### Scenario: Primitive only quoted
- **WHEN** the only text that names the timer primitive is a quote of bundled code that calls `core.timer`
- **THEN** gband's test suite fails and names `gband.core.timer`

### Requirement: Internals chapter directories
An internals chapter MAY have a directory `examples/internals/NN-<slug>/` with the chapter's name, and each directory under `examples/internals/` SHALL have a chapter. A chapter SHALL have one when it holds a fenced block whose info string is `lua` alone or starts with `screen`. The directory SHALL follow the layout, the test cases and the runs that the scripting tutorial's chapter directories follow: `user/`, `plugins/<name>/` and `tests/` with at least one `_spec.lua` file, and `gband test` run in it SHALL pass every case. A `lua` block whose info string is `lua` alone SHALL appear verbatim in a file under the chapter's directory. A `screen` block SHALL follow the scripting tutorial's rule for screens, with paths relative to the chapter's directory. `12-own-prelude.md` SHALL have a directory whose `user/lua/gband/prelude.lua` is the prelude the chapter has the reader write, and whose tests show the client loading it. gband's test suite SHALL run `gband test` in every directory under `examples/internals/`, and SHALL fail on a failed case, a missing directory, a `lua` block found in no file, or a `screen` block that differs from its reference, naming the chapter.

#### Scenario: Own prelude runs
- **WHEN** gband's test suite runs
- **THEN** `gband test` runs in `examples/internals/12-own-prelude/`, every case passes, and a case shows that the client loaded `user/lua/gband/prelude.lua` in place of the bundled prelude

#### Scenario: Reader code without a directory
- **WHEN** `docs/internals/04-bars.md` holds a block opened by `` ```lua `` alone and `examples/internals/04-bars/` does not exist
- **THEN** gband's test suite fails and names `docs/internals/04-bars.md`

#### Scenario: Quotes need no directory
- **WHEN** `docs/internals/02-highlights.md` holds only quotes and blocks whose info string starts with neither `lua` nor `screen`
- **THEN** no directory `examples/internals/02-highlights/` is required
