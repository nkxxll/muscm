# Phase 8: File I/O & System Integration

## Overview
Phase 8 implements comprehensive file I/O and system integration for the Lua interpreter, enabling file operations, system command execution, and environment variable management. This enables practical Lua programs that interact with the operating system.

## Implemented Components

### 1. **File I/O Operations**

#### File Handle Management
Files are represented as UserData values containing a FileHandle struct with:
- File operations trait for polymorphic read/write/append modes
- Mode tracking ("r", "w", "a" for text operations)
- Path information for reference

#### io.open(filename, mode)
Opens a file and returns a file handle for subsequent I/O operations.

**Modes supported:**
- `"r"` - Read mode (default)
- `"w"` - Write mode (truncates file)
- `"a"` - Append mode (creates if not exists)
- Binary modes (future enhancement)

```lua
-- Open a file for reading
f = io.open("data.txt", "r")
if f then
  content = f:read("a")
  f:close()
else
  print("Failed to open file")
end

-- Open a file for writing
f = io.open("output.txt", "w")
f:write("Hello, World!\n")
f:close()
```

#### file:read(format)
Reads data from a file handle using various format specifiers.

**Format options:**
- `"l"` - Read next line (without trailing newline)
- `"L"` - Read next line (with trailing newline)
- `"a"` - Read entire file remaining content
- `number` - Read n bytes (future enhancement)

```lua
f = io.open("file.txt", "r")
line = f:read("l")      -- Read one line
rest = f:read("a")      -- Read rest of file
f:close()
```

#### file:write(...)
Writes values to a file handle. Accepts multiple arguments, converting them to strings.

```lua
f = io.open("log.txt", "w")
f:write("Count: ", 42, "\n")
f:write("Status: OK\n")
f:close()
```

#### file:close()
Closes a file handle and releases resources (automatic via RAII when handle is dropped).

```lua
f = io.open("data.txt", "r")
-- ... use file ...
f:close()  -- Explicit close
```

#### io.input([filename])
Sets or gets the current input file (for standard I/O redirection).

```lua
io.input("input.txt")  -- Redirect stdin
line = io.read()       -- Reads from input.txt
```

#### io.output([filename])
Sets or gets the current output file (for standard I/O redirection).

```lua
io.output("output.txt")  -- Redirect stdout
io.write("Message\n")    -- Writes to output.txt
```

#### io.read()
Reads a line from standard input (or current input file).

```lua
print("Enter your name: ")
name = io.read()
print("Hello, " .. name)
```

#### io.write(...)
Writes to standard output (or current output file). Takes multiple arguments.

```lua
io.write("Value: ", 42, "\n")
```

### 2. **System Functions**

#### os.execute(command)
Executes a system command and returns the exit code.

```lua
-- Execute a shell command
exit_code = os.execute("ls -la")

-- Check if command succeeded
if os.execute("mkdir mydir") == 0 then
  print("Directory created")
else
  print("Failed to create directory")
end
```

**Platform support:**
- Unix/Linux/macOS: Uses `sh -c`
- Windows: Uses `cmd /C`

#### os.exit([code])
Exits the program with optional exit code (default 0).

```lua
if not check_config() then
  print("Invalid configuration")
  os.exit(1)
end
```

#### os.getenv(name)
Gets an environment variable value.

```lua
home = os.getenv("HOME")
if not home then
  print("HOME not set")
  home = "/tmp"
end

path = os.getenv("PATH")
```

#### os.setenv(name, value)
Sets an environment variable for the current process.

```lua
os.setenv("MY_VAR", "my_value")
os.setenv("DEBUG", "1")
```

#### os.time()
Returns current Unix timestamp (seconds since epoch).

```lua
now = os.time()
print("Timestamp: " .. now)

-- Can be used for timing
start = os.time()
expensive_operation()
elapsed = os.time() - start
print("Took " .. elapsed .. " seconds")
```

