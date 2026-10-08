local t = require("gband.test")

local GROUPS = {
  "WindowBorder", "WindowBorderFocused", "ErrorBanner", "Bar",
  "SidebarMode", "SidebarBand", "SidebarBandActive",
  "PluginWindow", "PluginWindowBorder", "PluginWindowTitle", "PluginWindowCursorLine",
  "SettingsLabel",
}

local RESOLVED = [[
local groups = ...
local resolved = { palette = gband.palette.get(), groups = {} }
for _, name in ipairs(groups) do
  resolved.groups[name] = gband.hl.get(name, { resolve = true })
end
return resolved
]]

t.case("Copied sidebar", function(g)
  g.start({ size = "40x8", bundled_copies = true, server_config = "error('first', 0)" })
  g.wait_text("│$")
  g.settle()
  local screen = g.screen()
  t.eq(screen.cell(0, 0).char, "I")
  t.eq(screen.cell(2, 0).char, "1")
  t.eq(screen.cell(7, 0).char, "!")
  t.ok(not screen.text():find("first", 1, true), "no banner is drawn")
  g.expect_screenshot("error")
end)

t.case("Copied prompt", function(g)
  g.start({ bundled_copies = true })
  g.wait_text("$")
  g.keys("ctrl+space :")
  g.settle()
  g.type('error("boom")')
  g.keys("enter")
  g.settle()
  t.eq(g.client("return gband.errors()"), { "prompt:1: boom" })
end)

t.case("Copied desktop plugin", function(g)
  g.start({ bundled_copies = true, files = { ["user/keystyle.lua"] = 'return "floating"\n' } })
  g.wait_text("│$")
  g.settle()
  t.ok(g.client([[
    local file = io.open(gband.config_dir .. "/user/lua/gband/desktop.lua")
    if file then
      file:close()
    end
    return file ~= nil
  ]]), "the desktop plugin is copied")
  t.match(g.screen().row(2), "╭[─]*%[_%]%[□%]%[X%]─╮$")
  g.keys("ctrl+space")
  g.settle()
  t.match(g.screen().text(), "┌windows[─]*%[□%]%[X%]─┐")
  t.eq(g.client("return gband.keymap.current_table()"), "root")
end)

t.case("Copied theme", function(g)
  g.start({ size = "40x8", bundled_copies = true, config = 'gband.colorscheme("nord")' })
  g.wait_text("$")
  g.settle()
  t.eq(g.client("return gband.colorscheme()"), "nord")
  local copied = g.client(RESOLVED, GROUPS)
  local bundled = g.client("gband.core.bundled('nord')()\n" .. RESOLVED, GROUPS)
  t.eq(copied, bundled)
  g.expect_screenshot("nord")
end)

t.case("Bundled copies", function(g)
  g.start({ bundled_copies = true })
  g.wait_text("$")
  local same, alias = g.client([[
    local function read(path)
      local file = io.open(gband.config_dir .. "/" .. path)
      if not file then
        return nil
      end
      local text = file:read("a")
      file:close()
      return text
    end
    local same = true
    for _, path in ipairs({ "lua/gband/sidebar.lua", "colors/nord.lua" }) do
      local copy = read("user/" .. path)
      same = same and copy ~= nil and copy == read("defaults/" .. path)
    end
    return same, read("user/colors/catppuccin.lua") ~= nil
  ]])
  t.ok(same, "the copies hold the bundled text")
  t.ok(not alias, "the alias is not copied")
end)
