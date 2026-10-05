local host = ...

local t = {}
local registered = {}

function host.wrap(fn)
  return function(...)
    local results = table.pack(fn(...))
    if not results[1] then
      error(results[2], results[3] and 0 or 2)
    end
    return table.unpack(results, 2, results.n)
  end
end

local function is_identifier(key)
  return type(key) == "string" and key:match("^[%a_][%w_]*$") ~= nil
end

local function describe(value, open)
  if type(value) == "string" then
    return string.format("%q", value)
  end
  if type(value) ~= "table" then
    return tostring(value)
  end
  open = open or {}
  if open[value] then
    return "<cycle>"
  end
  open[value] = true
  local parts = {}
  local length = #value
  for index = 1, length do
    parts[#parts + 1] = describe(value[index], open)
  end
  local keys = {}
  for key in pairs(value) do
    if not (math.type(key) == "integer" and key >= 1 and key <= length) then
      keys[#keys + 1] = key
    end
  end
  table.sort(keys, function(a, b)
    if type(a) == type(b) and (type(a) == "string" or type(a) == "number") then
      return a < b
    end
    return type(a) < type(b)
  end)
  for _, key in ipairs(keys) do
    local name = is_identifier(key) and key or "[" .. describe(key, open) .. "]"
    parts[#parts + 1] = name .. " = " .. describe(value[key], open)
  end
  open[value] = nil
  if #parts == 0 then
    return "{}"
  end
  return "{ " .. table.concat(parts, ", ") .. " }"
end

local function equal(a, b, open)
  if a == b then
    return true
  end
  if type(a) ~= "table" or type(b) ~= "table" then
    return false
  end
  open = open or {}
  if open[a] == b then
    return true
  end
  open[a] = b
  for key, value in pairs(a) do
    if not equal(value, b[key], open) then
      return false
    end
  end
  for key in pairs(b) do
    if a[key] == nil then
      return false
    end
  end
  return true
end

local function fail(text, message)
  if message ~= nil then
    text = tostring(message) .. ": " .. text
  end
  error(text, 3)
end

function t.case(name, opts, fn)
  if fn == nil then
    fn, opts = opts, nil
  end
  if type(name) ~= "string" or name == "" then
    error("t.case expects the case's name as a non-empty string", 2)
  end
  if registered[name] then
    error(string.format("the case %q is already registered in this file", name), 2)
  end
  if opts ~= nil and type(opts) ~= "table" then
    error("the options of t.case must be a table", 2)
  end
  local timeout = opts and opts.timeout
  if timeout ~= nil and (type(timeout) ~= "number" or timeout <= 0) then
    error("the timeout of t.case must be a positive number of seconds", 2)
  end
  if type(fn) ~= "function" then
    error("t.case expects the case as a function", 2)
  end
  registered[name] = true
  host.register(name, timeout, fn)
end

function t.eq(actual, expected, message)
  if not equal(actual, expected) then
    fail(string.format("expected %s, got %s", describe(expected), describe(actual)), message)
  end
end

function t.ok(value, message)
  if value == nil or value == false then
    fail("expected a value other than nil and false, got " .. describe(value), message)
  end
end

function t.match(text, pattern, message)
  if type(text) ~= "string" then
    fail(string.format("expected a string matching %q, got %s", pattern, describe(text)), message)
  end
  if not text:find(pattern) then
    fail(string.format("expected %s to match %q", describe(text), pattern), message)
  end
end

return t
