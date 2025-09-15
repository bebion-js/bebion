//! JavaScript to bytecode compiler

use crate::bytecode::{Bytecode, Constant, Instruction};
use crate::{CompileError, CompileResult};
use bebion_parser::ast::*;
use std::collections::HashMap;
use tracing::debug;

pub struct Compiler {
    scopes: Vec<Scope>,
    loop_stack: Vec<LoopInfo>,
    function_depth: usize,
}

#[derive(Debug, Clone)]
struct Scope {
    variables: HashMap<String, Variable>,
    depth: usize,
}

#[derive(Debug, Clone)]
struct Variable {
    index: usize,
    kind: VarKind,
    is_captured: bool,
}

#[derive(Debug, Clone)]
struct LoopInfo {
    break_jumps: Vec<usize>,
    continue_jumps: Vec<usize>,
}

impl Compiler {
    pub fn new() -> Self {
        let global_scope = Scope {
            variables: HashMap::new(),
            depth: 0,
        };
        
        Self {
            scopes: vec![global_scope],
            loop_stack: Vec::new(),
            function_depth: 0,
        }
    }

    pub fn compile(&mut self, program: &Program) -> CompileResult<Bytecode> {
        debug!("Compiling program with {} statements", program.body.len());
        
        let mut bytecode = Bytecode::new();
        
        for statement in &program.body {
            self.compile_statement(statement, &mut bytecode)?;
        }
        
        // End with halt instruction
        bytecode.emit(Instruction::Halt);
        
        // Optimize the bytecode
        bytecode.optimize();
        
        debug!("Generated {} instructions", bytecode.len());
        Ok(bytecode)
    }

    fn compile_statement(&mut self, stmt: &AstNode, bytecode: &mut Bytecode) -> CompileResult<()> {
        match stmt {
            AstNode::ExpressionStatement { expression, .. } => {
                self.compile_expression(expression, bytecode)?;
                bytecode.emit(Instruction::Pop); // Discard expression result
            }
            
            AstNode::VariableDeclaration { declarations, kind, .. } => {
                for decl in declarations {
                    self.compile_variable_declarator(decl, kind, bytecode)?;
                }
            }
            
            AstNode::FunctionDeclaration { id, params, body, is_async, is_generator, .. } => {
                self.compile_function_declaration(id, params, body, *is_async, *is_generator, bytecode)?;
            }
            
            AstNode::BlockStatement { body, .. } => {
                self.begin_scope();
                for statement in body {
                    self.compile_statement(statement, bytecode)?;
                }
                self.end_scope();
            }
            
            AstNode::IfStatement { test, consequent, alternate, .. } => {
                self.compile_if_statement(test, consequent, alternate.as_deref(), bytecode)?;
            }
            
            AstNode::WhileStatement { test, body, .. } => {
                self.compile_while_statement(test, body, bytecode)?;
            }
            
            AstNode::ForStatement { init, test, update, body, .. } => {
                self.compile_for_statement(init.as_deref(), test.as_deref(), update.as_deref(), body, bytecode)?;
            }
            
            AstNode::ReturnStatement { argument, .. } => {
                if let Some(arg) = argument {
                    self.compile_expression(arg, bytecode)?;
                } else {
                    let undefined_idx = bytecode.add_constant(Constant::Undefined);
                    bytecode.emit(Instruction::LoadConstant(undefined_idx));
                }
                bytecode.emit(Instruction::Return);
            }
            
            AstNode::BreakStatement { .. } => {
                if let Some(loop_info) = self.loop_stack.last_mut() {
                    let jump_idx = bytecode.emit(Instruction::Jump(0));
                    loop_info.break_jumps.push(jump_idx);
                } else {
                    return Err(CompileError::InvalidSyntax("break statement not in loop".to_string()));
                }
            }
            
            AstNode::ContinueStatement { .. } => {
                if let Some(loop_info) = self.loop_stack.last_mut() {
                    let jump_idx = bytecode.emit(Instruction::Jump(0));
                    loop_info.continue_jumps.push(jump_idx);
                } else {
                    return Err(CompileError::InvalidSyntax("continue statement not in loop".to_string()));
                }
            }
            
            AstNode::ThrowStatement { argument, .. } => {
                self.compile_expression(argument, bytecode)?;
                bytecode.emit(Instruction::Throw);
            }
            
            AstNode::TryStatement { block, handler, finalizer, .. } => {
                self.compile_try_statement(block, handler.as_deref(), finalizer.as_deref(), bytecode)?;
            }
            
            _ => {
                return Err(CompileError::UnsupportedFeature(
                    format!("Statement: {:?}", std::mem::discriminant(stmt))
                ));
            }
        }
        
        Ok(())
    }

