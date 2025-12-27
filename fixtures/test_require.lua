-- Test module loading
local simple = require("simple")
print("simple.add(2, 3) = " .. tostring(simple.add(2, 3)))
print("simple.value = " .. tostring(simple.value))

local config = require("config")
print("config.host = " .. config.host)
print("config.port = " .. tostring(config.port))
print("config.get_connection_string() = " .. config.get_connection_string())

local math_utils = require("utils.math")
print("math_utils.square(5) = " .. tostring(math_utils.square(5)))
print("math_utils.cube(3) = " .. tostring(math_utils.cube(3)))