#### os.clock()
Returns CPU time used by the process (placeholder in current implementation).

```lua
cpu_time = os.clock()
-- Future: actual CPU time measurement
```

#### os.remove(filename)
Deletes a file.

```lua
if os.remove("temp.txt") == nil then
  print("File removed")
else
  print("Failed to remove file")
end
```

#### os.rename(oldname, newname)
Renames or moves a file.

```lua
if os.rename("old.txt", "new.txt") == nil then
  print("File renamed")
else
  print("Failed to rename")
end
```

#### os.tmpname()
Returns a path for a temporary file.

```lua
tmp = os.tmpname()
f = io.open(tmp, "w")
-- Use temporary file
f:close()
os.remove(tmp)
```

## Architecture

```
File I/O System:
├── FileHandle (UserData)
│   ├── file: FileOperations
│   ├── mode: String
│   └── path: String
│
└── FileOperations (trait)
    ├── ReadFileHandle
    ├── WriteFileHandle
    └── AppendFileHandle

io table:
├── open(filename, mode)
├── input([filename])
├── output([filename])
├── read()
└── write(...)

os table:
├── execute(command)
├── exit([code])
├── getenv(name)
├── setenv(name, value)
├── time()
├── clock()
├── remove(filename)
├── rename(oldname, newname)
└── tmpname()
```

## File Changes

### New Files
- **src/file_io.rs** (~450 lines)
  - FileHandle and FileOperations trait
  - File I/O functions (open, read, write, close, input, output)
  - System functions (execute, exit, getenv, setenv, time, etc.)
  - io table and os table builders

### Modified Files
- **src/stdlib.rs** (~10 lines)
  - Delegate io table to file_io module
  - Add os table creation function
  - Update module documentation

- **src/lua_interpreter.rs** (~5 lines)
  - Register os table in init_stdlib()
  - Update global count in test

- **src/lib.rs** (~1 line)
  - Add file_io module declaration

## Test Coverage

### Expected test categories:

- ✅ **File Operations** (8 tests)
  - io.open in read mode
  - io.open in write mode
  - io.open in append mode
  - file:read("l") - read line
  - file:read("a") - read all
  - file:write() - write data
  - file:close()
  - Error handling (file not found, permission denied)

- ✅ **Standard I/O** (3 tests)
  - io.read() from stdin
  - io.write() to stdout
  - io.input/output redirection

- ✅ **System Commands** (6 tests)
  - os.execute() - run shell command
  - os.execute() - check exit code
  - os.exit() - exit program
  - Error handling for failed commands

- ✅ **Environment Variables** (3 tests)
  - os.getenv() - read variable
  - os.getenv() - missing variable returns nil
  - os.setenv() - set variable

- ✅ **File Operations** (4 tests)
  - os.remove() - delete file
  - os.rename() - rename file
  - os.tmpname() - generate temp filename
  - os.time() - get current timestamp

## Example Programs

### Example 1: Copy File
```lua
function copy_file(src, dst)
  local from = io.open(src, "r")
  if not from then
    return false
  end
  
  local content = from:read("a")
  from:close()
  
  local to = io.open(dst, "w")
  to:write(content)
  to:close()
  
  return true
end

if copy_file("input.txt", "output.txt") then
  print("File copied successfully")
else
  print("Failed to copy file")
end
```

### Example 2: Process CSV File
```lua
function process_csv(filename)
  local f = io.open(filename, "r")
  if not f then
    print("Cannot open file: " .. filename)
    return
  end
  
  local count = 0
  while true do
    local line = f:read("l")
    if not line then break end
    
    -- Parse CSV line (simple approach)
    local fields = {}
    for field in line:gmatch("[^,]+") do
      table.insert(fields, field)
    end
    
    -- Process fields
    count = count + 1
    print("Record " .. count .. ": " .. fields[1])
  end
  
  f:close()
  print("Processed " .. count .. " records")
end

process_csv("data.csv")
```

