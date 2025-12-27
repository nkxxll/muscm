local M = {}

function M.sleep(n)
  os.execute("sleep " .. tonumber(n))
end

function M.init()
end

function M:render()
end

function M:loop()
	self:render()
	self.sleep()
end

return M