    fn compile_expression(&mut self, expr: &AstNode, bytecode: &mut Bytecode) -> CompileResult<()> {
        match expr {
            AstNode::Identifier { name, .. } => {
                self.compile_identifier(name, bytecode)?;
            }
            
            AstNode::Literal { value, .. } => {
                let constant = match value {
                    LiteralValue::String(s) => Constant::String(s.clone()),
                    LiteralValue::Number(n) => Constant::Number(*n),
                    LiteralValue::Boolean(b) => Constant::Boolean(*b),
                    LiteralValue::Null => Constant::Null,
                    LiteralValue::Undefined => Constant::Undefined,
                    LiteralValue::RegExp { pattern, flags } => {
                        // treat regex as string
                        Constant::String(format!("/{}/{}", pattern, flags))
                    }
                };
                
                let idx = bytecode.add_constant(constant);
                bytecode.emit(Instruction::LoadConstant(idx));
            }
            
            AstNode::BinaryExpression { operator, left, right, .. } => {
                self.compile_expression(left, bytecode)?;
                self.compile_expression(right, bytecode)?;
                
                let instruction = match operator {
                    BinaryOperator::Add => Instruction::Add,
                    BinaryOperator::Sub => Instruction::Subtract,
                    BinaryOperator::Mul => Instruction::Multiply,
                    BinaryOperator::Div => Instruction::Divide,
                    BinaryOperator::Mod => Instruction::Modulo,
                    BinaryOperator::Pow => Instruction::Power,
                    BinaryOperator::Equal => Instruction::Equal,
                    BinaryOperator::NotEqual => Instruction::NotEqual,
                    BinaryOperator::StrictEqual => Instruction::StrictEqual,
                    BinaryOperator::StrictNotEqual => Instruction::StrictNotEqual,
                    BinaryOperator::Less => Instruction::Less,
                    BinaryOperator::Greater => Instruction::Greater,
                    BinaryOperator::LessEqual => Instruction::LessEqual,
                    BinaryOperator::GreaterEqual => Instruction::GreaterEqual,
                    BinaryOperator::LogicalAnd => Instruction::LogicalAnd,
                    BinaryOperator::LogicalOr => Instruction::LogicalOr,
                    BinaryOperator::BitwiseAnd => Instruction::BitwiseAnd,
                    BinaryOperator::BitwiseOr => Instruction::BitwiseOr,
                    BinaryOperator::BitwiseXor => Instruction::BitwiseXor,
                    BinaryOperator::LeftShift => Instruction::LeftShift,
                    BinaryOperator::RightShift => Instruction::RightShift,
                    BinaryOperator::UnsignedRightShift => Instruction::UnsignedRightShift,
                    _ => return Err(CompileError::UnsupportedFeature(format!("Binary operator: {:?}", operator))),
                };
                
                bytecode.emit(instruction);
            }
            
            AstNode::UnaryExpression { operator, argument, .. } => {
                self.compile_expression(argument, bytecode)?;
                
                let instruction = match operator {
                    UnaryOperator::Plus => Instruction::UnaryPlus,
                    UnaryOperator::Minus => Instruction::UnaryMinus,
                    UnaryOperator::Not => Instruction::LogicalNot,
                    UnaryOperator::BitwiseNot => Instruction::BitwiseNot,
                    UnaryOperator::TypeOf => Instruction::TypeOf,
                    _ => return Err(CompileError::UnsupportedFeature(format!("Unary operator: {:?}", operator))),
                };
                
                bytecode.emit(instruction);
            }
            
            AstNode::AssignmentExpression { left, right, operator, .. } => {
                match operator {
                    AssignmentOperator::Assign => {
                        self.compile_expression(right, bytecode)?;
                        self.compile_assignment_target(left, bytecode)?;
                    }
                    _ => {
                        // For compound assignments, load current value, perform operation, then store
                        self.compile_expression(left, bytecode)?;
                        self.compile_expression(right, bytecode)?;
                        
                        let op_instruction = match operator {
                            AssignmentOperator::AddAssign => Instruction::Add,
                            AssignmentOperator::SubAssign => Instruction::Subtract,
                            AssignmentOperator::MulAssign => Instruction::Multiply,
                            AssignmentOperator::DivAssign => Instruction::Divide,
                            AssignmentOperator::ModAssign => Instruction::Modulo,
                            AssignmentOperator::PowAssign => Instruction::Power,
                            _ => return Err(CompileError::UnsupportedFeature(format!("Assignment operator: {:?}", operator))),
                        };
                        
                        bytecode.emit(op_instruction);
                        self.compile_assignment_target(left, bytecode)?;
                    }
                }
            }
            
            AstNode::CallExpression { callee, arguments, .. } => {
                self.compile_expression(callee, bytecode)?;
                
                for arg in arguments {
                    self.compile_expression(arg, bytecode)?;
                }
                
                bytecode.emit(Instruction::Call(arguments.len()));
            }
            
            AstNode::MemberExpression { object, property, computed, .. } => {
                self.compile_expression(object, bytecode)?;
                
                if *computed {
                    self.compile_expression(property, bytecode)?;
                    bytecode.emit(Instruction::GetElement);
                } else {
                    self.compile_expression(property, bytecode)?;
                    bytecode.emit(Instruction::GetProperty);
                }
            }
            
            AstNode::ArrayExpression { elements, .. } => {
                let mut element_count = 0;
                
                for element in elements {
                    if let Some(elem) = element {
                        self.compile_expression(elem, bytecode)?;
                        element_count += 1;
                    } else {
                        let undefined_idx = bytecode.add_constant(Constant::Undefined);
                        bytecode.emit(Instruction::LoadConstant(undefined_idx));
                        element_count += 1;
                    }
                }
                
                bytecode.emit(Instruction::NewArray(element_count));
            }
            
            AstNode::ObjectExpression { properties, .. } => {
                bytecode.emit(Instruction::NewObject);
                
                for property in properties {
                    if let AstNode::Property { key, value, .. } = property {
                        bytecode.emit(Instruction::Duplicate); // Duplicate object reference
                        self.compile_expression(key, bytecode)?;
                        self.compile_expression(value, bytecode)?;
                        bytecode.emit(Instruction::SetProperty);
                    }
                }
            }
            
            AstNode::FunctionExpression { id, params, body, is_async, is_generator, .. } => {
                self.compile_function_expression(id.as_deref(), params, body, *is_async, *is_generator, bytecode)?;
            }
            
            AstNode::ConditionalExpression { test, consequent, alternate, .. } => {
                self.compile_expression(test, bytecode)?;
                
                let else_jump = bytecode.emit(Instruction::JumpIfFalse(0));
                self.compile_expression(consequent, bytecode)?;
                let end_jump = bytecode.emit(Instruction::Jump(0));
                
                let else_target = bytecode.len();
                bytecode.patch_jump(else_jump, else_target);
                self.compile_expression(alternate, bytecode)?;
                
                let end_target = bytecode.len();
                bytecode.patch_jump(end_jump, end_target);
            }
            
            _ => {
                return Err(CompileError::UnsupportedFeature(
                    format!("Expression: {:?}", std::mem::discriminant(expr))
                ));
            }
        }
        
        Ok(())
    }

