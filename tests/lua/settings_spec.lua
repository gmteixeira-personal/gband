local t = require("gband.test")

local function settings_open(g)
  g.wait(function(screen)
    return screen.text():find("┌settings", 1, true) and "open"
  end)
end

t.case("the settings window beside the default sidebar on the first start", function(g)
  g.start({ keystyle = false, theme = false })
  g.wait_text("$")
  settings_open(g)
  g.settle()
  t.match(g.screen().row(0), "^I")
  t.match(g.screen().row(8), "^ │ +┌settings.-┐$")
  t.match(g.screen().row(9), "^ │ +│theme    default +│")
  t.match(g.screen().row(10), "^ │ +│sidebar  on +│")
  t.match(g.screen().row(11), "^ │ +│keys     modal +│")
  t.match(g.screen().row(12), "^─│ +│I on new off +│")
  t.match(g.screen().row(13), "^ │ +│reload   ok +│")
  t.eq(g.screen().cell(8, 25).char, "┌")
  t.eq(g.screen().cell(14, 55).char, "┘")
  g.keys("j k")
  g.settle()
  g.expect_screenshot("first start")
end)

t.case("the settings window with the direct key style", function(g)
  g.start({ keystyle = "direct" })
  g.wait_text("$")
  g.keys("ctrl+space s")
  settings_open(g)
  g.settle()
  t.match(g.screen().row(12), "^─│ +│keys     direct +│")
  t.match(g.screen().row(13), "^ │ +│reload   ok +│")
  t.eq(g.screen().cell(9, 25).char, "┌")
  t.eq(g.screen().cell(14, 55).char, "┘")
  t.eq(g.screen().text():find("I on new", 1, true), nil)
  g.expect_screenshot("direct")
end)

t.case("the settings window with the floating key style", function(g)
  g.start({ files = { ["user/keystyle.lua"] = 'return "floating"\n' } })
  g.wait_text("$")
  g.keys("ctrl+space")
  g.settle()
  g.keys("s")
  settings_open(g)
  g.settle()
  t.match(g.screen().row(12), "│keys     floating +│")
  t.match(g.screen().row(13), "│reload   ok +│")
  t.eq(g.screen().cell(9, 25).char, "┌")
  t.eq(g.screen().cell(14, 55).char, "┘")
  t.eq(g.screen().text():find("I on new", 1, true), nil)
  t.eq(g.screen().text():find("┌windows", 1, true), nil)
  g.expect_screenshot("floating")
end)

t.case("the theme list beside the default sidebar", function(g)
  g.start({})
  g.wait_text("$")
  g.keys("ctrl+space s")
  settings_open(g)
  g.keys("enter")
  g.wait_text("┌theme")
  g.settle()
  t.eq(g.screen().cell(3, 29).char, "┌")
  t.eq(g.screen().cell(19, 50).char, "┘")
  t.match(g.screen().row(4), "│default +│")
  t.match(g.screen().row(18), "│vesper +│")
  g.expect_screenshot("theme list")
end)

