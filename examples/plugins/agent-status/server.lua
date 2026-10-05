local TAIL = 512

local PROMPTS = {
  "Do you want to proceed?",
  "Do you want to make this edit",
  "Do you want to create",
  "Allow this action?",
}

local tails = {}

local function key(session, pane)
  return session .. "/" .. pane
end

local function plain(text)
  return (text:gsub("\27%[[%d;?]*[%a@~]", ""):gsub("\27%][^\7]*\7", ""))
end

local function waiting(text)
  for _, prompt in ipairs(PROMPTS) do
    if text:find(prompt, 1, true) then
      return true
    end
  end
  return false
end

gband.on("PaneOutput", function(ev)
  local id = key(ev.session, ev.pane)
  local tail = (tails[id] or "") .. ev.data
  if #tail > TAIL then
    tail = tail:sub(-TAIL)
  end
  tails[id] = tail
  if not waiting(plain(tail)) then
    return
  end
  tails[id] = ""
  local state = gband.pane_state(ev.session, ev.pane)
  if state == nil or state.agent == "waiting" then
    return
  end
  state.agent = "waiting"
  gband.emit("agent.waiting", { session = ev.session, pane = ev.pane }, { session = ev.session })
end)

gband.on("PaneInput", function(ev)
  tails[key(ev.session, ev.pane)] = nil
  local state = gband.pane_state(ev.session, ev.pane)
  if state ~= nil and state.agent ~= nil then
    state.agent = nil
  end
end)

gband.on("PaneClosed", function(ev)
  tails[key(ev.session, ev.pane)] = nil
end)

gband.cmd.register("next_waiting", function(args, ctx)
  if ctx.session == nil then
    return nil
  end
  local session = gband.session(ctx.session)
  if session == nil then
    return nil
  end
  local after = args.after or 0
  local first, later
  for _, band in ipairs(session.bands) do
    for _, column in ipairs(band.columns) do
      for _, entry in ipairs(column.panes) do
        local state = gband.pane_state(ctx.session, entry.pane)
        if state ~= nil and state.agent == "waiting" then
          first = first or entry.pane
          if later == nil and entry.pane > after then
            later = entry.pane
          end
        end
      end
    end
  end
  local target = later or first
  if target ~= nil then
    ctx.focus(target)
  end
  return target
end, { desc = "focus the next pane whose agent waits for input", args = { "after" } })