    fn compile_identifier(&mut self, name: &str, bytecode: &mut Bytecode) -> CompileResult<()> {
        if let Some(var) = self.resolve_variable(name) {
            if var.index < 256 {
                bytecode.emit(Instruction::LoadLocal(var.index));
            } else {
                return Err(CompileError::InternalError("Too many local variables".to_string()));
            }
        } else {
            let name_idx = bytecode.add_name(name.to_string());
            bytecode.emit(Instruction::LoadGlobal(name_idx));
        }
        
        Ok(())
    }

    fn compile_assignment_target(&mut self, target: &AstNode, bytecode: &mut Bytecode) -> CompileResult<()> {
        match target {
            AstNode::Identifier { name, .. } => {
                if let Some(var) = self.resolve_variable(name) {
                    bytecode.emit(Instruction::StoreLocal(var.index));
                } else {
                    let name_idx = bytecode.add_name(name.to_string());
                    bytecode.emit(Instruction::StoreGlobal(name_idx));
                }
            }
            AstNode::MemberExpression { object, property, computed, .. } => {
                self.compile_expression(object, bytecode)?;
                self.compile_expression(property, bytecode)?;
                
                if *computed {
                    bytecode.emit(Instruction::SetElement);
                } else {
                    bytecode.emit(Instruction::SetProperty);
                }
            }
            _ => {
                return Err(CompileError::InvalidSyntax("Invalid assignment target".to_string()));
            }
        }
        
        Ok(())
    }

