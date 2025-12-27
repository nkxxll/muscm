-- Phase 8 File I/O & System Integration Demo
-- Demonstrates file operations, system functions, and practical patterns

print("=== Phase 8: File I/O & System Integration Demo ===\n")

-- Demo 1: Basic File Writing and Reading
print("Demo 1: Write and Read File")
print("-" .. string.rep("-", 40))

local f = io.open("demo_output.txt", "w")
f:write("Phase 8 Demo\n")
f:write("=============\n")
f:write("Line 1\n")
f:write("Line 2\n")
f:write("Line 3\n")
f:close()

f = io.open("demo_output.txt", "r")
local content = f:read("a")
f:close()

print("File content:")
print(content)

-- Demo 2: Environment Variables
print("\nDemo 2: Environment Variables")
print("-" .. string.rep("-", 40))

os.setenv("DEMO_VAR", "Hello from Phase 8")
local env_value = os.getenv("DEMO_VAR")
print("DEMO_VAR = " .. env_value)

-- Demo 3: Line-by-Line Processing
print("\nDemo 3: Line Processing")
print("-" .. string.rep("-", 40))

function count_lines(filename)
  local f = io.open(filename, "r")
  if not f then
    return 0
  end
  
  local count = 0
  while true do
    local line = f:read("l")
    if not line then break end
    count = count + 1
  end
  f:close()
  return count
end

local lines = count_lines("demo_output.txt")
print("File has " .. lines .. " lines")

-- Demo 4: File Operations
print("\nDemo 4: File Operations")
print("-" .. string.rep("-", 40))

-- Create a test file
local f = io.open("test_original.txt", "w")
f:write("Original content")
f:close()
print("Created: test_original.txt")

-- Rename it
os.rename("test_original.txt", "test_renamed.txt")
print("Renamed to: test_renamed.txt")

-- Read renamed file
f = io.open("test_renamed.txt", "r")
print("Content: " .. f:read("a"))
f:close()

-- Delete it
os.remove("test_renamed.txt")
print("Deleted: test_renamed.txt")

-- Demo 5: System Command Execution
print("\nDemo 5: System Commands")
print("-" .. string.rep("-", 40))

-- Show current timestamp
local timestamp = os.time()
print("Current timestamp: " .. timestamp)

-- Demo 6: Temporary Files
print("\nDemo 6: Temporary Files")
print("-" .. string.rep("-", 40))

local tmpfile = os.tmpname()
print("Generated temp filename: " .. tmpfile)

-- Use it
local f = io.open(tmpfile, "w")
f:write("Temporary data")
f:close()
print("Created temp file")

-- Clean up
os.remove(tmpfile)
print("Removed temp file")

-- Demo 7: Copy File Function
print("\nDemo 7: Copy File Function")
print("-" .. string.rep("-", 40))

function copy_file(source, dest)
  local src = io.open(source, "r")
  if not src then
    return false, "Cannot open source file"
  end
  
  local data = src:read("a")
  src:close()
  
  local dst = io.open(dest, "w")
  dst:write(data)
  dst:close()
  
  return true
end

-- Create source file
local f = io.open("original.txt", "w")
f:write("This is the original file\n")
f:write("With multiple lines\n")
f:close()

-- Copy it
copy_file("original.txt", "copy.txt")
print("Copied original.txt to copy.txt")

-- Verify
f = io.open("copy.txt", "r")
print("Verification: " .. f:read("l"))
f:close()

-- Demo 8: Log File Writer
print("\nDemo 8: Log File Writer")
print("-" .. string.rep("-", 40))

function create_logger(filename)
  return function(level, message)
    local f = io.open(filename, "a")
    local entry = string.format("[%s] %s\n", level, message)
    f:write(entry)
    f:close()
  end
end

local log = create_logger("demo.log")
log("INFO", "Application started")
log("DEBUG", "Processing request")
log("WARNING", "High memory usage")

-- Show log contents
print("\nLog file contents:")
f = io.open("demo.log", "r")
print(f:read("a"))
f:close()

-- Demo 9: Read and Filter
print("Demo 9: Read and Filter")
print("-" .. string.rep("-", 40))

function read_lines(filename)
  local lines = {}
  local f = io.open(filename, "r")
  if not f then return lines end
  
  while true do
    local line = f:read("l")
    if not line then break end
    table.insert(lines, line)
  end
  f:close()
  return lines
end

local all_lines = read_lines("demo_output.txt")
print("Read " .. #all_lines .. " lines from demo_output.txt")

-- Demo 10: Safe File Operations with Error Handling
print("\nDemo 10: Safe File Operations")
print("-" .. string.rep("-", 40))

function safe_read(filename)
  local ok, result = pcall(function()
    local f = io.open(filename, "r")
    if not f then
      error("File not found: " .. filename)
    end
    local content = f:read("a")
    f:close()
    return content
  end)
  
  if ok then
    return result
  else
    return nil
  end
end

local data = safe_read("demo_output.txt")
if data then
  print("Successfully read file (" .. string.len(data) .. " bytes)")
else
  print("Failed to read file")
end

-- Cleanup
print("\n" .. "=" .. string.rep("=", 40))
print("Cleaning up demo files...")
os.remove("demo_output.txt")
os.remove("original.txt")
os.remove("copy.txt")
os.remove("demo.log")
print("Done!")
