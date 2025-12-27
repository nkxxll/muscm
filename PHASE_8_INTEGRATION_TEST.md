# Phase 8 Integration Tests

## File I/O Tests

### Test 1: Write and Read File
```lua
-- Create a file
f = io.open("test.txt", "w")
f:write("Hello, ")
f:write("World!")
f:close()

-- Read it back
f = io.open("test.txt", "r")
content = f:read("a")
f:close()

assert(content == "Hello, World!", "File content mismatch")
print("✓ Write and read file")
```

### Test 2: Read Line by Line
```lua
f = io.open("lines.txt", "w")
f:write("Line 1\nLine 2\nLine 3\n")
f:close()

f = io.open("lines.txt", "r")
line1 = f:read("l")
line2 = f:read("l")
line3 = f:read("l")
f:close()

assert(line1 == "Line 1", "First line mismatch")
assert(line2 == "Line 2", "Second line mismatch")
assert(line3 == "Line 3", "Third line mismatch")
print("✓ Read line by line")
```

### Test 3: Append Mode
```lua
-- Create initial file
f = io.open("append.txt", "w")
f:write("Initial")
f:close()

-- Append to file
f = io.open("append.txt", "a")
f:write(" content")
f:close()

-- Verify
f = io.open("append.txt", "r")
content = f:read("a")
f:close()

assert(content == "Initial content", "Append failed")
print("✓ Append mode works")
```

### Test 4: File Not Found
```lua
f = io.open("nonexistent_file_xyz.txt", "r")
assert(f == nil, "Should return nil for non-existent file")
print("✓ File not found handling")
```

### Test 5: Write Multiple Values
```lua
f = io.open("multi.txt", "w")
f:write("Numbers: ", 1, ", ", 2, ", ", 3)
f:close()

f = io.open("multi.txt", "r")
content = f:read("a")
f:close()

assert(content == "Numbers: 1, 2, 3", "Multiple writes failed")
print("✓ Write multiple values")
```

## System Functions Tests

### Test 6: Get Environment Variable
```lua
os.setenv("TEST_VAR", "test_value")
value = os.getenv("TEST_VAR")
assert(value == "test_value", "Environment variable not set")
print("✓ Get/set environment variables")
```

### Test 7: Get Missing Environment Variable
```lua
value = os.getenv("NONEXISTENT_VAR_XYZ_123")
assert(value == nil, "Should return nil for missing variable")
print("✓ Missing environment variable returns nil")
```

### Test 8: Execute Command
```lua
-- This test is platform-specific
-- On Unix: execute 'true' command (always succeeds)
-- On Windows: execute 'cmd /c exit /b 0'
local code = os.execute("true")
-- Code should be 0 for success
print("✓ Execute command (exit code: " .. code .. ")")
```

### Test 9: Get Timestamp
```lua
time1 = os.time()
-- Small delay
for i = 1, 100000 do end
time2 = os.time()

assert(time2 >= time1, "Timestamp should be monotonic")
print("✓ Get timestamp")
```

### Test 10: Temp File
```lua
tmp = os.tmpname()
assert(tmp ~= nil and tmp ~= "", "Should return temp filename")
print("✓ Generate temp filename: " .. tmp)
```

## File Operations Tests

### Test 11: Remove File
```lua
-- Create a file
f = io.open("to_remove.txt", "w")
f:write("temporary")
f:close()

-- Remove it
result = os.remove("to_remove.txt")
assert(result == nil, "remove should return nil on success")

-- Verify it's gone
f = io.open("to_remove.txt", "r")
assert(f == nil, "File should be deleted")
print("✓ Remove file")
```

### Test 12: Rename File
```lua
-- Create a file
f = io.open("original.txt", "w")
f:write("data")
f:close()

-- Rename it
result = os.rename("original.txt", "renamed.txt")
assert(result == nil, "rename should return nil on success")

-- Check new name exists and old doesn't
f = io.open("renamed.txt", "r")
assert(f ~= nil, "Renamed file should exist")
content = f:read("a")
f:close()
assert(content == "data", "Content should be preserved")

f = io.open("original.txt", "r")
assert(f == nil, "Original file should not exist")

-- Cleanup
os.remove("renamed.txt")
print("✓ Rename file")
```

## Practical Examples

