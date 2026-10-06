local t = require("gband.test")

local SIZE = "60x16"

local function start(g)
  g.start({ size = SIZE, server_config = "error('first', 0)" })
  g.wait_text("│$")
end

local function errors(g)
  return g.client("return gband.errors()")
end

local function open(g)
  g.client("gband.action['errors.open']()")
  g.settle()
  local win = g.client("return gband.view().plugin_window")
  t.ok(win, "the error list is focused")
  return win
end

local function focused(g, win)
  return g.client("return gband.win.info(...).focused", win)
end

local function clear_with_c(g)
  g.keys("c")
  g.settle()
end

t.case("clear with c", function(g)
  start(g)
  g.client("gband.bar.add({ id = 'bad', side = 'right', on_resize = function() error('second', 0) end })")
  g.settle()
  g.client("gband.bar.remove('bad')")
  g.settle()
  t.eq(errors(g), { "server: first", "second" })
  local win = open(g)
  t.eq(g.screen().cell(15, 0).char, "!")
  g.expect_screenshot("two errors")
  clear_with_c(g)
  t.eq(errors(g), {})
  t.ok(focused(g, win), "the error list stays focused")
  t.ok(g.screen().cell(15, 0).char ~= "!", "the error marker is gone")
  t.ok(g.screen().text():find("no errors", 1, true), "the list shows no errors")
  g.expect_screenshot("cleared")
end)

t.case("cleared list wrapped again", function(g)
  start(g)
  open(g)
  clear_with_c(g)
  g.resize("50x16")
  g.settle()
  t.ok(g.screen().text():find("no errors", 1, true), "the list still shows no errors")
  t.ok(not g.screen().text():find("server: first", 1, true), "the cleared error stays gone")
  g.expect_screenshot()
end)

t.case("prefix c is not the list's c", function(g)
  start(g)
  open(g)
  g.keys("ctrl+space c")
  g.settle()
  t.eq(errors(g), { "server: first" })
  t.ok(g.screen().text():find("server: first", 1, true), "the list still holds the error")
end)
