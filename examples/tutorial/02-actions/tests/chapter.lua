local M = {}

function M.read(path)
  local file = io.open(path)
  if file == nil then
    return nil
  end
  local text = file:read("a")
  file:close()
  return text
end

function M.start(g, opts)
  opts = opts or {}
  opts.config = opts.config or M.read("user/init.lua")
  opts.server_config = opts.server_config or M.read("user/server.lua")
  local files = {}
  for _, path in ipairs(opts.files or {}) do
    files[path] = assert(M.read(path), path)
  end
  opts.files = files
  g.start(opts)
end

return M
