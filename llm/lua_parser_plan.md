# Lua Parser Implementation Plan

## Current Status

**Tokenizer**: ✅ Complete with 40+ passing tests
- All keywords and symbols recognized
- Whitespace and comment handling
- String literals, numbers, identifiers

**AST Types**: ✅ Phase 2 Complete
- Expression types fully defined (including function calls, table constructors, etc.)
- Binary/Unary operators defined
- Function bodies and parameters
- Table fields (bracket, identifier, array indices)
- Statement enum is empty (needed for Phase 3)
- Block, FunctionBody structs defined

**Parser**: ✅ Phase 2 Complete (11 passing tests)
- Literal parsing ✅
- Expression parsing ✅ (binary ops, unary ops)
- Prefix expressions ✅ (identifiers, function calls, method calls, table indexing, field access)
- Table constructors ✅
- Function definitions ✅
- Function arguments ✅ (parens, table, string)
- All statement parsing still needed (Phase 3)

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
- [ ] Implement `parse_statement_list` - parse multiple statements
- [ ] Implement `parse_block` - statements + optional return
- [ ] Implement `parse_chunk` - entry point (chunk = block)

### 4.2 Helper Functions
- [ ] `token_tag(token)` - already partially done, complete if needed
- [ ] `consume_token(token_type)` - expect specific token
- [ ] `optional_token(token_type)` - consume if present
- [ ] Error handling/recovery if needed

---

## Phase 5: AST Refinement & Testing

### 5.1 Complete AST Definitions
- [ ] Implement full `Statement` enum with all variants
- [ ] Add `TableConstructor` variant to `Expression`
- [ ] Add `FunctionDef` variant to `Expression`
- [ ] Add `TableIndexing` and `FieldAccess` to `Expression`
- [ ] Add `FunctionCall` to `Expression`

### 5.2 Integration Tests
- [ ] Test complete programs (multiple statements)
- [ ] Test nested structures (loops in loops, functions in functions)
- [ ] Test mixed statements and expressions
- [ ] Test error recovery (if applicable)

### 5.3 Edge Cases
- [ ] Operator precedence edge cases
- [ ] Whitespace handling (already tokenizer-tested)
- [ ] Comment handling (already tokenizer-tested)
- [ ] Empty blocks
- [ ] Optional commas/semicolons where allowed

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
