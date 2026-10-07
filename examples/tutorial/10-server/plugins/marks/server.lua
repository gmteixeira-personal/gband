local KEY = "marks.letter"
local group = gband.augroup("marks")

local function marked(session)
  local found = {}
  local layout = gband.session(session)
  if layout == nil then
    return found
  end
  for _, band in ipairs(layout.bands) do
    local windows = {}
    for _, column in ipairs(band.columns) do
      for _, tile in ipairs(column.windows) do
        windows[#windows + 1] = tile.window
      end
    end
    for _, box in ipairs(band.floating) do
      windows[#windows + 1] = box.window
    end
    for _, window in ipairs(windows) do
      local letter = gband.window_state(session, window)[KEY]
      if letter then
        found[letter] = window
      end
    end
  end
  return found
end

gband.cmd.register("toggle", function(args, ctx)
  local state = gband.window_state(ctx.session, args.window)
  if state == nil then
    error("no window " .. tostring(args.window) .. " in session " .. tostring(ctx.session))
  end
  if state[KEY] then
    state[KEY] = nil
    return nil
  end
  local taken = marked(ctx.session)
  for letter in args.letters:gmatch(".") do
    if not taken[letter] then
      state[KEY] = letter
      return letter
    end
  end
  return false
end, { desc = "mark or unmark a window", args = { "window", "letters" } })

gband.cmd.register("list", function(_, ctx)
  return marked(ctx.session)
end, { desc = "list the marks of the caller's session" })

gband.on("WindowExited", function(event)
  local state = gband.window_state(event.session, event.window)
  local letter = state and state[KEY]
  if letter then
    gband.emit("marks.exited", { window = event.window, letter = letter }, { session = event.session })
  end
end, { group = group })

gband.on("ConfigReloaded", function()
  for _, session in ipairs(gband.sessions()) do
    local count = 0
    for _ in pairs(marked(session)) do
      count = count + 1
    end
    print(session .. " keeps " .. count .. " marks")
  end
end, { group = group })
