## REMOVED Requirements

### Requirement: Placement
**Reason**: The status line is removed. The one-column sidebar shows the mode and the bands.
**Migration**: Set up `gband.sidebar`, or add a bar with `gband.bar.add`.

### Requirement: Drawing the line
**Reason**: The status line is removed.
**Migration**: A plugin draws its own bar with `gband.bar.set_lines`, in the bar's `hl` group.

### Requirement: Components
**Reason**: The component API `gband.ui.statusline` is removed with the status line.
**Migration**: A plugin adds its own bar with `gband.bar.add` and writes its lines with `gband.bar.set_lines`.

### Requirement: Render output
**Reason**: Components are removed with the status line.
**Migration**: Bar lines take the plugin window line form, as the bars capability defines.

### Requirement: Render context
**Reason**: Components are removed with the status line.
**Migration**: Read the view with `gband.view()`, the layout with `gband.layout()` and the active table with `gband.keymap.current_table()`.

### Requirement: Render triggers
**Reason**: Components are removed with the status line.
**Migration**: Redraw a bar from `gband.on` handlers of the events the content depends on.

### Requirement: Layout
**Reason**: The status line's vertical layout is removed with it.
**Migration**: A bar shows its lines from its top row, as the bars capability defines.

### Requirement: Render errors
**Reason**: Components are removed with the status line.
**Migration**: An error in a bar plugin's event handler is reported as a plugin error.

### Requirement: Error item
**Reason**: The sidebar's error marker replaces the status line's error item.
**Migration**: Set up `gband.sidebar`, which shows `!` while an error is reported. The error list shows the messages.

### Requirement: Built-in groups
**Reason**: The `StatusLine` groups are removed with the status line.
**Migration**: Use the `Bar` group and the sidebar's groups, `SidebarMode`, `SidebarBand`, `SidebarBandActive` and `SidebarError`.

### Requirement: Display width helpers
**Reason**: The helpers stay, and move out of the retired status-line capability.
**Migration**: Moved unchanged to the plugin-windows capability's "Display width helpers".

### Requirement: Bundled segment plugins
**Reason**: The segment plugins `gband.statusline.band`, `.mode`, `.position` and `.clock` are removed with the status line.
**Migration**: The sidebar shows the mode and the viewed band. A plugin that needs another value adds its own bar.
