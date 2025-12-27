local AsciiRender = require("ascii_render")
local canvas = AsciiRender.init(80, 24)

print("Calling canvas:set(1, 1, 'x')")
canvas:set(1, 1, "x")
print("Done set, calling canvas:write(2, 1, 'test')")
canvas:write(2, 1, "test")
print("Done write")
