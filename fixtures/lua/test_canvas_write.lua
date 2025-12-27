local AsciiRender = require("ascii_render")
local canvas = AsciiRender.init(80, 24)

print("canvas:", type(canvas))
print("canvas.write:", type(canvas.write))

canvas:write(10, 5, "test")
print("Done")