local function tiled(g)
  return g.client([[
    local out = {}
    for _, column in ipairs(gband.layout().bands[1].columns) do
      local ids = {}
      for _, window in ipairs(column.windows) do
        ids[#ids + 1] = window.id
      end
      out[#out + 1] = ids
    end
    return out
  ]])
end

local function floating(g)
  return g.client([[
    local out = {}
    for _, box in ipairs(gband.layout().bands[1].floating) do
      if box.name then
        out[#out + 1] = box.id
      end
    end
    return out
  ]])
end

local function switch_keys(g, key, style)
  g.keys(key)
  g.wait_text("keys     " .. style)
  g.settle()
end

local function open_keys_line(g)
  g.keys("ctrl+space s")
  settings_open(g)
  g.keys("j j")
  g.settle()
end

local function record(g)
  return g.client([[
    local chunk = loadfile(gband.config_dir .. "/user/floating_restore.lua", "t", {})
    return chunk and chunk()
  ]])
end

t.case("a record kept entering floating", function(g)
  g.start({})
  g.client([[gband.action.open_window({ after = 1 })]])
  g.settle()
  open_keys_line(g)
  switch_keys(g, "h", "floating")
  t.eq(g.client([[return gband.keystyle.saved()]]), "floating")
  t.eq(tiled(g), {})
  t.eq(floating(g), { 1, 2 })
  t.eq(record(g), {
    bands = {
      {
        id = 1,
        columns = {
          { width = 0.5, full_width = false, tabbed = false, windows = { { id = 1, weight = 1 } } },
          { width = 0.5, full_width = false, tabbed = false, windows = { { id = 2, weight = 1 } } },
        },
        floating = {},
      },
      { id = 2, columns = {}, floating = {} },
    },
  })
end)

t.case("columns restored across a round trip", function(g)
  g.start({})
  g.client([[gband.action.open_window({ after = 1 })]])
  g.settle()
  open_keys_line(g)
  switch_keys(g, "h", "floating")
  switch_keys(g, "l", "modal")
  t.eq(g.client([[return gband.keystyle.saved()]]), "modal")
  t.eq(tiled(g), { { 1 }, { 2 } })
  t.eq(floating(g), {})
  t.eq(record(g), nil)
end)

t.case("a stacked column restored", function(g)
  g.start({})
  g.client([[gband.action.open_window({ after = 1 })]])
  g.settle()
  g.client([[gband.action.consume_or_expel_left({ window = 2 })]])
  g.settle()
  t.eq(tiled(g), { { 1, 2 } })
  open_keys_line(g)
  switch_keys(g, "h", "floating")
  switch_keys(g, "l", "modal")
  t.eq(tiled(g), { { 1, 2 } })
end)

t.case("a column width restored", function(g)
  g.start({})
  g.client([[gband.action.open_window({ after = 1 })]])
  g.settle()
  g.client([[gband.window.set_width(1, 1 / 3)]])
  g.settle()
  open_keys_line(g)
  switch_keys(g, "h", "floating")
  switch_keys(g, "l", "modal")
  t.eq(tiled(g), { { 1 }, { 2 } })
  t.eq(g.client([[return gband.layout().bands[1].columns[1].width]]), 1 / 3)
end)

t.case("a window opened while floating tiles after the restored windows", function(g)
  g.start({})
  open_keys_line(g)
  switch_keys(g, "h", "floating")
  g.client([[gband.action.open_window({ floating = true })]])
  g.settle()
  t.eq(floating(g), { 1, 2 })
  g.client([[gband.win.focus(gband.win.list()[1])]])
  g.settle()
  switch_keys(g, "l", "modal")
  t.eq(tiled(g), { { 1 }, { 2 } })
  t.eq(floating(g), {})
end)

t.case("a previously floating window keeps its box", function(g)
  g.start({})
  g.client([[gband.action.open_window({ after = 1 })]])
  g.settle()
  g.client([[gband.action.open_window({ after = 2 })]])
  g.settle()
  g.client([[gband.action.close_window({ window = 2 })]])
  g.settle()
  g.client([[gband.action.toggle_window_floating({ window = 3, floating = true })]])
  g.settle()
  g.client([[
    gband.window.set_width(3, 1 / 3)
    gband.window.set_height(3, { rows = 10 })
    gband.window.set_position(3, { col = 5, row = 3 })
  ]])
  g.settle()
  open_keys_line(g)
  switch_keys(g, "h", "floating")
  switch_keys(g, "l", "modal")
  t.eq(tiled(g), { { 1 } })
  local box = g.client([[return gband.layout().bands[1].floating[1] ]])
  t.eq({ box.id, box.col, box.row, box.width, box.rows }, { 3, 5, 3, 1 / 3, 10 })
end)

t.case("no record tiles every floating window", function(g)
  g.start({ files = { ["user/keystyle.lua"] = 'return "floating"\n' } })
  g.wait_text("$")
  g.client([[gband.action.open_window({ floating = true })]])
  g.settle()
  t.eq(floating(g), { 1, 2 })
  g.keys("ctrl+space")
  g.settle()
  g.keys("s")
  settings_open(g)
  g.keys("j j")
  g.settle()
  switch_keys(g, "l", "modal")
  t.eq(g.client([[return gband.keystyle.saved()]]), "modal")
  t.eq(#tiled(g), 2)
  t.eq(floating(g), {})
end)

t.case("a gruvbox frame", function(g)
  g.start({ theme = "gruvbox" })
  g.wait_text("$")
  g.run("printf '\\033[31m%s\\033[0m\\n' red")
  g.wait_text("$ printf")
  g.wait(function(screen)
    return screen.text():find("│red", 1, true) and "printed"
  end)
  g.settle()
  t.eq(g.screen().cell(0, 0).bg, "#3c3836")
  g.expect_screenshot("gruvbox")
end)
