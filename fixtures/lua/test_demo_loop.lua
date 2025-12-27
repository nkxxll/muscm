print("Start of loop test")

local start = os.time()
print("Start time:", start)

print("Trying first condition check")
local cond = os.difftime(os.time(), start) < 10
print("Condition result:", cond)

print("Trying while loop")
while os.difftime(os.time(), start) < 1 do
  print("In loop")
  break
end
print("Done with while loop")