    fn compile_variable_declarator(&mut self, decl: &AstNode, kind: &VarKind, bytecode: &mut Bytecode) -> CompileResult<()> {
        if let AstNode::VariableDeclarator { id, init, .. } = decl {
            if let AstNode::Identifier { name, .. } = id.as_ref() {
                // Compile initializer if present
                if let Some(init_expr) = init {
                    self.compile_expression(init_expr, bytecode)?;
                } else {
                    let undefined_idx = bytecode.add_constant(Constant::Undefined);
                    bytecode.emit(Instruction::LoadConstant(undefined_idx));
                }
                
                // Declare variable
                let var_index = self.declare_variable(name, kind.clone())?;
                
                let instruction = match kind {
                    VarKind::Var => Instruction::DeclareVar(var_index),
                    VarKind::Let => Instruction::DeclareLet(var_index),
                    VarKind::Const => Instruction::DeclareConst(var_index),
                };
                
                bytecode.emit(instruction);
            }
        }
        
        Ok(())
    }

    fn compile_function_declaration(
        &mut self,
        id: &Option<Box<AstNode>>,
        params: &[AstNode],
        body: &AstNode,
        is_async: bool,
        is_generator: bool,
        bytecode: &mut Bytecode,
    ) -> CompileResult<()> {
        let name = if let Some(id_node) = id {
            if let AstNode::Identifier { name, .. } = id_node.as_ref() {
                Some(name.clone())
            } else {
                None
            }
        } else {
            None
        };
        
        let function_bytecode = self.compile_function_body(params, body, is_async, is_generator)?;
        
        let constant = Constant::Function {
            name: name.clone(),
            param_count: params.len(),
            bytecode: function_bytecode,
            is_async,
            is_generator,
        };
        
        let const_idx = bytecode.add_constant(constant);
        bytecode.emit(Instruction::LoadConstant(const_idx));
        
        if let Some(func_name) = name {
            let name_idx = bytecode.add_name(func_name.clone());
            bytecode.emit(Instruction::StoreGlobal(name_idx));
            self.declare_variable(&func_name, VarKind::Var)?;
        }
        
        Ok(())
    }

