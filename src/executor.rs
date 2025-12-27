/// Phase 3: Core Execution Engine for Lua AST Interpreter
///
/// This module implements the core execution engine that interprets the parsed Lua AST.
/// It includes:
/// - Block executor: walks Block nodes and executes statements
/// - Statement executor: pattern matches on Statement enum and executes each type
/// - Expression evaluator: recursively evaluates expressions with proper type coercion
/// - Function call mechanism: invokes functions using call frames from Phase 2

use crate::lua_interpreter::LuaInterpreter;
use crate::lua_value::LuaValue;
use crate::lua_parser::{Block, Statement, Expression, BinaryOp, UnaryOp, Field, FieldKey, FunctionBody};
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

// Used in Phase 6 tests
#[cfg(test)]
use crate::lua_value::{LuaFunction, LuaTable};

/// Control flow signals used to handle break, return, and goto statements
#[derive(Debug, Clone)]
pub enum ControlFlow {
    /// Normal execution continues
    Normal,
    /// Return from current block with values
    Return(Vec<LuaValue>),
    /// Break from current loop
    Break,
    /// Jump to a label with target name
    Goto(String),
}

/// Executor for the Lua AST interpreter
pub struct Executor {
    /// For tracking labeled positions (basic support)
    labels: HashMap<String, usize>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            labels: HashMap::new(),
        }
    }

    /// Execute a block of statements with the given interpreter context
    /// Returns ControlFlow indicating how execution completed (normal, return, break, etc)
    pub fn execute_block(
        &mut self,
        block: &Block,
        interp: &mut LuaInterpreter,
    ) -> Result<ControlFlow, String> {
        for statement in &block.statements {
            match self.execute_statement(statement, interp)? {
                ControlFlow::Normal => continue,
                // Propagate non-normal control flow
                cf => return Ok(cf),
            }
        }

        // Check for return statement at end of block
        if let Some(ret) = &block.return_statement {
            let values = self.eval_expression_list(&ret.expression_list, interp)?;
            return Ok(ControlFlow::Return(values));
        }

        Ok(ControlFlow::Normal)
    }

    /// Execute a single statement
    fn execute_statement(
        &mut self,
        stmt: &Statement,
        interp: &mut LuaInterpreter,
    ) -> Result<ControlFlow, String> {
        match stmt {
            Statement::Empty => Ok(ControlFlow::Normal),

            Statement::Assignment { variables, values } => {
                self.execute_assignment(variables, values, interp)?;
                Ok(ControlFlow::Normal)
            }

            Statement::FunctionCall(expr) => {
                self.eval_expression(expr, interp)?;
                Ok(ControlFlow::Normal)
            }

            Statement::Break => Ok(ControlFlow::Break),

            Statement::Label(name) => {
                // Store label position for goto
                self.labels.insert(name.clone(), 0); // Simplified: just mark it exists
                Ok(ControlFlow::Normal)
            }

            Statement::Goto(name) => {
                // Return control flow signal to jump to label
                Ok(ControlFlow::Goto(name.clone()))
            }

            Statement::Do(block) => {
                // Create new scope for do block
                interp.push_scope();
                let result = self.execute_block(block, interp);
                interp.pop_scope();
                match result? {
                    ControlFlow::Normal => Ok(ControlFlow::Normal),
                    other => Ok(other),
                }
            }

            Statement::While { condition, body } => {
                self.execute_while(condition, body, interp)
            }

            Statement::Repeat { body, condition } => {
                self.execute_repeat(body, condition, interp)
            }

            Statement::If {
                condition,
                then_block,
                elseif_parts,
                else_block,
            } => self.execute_if(condition, then_block, elseif_parts, else_block, interp),

            Statement::ForNumeric {
                var,
                start,
                end,
                step,
                body,
            } => self.execute_for_numeric(var, start, end, step.as_ref(), body, interp),

            Statement::ForGeneric {
                vars,
                iterables,
                body,
            } => self.execute_for_generic(vars, iterables, body, interp),

            Statement::FunctionDecl { name, body } => {
                let is_method = name.contains(':');
                let func_value = if is_method {
                    // For methods, we need to prepend 'self' to the parameters
                    let mut new_body = body.as_ref().clone();
                    new_body.params.insert(0, "self".to_string());
                    self.create_function(Box::new(new_body), interp)?
                } else {
                    self.create_function(body.clone(), interp)?
                };
                
                // Check if this is a qualified name (e.g., M.test or M:method)
                if name.contains('.') || name.contains(':') {
                    // Parse qualified name and assign to table
                    let parts: Vec<&str> = if name.contains(':') {
                        name.split(':').collect()
                    } else {
                        name.split('.').collect()
                    };
                    
                    if parts.len() >= 2 {
                        // Get the base table
                        let base_name = parts[0];
                        let mut table = interp.lookup(base_name)
                            .ok_or_else(|| format!("Table '{}' not found", base_name))?;
                        
                        // Navigate through intermediate tables
                        for i in 1..parts.len() - 1 {
                            match table {
                                LuaValue::Table(t) => {
                                    let key = LuaValue::String(parts[i].to_string());
                                    let next = t.borrow().data.get(&key)
                                        .cloned()
                                        .ok_or_else(|| format!("Key '{}' not found in table", parts[i]))?;
                                    table = next;
                                }
                                _ => return Err(format!("'{}' is not a table", parts[i-1])),
                            }
                        }
                        
                        // Set the final key
                        if let LuaValue::Table(t) = table {
                            let final_key = LuaValue::String(parts[parts.len() - 1].to_string());
                            t.borrow_mut().data.insert(final_key, func_value);
                        } else {
                            return Err("Cannot assign to non-table".to_string());
                        }
                    }
                } else {
                    // Simple name
                    interp.define(name.clone(), func_value);
                }
                Ok(ControlFlow::Normal)
            }

            Statement::LocalFunction { name, body } => {
                let func_value = self.create_function(body.clone(), interp)?;
                interp.define(name.clone(), func_value);
                Ok(ControlFlow::Normal)
            }

            Statement::LocalVars { names, values } => {
                let vals = if let Some(value_exprs) = values {
                    self.eval_expression_list(value_exprs, interp)?
                } else {
                    vec![LuaValue::Nil; names.len()]
                };

                // Define each local variable
                for (name, val) in names.iter().zip(vals.iter()) {
                    interp.define(name.clone(), val.clone());
                }
                Ok(ControlFlow::Normal)
            }
        }
    }

    /// Execute assignment statement
    fn execute_assignment(
        &mut self,
        variables: &[Expression],
        values: &[Expression],
        interp: &mut LuaInterpreter,
    ) -> Result<(), String> {
        // Evaluate all RHS expressions
        let mut rhs_values = self.eval_expression_list(values, interp)?;

        // Pad with nil if not enough values
        while rhs_values.len() < variables.len() {
            rhs_values.push(LuaValue::Nil);
        }

        // Assign to each variable
        for (var_expr, value) in variables.iter().zip(rhs_values.iter()) {
            match var_expr {
                Expression::Identifier(name) => {
                    // Update existing variable or create new one
                    if interp.lookup(name).is_some() {
                        interp.update(name, value.clone())?;
                    } else {
                        interp.define(name.clone(), value.clone());
                    }
                }

                Expression::TableIndexing { object, index } => {
                    // Handle table[key] = value
                    let table = self.eval_expression(object, interp)?;
                    let key = self.eval_expression(index, interp)?;
                    self.table_set(&table, key, value.clone())?;
                }

                Expression::FieldAccess { object, field } => {
                    // Handle table.field = value (sugar for table["field"])
                    let table = self.eval_expression(object, interp)?;
                    let key = LuaValue::String(field.clone());
                    self.table_set(&table, key, value.clone())?;
                }

                _ => return Err("Invalid assignment target".to_string()),
            }
        }

        Ok(())
    }

    /// Execute while loop
    fn execute_while(
        &mut self,
        condition: &Expression,
        body: &Block,
        interp: &mut LuaInterpreter,
    ) -> Result<ControlFlow, String> {
        loop {
            let cond_val = self.eval_expression(condition, interp)?;
            if !cond_val.is_truthy() {
                break;
            }

            match self.execute_block(body, interp)? {
                ControlFlow::Normal => continue,
                ControlFlow::Break => break,
                ControlFlow::Return(vals) => return Ok(ControlFlow::Return(vals)),
                ControlFlow::Goto(_) => {
                    return Err("Goto not yet fully supported".to_string())
                }
            }
        }
        Ok(ControlFlow::Normal)
    }

    /// Execute repeat-until loop
    fn execute_repeat(
        &mut self,
        body: &Block,
        condition: &Expression,
        interp: &mut LuaInterpreter,
    ) -> Result<ControlFlow, String> {
        loop {
            match self.execute_block(body, interp)? {
                ControlFlow::Normal => {},
                ControlFlow::Break => return Ok(ControlFlow::Normal),
                ControlFlow::Return(vals) => return Ok(ControlFlow::Return(vals)),
                ControlFlow::Goto(_) => {
                    return Err("Goto not yet fully supported".to_string())
                }
            }

            let cond_val = self.eval_expression(condition, interp)?;
            if cond_val.is_truthy() {
                break;
            }
        }
        Ok(ControlFlow::Normal)
    }

    /// Execute if statement
    fn execute_if(
        &mut self,
        condition: &Expression,
        then_block: &Block,
        elseif_parts: &[(Expression, Block)],
        else_block: &Option<Box<Block>>,
        interp: &mut LuaInterpreter,
    ) -> Result<ControlFlow, String> {
        let cond_val = self.eval_expression(condition, interp)?;
        if cond_val.is_truthy() {
            return self.execute_block(then_block, interp);
        }

        // Check elseif conditions
        for (elseif_cond, elseif_block) in elseif_parts {
            let cond_val = self.eval_expression(elseif_cond, interp)?;
            if cond_val.is_truthy() {
                return self.execute_block(elseif_block, interp);
            }
        }

        // Execute else block if present
        if let Some(else_blk) = else_block {
            self.execute_block(else_blk, interp)
        } else {
            Ok(ControlFlow::Normal)
        }
    }

    /// Execute numeric for loop: for i = start, end, step do ... end
    fn execute_for_numeric(
        &mut self,
        var: &str,
        start: &Expression,
        end: &Expression,
        step: Option<&Expression>,
        body: &Block,
        interp: &mut LuaInterpreter,
    ) -> Result<ControlFlow, String> {
        let start_val = self.eval_expression(start, interp)?.to_number()?;
        let end_val = self.eval_expression(end, interp)?.to_number()?;
        let step_val = if let Some(s) = step {
            self.eval_expression(s, interp)?.to_number()?
        } else {
            1.0
        };

        if step_val == 0.0 {
            return Err("for step cannot be zero".to_string());
        }

        // Create new scope for loop variable
        interp.push_scope();

        let mut i = start_val;
        let continue_loop = if step_val > 0.0 {
            |i: f64, end: f64| i <= end
        } else {
            |i: f64, end: f64| i >= end
        };

        while continue_loop(i, end_val) {
            interp.define(var.to_string(), LuaValue::Number(i));

            match self.execute_block(body, interp)? {
                ControlFlow::Normal => {},
                ControlFlow::Break => break,
                ControlFlow::Return(vals) => {
                    interp.pop_scope();
                    return Ok(ControlFlow::Return(vals));
                }
                ControlFlow::Goto(_) => {
                    interp.pop_scope();
                    return Err("Goto not yet fully supported".to_string());
                }
            }

            i += step_val;
        }

        interp.pop_scope();
        Ok(ControlFlow::Normal)
    }

    /// Execute generic for loop: for k, v in iterables do ... end
    fn execute_for_generic(
        &mut self,
        vars: &[String],
        iterables: &[Expression],
        body: &Block,
        interp: &mut LuaInterpreter,
    ) -> Result<ControlFlow, String> {
        // Evaluate iterator expressions
        let iterator_vals = self.eval_expression_list(iterables, interp)?;

        // Simple implementation: support table iteration
        // Real Lua would use metamethods (__iter), we'll keep it simple
        for iterable in iterator_vals {
            match iterable {
                LuaValue::Table(table) => {
                    interp.push_scope();

                    // Collect keys and values before iteration to avoid borrow issues
                    let entries: Vec<(LuaValue, LuaValue)> = {
                        let table_ref = table.borrow();
                        table_ref.data.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                    };

                    for (key, value) in entries {
                        // Bind variables: vars[0] = key, vars[1] = value, ...
                        if !vars.is_empty() {
                            interp.define(vars[0].clone(), key);
                        }
                        if vars.len() > 1 {
                            interp.define(vars[1].clone(), value);
                        }

                        match self.execute_block(body, interp)? {
                            ControlFlow::Normal => {},
                            ControlFlow::Break => {
                                interp.pop_scope();
                                return Ok(ControlFlow::Normal);
                            }
                            ControlFlow::Return(vals) => {
                                interp.pop_scope();
                                return Ok(ControlFlow::Return(vals));
                            }
                            ControlFlow::Goto(_) => {
                                interp.pop_scope();
                                return Err("Goto not yet fully supported".to_string());
                            }
                        }
                    }

                    interp.pop_scope();
                }
                _ => {
                    return Err(format!(
                        "Cannot iterate over {} value",
                        iterable.type_name()
                    ))
                }
            }
        }

        Ok(ControlFlow::Normal)
    }

    /// Evaluate a single expression
    pub fn eval_expression(
        &mut self,
        expr: &Expression,
        interp: &mut LuaInterpreter,
    ) -> Result<LuaValue, String> {
        match expr {
            Expression::Nil => Ok(LuaValue::Nil),
            Expression::Boolean(b) => Ok(LuaValue::Boolean(*b)),
            Expression::Number(s) => {
                let n = s.parse::<f64>()
                    .map_err(|_| format!("Invalid number: {}", s))?;
                Ok(LuaValue::Number(n))
            }
            Expression::String(s) => Ok(LuaValue::String(s.clone())),
            Expression::Varargs => {
                // Simplified: return nil. Full implementation needs context
                Ok(LuaValue::Nil)
            }
            Expression::Identifier(name) => {
                interp.lookup(name)
                    .ok_or_else(|| format!("Undefined variable: {}", name))
            }
            Expression::BinaryOp { left, op, right } => {
                self.eval_binary_op(left, op, right, interp)
            }
            Expression::UnaryOp { op, operand } => {
                self.eval_unary_op(op, operand, interp)
            }
            Expression::TableIndexing { object, index } => {
                let table = self.eval_expression(object, interp)?;
                let key = self.eval_expression(index, interp)?;
                self.table_get(&table, key)
            }
            Expression::FieldAccess { object, field } => {
                let table = self.eval_expression(object, interp)?;
                let key = LuaValue::String(field.clone());
                self.table_get(&table, key)
            }
            Expression::FunctionCall {
                function,
                args,
            } => {
                let func = self.eval_expression(function, interp)?;
                let arg_vals = self.eval_expression_list(args, interp)?;
                self.call_function(func, arg_vals, interp)
            }
            Expression::MethodCall {
                object,
                method,
                args,
            } => {
                // Method call: obj:method(args) -> method(obj, args)
                let obj = self.eval_expression(object, interp)?;
                let key = LuaValue::String(method.clone());
                
                let method_func = match &obj {
                    LuaValue::String(_) => {
                        // For strings, look up method in the string library
                        let string_lib = interp.lookup("string")
                            .ok_or_else(|| "string library not found".to_string())?;
                        self.table_get(&string_lib, key)?
                    }
                    _ => {
                        // For other types, look up in the object's table
                        self.table_get(&obj, key)?
                    }
                };

                let mut all_args = vec![obj];
                all_args.extend(self.eval_expression_list(args, interp)?);
                self.call_function(method_func, all_args, interp)
            }
            Expression::TableConstructor { fields } => {
                self.create_table(fields, interp)
            }
            Expression::FunctionDef(body) => {
                self.create_function(body.clone(), interp)
            }
        }
    }

    /// Evaluate a list of expressions
    fn eval_expression_list(
        &mut self,
        exprs: &[Expression],
        interp: &mut LuaInterpreter,
    ) -> Result<Vec<LuaValue>, String> {
        let mut results = Vec::new();
        for expr in exprs {
            results.push(self.eval_expression(expr, interp)?);
        }
        Ok(results)
    }

    /// Evaluate binary operations
    fn eval_binary_op(
        &mut self,
        left: &Expression,
        op: &BinaryOp,
        right: &Expression,
        interp: &mut LuaInterpreter,
    ) -> Result<LuaValue, String> {
        // Short-circuit evaluation for 'and' and 'or'
        match op {
            BinaryOp::And => {
                let left_val = self.eval_expression(left, interp)?;
                if !left_val.is_truthy() {
                    return Ok(left_val);
                }
                self.eval_expression(right, interp)
            }
            BinaryOp::Or => {
                let left_val = self.eval_expression(left, interp)?;
                if left_val.is_truthy() {
                    return Ok(left_val);
                }
                self.eval_expression(right, interp)
            }
            _ => {
                let left_val = self.eval_expression(left, interp)?;
                let right_val = self.eval_expression(right, interp)?;
                self.apply_binary_op(&left_val, op, &right_val)
            }
        }
    }

    /// Apply binary operation to two values
    fn apply_binary_op(
        &self,
        left: &LuaValue,
        op: &BinaryOp,
        right: &LuaValue,
    ) -> Result<LuaValue, String> {
        match op {
            BinaryOp::Add => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(LuaValue::Number(l + r))
            }
            BinaryOp::Subtract => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(LuaValue::Number(l - r))
            }
            BinaryOp::Multiply => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(LuaValue::Number(l * r))
            }
            BinaryOp::Divide => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                if r == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(LuaValue::Number(l / r))
            }
            BinaryOp::FloorDivide => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                if r == 0.0 {
                    return Err("Division by zero".to_string());
                }
                Ok(LuaValue::Number((l / r).floor()))
            }
            BinaryOp::Modulo => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                if r == 0.0 {
                    return Err("Modulo by zero".to_string());
                }
                Ok(LuaValue::Number(l % r))
            }
            BinaryOp::Power => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(LuaValue::Number(l.powf(r)))
            }
            BinaryOp::Concat => {
                let l = left.to_string_value();
                let r = right.to_string_value();
                Ok(LuaValue::String(format!("{}{}", l, r)))
            }
            BinaryOp::Lt => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(LuaValue::Boolean(l < r))
            }
            BinaryOp::Lte => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(LuaValue::Boolean(l <= r))
            }
            BinaryOp::Gt => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(LuaValue::Boolean(l > r))
            }
            BinaryOp::Gte => {
                let l = left.to_number()?;
                let r = right.to_number()?;
                Ok(LuaValue::Boolean(l >= r))
            }
            BinaryOp::Eq => {
                Ok(LuaValue::Boolean(left == right))
            }
            BinaryOp::Neq => {
                Ok(LuaValue::Boolean(left != right))
            }
            BinaryOp::BitAnd => {
                let l = left.to_number()? as i64;
                let r = right.to_number()? as i64;
                Ok(LuaValue::Number((l & r) as f64))
            }
            BinaryOp::BitOr => {
                let l = left.to_number()? as i64;
                let r = right.to_number()? as i64;
                Ok(LuaValue::Number((l | r) as f64))
            }
            BinaryOp::BitXor => {
                let l = left.to_number()? as i64;
                let r = right.to_number()? as i64;
                Ok(LuaValue::Number((l ^ r) as f64))
            }
            BinaryOp::LeftShift => {
                let l = left.to_number()? as i64;
                let r = right.to_number()? as i64;
                Ok(LuaValue::Number((l << r) as f64))
            }
            BinaryOp::RightShift => {
                let l = left.to_number()? as i64;
                let r = right.to_number()? as i64;
                Ok(LuaValue::Number((l >> r) as f64))
            }
            BinaryOp::And | BinaryOp::Or => {
                unreachable!("Short-circuit ops should be handled separately")
            }
        }
    }

    /// Evaluate unary operations
    fn eval_unary_op(
        &mut self,
        op: &UnaryOp,
        operand: &Expression,
        interp: &mut LuaInterpreter,
    ) -> Result<LuaValue, String> {
        let val = self.eval_expression(operand, interp)?;
        match op {
            UnaryOp::Minus => {
                let n = val.to_number()?;
                Ok(LuaValue::Number(-n))
            }
            UnaryOp::Not => {
                Ok(LuaValue::Boolean(!val.is_truthy()))
            }
            UnaryOp::BitNot => {
                let n = val.to_number()? as i64;
                Ok(LuaValue::Number((!n) as f64))
            }
            UnaryOp::Length => {
                match val {
                    LuaValue::String(s) => Ok(LuaValue::Number(s.len() as f64)),
                    LuaValue::Table(t) => {
                        // Simple length: count elements (not counting string keys)
                        let table = t.borrow();
                        let count = table.data.iter()
                            .filter(|(k, _)| matches!(k, LuaValue::Number(_)))
                            .count();
                        Ok(LuaValue::Number(count as f64))
                    }
                    _ => Err(format!("Cannot get length of {}", val.type_name())),
                }
            }
        }
    }

    /// Get value from table
    fn table_get(&self, table: &LuaValue, key: LuaValue) -> Result<LuaValue, String> {
        match table {
            LuaValue::Table(t) => {
                let table_ref = t.borrow();
                // Try to get the key directly
                if let Some(value) = table_ref.data.get(&key) {
                    return Ok(value.clone());
                }
                
                // If not found, check metatable for __index
                let index_handler = if let Some(mt) = &table_ref.metatable {
                    mt.get("__index").cloned()
                } else {
                    None
                };
                
                drop(table_ref);
                
                if let Some(handler) = index_handler {
                    // __index can be a table or a function
                    match handler {
                        LuaValue::Table(_) => {
                            // Recursively look up in __index table
                            return self.table_get(&handler, key);
                        }
                        LuaValue::Function(_) => {
                            // For functions, we'd need to call them - for now just return nil
                            return Ok(LuaValue::Nil);
                        }
                        _ => {}
                    }
                }
                
                Ok(LuaValue::Nil)
            }
            _ => Err(format!("table get: Cannot index {}", table.type_name())),
        }
    }

    /// Set value in table
    fn table_set(&self, table: &LuaValue, key: LuaValue, value: LuaValue) -> Result<(), String> {
        match table {
            LuaValue::Table(t) => {
                let mut table_ref = t.borrow_mut();
                table_ref.data.insert(key, value);
                Ok(())
            }
            _ => Err(format!("table set: Cannot index {}", table.type_name())),
        }
    }

    /// Create a table from field list
    fn create_table(
        &mut self,
        fields: &[Field],
        interp: &mut LuaInterpreter,
    ) -> Result<LuaValue, String> {
        let table = interp.create_table();
        match table {
            LuaValue::Table(t) => {
                let mut table_ref = t.borrow_mut();
                let mut index = 1.0; // Lua tables are 1-indexed by default

                for field in fields {
                    let key = match &field.key {
                        FieldKey::Bracket(expr) => self.eval_expression(expr, interp)?,
                        FieldKey::Identifier(name) => LuaValue::String(name.clone()),
                        FieldKey::Index(_) => LuaValue::Number(index),
                    };

                    let value = self.eval_expression(&field.value, interp)?;
                    table_ref.data.insert(key, value);

                    // Increment index for positional fields
                    if matches!(field.key, FieldKey::Index(_)) {
                        index += 1.0;
                    }
                }

                drop(table_ref);
                Ok(LuaValue::Table(t))
            }
            _ => unreachable!(),
        }
    }

    /// Create a function value with closure support
    fn create_function(
        &self,
        body: Box<FunctionBody>,
        interp: &LuaInterpreter,
    ) -> Result<LuaValue, String> {
        // Capture variables from current scope (closure)
        // For now, capture all accessible variables
        let mut captured = HashMap::new();
        
        // Capture from innermost scope first, then globals
        for scope in interp.scope_stack.iter().rev() {
            for (name, value) in scope {
                captured.insert(name.clone(), value.clone());
            }
        }
        
        // Add globals
        for (name, value) in &interp.globals {
            // Only capture if not already in a local scope
            if !captured.contains_key(name) {
                captured.insert(name.clone(), value.clone());
            }
        }

        let func = crate::lua_value::LuaFunction::User {
            params: body.params.clone(),
            varargs: body.varargs,
            body: body.block.clone(),
            captured: Rc::new(RefCell::new(captured)),
        };

        Ok(LuaValue::Function(Rc::new(func)))
    }

    /// Call a function with arguments
    fn call_function(
        &mut self,
        func: LuaValue,
        args: Vec<LuaValue>,
        interp: &mut LuaInterpreter,
    ) -> Result<LuaValue, String> {
        match func {
            LuaValue::Function(f) => match f.as_ref() {
                crate::lua_value::LuaFunction::Builtin(builtin) => {
                    // Try to call the builtin
                    match builtin(args.clone()) {
                        // If require() needs special handling, extract module name from error
                        Err(err) if err.contains("require() must be called through executor") => {
                            if args.len() == 1 {
                                if let LuaValue::String(module_name) = &args[0] {
                                    return self.execute_require(module_name, interp);
                                }
                            }
                            Err(err)
                        }
                        result => result,
                    }
                }
                crate::lua_value::LuaFunction::User { params, varargs, body, captured } => {
                    // Create new scope for function execution
                    interp.push_scope();
                    
                    // Restore captured variables from shared closure state
                    let captured_vars = captured.borrow();
                    for (name, value) in captured_vars.iter() {
                        interp.define(name.clone(), value.clone());
                    }
                    drop(captured_vars);

                    // Bind parameters to arguments
                    for (i, param) in params.iter().enumerate() {
                        let value = args.get(i).cloned().unwrap_or(LuaValue::Nil);
                        interp.define(param.clone(), value);
                    }

                    // Handle varargs if present
                    if *varargs {
                        // Collect extra arguments as varargs
                        let _varargs_vec: Vec<LuaValue> = if args.len() > params.len() {
                            args[params.len()..].to_vec()
                        } else {
                            Vec::new()
                        };
                        // Store varargs as a special table that can be accessed via ...
                        // For now, we store it as a pseudo-variable for expression evaluation
                        interp.define("...".to_string(), LuaValue::Nil); // Placeholder
                    }

                    // Execute function body
                    let result = self.execute_block(body, interp);
                    
                    // Before popping scope, sync modified captured variables back to the closure
                    if let Some(current_scope) = interp.scope_stack.last() {
                        let mut captured_mut = captured.borrow_mut();
                        for (name, value) in captured_mut.iter_mut() {
                            // Update with new value if it exists in current scope
                            if let Some(new_value) = current_scope.get(name) {
                                *value = new_value.clone();
                            }
                        }
                    }
                    
                    // Pop scope and get return values
                    interp.pop_scope();

                    match result? {
                        ControlFlow::Normal => Ok(LuaValue::Nil),
                        ControlFlow::Return(values) => {
                            // Return first value or nil if no return
                            Ok(values.first().cloned().unwrap_or(LuaValue::Nil))
                        }
                        _ => Err("Unexpected control flow in function".to_string()),
                    }
                }
            },
            _ => Err(format!("Cannot call {}", func.type_name())),
        }
    }

    /// Handle require() function call which needs special access to executor and interpreter
    fn execute_require(
        &mut self,
        module_name: &str,
        interp: &mut LuaInterpreter,
    ) -> Result<LuaValue, String> {
        use crate::lua_parser::{self, TokenSlice};

        // Check cache first (without needing to hold borrow)
        {
            let loader = interp.module_loader.borrow();
            if let Some(cached) = loader.loaded_modules.get(module_name) {
                return Ok(cached.clone());
            }
            // Check if currently loading (circular dependency)
            if loader.loading.contains(module_name) {
                return Ok(interp.create_table());
            }
        }

        // Mark as loading
        interp.module_loader.borrow_mut().loading.insert(module_name.to_string());

        // Resolve path
        let path = {
            let loader = interp.module_loader.borrow();
            loader.resolve_module(module_name)
        };

        let path = match path {
            Ok(p) => p,
            Err(e) => {
                interp.module_loader.borrow_mut().loading.remove(module_name);
                return Err(e);
            }
        };

        // Read file
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                interp.module_loader.borrow_mut().loading.remove(module_name);
                return Err(format!("Cannot read module '{}': {}", module_name, e));
            }
        };

        // Tokenize
        let tokens = match lua_parser::tokenize(&content) {
            Ok(t) => t,
            Err(e) => {
                interp.module_loader.borrow_mut().loading.remove(module_name);
                return Err(format!("Tokenize error in module '{}': {}", module_name, e));
            }
        };

        // Parse
        let token_slice = TokenSlice::from(tokens.as_slice());
        let ast = match lua_parser::parse(token_slice) {
            Ok((_, block)) => block,
            Err(e) => {
                interp.module_loader.borrow_mut().loading.remove(module_name);
                return Err(format!("Parse error in module '{}': {}", module_name, e));
            }
        };

        // Execute in isolated scope
        interp.push_scope();

        let result = match self.execute_block(&ast, interp) {
            Ok(control_flow) => {
                use crate::executor::ControlFlow;

                match control_flow {
                    ControlFlow::Return(values) if !values.is_empty() => {
                        values[0].clone()
                    }
                    _ => {
                        interp
                            .lookup("exports")
                            .unwrap_or(LuaValue::Nil)
                    }
                }
            }
            Err(e) => {
                interp.pop_scope();
                interp.module_loader.borrow_mut().loading.remove(module_name);
                return Err(format!("Runtime error in module '{}': {}", module_name, e));
            }
        };

        interp.pop_scope();

        // Mark as loaded and cache
        {
            let mut loader = interp.module_loader.borrow_mut();
            loader.loading.remove(module_name);
            loader.loaded_modules.insert(module_name.to_string(), result.clone());
        }

        Ok(result)
    }
}

