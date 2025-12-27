# Phase 3: Core Execution Engine

## Overview
Phase 3 implements the AST interpreter that executes parsed Lua code. It bridges the parser output from Phase 1 with the runtime environment from Phase 2, creating a fully functional interpreter for basic Lua programs.

## Implemented Components

### 1. **ControlFlow Enum**
Signals for control flow in statements:
```rust
pub enum ControlFlow {
    Normal,           // Execution continues normally
    Return(Vec<LuaValue>),  // Function return with values
    Break,            // Break from loop
    Goto(String),     // Jump to label
}
```

### 2. **Block Executor**
Walks through a Block's statements and executes them in order:
```rust
pub fn execute_block(&mut self, block: &Block, interp: &mut LuaInterpreter) -> Result<ControlFlow, String>
```

**Key behavior:**
- Executes statements sequentially
- Propagates control flow signals (break, return, goto)
- Handles return statements at block end
- Returns ControlFlow enum indicating how block terminated

### 3. **Statement Executor**
Pattern matches on Statement enum and executes each type:

#### Simple Statements
- `Empty`: No-op
- `Break`: Signal loop break
- `Label`: Mark label position
- `Goto`: Jump to label

#### Control Flow Statements
- **While**: Loop while condition is truthy
- **Repeat-Until**: Loop until condition is true
- **If-ElseIf-Else**: Conditional branching with multiple alternatives
- **For (Numeric)**: `for i = start, end, step do ... end`
- **For (Generic)**: `for k, v in iterables do ... end`

#### Definition Statements
- **Assignment**: Evaluate RHS, bind to LHS variables
- **FunctionCall**: Execute function for side effects
- **FunctionDecl**: Create and bind function
- **LocalFunction**: Create local-scoped function
- **LocalVars**: Define local variables with optional initialization

#### Do Block
- Create new scope for statements in block

### 4. **Expression Evaluator**
Recursively evaluates expressions with proper type handling:

#### Literals
- Nil, Boolean, Number, String

#### Variables
- Identifier lookup with undefined variable error

#### Operators
- **Binary**: +, -, *, /, //, %, ^, .., <, <=, >, >=, ==, ~=, and, or, &, |, ^, <<, >>
- **Unary**: -, not, ~, # (length)
- **Short-circuit evaluation** for `and` and `or`

#### Complex Expressions
- `Table[key]` - Table indexing
- `Table.field` - Field access (sugar for `Table["field"]`)
- `function(args)` - Function calls
- `object:method(args)` - Method calls
- `{fields}` - Table constructor
- `function(params) ... end` - Anonymous functions

### 5. **Binary Operations**
Implements all Lua binary operators with proper type coercion:

- **Arithmetic**: `+`, `-`, `*`, `/`, `//`, `%`, `^`
- **String**: `.` (concatenation)
- **Comparison**: `<`, `<=`, `>`, `>=`, `==`, `~=`
- **Logical**: `and`, `or` (short-circuit)
- **Bitwise**: `&`, `|`, `^`, `<<`, `>>`

### 6. **Unary Operations**
- `-` (negation): Converts to number and negates
- `not` (logical not): Returns boolean negation of truthiness
- `~` (bitwise not): Bitwise negation on integer part
- `#` (length): String length or table size

### 7. **Table Operations**
- `table_get(table, key)`: Retrieve value from table (returns nil if missing)
- `table_set(table, key, value)`: Set value in table
- `create_table(fields, interp)`: Build table from field list

### 8. **Loop Implementation**

#### While Loop
```rust
while condition_is_true {
    execute_body;
    if break_signal: exit loop
}
```

#### Repeat-Until Loop
```rust
loop {
    execute_body;
    if break_signal: exit loop
    if condition_is_true: break
}
```

#### Numeric For Loop
```rust
for var = start, end, step do
    execute_body
end
```
- Creates new scope for loop variable
- Handles positive and negative step values
- Returns error if step is zero

#### Generic For Loop
```rust
for k, v in iterable do
    execute_body
end
```
- Collects iterator entries first (avoids borrow issues)
- Binds key and value to variables
- Currently supports table iteration

### 9. **Scope Management**
Integrates with LuaInterpreter's scope stack:
- `push_scope()` for block/function entry
- `pop_scope()` for block/function exit
- Variable lookup checks innermost scope first, then globals

### 10. **Type Coercion**
Uses LuaValue's type coercion methods:
- `to_number()`: Convert to f64 with Lua semantics
- `to_string_value()`: Convert to string
- `is_truthy()`: Test truthiness (false and nil are falsy)
- `type_name()`: Get type name for errors

## Architecture Diagram

