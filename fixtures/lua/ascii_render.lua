-- ASCII renderer for TUI graphics
local M = {}

function M.init(width, height, fillchar)
	local w = width or 80
	local h = height or 24
	local f = fillchar or "."
	local self = {
		width = w,
		height = h,
		fillchar = f,
		buffer = {},
	}
	setmetatable(self, { __index = M })
	return self
end

function M:clear()
	self.buffer = {}
	for y = 1, self.height do
		self.buffer[y] = {}
		for x = 1, self.width do
			self.buffer[y][x] = " "
		end
	end
end

function M:set(x, y, char)
	if x >= 1 and x <= self.width and y >= 1 and y <= self.height then
		self.buffer[y][x] = char
	end
end

function M:write(x, y, text)
	for i = 1, #text do
		self:set(x + i - 1, y, text:sub(i, i))
	end
end

function M:render()
	for y = 1, self.height do
		for x = 1, self.width do
			io.write(self.buffer[y][x] or " ")
		end
		io.write("\n")
	end
end

return M
