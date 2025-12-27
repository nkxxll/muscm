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

/// Control flow signals used to handle break, return, and goto statements
#[derive(Debug, Clone)]
pub enum ControlFlow {
    /// Normal execution continues
    Normal,
    /// Return from current block with values
    Return(Vec<LuaValue>),
    /// Break from current loop
    Break,
    /// Jump to a label (not fully implemented yet)
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
                let func_value = self.create_function(body.clone(), interp)?;
                interp.define(name.clone(), func_value);
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
                let table = self.eval_expression(object, interp)?;
                let key = LuaValue::String(method.clone());
                let method_func = self.table_get(&table, key)?;

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
                Ok(table_ref.data.get(&key).cloned().unwrap_or(LuaValue::Nil))
            }
            _ => Err(format!("Cannot index {}", table.type_name())),
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
            _ => Err(format!("Cannot index {}", table.type_name())),
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
            captured,
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
                    // Call built-in function
                    builtin(args)
                }
                crate::lua_value::LuaFunction::User { params, varargs, body, captured } => {
                    // Create new scope for function execution
                    interp.push_scope();
                    
                    // Restore captured variables
                    for (name, value) in captured {
                        interp.define(name.clone(), value.clone());
                    }

                    // Bind parameters to arguments
                    for (i, param) in params.iter().enumerate() {
                        let value = args.get(i).cloned().unwrap_or(LuaValue::Nil);
                        interp.define(param.clone(), value);
                    }

                    // Handle varargs if present
                    if *varargs && args.len() > params.len() {
                        // For now, varargs are not fully supported
                        // In a full implementation, we'd bind ... to remaining args
                    }

                    // Execute function body
                    let result = self.execute_block(body, interp);
                    
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
}
