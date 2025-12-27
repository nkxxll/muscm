-- Math utilities module
local exports = {}

exports.square = function(x)
    return x * x
end

exports.cube = function(x)
    return x * x * x
end

exports.factorial = function(n)
    if n <= 1 then
        return 1
    else
        return n * exports.factorial(n - 1)
    end
end

return exports
