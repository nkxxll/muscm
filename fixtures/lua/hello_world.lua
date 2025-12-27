local gamelib = require("gamelib")
local function main()
	local game = gamelib.init()
	while true do
		game::loop()
	end
end

main()