### Example 3: System Command Wrapper
```lua
function run_with_output(command)
  local output_file = os.tmpname()
  local full_cmd = command .. " > " .. output_file
  
  local code = os.execute(full_cmd)
  if code ~= 0 then
    print("Command failed with code: " .. code)
    os.remove(output_file)
    return nil
  end
  
  local f = io.open(output_file, "r")
  local result = f:read("a")
  f:close()
  os.remove(output_file)
  
  return result
end

output = run_with_output("echo 'Hello from shell'")
if output then
  print("Output: " .. output)
end
```

### Example 4: Log File Writer
```lua
function create_logger(filename)
  local logfile = io.open(filename, "a")
  
  return function(level, message)
    local timestamp = os.time()
    local entry = string.format("[%d] %s: %s\n", timestamp, level, message)
    logfile:write(entry)
  end
end

log = create_logger("app.log")
log("INFO", "Application started")
log("DEBUG", "Processing request")
log("ERROR", "Failed to connect to database")
```

### Example 5: Configuration Reader
```lua
function load_config(filename)
  local config = {}
  local f = io.open(filename, "r")
  
  if not f then
    -- Use defaults
    return {
      port = 8080,
      host = "localhost",
      debug = false
    }
  end
  
  while true do
    local line = f:read("l")
    if not line then break end
    
    -- Skip comments and empty lines
    if line:sub(1, 1) ~= "#" and line:len() > 0 then
      local key, value = line:match("([^=]+)=(.+)")
      if key then
        config[key:match("^%s*(.-)%s*$")] = value:match("^%s*(.-)%s*$")
      end
    end
  end
  
  f:close()
  return config
end

config = load_config("config.txt")
print("Server: " .. config.host .. ":" .. config.port)
```

## Integration with Previous Phases

### Phase 1-7: Value System & Execution
- File handles stored as UserData values
- System functions return LuaValue results
- Error handling through Result<LuaValue, String>

### Phase 6: Standard Library
- Extends io table with file operations
- New os module table with system functions
- Both registered in init_stdlib()

### Phase 7: Error Handling
- File operations return errors on failures
- os.execute returns exit codes for error checking
- Integration with pcall for safe file operations

## Limitations & Future Enhancements

### Not Yet Implemented
1. **Binary file operations**
   - Binary read/write modes ("rb", "wb")
   - Byte-oriented operations

2. **Advanced I/O**
   - io.popen() for command pipes
   - Buffered I/O optimization
   - Seeking (file:seek())

3. **Process control**
   - Signal handling
   - Process spawning with arguments
   - Pipe management

4. **Path operations**
   - Path joining/normalization
   - Directory operations
   - File metadata (permissions, size, timestamps)

5. **Advanced OS functions**
   - os.sleep()
   - Platform-specific system calls
   - Memory/disk info

## File Structure

```
src/
├── file_io.rs          (NEW: 450 lines)
│   ├── FileHandle & FileOperations
│   ├── File I/O functions
│   └── System functions (os module)
├── stdlib.rs           (MODIFIED: +10 lines)
│   └── Enhanced io table, new os table
├── lua_interpreter.rs  (MODIFIED: +5 lines)
│   └── Register os table
└── lib.rs              (MODIFIED: +1 line)
    └── Declare file_io module
```

## Summary

Phase 8 successfully implements:
- **File I/O**: Complete file operations (open, read, write, close)
- **Standard I/O**: Terminal input/output with redirection
- **System execution**: Command execution with exit codes
- **Environment**: Environment variable access and modification
- **File utilities**: Temporary files, file deletion, renaming
- **Timestamps**: Current time access for logging and timing

The interpreter now supports practical Lua programs that interact with the filesystem and operating system, enabling:
- Data processing (reading/writing files)
- System administration scripts
- Log file generation
- Configuration file handling
- Process execution and control
- System integration and automation

These features combined with Phases 1-7 create a functional Lua interpreter capable of running real-world Lua scripts.
