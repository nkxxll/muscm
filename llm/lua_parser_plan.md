# Lua Parser Implementation Plan

## Summary

**Current Status**: ✅ Phases 1-5 COMPLETE  
**Total Tests**: 113 lua_parser tests + 31 tokenizer tests = 144 passing

The Lua parser is now fully functional with complete expression, statement, and block parsing capabilities. All core language features are supported including binary/unary operators, control flow structures, function declarations, and table operations.

---

## Current Status (Detailed)

**Tokenizer**: ✅ Complete with 31 passing tests
- All keywords and symbols recognized
- Whitespace and comment handling
- String literals (double-quoted), numbers, identifiers

**AST Types**: ✅ Complete
- Expression types fully defined (literals, operators, table ops, function calls, table/function construction)
- Binary/Unary operators defined (21 binary, 4 unary)
- Statement enum with 12 variants
- Function bodies and parameters with varargs support
- Table fields (bracket expressions, named fields, positional)
- Block structure with optional return statements
- Complete type definitions

**Parser**: ✅ Phase 5 Complete (113 lua_parser tests)
- Tokenization ✅
- Literal parsing ✅
- Expression parsing ✅ (binary/unary ops with correct precedence)
- Prefix expressions ✅ (variables, calls, method calls, table ops)
- Table constructors ✅
- Function definitions ✅
- Statement parsing ✅ (all 12 statement types)
- Block parsing ✅
- Integration tests ✅ (26 comprehensive tests)

---

## Phase 1: Core Expression Parsing

### 1.1 Binary Operator Precedence
- [x] Implement operator precedence table (Lua has 14 levels)
- [x] Write `parse_binary_op` with proper precedence climbing
- [x] Add test cases for precedence (e.g., `a + b * c`)

### 1.2 Complete Expression Parsing
- [x] `parse_unary_expr` (handle `-`, `not`, `#`, `~`)
- [x] `parse_power_expr` (right-associative `^`)
- [x] `parse_multiplicative_expr` (`*`, `/`, `//`, `%`)
- [x] `parse_additive_expr` (`+`, `-`)
- [x] `parse_concat_expr` (`.., right-associative`)
- [x] `parse_relational_expr` (`<`, `<=`, `>`, `>=`, `==`, `~=`)
- [x] `parse_logical_and_expr` (`and`)
- [x] `parse_logical_or_expr` (`or`)
- [x] Test each level with edge cases

### 1.3 Primary Expressions
- [x] `parse_parenthesized_expr` - `(exp)`
- [x] `parse_table_constructor` - `{fieldlist}`
  - [x] Parse field variants: `[exp] = exp`, `name = exp`, `exp`
  - [x] Handle field separators (`,` or `;`)
- [x] `parse_function_def` - `function funcbody`
- [x] Test combinations with proper precedence

**Phase 1 Status**: ✅ COMPLETE (55 passing tests)

---

## Phase 2: Prefix Expressions & Function Calls

### 2.1 Prefix Expressions
- [x] Define `Prefix` AST type or extend `Expression`
- [x] Implement `parse_prefix_exp`:
   - [x] Variable: identifier
   - [x] Function call: `prefixexp args`
   - [x] Method call: `prefixexp : name args`
   - [x] Parenthesized: `(exp)`
   - [x] Table indexing: `[exp]`
   - [x] Field access: `.name`

### 2.2 Function Arguments
- [x] `parse_args` - parenthesized, table, or string literal
- [x] Handle optional parentheses for string/table args

### 2.3 Variable References
- [x] `parse_var` - extends prefix expressions (via `parse_prefix_exp`)
- [x] Table indexing: `var[exp]`
- [x] Field access: `var.name`

---

## Phase 3: Statement Parsing

### 3.1 Simple Statements
- [x] `parse_empty_statement` - `;`
- [x] `parse_assignment` - `varlist = explist`
- [x] `parse_function_call` - `functioncall`
- [x] `parse_break` - `break`
- [x] `parse_label` - `:: name ::`
- [x] `parse_goto` - `goto name`

### 3.2 Block Statements (require recursion)
- [x] `parse_do_block` - `do block end`
- [x] `parse_while_loop` - `while exp do block end`
- [x] `parse_repeat_until` - `repeat block until exp`
- [x] `parse_if_statement` - full `if/elseif/else` structure
- [x] `parse_for_numeric` - `for name = exp, exp [, exp] do block end`
- [x] `parse_for_generic` - `for namelist in explist do block end`

