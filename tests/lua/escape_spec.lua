local t = require("gband.test")

t.case("escape then settle, again and again", function(g)
  g.start({ size = "40x8" })
  g.wait_text("$")
  for _ = 1, 100 do
    g.keys("ctrl+space escape")
    g.settle()
    t.eq(g.screen().row(0):sub(1, 1), "I")
  end
end)

t.case("escape reaches the window apart from the next key", function(g)
  g.start({ size = "40x8" })
  g.wait_text("$")
  g.run("cat -v")
  g.keys("escape up enter")
  g.wait(function(screen)
    local _, found = screen.text():gsub("%^%[%^%[%[A", "")
    return found == 2
  end)
  t.ok(not g.screen().text():find("[I", 1, true))
end)
