-- Simple demo with moving ASCII art

local AsciiRender = require("ascii_render")
local RenderLib = require("renderlib")

-- Create canvas
local canvas = AsciiRender.init(80, 24)
canvas:clear()

-- Create engine
local engine = RenderLib.init(canvas, 10)

-- Animation state
local x = 10
local dx = 1

-- Override update to move the ASCII art
function engine:update()
	-- Draw a simple ASCII character moving horizontally
	self.canvas:write(x, 5, "+-------+")
	self.canvas:write(x, 6, "| HELLO |")
	self.canvas:write(x, 7, "+-------+")

	-- Update position
	x = x + dx
	if x >= 70 or x <= 1 then
		dx = -dx
	end

	-- Draw borders
	for i = 1, self.canvas.width do
		self.canvas:set(i, 1, "-")
		self.canvas:set(i, self.canvas.height, "-")
	end
	for i = 1, self.canvas.height do
		self.canvas:set(1, i, "|")
		self.canvas:set(self.canvas.width, i, "|")
	end
	self.canvas:set(1, 1, "+")
	self.canvas:set(self.canvas.width, 1, "+")
	self.canvas:set(1, self.canvas.height, "+")
	self.canvas:set(self.canvas.width, self.canvas.height, "+")
end

-- Run for 10 seconds then stop
local start = os.time()
while os.difftime(os.time(), start) < 10 do
	engine.canvas:clear()
	engine:update()
	RenderLib.clear_screen()
	engine:render()
	RenderLib.sleep(0.1)
end

print("Demo complete!")
