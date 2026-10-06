local M = { name = "hello", api = 1 }

function M.setup(opts)
  gband.opt.declare("greeting", {
    type = "string",
    default = "hello",
    desc = "the text the greeting window prints",
  })
  if opts.greeting ~= nil then
    gband.opt["hello.greeting"] = opts.greeting
  end

  gband.action.register("greet", function()
    gband.spawn({
      cmd = { "sh", "-c", 'printf "%s\\n" "$1"; exec "${SHELL:-sh}"', "sh", gband.opt["hello.greeting"] },
    })
  end, { desc = "open a window that prints the greeting" })

  gband.cmd.register("say", function(args)
    print(gband.opt["hello.greeting"] .. ", " .. (args.who or "world"))
  end, { desc = "log the greeting", args = { "who" } })

  local group = gband.augroup("hello")
  gband.on("FocusChanged", function(event)
    print("focused window", event.window)
  end, { group = group })
end

return M
