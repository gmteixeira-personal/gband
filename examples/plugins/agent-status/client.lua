gband.on("ServerEvent", function(ev)
  gband.notify("Agent waiting in pane " .. tostring(ev.data.pane), { title = "gband" })
end, { pattern = "agent.waiting" })

gband.ui.statusline.add({
  id = "waiting",
  align = "right",
  redraw_on = { "PaneStateChanged", "LayoutChanged" },
  render = function(ctx)
    local waiting = 0
    for _, entry in ipairs(ctx.panes) do
      if entry.state.agent == "waiting" then
        waiting = waiting + 1
      end
    end
    if waiting == 0 then
      return nil
    end
    return "agents waiting: " .. waiting
  end,
})

gband.keymap.set("prefix", "a", function()
  gband.rpc("agent-status.next_waiting", { after = gband.view().pane })
end, { desc = "focus the next waiting agent" })