    fn compile_function_expression(
        &mut self,
        id: Option<&AstNode>,
        params: &[AstNode],
        body: &AstNode,
        is_async: bool,
        is_generator: bool,
        bytecode: &mut Bytecode,
    ) -> CompileResult<()> {
        let name = if let Some(id_node) = id {
            if let AstNode::Identifier { name, .. } = id_node {
                Some(name.clone())
            } else {
                None
            }
        } else {
            None
        };
        
        let function_bytecode = self.compile_function_body(params, body, is_async, is_generator)?;
        
        let constant = Constant::Function {
            name,
            param_count: params.len(),
            bytecode: function_bytecode,
            is_async,
            is_generator,
        };
        
        let const_idx = bytecode.add_constant(constant);
        bytecode.emit(Instruction::LoadConstant(const_idx));
        
        Ok(())
    }

    fn compile_function_body(
        &mut self,
        params: &[AstNode],
        body: &AstNode,
        _is_async: bool,
        _is_generator: bool,
    ) -> CompileResult<Bytecode> {
        self.function_depth += 1;
        self.begin_scope();
        
        let mut function_bytecode = Bytecode::new();
        
        // Declare parameters as local variables
        for param in params {
            if let AstNode::Identifier { name, .. } = param {
                self.declare_variable(name, VarKind::Var)?;
            }
        }
        
        // Compile function body
        self.compile_statement(body, &mut function_bytecode)?;
        
        // Ensure function returns undefined if no explicit return
        let undefined_idx = function_bytecode.add_constant(Constant::Undefined);
        function_bytecode.emit(Instruction::LoadConstant(undefined_idx));
        function_bytecode.emit(Instruction::Return);
        
        self.end_scope();
        self.function_depth -= 1;
        
        Ok(function_bytecode)
    }

    fn compile_if_statement(
        &mut self,
        test: &AstNode,
        consequent: &AstNode,
        alternate: Option<&AstNode>,
        bytecode: &mut Bytecode,
    ) -> CompileResult<()> {
        self.compile_expression(test, bytecode)?;
        
        let else_jump = bytecode.emit(Instruction::JumpIfFalse(0));
        self.compile_statement(consequent, bytecode)?;
        
        if let Some(alternate_stmt) = alternate {
            let end_jump = bytecode.emit(Instruction::Jump(0));
            let else_target = bytecode.len();
            bytecode.patch_jump(else_jump, else_target);
            
            self.compile_statement(alternate_stmt, bytecode)?;
            
            let end_target = bytecode.len();
            bytecode.patch_jump(end_jump, end_target);
        } else {
            let end_target = bytecode.len();
            bytecode.patch_jump(else_jump, end_target);
        }
        
        Ok(())
    }

    fn compile_while_statement(&mut self, test: &AstNode, body: &AstNode, bytecode: &mut Bytecode) -> CompileResult<()> {
        let loop_start = bytecode.len();
        
        self.loop_stack.push(LoopInfo {
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
        });
        
        self.compile_expression(test, bytecode)?;
        let exit_jump = bytecode.emit(Instruction::JumpIfFalse(0));
        
        self.compile_statement(body, bytecode)?;
        
        // Continue target
        let continue_target = bytecode.len();
        bytecode.emit(Instruction::Jump(loop_start as isize - bytecode.len() as isize - 1));
        
        // Break target
        let break_target = bytecode.len();
        bytecode.patch_jump(exit_jump, break_target);
        
        // Patch all break and continue jumps
        if let Some(loop_info) = self.loop_stack.pop() {
            for jump in loop_info.break_jumps {
                bytecode.patch_jump(jump, break_target);
            }
            for jump in loop_info.continue_jumps {
                bytecode.patch_jump(jump, continue_target);
            }
        }
        
        Ok(())
    }

