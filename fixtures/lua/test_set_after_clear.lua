local AsciiRender = require("ascii_render")
local canvas = AsciiRender.init(80, 24)

print("Calling canvas:clear()")
canvas:clear()
print("Calling canvas:set(1, 1, 'x')")
canvas:set(1, 1, "x")
print("Done!")