```
┌─────────────────────────────────────────────────┐
│              Executor                           │
├─────────────────────────────────────────────────┤
│                                                 │
│  execute_block(Block)                           │
│  └─> execute_statement(Statement) for each      │
│      ├─ Assignment: RHS → LHS                   │
│      ├─ While: condition → loop                 │
│      ├─ If: condition → branch                  │
│      ├─ For: init → loop                        │
│      ├─ Do: new scope → block                   │
│      └─ Others: simple dispatch                 │
│                                                 │
│  eval_expression(Expression)                    │
│  ├─ Literals: return directly                   │
│  ├─ Identifiers: lookup in scope                │
│  ├─ BinaryOp: eval left & right → apply op    │
│  ├─ UnaryOp: eval operand → apply op           │
│  ├─ Table[key]: eval both → index              │
│  ├─ function(args): eval → call                 │
│  └─ {fields}: collect → create table            │
│                                                 │
│  apply_binary_op(LuaValue, Op, LuaValue)       │
│  └─> Operator-specific logic with coercion     │
│                                                 │
└──────────────────┬──────────────────────────────┘
                   │
                   ▼
        ┌──────────────────────┐
        │   LuaInterpreter     │
        │  (from Phase 2)      │
        │                      │
        │ - globals            │
        │ - scope_stack        │
        │ - call_stack         │
        │ - value_stack        │
        │ - define/lookup      │
        └──────────────────────┘
```

## Execution Flow Example

```
Lua Code:
  if x > 5 then
    y = 10
  else
    y = 20
  end

Execution:
1. evaluate_expression(x > 5)
   - lookup "x" → LuaValue
   - eval_expression(5) → LuaValue::Number(5.0)
   - apply_binary_op(LuaValue, Gt, Number(5.0))
   - return Boolean(x > 5)

2. Check truthiness
   - is_truthy() → true/false

3. Execute corresponding block
   - eval_expression(10)
   - execute_assignment("y", 10)
   - define("y", Number(10)) or update("y", Number(10))
```

## Error Handling

| Error | Handler | Example |
|-------|---------|---------|
| Undefined variable | `eval_expression(Identifier)` | `"Undefined variable: x"` |
| Invalid assignment target | `execute_assignment` | Cannot assign to literal |
| Division by zero | `apply_binary_op(Divide)` | `"Division by zero"` |
| Cannot index non-table | `table_get` | `"Cannot index number"` |
| Invalid function call | `call_function` | `"Cannot call nil"` |
| Loop step is zero | `execute_for_numeric` | `"for step cannot be zero"` |

## Test Coverage

All 14 tests pass:
- ✅ Executor creation
- ✅ Empty block execution
- ✅ Literal expressions (nil, boolean, number, string)
- ✅ Simple variable assignment
- ✅ Multiple assignment
- ✅ Arithmetic operations (+, -, *, /)
- ✅ Comparison operations (<, >, ==)
- ✅ Logical operations (and, or with short-circuit)
- ✅ Unary operations (-, not)
- ✅ String concatenation
- ✅ Table creation and indexing
- ✅ If statement (true condition)
- ✅ If statement (false with else)

## Limitations

### Not Yet Implemented (Phases 4-6)
1. **Function calls** - Phase 4-5 needed for user functions
   - Placeholder returns error
   - Built-in functions not yet added
   - Variadic arguments not supported

2. **Goto statements** - Partial implementation
   - Labels are marked but not used
   - Goto control flow not yet implemented
   - Would need label tracking and code position jumping

3. **Method calls** - Placeholder implementation
   - Object:method(args) syntax parsed
   - Metatable lookup not yet implemented

4. **Varargs** - Returns nil as placeholder
   - `...` parameter not captured
   - Variadic function support in Phase 5

5. **Metatables** - Not implemented
   - __index, __newindex, __tostring etc.
   - Table metamethods for operators
   - Type coercion rules simplified

6. **String escape sequences** - Not fully handled
   - Only basic string support
   - No \n, \t, \x escapes yet

## Integration with Previous Phases

### Phase 1: Value System
- Uses `LuaValue` enum for all runtime values
- Uses type coercion methods (`to_number()`, `to_string_value()`)
- Uses truthiness testing (`is_truthy()`)

### Phase 2: Runtime Environment
- Uses `LuaInterpreter` for execution context
- Uses `scope_stack` for variable scoping
- Uses `call_stack` for function calls (Phase 4+)
- Uses `value_stack` for expression evaluation

### Parser Output
- Consumes `Block`, `Statement`, `Expression` AST nodes
- All types made public for executor access
- Proper error propagation from parser

## Next Steps (Phase 4)

Phase 4 will implement:
- **Function calls** - Invoke user-defined and built-in functions
- **Call frame management** - Use Phase 2's call_stack for function context
- **Parameter binding** - Map arguments to function parameters
- **Return value handling** - Capture and return multiple values
- **Closure capture** - Capture variables from defining scope

This will enable actual Lua programs with functions to run.

## File Changes

- **src/executor.rs** - New file (~780 lines)
  - Executor struct with all execution logic
  - 14 comprehensive tests
  - Block, Statement, and Expression execution
  - All operator implementations

- **src/lua_parser.rs** - Made types public
  - `Block`, `Statement`, `Expression` structs/enums
  - `BinaryOp`, `UnaryOp`, `Field`, `FieldKey`, `FunctionBody`
  - All fields made public for executor access

- **src/lib.rs** - Added executor module
  - `pub mod executor;`

## Key Design Decisions

1. **ControlFlow enum** instead of exceptions provides compile-time safety for control flow
2. **Recursive evaluator** matches Lua's evaluation semantics naturally
3. **Short-circuit evaluation** implemented at operator level for efficiency
4. **Scope stack integration** leverages Phase 2's infrastructure
5. **Type coercion in operators** keeps type logic centralized in LuaValue
6. **Table entry collection** in for-generic loop avoids borrow checker issues
7. **Placeholder functions** (call_function, create_function) for Phase 4-5 integration
