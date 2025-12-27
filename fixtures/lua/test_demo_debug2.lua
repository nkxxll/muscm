print("1")
local AsciiRender = require("ascii_render")
print("2")
local RenderLib = require("renderlib")
print("3")
local canvas = AsciiRender.init(80, 24)
print("4")
canvas:clear()
print("5")
local engine = RenderLib.init(canvas, 10)
print("6")

-- Animation state
local x = 10
local dx = 1
print("7")

-- Override update to move the ASCII art
function engine:update()
  print("Update called")
end
print("8")
