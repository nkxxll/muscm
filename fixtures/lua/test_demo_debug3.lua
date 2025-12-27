print("Start")
local AsciiRender = require("ascii_render")
local RenderLib = require("renderlib")

local canvas = AsciiRender.init(80, 24)
local engine = RenderLib.init(canvas, 10)

local x = 10
local dx = 1

function engine:update()
  self.canvas:write(x, 5, "+--------+")
  self.canvas:write(x, 6, "|  HELLO |")
  self.canvas:write(x, 7, "+--------+")
  
  x = x + dx
  if x >= 70 or x <= 1 then
    dx = -dx
  end
end

print("About to call update")
engine:update()
print("Done with update")
