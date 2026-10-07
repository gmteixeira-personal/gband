gband.keystyle.use()

gband.plugin("gband.errors")
gband.plugin("gband.sidebar")

gband.keymap.set("prefix", "e", gband.action["errors.open"], { desc = "list the errors" })
