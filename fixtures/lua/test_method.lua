local M = {}
function M:clear()
  print("clear called with self:", type(self))
end
M.x = 10

local obj = M
obj:clear()
