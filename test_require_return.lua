local TestMod = require("test_module_return")
print("Type of TestMod:", type(TestMod))
if TestMod then
  print("TestMod loaded successfully")
else
  print("TestMod is nil!")
end