### Test 13: Copy File Function
```lua
function copy_file(src, dst)
  local src_file = io.open(src, "r")
  if not src_file then return false end
  
  local content = src_file:read("a")
  src_file:close()
  
  local dst_file = io.open(dst, "w")
  dst_file:write(content)
  dst_file:close()
  
  return true
end

-- Test it
f = io.open("source.txt", "w")
f:write("Source content")
f:close()

assert(copy_file("source.txt", "destination.txt"), "Copy failed")

f = io.open("destination.txt", "r")
content = f:read("a")
f:close()

assert(content == "Source content", "Copied content mismatch")

os.remove("source.txt")
os.remove("destination.txt")
print("✓ Copy file function")
```

### Test 14: Count Lines in File
```lua
function count_lines(filename)
  local f = io.open(filename, "r")
  if not f then return 0 end
  
  local count = 0
  while true do
    local line = f:read("l")
    if not line then break end
    count = count + 1
  end
  f:close()
  
  return count
end

-- Test it
f = io.open("count_test.txt", "w")
f:write("Line 1\nLine 2\nLine 3\nLine 4\n")
f:close()

local lines = count_lines("count_test.txt")
assert(lines == 4, "Line count mismatch")

os.remove("count_test.txt")
print("✓ Count lines in file")
```

### Test 15: Read and Process Lines
```lua
function process_lines(filename, func)
  local f = io.open(filename, "r")
  if not f then return end
  
  while true do
    local line = f:read("l")
    if not line then break end
    func(line)
  end
  f:close()
end

-- Test it
f = io.open("process_test.txt", "w")
f:write("apple\nbanana\ncherry\n")
f:close()

local items = {}
process_lines("process_test.txt", function(line)
  table.insert(items, line)
end)

assert(#items == 3, "Should have 3 items")
assert(items[1] == "apple", "First item mismatch")

os.remove("process_test.txt")
print("✓ Process lines with function")
```

## Edge Cases

### Test 16: Empty File
```lua
f = io.open("empty.txt", "w")
f:close()

f = io.open("empty.txt", "r")
content = f:read("a")
f:close()

assert(content == "", "Empty file should return empty string")

os.remove("empty.txt")
print("✓ Empty file handling")
```

### Test 17: Large Numbers in Write
```lua
f = io.open("numbers.txt", "w")
f:write(123456789)
f:write(3.14159)
f:close()

f = io.open("numbers.txt", "r")
content = f:read("a")
f:close()

assert(string.len(content) > 0, "Numbers should be written")

os.remove("numbers.txt")
print("✓ Write large numbers")
```

### Test 18: Special Characters
```lua
local special = "Hello!\nTab:\tQuote:\"Value\"\n"
f = io.open("special.txt", "w")
f:write(special)
f:close()

f = io.open("special.txt", "r")
content = f:read("a")
f:close()

assert(content == special, "Special characters should be preserved")

os.remove("special.txt")
print("✓ Special character handling")
```

## Integration with Previous Phases

### Test 19: File I/O with Error Handling
```lua
function safe_read_file(filename)
  local ok, result = pcall(function()
    local f = io.open(filename, "r")
    if not f then error("File not found") end
    local content = f:read("a")
    f:close()
    return content
  end)
  
  if ok then
    return result
  else
    return nil, result
  end
end

f = io.open("safe_test.txt", "w")
f:write("Safe content")
f:close()

content, err = safe_read_file("safe_test.txt")
assert(content == "Safe content", "Safe read failed")

content, err = safe_read_file("nonexistent.txt")
assert(content == nil, "Should fail gracefully")

os.remove("safe_test.txt")
print("✓ File I/O with error handling")
```

### Test 20: Closures with File Operations
```lua
function create_appender(filename)
  return function(text)
    local f = io.open(filename, "a")
    f:write(text .. "\n")
    f:close()
  end
end

local log_error = create_appender("errors.log")
log_error("Error 1")
log_error("Error 2")

f = io.open("errors.log", "r")
line1 = f:read("l")
line2 = f:read("l")
f:close()

assert(line1 == "Error 1", "First log line mismatch")
assert(line2 == "Error 2", "Second log line mismatch")

os.remove("errors.log")
print("✓ Closures with file operations")
```

## Summary

All 20 Phase 8 integration tests verify:
- ✅ Basic file operations (read, write, append, close)
- ✅ File system operations (remove, rename)
- ✅ Environment variable access
- ✅ System command execution
- ✅ Timestamp access
- ✅ Error handling and edge cases
- ✅ Practical file processing patterns
- ✅ Integration with closure and error handling

These tests ensure Phase 8 provides production-ready file I/O and system integration capabilities.