### 3.3 Declaration Statements
- [x] `parse_function_decl` - `function funcname funcbody`
- [x] `parse_local_function` - `local function name funcbody`
- [x] `parse_local_vars` - `local namelist [= explist]`

### 3.4 Return Statement
- [x] `parse_return_stmt` - `return [explist] [;]` (already done in Phase 2)

### 3.5 Update Statement Enum
- [x] Replace empty `enum Statement` with all variants
- [x] Ensure each variant has necessary fields

**Phase 3 Status**: ✅ COMPLETE (21 new tests added, 76 total passing)

---

## Phase 4: Top-Level Block Parsing

### 4.1 Main Parsers
- [x] Implement `parse_statement_list` - parse multiple statements
- [x] Implement `parse_block` - statements + optional return
- [x] Implement `parse_chunk` - entry point (chunk = block)

### 4.2 Helper Functions
- [x] `token_tag(token)` - already partially done, complete if needed
- [x] `consume_token(token_type)` - expect specific token
- [x] `optional_token(token_type)` - consume if present
- [x] Error handling/recovery if needed

**Phase 4 Status**: ✅ COMPLETE (87 passing tests)

---

## Phase 5: AST Refinement & Testing

### 5.1 Complete AST Definitions
- [x] Implement full `Statement` enum with all variants
- [x] Add `TableConstructor` variant to `Expression`
- [x] Add `FunctionDef` variant to `Expression`
- [x] Add `TableIndexing` and `FieldAccess` to `Expression`
- [x] Add `FunctionCall` to `Expression`

### 5.2 Integration Tests
- [x] Test complete programs (multiple statements)
- [x] Test nested structures (loops in loops, functions in functions)
- [x] Test mixed statements and expressions
- [x] Test error recovery (if applicable)

### 5.3 Edge Cases
- [x] Operator precedence edge cases
- [x] Whitespace handling (already tokenizer-tested)
- [x] Comment handling (already tokenizer-tested)
- [x] Empty blocks
- [x] Optional commas/semicolons where allowed

**Phase 5 Status**: ✅ COMPLETE (26 new integration tests added, 113 total lua_parser tests)

---

## Phase 6: Interpreter / Code Generator

### 6.1 Lua Runtime Environment
- [ ] Implement `LuaValue` enum for runtime values (nil, bool, number, string, table, function)
- [ ] Implement `LuaTable` as HashMap for associative arrays
- [ ] Create `Environment` struct for variable scoping
- [ ] Implement global and local variable scoping

### 6.2 Expression Evaluation
- [ ] `eval_expression` - evaluate an expression to a value
- [ ] Support literals (nil, booleans, numbers, strings)
- [ ] Support binary operators with type coercion
- [ ] Support unary operators
- [ ] Support table operations (indexing, field access)
- [ ] Support function calls (both user-defined and built-ins)

### 6.3 Statement Execution
- [ ] `execute_statement` - execute individual statements
- [ ] Handle assignments (including multiple assignment)
- [ ] Handle control flow (if/elseif/else, while, repeat/until, for)
- [ ] Handle function declarations and local functions
- [ ] Handle local variable declarations
- [ ] Handle break statements (requires control flow context)

### 6.4 Block & Program Execution
- [ ] `execute_block` - execute a block of statements
- [ ] Return value handling (explicit returns and implicit nil)
- [ ] Scope management for blocks
- [ ] Entry point `execute` for running a full program

### 6.5 Built-in Functions
- [ ] `print` - output values
- [ ] `type` - return type of value
- [ ] `tostring`/`tonumber` - type conversions
- [ ] `pairs`/`ipairs` - table iteration
- [ ] `table.insert`, `table.remove` - table operations

**Phase 6 Status**: ⏳ NOT STARTED

---

## Implementation Order Recommendation

1. **Binary operator precedence** (Phase 1.1-1.3) - foundational
2. **Primary expressions** (Phase 1.3) - enables testing higher-level constructs
3. **Simple statements** (Phase 3.1) - non-recursive, quick wins
4. **Block statements** (Phase 3.2) - enables program structure
5. **Declarations** (Phase 3.3) - function/local variable support
6. **Top-level parsing** (Phase 4) - brings it all together
7. **Polish & integration** (Phase 5) - comprehensive testing

---

## Notes

- The grammar is defined in the file header comments - use as reference
- Use `nom` combinators: `alt`, `many0`, `sequence`, etc.
- Each parsing function should return `IResult<TokenSlice, AstType>`
- Consider left-recursion issues (nom doesn't support it directly)
- Test incrementally after each phase
