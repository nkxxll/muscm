local AsciiRender = require("ascii_render")
local RenderLib = require("renderlib")

local canvas = AsciiRender.init(80, 24)
local engine = RenderLib.init(canvas, 10)

print("engine.canvas:", type(engine.canvas))
print("engine.canvas.clear:", type(engine.canvas.clear))

engine.canvas:clear()
print("Success!")
