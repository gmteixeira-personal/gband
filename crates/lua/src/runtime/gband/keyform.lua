local function key_form(written)
  local modifiers, key
  if written == "+" then
    modifiers, key = "", "+"
  elseif written:sub(-2) == "++" then
    modifiers, key = written:sub(1, -3), "+"
  else
    modifiers, key = written:match("^(.*)%+([^+]*)$")
    if not modifiers then
      modifiers, key = "", written
    end
  end
  local held = {}
  for modifier in modifiers:gmatch("[^+]+") do
    held[modifier:lower()] = true
  end
  if utf8.len(key) == 1 then
    if held.shift and key:match("^%l$") then
      key = key:upper()
      held.shift = nil
    end
  else
    key = key:lower()
    if key == "escape" then
      key = "esc"
    end
  end
  return (held.ctrl and "C-" or "") .. (held.alt and "A-" or "") .. (held.shift and "S-" or "") .. key
end

return key_form
