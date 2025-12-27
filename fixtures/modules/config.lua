-- Configuration module using exports pattern
local exports = {}

exports.host = "localhost"
exports.port = 8080
exports.debug = true

function exports.get_connection_string()
    return exports.host .. ":" .. tostring(exports.port)
end

return exports
