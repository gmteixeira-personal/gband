local theme = {}

function theme.apply(spec)
  local palette, ui = spec.palette, spec.ui
  local fg, bg = palette.fg, palette.bg
  gband.palette.set(palette)
  gband.hl.set("Bar", { bg = ui.surface })
  gband.hl.set("SidebarMode", { fg = ui.accent, bold = true })
  gband.hl.set("SidebarBand", { fg = ui.muted })
  gband.hl.set("SidebarBandActive", { fg = fg, bold = true })
  gband.hl.set("SidebarError", { fg = ui.error, bold = true })
  gband.hl.set("KeyListKey", { fg = ui.accent, bold = true })
  gband.hl.set("KeyListMuted", { fg = ui.muted })
  gband.hl.set("PluginWindow", { fg = fg, bg = ui.surface })
  gband.hl.set("PluginWindowBorder", { fg = ui.muted, bg = ui.surface })
  gband.hl.set("PluginWindowTitle", { fg = ui.accent, bg = ui.surface, bold = true })
  gband.hl.set("PluginWindowCursorLine", { fg = fg, bg = ui.selection, bold = true })
  gband.hl.set("PromptCursor", { fg = bg, bg = ui.accent })
  gband.hl.set("SettingsLabel", { fg = ui.muted })
  gband.hl.set("WindowBorder", { fg = ui.muted })
  gband.hl.set("WindowBorderFocused", { fg = ui.accent, bold = true })
  gband.hl.set("ErrorBanner", { fg = bg, bg = ui.error, bold = true })
end

return theme
