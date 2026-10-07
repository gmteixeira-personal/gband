# 10 · The server half

So far the marks live in one client's Lua, and they vanish when it detaches or reloads.
This chapter moves them into the server, where every client attached to the session sees the same marks, and where the server can watch a marked window even while no client is attached.

The finished files are in [examples/tutorial/10-server/](../../examples/tutorial/10-server/).

## Two processes

gband is a server, which runs the windows, and clients, which draw them and take your keys.
Each runs its own Lua, and the two never share a Lua value: they exchange plain data only, such as tables of strings and numbers.
The rule for where code goes:

- What concerns one user's screen and keys runs in the **client**: bindings, bars, plugin windows, notifications.
- What must be shared by every client, or keep working with none attached, runs in the **server**.

The marks belong to the windows, so they move to the server.
The keys, the list and the bar stay in the client.
[Two sides](../plugins.md#two-sides) explains the split.

## The server's file

A plugin's `server.lua` runs in the server, as `client.lua` runs in each client.
The manifest now says which client half the server half needs, so a client with an older copy of the plugin reports it:

```lua
return {
  name = "marks",
  version = "0.2.0",
  client = ">= 0.2",
}
```

[Requirements](../plugins.md#requirements) describes the comparison.

## Window state

Every window has a state in the server: a table of keys and plain values, removed when the window closes.
`gband.window_state(session, window)` in the server returns it for reading and writing.
The server sends every change to the clients of the window's session, and a reload of either side keeps every state.

A mark is the key `marks.letter`.
Keys are shared by every plugin, so the plugin's name in front keeps it apart:

```lua
local KEY = "marks.letter"
local group = gband.augroup("marks")
```

[Window state](../plugins.md#window-state-gbandwindow_state) has the rules on keys, values and size.

## Reading the session

The server has no view and no focus, which are client state, but it knows each session's layout.
`gband.session(name)` returns it, with `window` numbers rather than the client's `id`:

```lua
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
```

`gband.sessions()` lists the session names.
After a reload the server counts the marks it kept in each session, which shows that window state outlives the Lua that wrote it:

```lua
gband.on("ConfigReloaded", function()
  for _, session in ipairs(gband.sessions()) do
    local count = 0
    for _ in pairs(marked(session)) do
      count = count + 1
    end
    print(session .. " keeps " .. count .. " marks")
  end
end, { group = group })
```

[Reading structure](../plugins.md#reading-structure-gbandsessions-gbandsession) lists the fields.

## Server commands

A server command takes the arguments a client sends and a context naming the client's session, and returns a plain value to that client.
Choosing the letter in the server means two clients marking at once can never take the same letter:

```lua
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
```

It returns the new letter, nil once it unmarks, or `false` when every letter is taken.
The letters come from the client, since `marks.letters` is a client option.
A second command returns the session's marks for any client that asks:

```lua
gband.cmd.register("list", function(_, ctx)
  return marked(ctx.session)
end, { desc = "list the marks of the caller's session" })
```

[Server commands](../plugins.md#server-commands) describes the context.

## Events for the clients

The server sees what no client can: a window's program exiting while you are away.
`gband.emit` in the server sends an event to the clients of a session, and queues it while none is attached:

```lua
gband.on("WindowExited", function(event)
  local state = gband.window_state(event.session, event.window)
  local letter = state and state[KEY]
  if letter then
    gband.emit("marks.exited", { window = event.window, letter = letter }, { session = event.session })
  end
end, { group = group })
```

[Server events](../plugins.md#server-events) lists what the server can watch, and [Emitting events](../plugins.md#emitting-events-gbandemit) how the queue works.

## The client half

In the client, `gband.window_state(window)` returns a copy of a window's state, and the marks table of the earlier chapters gives way to it:

```lua
local function letter_of(window)
  local state = gband.window_state(window)
  return state and state[KEY]
end
```

`gband.rpc(name, args, callback)` asks the server to run a command, and calls `callback` with the result once it arrives.
`toggle` sends the window and the letters, and reacts to the answer:

```lua
local function toggle(window)
  if band_of(window) == nil then
    return
  end
  gband.rpc("marks.toggle", { window = window, letters = gband.opt[letters] }, function(ok, letter)
    if not ok then
      print(letter)
      return
    end
    if letter == false then
      gband.bell()
      return
    end
    if band_of(window) then
      gband.window.rename(window, letter and ("mark " .. letter))
    end
    if letter then
      gband.notify("marked window " .. window .. " as " .. letter)
    else
      gband.notify("unmarked window " .. window)
    end
  end)
end
```

Only the client that marked notifies, but every client of the session hears of the change, through `WindowStateChanged`, and redraws its bar and list:

```lua
  gband.on("WindowStateChanged", function(event)
    if event.key == KEY then
      changed(event.window)
    end
  end, { group = group })
```

The server's event arrives as `ServerEvent`:

```lua
  gband.on("ServerEvent", function(event)
    gband.notify("window " .. event.data.window .. ", mark " .. event.data.letter .. ", exited")
  end, { group = group, pattern = "marks.exited" })
```

The marks now survive a client reload, so the bar draws after one as well as on attaching:

```lua
  gband.on("ConfigReloaded", function()
    draw(gband.view().window)
  end, { group = group })
```

[The client side of the bridge](../plugins.md#the-client-side-of-the-bridge) covers `ServerEvent`, window state in the client and `gband.rpc`.

## Installing both halves

The server reads plugins from the plugins directory of the machine it runs on, and each client from its own.
On one machine, one copy of `plugins/marks/` serves both.

## The whole plugin

The complete files are in [examples/tutorial/10-server/plugins/marks/](../../examples/tutorial/10-server/plugins/marks/), with the new [server.lua](../../examples/tutorial/10-server/plugins/marks/server.lua).

Next: [11 · Testing](11-testing.md).
