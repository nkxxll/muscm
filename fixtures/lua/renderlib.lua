local M = {}

function M.sleep(n)
	n = tonumber(n) or 0.1
	os.execute("sleep " .. n)
end

function M.clear_screen()
	os.execute("clear")
end

function M.init(canvas, fps)
	local self = {
		canvas = canvas,
		fps = fps or 10,
		frame_time = 1 / (fps or 10),
		running = true,
	}
	setmetatable(self, { __index = M })
	return self
end

function M:render()
	self.canvas:render()
end

function M:update()
	-- Override this in subclasses
end

function M:loop()
	local last_time = os.time()
	while self.running do
		self.canvas:clear()
		self:update()
		M.clear_screen()
		self:render()

		local elapsed = os.difftime(os.time(), last_time)
		if elapsed < self.frame_time then
			M.sleep(self.frame_time - elapsed)
		end
		last_time = os.time()
	end
end

function M:stop()
	self.running = false
end

return M