impl Default for Executor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = Executor::new();
        assert!(executor.labels.is_empty());
    }

    #[test]
    fn test_empty_block_execution() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();
        
        let block = Block {
            statements: vec![],
            return_statement: None,
        };

        let result = executor.execute_block(&block, &mut interp);
        assert!(result.is_ok());
        match result.unwrap() {
            ControlFlow::Normal => {},
            _ => panic!("Expected Normal control flow"),
        }
    }

    #[test]
    fn test_literal_expressions() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Test nil
        let nil_expr = Expression::Nil;
        let result = executor.eval_expression(&nil_expr, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Nil);

        // Test boolean
        let bool_expr = Expression::Boolean(true);
        let result = executor.eval_expression(&bool_expr, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Boolean(true));

        // Test number
        let num_expr = Expression::Number("42.5".to_string());
        let result = executor.eval_expression(&num_expr, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Number(42.5));

        // Test string
        let str_expr = Expression::String("hello".to_string());
        let result = executor.eval_expression(&str_expr, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::String("hello".to_string()));
    }

    #[test]
    fn test_simple_assignment() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        let var = Expression::Identifier("x".to_string());
        let val = Expression::Number("42".to_string());
        
        let result = executor.execute_assignment(&[var.clone()], &[val], &mut interp);
        assert!(result.is_ok());

        // Check that variable was assigned
        let x = interp.lookup("x");
        assert_eq!(x, Some(LuaValue::Number(42.0)));
    }

    #[test]
    fn test_multiple_assignment() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        let vars = vec![
            Expression::Identifier("a".to_string()),
            Expression::Identifier("b".to_string()),
        ];
        let vals = vec![
            Expression::Number("1".to_string()),
            Expression::Number("2".to_string()),
        ];
        
        let result = executor.execute_assignment(&vars, &vals, &mut interp);
        assert!(result.is_ok());

        assert_eq!(interp.lookup("a"), Some(LuaValue::Number(1.0)));
        assert_eq!(interp.lookup("b"), Some(LuaValue::Number(2.0)));
    }

    #[test]
    fn test_arithmetic_operations() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Test addition
        let add = Expression::BinaryOp {
            left: Box::new(Expression::Number("5".to_string())),
            op: BinaryOp::Add,
            right: Box::new(Expression::Number("3".to_string())),
        };
        let result = executor.eval_expression(&add, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Number(8.0));

        // Test multiplication
        let mul = Expression::BinaryOp {
            left: Box::new(Expression::Number("4".to_string())),
            op: BinaryOp::Multiply,
            right: Box::new(Expression::Number("3".to_string())),
        };
        let result = executor.eval_expression(&mul, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Number(12.0));

        // Test subtraction
        let sub = Expression::BinaryOp {
            left: Box::new(Expression::Number("10".to_string())),
            op: BinaryOp::Subtract,
            right: Box::new(Expression::Number("4".to_string())),
        };
        let result = executor.eval_expression(&sub, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Number(6.0));

        // Test division
        let div = Expression::BinaryOp {
            left: Box::new(Expression::Number("12".to_string())),
            op: BinaryOp::Divide,
            right: Box::new(Expression::Number("3".to_string())),
        };
        let result = executor.eval_expression(&div, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Number(4.0));
    }

    #[test]
    fn test_comparison_operations() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Test less than
        let lt = Expression::BinaryOp {
            left: Box::new(Expression::Number("3".to_string())),
            op: BinaryOp::Lt,
            right: Box::new(Expression::Number("5".to_string())),
        };
        let result = executor.eval_expression(&lt, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Boolean(true));

        // Test greater than
        let gt = Expression::BinaryOp {
            left: Box::new(Expression::Number("5".to_string())),
            op: BinaryOp::Gt,
            right: Box::new(Expression::Number("3".to_string())),
        };
        let result = executor.eval_expression(&gt, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Boolean(true));

        // Test equality
        let eq = Expression::BinaryOp {
            left: Box::new(Expression::Number("5".to_string())),
            op: BinaryOp::Eq,
            right: Box::new(Expression::Number("5".to_string())),
        };
        let result = executor.eval_expression(&eq, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Boolean(true));
    }

    #[test]
    fn test_logical_operations() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Test 'and' short-circuit
        let and_expr = Expression::BinaryOp {
            left: Box::new(Expression::Boolean(false)),
            op: BinaryOp::And,
            right: Box::new(Expression::Boolean(true)),
        };
        let result = executor.eval_expression(&and_expr, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Boolean(false));

        // Test 'or' short-circuit
        let or_expr = Expression::BinaryOp {
            left: Box::new(Expression::Boolean(true)),
            op: BinaryOp::Or,
            right: Box::new(Expression::Boolean(false)),
        };
        let result = executor.eval_expression(&or_expr, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Boolean(true));
    }

    #[test]
    fn test_unary_operations() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Test negation
        let neg = Expression::UnaryOp {
            op: UnaryOp::Minus,
            operand: Box::new(Expression::Number("42".to_string())),
        };
        let result = executor.eval_expression(&neg, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Number(-42.0));

        // Test logical not
        let not = Expression::UnaryOp {
            op: UnaryOp::Not,
            operand: Box::new(Expression::Boolean(true)),
        };
        let result = executor.eval_expression(&not, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::Boolean(false));
    }

    #[test]
    fn test_string_concatenation() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        let concat = Expression::BinaryOp {
            left: Box::new(Expression::String("hello".to_string())),
            op: BinaryOp::Concat,
            right: Box::new(Expression::String(" world".to_string())),
        };
        let result = executor.eval_expression(&concat, &mut interp);
        assert_eq!(result.unwrap(), LuaValue::String("hello world".to_string()));
    }

    #[test]
    fn test_table_creation() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        let fields = vec![
            Field {
                key: FieldKey::Identifier("x".to_string()),
                value: Expression::Number("10".to_string()),
            },
            Field {
                key: FieldKey::Identifier("y".to_string()),
                value: Expression::Number("20".to_string()),
            },
        ];

        let table_expr = Expression::TableConstructor { fields };
        let result = executor.eval_expression(&table_expr, &mut interp);
        assert!(matches!(result.unwrap(), LuaValue::Table(_)));
    }

    #[test]
    fn test_table_indexing() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Create table and assign it
        let table = interp.create_table();
        interp.define("t".to_string(), table);

        // Set a value in the table
        if let Some(LuaValue::Table(t)) = interp.lookup("t") {
            let mut table_ref = t.borrow_mut();
            table_ref.data.insert(LuaValue::String("key".to_string()), LuaValue::Number(42.0));
        }

        // Access the value
        let table_val = interp.lookup("t").unwrap();
        let result = executor.table_get(&table_val, LuaValue::String("key".to_string()));
        assert_eq!(result.unwrap(), LuaValue::Number(42.0));
    }

    #[test]
    fn test_if_statement_true() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        let then_stmt = Statement::Assignment {
            variables: vec![Expression::Identifier("x".to_string())],
            values: vec![Expression::Number("1".to_string())],
        };

        let then_block = Block {
            statements: vec![then_stmt],
            return_statement: None,
        };

        let if_stmt = Statement::If {
            condition: Expression::Boolean(true),
            then_block: Box::new(then_block),
            elseif_parts: vec![],
            else_block: None,
        };

        let result = executor.execute_statement(&if_stmt, &mut interp);
        assert!(result.is_ok());
        assert_eq!(interp.lookup("x"), Some(LuaValue::Number(1.0)));
    }

    #[test]
    fn test_if_statement_false_with_else() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        let then_stmt = Statement::Assignment {
            variables: vec![Expression::Identifier("x".to_string())],
            values: vec![Expression::Number("1".to_string())],
        };
        let then_block = Block {
            statements: vec![then_stmt],
            return_statement: None,
        };

        let else_stmt = Statement::Assignment {
            variables: vec![Expression::Identifier("x".to_string())],
            values: vec![Expression::Number("2".to_string())],
        };
        let else_block = Block {
            statements: vec![else_stmt],
            return_statement: None,
        };

        let if_stmt = Statement::If {
            condition: Expression::Boolean(false),
            then_block: Box::new(then_block),
            elseif_parts: vec![],
            else_block: Some(Box::new(else_block)),
        };

        let result = executor.execute_statement(&if_stmt, &mut interp);
        assert!(result.is_ok());
        assert_eq!(interp.lookup("x"), Some(LuaValue::Number(2.0)));
    }

    #[test]
    fn test_function_creation() {
        let executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        let func_body = FunctionBody {
            params: vec!["x".to_string()],
            varargs: false,
            block: Box::new(Block {
                statements: vec![],
                return_statement: None,
            }),
        };

        let result = executor.create_function(Box::new(func_body), &interp);
        assert!(result.is_ok());
        match result.unwrap() {
            LuaValue::Function(_) => {},
            _ => panic!("Expected function"),
        }
    }

    #[test]
    fn test_function_call_simple() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Create function: function(x) return x + 1 end
        let return_stmt = crate::lua_parser::ReturnStatement {
            expression_list: vec![
                Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("x".to_string())),
                    op: BinaryOp::Add,
                    right: Box::new(Expression::Number("1".to_string())),
                }
            ],
        };

        let func_body = FunctionBody {
            params: vec!["x".to_string()],
            varargs: false,
            block: Box::new(Block {
                statements: vec![],
                return_statement: Some(return_stmt),
            }),
        };

        let func = executor.create_function(Box::new(func_body), &interp).unwrap();

        // Call function with argument 5
        let result = executor.call_function(func, vec![LuaValue::Number(5.0)], &mut interp);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LuaValue::Number(6.0));
    }

    #[test]
    fn test_function_call_with_defaults() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Create function: function(x, y) return x end
        // This returns only x, ignoring y which defaults to nil
        let return_stmt = crate::lua_parser::ReturnStatement {
            expression_list: vec![
                Expression::Identifier("x".to_string()),
            ],
        };

        let func_body = FunctionBody {
            params: vec!["x".to_string(), "y".to_string()],
            varargs: false,
            block: Box::new(Block {
                statements: vec![],
                return_statement: Some(return_stmt),
            }),
        };

        let func = executor.create_function(Box::new(func_body), &interp).unwrap();

        // Call with only one argument (y should default to nil)
        let result = executor.call_function(func, vec![LuaValue::Number(5.0)], &mut interp);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LuaValue::Number(5.0));
    }

    #[test]
    fn test_function_with_closure() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Define outer variable
        interp.define("outer".to_string(), LuaValue::Number(10.0));

        // Create function: function(x) return x + outer end
        let return_stmt = crate::lua_parser::ReturnStatement {
            expression_list: vec![
                Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("x".to_string())),
                    op: BinaryOp::Add,
                    right: Box::new(Expression::Identifier("outer".to_string())),
                }
            ],
        };

        let func_body = FunctionBody {
            params: vec!["x".to_string()],
            varargs: false,
            block: Box::new(Block {
                statements: vec![],
                return_statement: Some(return_stmt),
            }),
        };

        let func = executor.create_function(Box::new(func_body), &interp).unwrap();

        // Call function
        let result = executor.call_function(func, vec![LuaValue::Number(5.0)], &mut interp);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LuaValue::Number(15.0));
    }

    #[test]
    fn test_local_variable_shadowing() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Define global variable
        interp.define("x".to_string(), LuaValue::Number(1.0));
        assert_eq!(interp.lookup("x"), Some(LuaValue::Number(1.0)));

        // Push scope and define local variable with same name
        interp.push_scope();
        interp.define("x".to_string(), LuaValue::Number(2.0));
        assert_eq!(interp.lookup("x"), Some(LuaValue::Number(2.0)));

        // Pop scope and verify original value
        interp.pop_scope();
        assert_eq!(interp.lookup("x"), Some(LuaValue::Number(1.0)));
    }

    #[test]
    fn test_loop_break_statement() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Create a loop that breaks
        let break_stmt = Statement::Break;
        let loop_body = Block {
            statements: vec![break_stmt],
            return_statement: None,
        };

        let while_stmt = Statement::While {
            condition: Expression::Boolean(true),
            body: Box::new(loop_body),
        };

        let result = executor.execute_statement(&while_stmt, &mut interp);
        assert!(result.is_ok());
        match result.unwrap() {
            ControlFlow::Normal => {}, // Loop should exit normally after break
            _ => panic!("Expected normal control flow after break"),
        }
    }

    #[test]
    fn test_local_variable_declaration() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Define global variable
        interp.define("x".to_string(), LuaValue::Number(1.0));

        // Push scope for local declaration (simulating block context)
        interp.push_scope();

        // Create local variable declaration
        let local_stmt = Statement::LocalVars {
            names: vec!["y".to_string()],
            values: Some(vec![Expression::Number("2".to_string())]),
        };

        executor.execute_statement(&local_stmt, &mut interp).unwrap();

        // Local y should exist and be 2
        assert_eq!(interp.lookup("y"), Some(LuaValue::Number(2.0)));
        // Global x should still exist and be 1
        assert_eq!(interp.lookup("x"), Some(LuaValue::Number(1.0)));

        // Pop scope
        interp.pop_scope();

        // After popping, y should not be accessible
        assert_eq!(interp.lookup("y"), None);
        // But x should still be 1
        assert_eq!(interp.lookup("x"), Some(LuaValue::Number(1.0)));
    }

    #[test]
    fn test_do_block_scope() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Define global variable
        interp.define("x".to_string(), LuaValue::Number(1.0));

        // Create do block that redefines x
        let do_block = Block {
            statements: vec![
                Statement::LocalVars {
                    names: vec!["x".to_string()],
                    values: Some(vec![Expression::Number("2".to_string())]),
                }
            ],
            return_statement: None,
        };

        let do_stmt = Statement::Do(Box::new(do_block));
        executor.execute_statement(&do_stmt, &mut interp).unwrap();

        // Global x should still be 1
        assert_eq!(interp.lookup("x"), Some(LuaValue::Number(1.0)));
    }

    #[test]
    fn test_multiple_scope_levels() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Level 0 (global)
        interp.define("a".to_string(), LuaValue::Number(1.0));
        assert_eq!(interp.lookup("a"), Some(LuaValue::Number(1.0)));

        // Level 1
        interp.push_scope();
        interp.define("a".to_string(), LuaValue::Number(2.0));
        interp.define("b".to_string(), LuaValue::Number(20.0));
        assert_eq!(interp.lookup("a"), Some(LuaValue::Number(2.0)));
        assert_eq!(interp.lookup("b"), Some(LuaValue::Number(20.0)));

        // Level 2
        interp.push_scope();
        interp.define("a".to_string(), LuaValue::Number(3.0));
        interp.define("c".to_string(), LuaValue::Number(30.0));
        assert_eq!(interp.lookup("a"), Some(LuaValue::Number(3.0)));
        assert_eq!(interp.lookup("b"), Some(LuaValue::Number(20.0))); // Accessible from level 1
        assert_eq!(interp.lookup("c"), Some(LuaValue::Number(30.0)));

        // Pop to level 1
        interp.pop_scope();
        assert_eq!(interp.lookup("a"), Some(LuaValue::Number(2.0)));
        assert_eq!(interp.lookup("b"), Some(LuaValue::Number(20.0)));
        assert_eq!(interp.lookup("c"), None); // Not accessible

        // Pop to level 0 (global)
        interp.pop_scope();
        assert_eq!(interp.lookup("a"), Some(LuaValue::Number(1.0)));
        assert_eq!(interp.lookup("b"), None);
    }

    #[test]
    fn test_repeat_until_loop() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Create repeat-until loop
        let increment = Statement::Assignment {
            variables: vec![Expression::Identifier("i".to_string())],
            values: vec![Expression::BinaryOp {
                left: Box::new(Expression::Identifier("i".to_string())),
                op: BinaryOp::Add,
                right: Box::new(Expression::Number("1".to_string())),
            }],
        };

        let loop_body = Block {
            statements: vec![increment],
            return_statement: None,
        };

        let repeat_stmt = Statement::Repeat {
            body: Box::new(loop_body),
            condition: Expression::BinaryOp {
                left: Box::new(Expression::Identifier("i".to_string())),
                op: BinaryOp::Gte,
                right: Box::new(Expression::Number("3".to_string())),
            },
        };

        interp.define("i".to_string(), LuaValue::Number(0.0));
        let result = executor.execute_statement(&repeat_stmt, &mut interp);
        assert!(result.is_ok());

        // i should be 3 after loop
        assert_eq!(interp.lookup("i"), Some(LuaValue::Number(3.0)));
    }

    #[test]
    fn test_for_numeric_loop() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Create accumulator variable
        interp.define("sum".to_string(), LuaValue::Number(0.0));

        // Create loop body that accumulates sum
        let sum_stmt = Statement::Assignment {
            variables: vec![Expression::Identifier("sum".to_string())],
            values: vec![Expression::BinaryOp {
                left: Box::new(Expression::Identifier("sum".to_string())),
                op: BinaryOp::Add,
                right: Box::new(Expression::Identifier("i".to_string())),
            }],
        };

        let loop_body = Block {
            statements: vec![sum_stmt],
            return_statement: None,
        };

        let for_stmt = Statement::ForNumeric {
            var: "i".to_string(),
            start: Expression::Number("1".to_string()),
            end: Expression::Number("5".to_string()),
            step: None,
            body: Box::new(loop_body),
        };

        executor.execute_statement(&for_stmt, &mut interp).unwrap();

        // sum should be 1+2+3+4+5 = 15
        assert_eq!(interp.lookup("sum"), Some(LuaValue::Number(15.0)));
    }

    #[test]
    fn test_for_numeric_with_step() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Create accumulator variable
        interp.define("sum".to_string(), LuaValue::Number(0.0));

        // Create loop body
        let sum_stmt = Statement::Assignment {
            variables: vec![Expression::Identifier("sum".to_string())],
            values: vec![Expression::BinaryOp {
                left: Box::new(Expression::Identifier("sum".to_string())),
                op: BinaryOp::Add,
                right: Box::new(Expression::Identifier("i".to_string())),
            }],
        };

        let loop_body = Block {
            statements: vec![sum_stmt],
            return_statement: None,
        };

        // for i = 1, 10, 2 do sum = sum + i end (1, 3, 5, 7, 9)
        let for_stmt = Statement::ForNumeric {
            var: "i".to_string(),
            start: Expression::Number("1".to_string()),
            end: Expression::Number("10".to_string()),
            step: Some(Expression::Number("2".to_string())),
            body: Box::new(loop_body),
        };

        executor.execute_statement(&for_stmt, &mut interp).unwrap();

        // sum should be 1+3+5+7+9 = 25
        assert_eq!(interp.lookup("sum"), Some(LuaValue::Number(25.0)));
    }

    #[test]
    fn test_label_definition() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Create label statement
        let label_stmt = Statement::Label("start".to_string());

        let result = executor.execute_statement(&label_stmt, &mut interp);
        assert!(result.is_ok());

        // Label should be marked as existing
        assert!(executor.labels.contains_key("start"));
    }

    #[test]
    fn test_function_with_varargs() {
        let mut executor = Executor::new();
        let mut interp = LuaInterpreter::new();

        // Create function: function(a, b, ...) return a + b end
        let return_stmt = crate::lua_parser::ReturnStatement {
            expression_list: vec![
                Expression::BinaryOp {
                    left: Box::new(Expression::Identifier("a".to_string())),
                    op: BinaryOp::Add,
                    right: Box::new(Expression::Identifier("b".to_string())),
                }
            ],
        };

        let func_body = FunctionBody {
            params: vec!["a".to_string(), "b".to_string()],
            varargs: true,
            block: Box::new(Block {
                statements: vec![],
                return_statement: Some(return_stmt),
            }),
        };

        let func = executor.create_function(Box::new(func_body), &interp).unwrap();

        // Call with extra arguments (should accept them without error)
        let result = executor.call_function(
            func,
            vec![
                LuaValue::Number(5.0),
                LuaValue::Number(3.0),
                LuaValue::Number(10.0), // Extra argument
                LuaValue::Number(20.0), // Extra argument
            ],
            &mut interp,
        );
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), LuaValue::Number(8.0)); // 5 + 3
    }

    // =====================
    // Phase 6 Tests: Standard Library
    // =====================

    #[test]
    fn test_print_function() {
        let interp = LuaInterpreter::new();
        // print() is available in globals
        assert!(interp.globals.contains_key("print"));
    }

    #[test]
    fn test_type_function() {
        let mut interp = LuaInterpreter::new();
        let mut executor = Executor::new();
        
        // Test type() on different values
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_type()))),
            vec![LuaValue::Number(42.0)],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::String("number".to_string()));
        
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_type()))),
            vec![LuaValue::String("hello".to_string())],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::String("string".to_string()));
    }

    #[test]
    fn test_tonumber_function() {
        let mut interp = LuaInterpreter::new();
        let mut executor = Executor::new();
        
        // Convert string to number
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_tonumber()))),
            vec![LuaValue::String("123".to_string())],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::Number(123.0));
        
        // Invalid string returns nil
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_tonumber()))),
            vec![LuaValue::String("abc".to_string())],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::Nil);
    }

    #[test]
    fn test_tostring_function() {
        let mut interp = LuaInterpreter::new();
        let mut executor = Executor::new();
        
        // Convert number to string
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_tostring()))),
            vec![LuaValue::Number(42.0)],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::String("42".to_string()));
        
        // Convert boolean to string
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_tostring()))),
            vec![LuaValue::Boolean(true)],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::String("true".to_string()));
    }

    #[test]
    fn test_string_len() {
        let mut interp = LuaInterpreter::new();
        let mut executor = Executor::new();
        
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_string_len()))),
            vec![LuaValue::String("hello".to_string())],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::Number(5.0));
    }

    #[test]
    fn test_string_upper() {
        let mut interp = LuaInterpreter::new();
        let mut executor = Executor::new();
        
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_string_upper()))),
            vec![LuaValue::String("hello".to_string())],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::String("HELLO".to_string()));
    }

    #[test]
    fn test_string_lower() {
        let mut interp = LuaInterpreter::new();
        let mut executor = Executor::new();
        
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_string_lower()))),
            vec![LuaValue::String("HELLO".to_string())],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::String("hello".to_string()));
    }

    #[test]
    fn test_string_sub() {
        let mut interp = LuaInterpreter::new();
        let mut executor = Executor::new();
        
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_string_sub()))),
            vec![
                LuaValue::String("hello".to_string()),
                LuaValue::Number(1.0),
                LuaValue::Number(3.0),
            ],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::String("hel".to_string()));
    }

    #[test]
    fn test_math_abs() {
        let mut interp = LuaInterpreter::new();
        let mut executor = Executor::new();
        
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_math_abs()))),
            vec![LuaValue::Number(-42.0)],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::Number(42.0));
    }

    #[test]
    fn test_math_floor() {
        let mut interp = LuaInterpreter::new();
        let mut executor = Executor::new();
        
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_math_floor()))),
            vec![LuaValue::Number(3.7)],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::Number(3.0));
    }

    #[test]
    fn test_math_ceil() {
        let mut interp = LuaInterpreter::new();
        let mut executor = Executor::new();
        
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_math_ceil()))),
            vec![LuaValue::Number(3.2)],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::Number(4.0));
    }

    #[test]
    fn test_math_min() {
        let mut interp = LuaInterpreter::new();
        let mut executor = Executor::new();
        
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_math_min()))),
            vec![
                LuaValue::Number(5.0),
                LuaValue::Number(2.0),
                LuaValue::Number(8.0),
            ],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::Number(2.0));
    }

    #[test]
    fn test_math_max() {
        let mut interp = LuaInterpreter::new();
        let mut executor = Executor::new();
        
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_math_max()))),
            vec![
                LuaValue::Number(5.0),
                LuaValue::Number(2.0),
                LuaValue::Number(8.0),
            ],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::Number(8.0));
    }

    #[test]
    fn test_table_insert() {
        let mut interp = LuaInterpreter::new();
        let mut executor = Executor::new();
        
        // Create a table
        let table = LuaValue::Table(Rc::new(RefCell::new(LuaTable {
            data: HashMap::new(),
            metatable: None,
        })));
        
        let result = executor.call_function(
            LuaValue::Function(Rc::new(LuaFunction::Builtin(crate::stdlib::create_table_insert()))),
            vec![table.clone(), LuaValue::Number(42.0)],
            &mut interp,
        );
        assert_eq!(result.unwrap(), LuaValue::Nil);
        
        // Verify the value was inserted
        if let LuaValue::Table(t) = table {
            let table_ref = t.borrow();
            assert!(table_ref.data.contains_key(&LuaValue::Number(1.0)));
        }
    }

    #[test]
    fn test_string_table_exists() {
        let interp = LuaInterpreter::new();
        let string_table = interp.globals.get("string");
        assert!(string_table.is_some());
        
        if let Some(LuaValue::Table(t)) = string_table {
            let table = t.borrow();
            assert!(table.data.contains_key(&LuaValue::String("len".to_string())));
            assert!(table.data.contains_key(&LuaValue::String("upper".to_string())));
            assert!(table.data.contains_key(&LuaValue::String("lower".to_string())));
            assert!(table.data.contains_key(&LuaValue::String("sub".to_string())));
        } else {
            panic!("string table not found or not a table");
        }
    }

    #[test]
    fn test_math_table_exists() {
        let interp = LuaInterpreter::new();
        let math_table = interp.globals.get("math");
        assert!(math_table.is_some());
        
        if let Some(LuaValue::Table(t)) = math_table {
            let table = t.borrow();
            assert!(table.data.contains_key(&LuaValue::String("abs".to_string())));
            assert!(table.data.contains_key(&LuaValue::String("floor".to_string())));
            assert!(table.data.contains_key(&LuaValue::String("ceil".to_string())));
            assert!(table.data.contains_key(&LuaValue::String("min".to_string())));
            assert!(table.data.contains_key(&LuaValue::String("max".to_string())));
        } else {
            panic!("math table not found or not a table");
        }
    }

    #[test]
    fn test_table_table_exists() {
        let interp = LuaInterpreter::new();
        let table_table = interp.globals.get("table");
        assert!(table_table.is_some());
        
        if let Some(LuaValue::Table(t)) = table_table {
            let table = t.borrow();
            assert!(table.data.contains_key(&LuaValue::String("insert".to_string())));
            assert!(table.data.contains_key(&LuaValue::String("remove".to_string())));
        } else {
            panic!("table table not found or not a table");
        }
    }

    // Phase 7: Metatables Tests
    
    #[test]
    fn test_setgetmetatable_basic() {
        let interp = LuaInterpreter::new();
        
        // Create a table
        let t = LuaValue::Table(Rc::new(RefCell::new(LuaTable {
            data: HashMap::new(),
            metatable: None,
        })));
        
        // Create a metatable
        let mt = LuaValue::Table(Rc::new(RefCell::new(LuaTable {
            data: HashMap::new(),
            metatable: None,
        })));
        
        // Call setmetatable(t, mt) via the function
        let setmetatable_fn = interp.lookup("setmetatable").unwrap();
        if let LuaValue::Function(f) = setmetatable_fn {
            if let crate::lua_value::LuaFunction::Builtin(builtin) = f.as_ref() {
                let result = builtin(vec![t.clone(), mt.clone()]);
                assert!(result.is_ok());
                
                // Verify that getmetatable returns the metatable
                let getmetatable_fn = interp.lookup("getmetatable").unwrap();
                if let LuaValue::Function(gf) = getmetatable_fn {
                    if let crate::lua_value::LuaFunction::Builtin(gbuiltin) = gf.as_ref() {
                        let mt_result = gbuiltin(vec![t.clone()]);
                        assert!(mt_result.is_ok());
                        assert!(matches!(mt_result.unwrap(), LuaValue::Table(_)));
                    }
                }
            }
        }
    }

    #[test]
    fn test_setmetatable_nil_clears() {
        let interp = LuaInterpreter::new();
        
        // Create a table with metatable
        let t = LuaValue::Table(Rc::new(RefCell::new(LuaTable {
            data: HashMap::new(),
            metatable: Some(Box::new(HashMap::new())),
        })));
        
        // Clear metatable with nil
        let setmetatable_fn = interp.lookup("setmetatable").unwrap();
        if let LuaValue::Function(f) = setmetatable_fn {
            if let crate::lua_value::LuaFunction::Builtin(builtin) = f.as_ref() {
                let result = builtin(vec![t.clone(), LuaValue::Nil]);
                assert!(result.is_ok());
                
                // Verify metatable is cleared
                let getmetatable_fn = interp.lookup("getmetatable").unwrap();
                if let LuaValue::Function(gf) = getmetatable_fn {
                    if let crate::lua_value::LuaFunction::Builtin(gbuiltin) = gf.as_ref() {
                        let mt_result = gbuiltin(vec![t.clone()]);
                        assert!(mt_result.is_ok());
                        assert_eq!(mt_result.unwrap(), LuaValue::Nil);
                    }
                }
            }
        }
    }

    #[test]
    fn test_getmetatable_nonexistent() {
        let interp = LuaInterpreter::new();
        
        // Create a table without metatable
        let t = LuaValue::Table(Rc::new(RefCell::new(LuaTable {
            data: HashMap::new(),
            metatable: None,
        })));
        
        // getmetatable should return nil
        let getmetatable_fn = interp.lookup("getmetatable").unwrap();
        if let LuaValue::Function(f) = getmetatable_fn {
            if let crate::lua_value::LuaFunction::Builtin(builtin) = f.as_ref() {
                let result = builtin(vec![t]);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), LuaValue::Nil);
            }
        }
    }

    #[test]
    fn test_getmetatable_non_table() {
        let interp = LuaInterpreter::new();
        
        // getmetatable on non-table should return nil
        let getmetatable_fn = interp.lookup("getmetatable").unwrap();
        if let LuaValue::Function(f) = getmetatable_fn {
            if let crate::lua_value::LuaFunction::Builtin(builtin) = f.as_ref() {
                let result = builtin(vec![LuaValue::Number(42.0)]);
                assert!(result.is_ok());
                assert_eq!(result.unwrap(), LuaValue::Nil);
            }
        }
    }

    // Phase 7: Error Handling Tests
    
    #[test]
    fn test_error_function() {
        let interp = LuaInterpreter::new();
        
        let error_fn = interp.lookup("error").unwrap();
        if let LuaValue::Function(f) = error_fn {
            if let crate::lua_value::LuaFunction::Builtin(builtin) = f.as_ref() {
                let result = builtin(vec![LuaValue::String("test error".to_string())]);
                assert!(result.is_err());
                assert_eq!(result.unwrap_err(), "test error");
            }
        }
    }

    #[test]
    fn test_pcall_requires_function() {
        let interp = LuaInterpreter::new();
        
        let pcall_fn = interp.lookup("pcall").unwrap();
        if let LuaValue::Function(f) = pcall_fn {
            if let crate::lua_value::LuaFunction::Builtin(builtin) = f.as_ref() {
                let result = builtin(vec![LuaValue::Number(42.0)]);
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn test_pcall_with_function() {
        let interp = LuaInterpreter::new();
        
        // Create a simple function
        let func = LuaValue::Function(Rc::new(LuaFunction::Builtin(
            Rc::new(|_| Ok(LuaValue::Number(42.0)))
        )));
        
        let pcall_fn = interp.lookup("pcall").unwrap();
        if let LuaValue::Function(f) = pcall_fn {
            if let crate::lua_value::LuaFunction::Builtin(builtin) = f.as_ref() {
                let result = builtin(vec![func]);
                assert!(result.is_ok());
            }
        }
    }

    #[test]
    fn test_xpcall_requires_functions() {
        let interp = LuaInterpreter::new();
        
        let xpcall_fn = interp.lookup("xpcall").unwrap();
        if let LuaValue::Function(f) = xpcall_fn {
            if let crate::lua_value::LuaFunction::Builtin(builtin) = f.as_ref() {
                let result = builtin(vec![LuaValue::Number(42.0), LuaValue::Number(0.0)]);
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn test_xpcall_with_functions() {
        let interp = LuaInterpreter::new();
        
        // Create two simple functions
        let func1 = LuaValue::Function(Rc::new(LuaFunction::Builtin(
            Rc::new(|_| Ok(LuaValue::Number(42.0)))
        )));
        let func2 = LuaValue::Function(Rc::new(LuaFunction::Builtin(
            Rc::new(|_| Ok(LuaValue::String("error handled".to_string())))
        )));
        
        let xpcall_fn = interp.lookup("xpcall").unwrap();
        if let LuaValue::Function(f) = xpcall_fn {
            if let crate::lua_value::LuaFunction::Builtin(builtin) = f.as_ref() {
                let result = builtin(vec![func1, func2]);
                assert!(result.is_ok());
            }
        }
    }

    // Phase 7: Coroutine Tests
    
    #[test]
    fn test_coroutine_table_exists() {
        let interp = LuaInterpreter::new();
        let coro_table = interp.globals.get("coroutine");
        assert!(coro_table.is_some());
        
        if let Some(LuaValue::Table(t)) = coro_table {
            let table = t.borrow();
            assert!(table.data.contains_key(&LuaValue::String("create".to_string())));
            assert!(table.data.contains_key(&LuaValue::String("resume".to_string())));
            assert!(table.data.contains_key(&LuaValue::String("yield".to_string())));
            assert!(table.data.contains_key(&LuaValue::String("status".to_string())));
        } else {
            panic!("coroutine table not found or not a table");
        }
    }

    #[test]
    fn test_phase7_functions_registered() {
        let interp = LuaInterpreter::new();
        
        // Check Phase 7 functions are registered
        assert!(interp.globals.contains_key("setmetatable"));
        assert!(interp.globals.contains_key("getmetatable"));
        assert!(interp.globals.contains_key("pcall"));
        assert!(interp.globals.contains_key("xpcall"));
        assert!(interp.globals.contains_key("error"));
        assert!(interp.globals.contains_key("coroutine"));
    }

    #[test]
    fn test_metatable_with_string_keys() {
        let interp = LuaInterpreter::new();
        
        // Create a table with string keys for metamethods
        let mut mt_data = HashMap::new();
        mt_data.insert(
            LuaValue::String("__add".to_string()),
            LuaValue::Function(Rc::new(LuaFunction::Builtin(
                Rc::new(|args| {
                    // Simple add that returns sum of first two numbers
                    if args.len() >= 2 {
                        let a = args[0].to_number().unwrap_or(0.0);
                        let b = args[1].to_number().unwrap_or(0.0);
                        Ok(LuaValue::Number(a + b))
                    } else {
                        Ok(LuaValue::Nil)
                    }
                })
            )))
        );
        
        let mt = LuaValue::Table(Rc::new(RefCell::new(LuaTable {
            data: mt_data,
            metatable: None,
        })));
        
        let t = LuaValue::Table(Rc::new(RefCell::new(LuaTable {
            data: HashMap::new(),
            metatable: None,
        })));
        
        let setmetatable_fn = interp.lookup("setmetatable").unwrap();
        if let LuaValue::Function(f) = setmetatable_fn {
            if let crate::lua_value::LuaFunction::Builtin(builtin) = f.as_ref() {
                let result = builtin(vec![t.clone(), mt.clone()]);
                assert!(result.is_ok());
                
                // Verify metamethod is accessible through getmetatable
                let getmetatable_fn = interp.lookup("getmetatable").unwrap();
                if let LuaValue::Function(gf) = getmetatable_fn {
                    if let crate::lua_value::LuaFunction::Builtin(gbuiltin) = gf.as_ref() {
                        let mt_retrieved = gbuiltin(vec![t.clone()]);
                        assert!(mt_retrieved.is_ok());
                        
                        if let Ok(LuaValue::Table(mt_table)) = mt_retrieved {
                            let mt_borrow = mt_table.borrow();
                            assert!(mt_borrow.data.contains_key(&LuaValue::String("__add".to_string())));
                        } else {
                            panic!("Expected table from getmetatable");
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn test_upvalues_module_loads() {
        // Just verify the upvalues module compiles and can be used
        use crate::upvalues::{Upvalue, ClosureState};
        
        let mut cs = ClosureState::new();
        let uv = Upvalue::new("x".to_string(), 0, LuaValue::Number(42.0));
        cs.add_upvalue(uv.clone());
        
        assert_eq!(cs.get_upvalue("x").unwrap().value, LuaValue::Number(42.0));
        
        cs.update_upvalue("x", LuaValue::Number(100.0));
        assert_eq!(cs.get_upvalue("x").unwrap().value, LuaValue::Number(100.0));
    }

    #[test]
    fn test_coroutines_module_loads() {
        // Just verify the coroutines module compiles and can be used
        use crate::coroutines::{Coroutine, CoroutineStatus, CoroutineRegistry};
        
        let mut co = Coroutine::new(1, vec![], vec![]);
        assert_eq!(co.status, CoroutineStatus::Suspended);
        
        let (ok, _) = co.resume(vec![]);
        assert!(ok);
        assert_eq!(co.status, CoroutineStatus::Running);
        
        let mut registry = CoroutineRegistry::new();
        let id = registry.create(vec![], vec![]);
        assert!(registry.get(id).is_some());
    }
}
