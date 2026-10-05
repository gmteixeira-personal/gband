## MODIFIED Requirements

### Requirement: Components
`gband.ui.statusline.add(spec)` SHALL add a component and return its full id. `spec` SHALL be a table with these fields:

| field | value | default |
|---|---|---|
| `id` | a non-empty string | the plugin's name |
| `render` | a function | required |
| `align` | `"left"`, `"center"` or `"right"` | `"left"` |
| `priority` | a number | `0` |
| `order` | a number | `0` |
| `hl` | a group name | `"StatusLineSegment"` |
| `redraw_on` | a list of built-in event names, as the lua-events capability lists them, and `"User"` | empty |
| `redraw_interval` | an integer number of milliseconds, at least 100 | none |
| `fill` | a boolean | `false` |

A component whose `fill` is true is a fill component, which "Render triggers" renders again when the width left to it changes.

A component added by code that belongs to a plugin SHALL belong to that plugin, and its full id SHALL follow the plugins capability's namespacing of names. When such code omits `id`, the full id SHALL be the plugin's name. Code that belongs to no plugin SHALL give `id`, which SHALL be used as given. A missing `render`, a field of the wrong type or value, an unknown event name, an omitted `id` outside a plugin, or a full id that another component already has SHALL be an error at the line of the call, and SHALL add nothing.

`gband.ui.statusline.remove(id)` SHALL remove the component whose full id is `id` and return `true`, or return `false` when no component has that id. `gband.ui.statusline.list()` SHALL return one table per component, in ascending byte order of full ids, each holding `id`, `align`, `priority`, `order`, `hl`, `fill`, `plugin` (the owner's name, or nil) and `enabled`. Changing a returned table SHALL NOT change the component. All three functions SHALL be callable while the configuration loads and in any callback.

#### Scenario: Plugin component without an id
- **WHEN** the plugin `pane` calls `gband.ui.statusline.add({ render = fn })`
- **THEN** the call returns `"pane"`

#### Scenario: Plugin component with an id
- **WHEN** the plugin `pane` calls `gband.ui.statusline.add({ id = "count", render = fn })`
- **THEN** the call returns `"pane.count"`

#### Scenario: User component
- **WHEN** `user/init.lua` calls `gband.ui.statusline.add({ id = "host", render = fn })`
- **THEN** the call returns `"host"`

#### Scenario: Duplicate id
- **WHEN** line 5 of `user/init.lua` adds a component with id `host` a second time
- **THEN** loading fails with an error at `user/init.lua` line 5 naming `host`

#### Scenario: Unknown event
- **WHEN** line 3 of a plugin's `setup` adds a component with `redraw_on = { "FocusChange" }`
- **THEN** a plugin error at that line names `FocusChange`

#### Scenario: Fill of the wrong type
- **WHEN** line 4 of `user/init.lua` adds a component with `fill = "yes"`
- **THEN** loading fails with an error at `user/init.lua` line 4 naming `fill`

#### Scenario: Remove a bundled segment
- **WHEN** the default configuration's segments are set up and a binding function calls `gband.ui.statusline.remove("mode")`
- **THEN** it returns `true` and the mode segment is no longer drawn

### Requirement: Render triggers
While the status line is drawn, the client SHALL call an enabled component's `render`:
- once when the client attaches, and once after each successful reload;
- once when the component is added after the configuration has loaded;
- once each time an event its `redraw_on` names is emitted, after the handlers of that event have run;
- once each `redraw_interval` milliseconds, when the component has one;
- once when the terminal's width changes, and once when the status line starts being drawn;
- once on each `HighlightChanged` and `ColorschemeChanged`;
- for a fill component, once more after any of the triggers above, when the line has been laid out and the component's available width differs from the `width` of its latest call.

When one trigger renders several components, the components that are not fill components SHALL render first, and the fill components after them, in descending `priority`, then in the order they were added. Once those renders are done, the line SHALL be laid out, and each enabled fill component whose available width, as the context defines it, then differs from the `width` of its latest call SHALL render once more, in the same order. Those renders SHALL NOT cause further renders for the same trigger.

The client SHALL NOT call `render` at any other time. Each frame SHALL be drawn from the components' latest outputs, and drawing a frame, including every frame of an animation, SHALL NOT call `render`. A component's output SHALL stay in use until its next call.

#### Scenario: Redraw on an event
- **WHEN** a component has `redraw_on = { "FocusChanged" }` and the user focuses the column to the left
- **THEN** `render` runs once more, and the line shows its new output

#### Scenario: Not per frame
- **WHEN** a component has rendered once and the camera animates over 20 frames with no event its `redraw_on` names
- **THEN** `render` is not called again

#### Scenario: Interval
- **WHEN** a component has `redraw_interval = 1000` and no other trigger occurs for 3.5 seconds
- **THEN** `render` runs 3 times in that period

#### Scenario: Width change
- **WHEN** the client's terminal changes from 80 to 60 columns
- **THEN** every enabled component renders once with `total_width` 60

#### Scenario: Fill renders after the others
- **WHEN** the fill component F and the component M both have `redraw_on = { "KeyTableChanged" }`, M is in F's region, and the user presses Ctrl+Space, after which M's output grows from empty to `prefix`
- **THEN** F renders once, after M, and its context's `width` counts `prefix` and the separator before it

#### Scenario: Fill follows another component's width
- **WHEN** the fill component F has no `redraw_on`, and the component P, with `redraw_on = { "FocusChanged" }`, changes its output from `2/3` to `10/12`
- **THEN** F renders once more, with a context `width` two cells smaller than in its latest call

#### Scenario: Fill width unchanged
- **WHEN** the fill component F has no `redraw_on`, and the component P, with `redraw_on = { "FocusChanged" }`, changes its output from `2/3` to `3/3`
- **THEN** F does not render