    fn compile_for_statement(
        &mut self,
        init: Option<&AstNode>,
        test: Option<&AstNode>,
        update: Option<&AstNode>,
        body: &AstNode,
        bytecode: &mut Bytecode,
    ) -> CompileResult<()> {
        self.begin_scope();
        
        // Compile initializer
        if let Some(init_stmt) = init {
            self.compile_statement(init_stmt, bytecode)?;
        }
        
        let loop_start = bytecode.len();
        
        self.loop_stack.push(LoopInfo {
            break_jumps: Vec::new(),
            continue_jumps: Vec::new(),
        });
        
        // Compile test condition
        let exit_jump = if let Some(test_expr) = test {
            self.compile_expression(test_expr, bytecode)?;
            Some(bytecode.emit(Instruction::JumpIfFalse(0)))
        } else {
            None
        };
        
        // Compile body
        self.compile_statement(body, bytecode)?;
        
        // Continue target (where update expression runs)
        let continue_target = bytecode.len();
        
        // Compile update expression
        if let Some(update_expr) = update {
            self.compile_expression(update_expr, bytecode)?;
            bytecode.emit(Instruction::Pop); // Discard update result
        }
        
        // Jump back to loop start
        bytecode.emit(Instruction::Jump(loop_start as isize - bytecode.len() as isize - 1));
        
        // Break target
        let break_target = bytecode.len();
        
        // Patch exit jump if present
        if let Some(jump) = exit_jump {
            bytecode.patch_jump(jump, break_target);
        }
        
        // Patch all break and continue jumps
        if let Some(loop_info) = self.loop_stack.pop() {
            for jump in loop_info.break_jumps {
                bytecode.patch_jump(jump, break_target);
            }
            for jump in loop_info.continue_jumps {
                bytecode.patch_jump(jump, continue_target);
            }
        }
        
        self.end_scope();
        
        Ok(())
    }

    fn compile_try_statement(
        &mut self,
        block: &AstNode,
        handler: Option<&AstNode>,
        finalizer: Option<&AstNode>,
        bytecode: &mut Bytecode,
    ) -> CompileResult<()> {
        let try_begin = bytecode.emit(Instruction::TryBegin(0));
        
        self.compile_statement(block, bytecode)?;
        
        bytecode.emit(Instruction::TryEnd);
        
        let try_end_jump = bytecode.emit(Instruction::Jump(0));
        
        // Catch handler
        let catch_start = bytecode.len();
        bytecode.patch_jump(try_begin, catch_start);
        
        if let Some(catch_clause) = handler {
            if let AstNode::CatchClause { param, body, .. } = catch_clause {
                bytecode.emit(Instruction::CatchBegin);
                
                // Bind exception to parameter if present
                if let Some(param_node) = param {
                    if let AstNode::Identifier { name, .. } = param_node.as_ref() {
                        let var_index = self.declare_variable(name, VarKind::Let)?;
                        bytecode.emit(Instruction::StoreLocal(var_index));
                    }
                }
                
                self.compile_statement(body, bytecode)?;
                
                bytecode.emit(Instruction::CatchEnd);
            }
        }
        
        let catch_end = bytecode.len();
        bytecode.patch_jump(try_end_jump, catch_end);
        
        // Finally block
        if let Some(finally_stmt) = finalizer {
            bytecode.emit(Instruction::FinallyBegin);
            self.compile_statement(finally_stmt, bytecode)?;
            bytecode.emit(Instruction::FinallyEnd);
        }
        
        Ok(())
    }

    // Scope management
    
    fn begin_scope(&mut self) {
        let depth = self.scopes.last().map(|s| s.depth + 1).unwrap_or(0);
        self.scopes.push(Scope {
            variables: HashMap::new(),
            depth,
        });
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn declare_variable(&mut self, name: &str, kind: VarKind) -> CompileResult<usize> {
        if let Some(scope) = self.scopes.last_mut() {
            let index = scope.variables.len();
            let variable = Variable {
                index,
                kind,
                is_captured: false,
            };
            scope.variables.insert(name.to_string(), variable);
            Ok(index)
        } else {
            Err(CompileError::InternalError("No scope available".to_string()))
        }
    }

    fn resolve_variable(&self, name: &str) -> Option<&Variable> {
        for scope in self.scopes.iter().rev() {
            if let Some(var) = scope.variables.get(name) {
                return Some(var);
            }
        }
        None
    }
}
