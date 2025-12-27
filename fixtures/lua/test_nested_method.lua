local M = {}
function M:test(str)
  print("In test, self:", type(self))
  print("In test, str:", str)
  local result = str:sub(1, 2)
  print("Result:", result)
  return result
end

local obj = M
local result = obj:test("hello")
print("Final result:", result)
