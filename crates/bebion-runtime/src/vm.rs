//! Virtual machine for executing bytecode

use crate::{RuntimeError, RuntimeResult, Value};
use bebion_compiler::bytecode::{Bytecode, Constant, Instruction};
use bebion_gc::{GarbageCollector, GcHandle, GcObjectType};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{debug, trace};

pub struct VirtualMachine {
    gc: Arc<Mutex<GarbageCollector>>,
    stack: Vec<Value>,
    call_stack: Vec<CallFrame>,
    globals: HashMap<String, Value>,
    max_stack_size: usize,
    max_call_depth: usize,
}

#[derive(Debug, Clone)]
struct CallFrame {
    bytecode: Arc<Bytecode>,
    pc: usize, // Program counter
    locals: Vec<Value>,
    base_stack_offset: usize,
}

impl VirtualMachine {
    pub fn new(gc: Arc<Mutex<GarbageCollector>>) -> Self {
        Self {
            gc,
            stack: Vec::with_capacity(1024),
            call_stack: Vec::with_capacity(256),
            globals: HashMap::new(),
            max_stack_size: 10000,
            max_call_depth: 1000,
        }
    }

    pub fn execute(&mut self, bytecode: &Bytecode) -> RuntimeResult<Value> {
        debug!("Executing bytecode with {} instructions", bytecode.len());
        
        let frame = CallFrame {
            bytecode: Arc::new(bytecode.clone()),
            pc: 0,
            locals: Vec::new(),
            base_stack_offset: self.stack.len(),
        };
        
        self.call_stack.push(frame);
        
        let result = self.run_interpreter_loop();
        
        // Clean up call stack
        self.call_stack.pop();
        
        result
    }

    fn run_interpreter_loop(&mut self) -> RuntimeResult<Value> {
        loop {
            let frame = self.call_stack.last_mut()
                .ok_or_else(|| RuntimeError::InvalidOperation("No call frame".to_string()))?;
            
            if frame.pc >= frame.bytecode.instructions.len() {
                // End of bytecode reached
                return Ok(self.stack.pop().unwrap_or(Value::Undefined));
            }
            
            let instruction = &frame.bytecode.instructions[frame.pc];
            trace!("PC: {}, Instruction: {:?}", frame.pc, instruction);
            
            match instruction {
                Instruction::LoadConstant(idx) => {
                    let constant = frame.bytecode.constants.get(*idx)
                        .ok_or_else(|| RuntimeError::InvalidBytecode(format!("Invalid constant index: {}", idx)))?;
                    
                    let value = self.constant_to_value(constant)?;
                    self.push_stack(value)?;
                    frame.pc += 1;
                }
                
                Instruction::LoadGlobal(idx) => {
                    let name = frame.bytecode.names.get(*idx)
                        .ok_or_else(|| RuntimeError::InvalidBytecode(format!("Invalid name index: {}", idx)))?;
                    
                    let value = self.globals.get(name).cloned().unwrap_or(Value::Undefined);
                    self.push_stack(value)?;
                    frame.pc += 1;
                }
                
                Instruction::StoreGlobal(idx) => {
                    let name = frame.bytecode.names.get(*idx)
                        .ok_or_else(|| RuntimeError::InvalidBytecode(format!("Invalid name index: {}", idx)))?;
                    
                    let value = self.pop_stack()?;
                    self.globals.insert(name.clone(), value);
                    frame.pc += 1;
                }
                
                Instruction::LoadLocal(idx) => {
                    let value = frame.locals.get(*idx).cloned().unwrap_or(Value::Undefined);
                    self.push_stack(value)?;
                    frame.pc += 1;
                }
                
                Instruction::StoreLocal(idx) => {
                    let value = self.pop_stack()?;
                    
                    // Extend locals vector if necessary
                    while frame.locals.len() <= *idx {
                        frame.locals.push(Value::Undefined);
                    }
                    
                    frame.locals[*idx] = value;
                    frame.pc += 1;
                }
                
                // Arithmetic operations
                Instruction::Add => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let result = crate::value::add_values(&left, &right)?;
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                Instruction::Subtract => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let result = crate::value::subtract_values(&left, &right)?;
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                Instruction::Multiply => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let result = crate::value::multiply_values(&left, &right)?;
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                Instruction::Divide => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let result = crate::value::divide_values(&left, &right)?;
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                Instruction::Modulo => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let result = crate::value::modulo_values(&left, &right)?;
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                Instruction::Power => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let result = crate::value::power_values(&left, &right)?;
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                // Comparison operations
                Instruction::Equal => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let result = Value::Boolean(left.loose_equals(&right));
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                Instruction::StrictEqual => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let result = Value::Boolean(left.strict_equals(&right));
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                Instruction::Less => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let left_num = left.to_number()?;
                    let right_num = right.to_number()?;
                    let result = Value::Boolean(left_num < right_num);
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                Instruction::Greater => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let left_num = left.to_number()?;
                    let right_num = right.to_number()?;
                    let result = Value::Boolean(left_num > right_num);
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                Instruction::LessEqual => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let left_num = left.to_number()?;
                    let right_num = right.to_number()?;
                    let result = Value::Boolean(left_num <= right_num);
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                Instruction::GreaterEqual => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let left_num = left.to_number()?;
                    let right_num = right.to_number()?;
                    let result = Value::Boolean(left_num >= right_num);
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                // Logical operations
                Instruction::LogicalAnd => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let result = if left.to_boolean() { right } else { left };
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                Instruction::LogicalOr => {
                    let right = self.pop_stack()?;
                    let left = self.pop_stack()?;
                    let result = if left.to_boolean() { left } else { right };
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                Instruction::LogicalNot => {
                    let value = self.pop_stack()?;
                    let result = Value::Boolean(!value.to_boolean());
                    self.push_stack(result)?;
                    frame.pc += 1;
                }
                
                // Control flow
                Instruction::Jump(offset) => {
                    frame.pc = ((frame.pc as isize) + offset + 1) as usize;
                }
                
                Instruction::JumpIfFalse(offset) => {
                    let condition = self.pop_stack()?;
                    if !condition.to_boolean() {
                        frame.pc = ((frame.pc as isize) + offset + 1) as usize;
                    } else {
                        frame.pc += 1;
                    }
                }
                
                Instruction::JumpIfTrue(offset) => {
                    let condition = self.pop_stack()?;
                    if condition.to_boolean() {
                        frame.pc = ((frame.pc as isize) + offset + 1) as usize;
                    } else {
                        frame.pc += 1;
                    }
                }
                
                Instruction::Call(arg_count) => {
                    self.handle_function_call(*arg_count)?;
                    // PC will be managed by the new call frame
                }
                
                Instruction::Return => {
                    let return_value = self.pop_stack().unwrap_or(Value::Undefined);
                    
                    // Clean up the current frame's stack space
                    let frame = self.call_stack.pop().unwrap();
                    self.stack.truncate(frame.base_stack_offset);
                    
                    // Push return value
                    if !self.call_stack.is_empty() {
                        self.push_stack(return_value)?;
                        // Continue execution in the calling frame
                        if let Some(caller_frame) = self.call_stack.last_mut() {
                            caller_frame.pc += 1;
                        }
                    } else {
                        // Main function returned
                        return Ok(return_value);
                    }
                }
                
                Instruction::NewObject => {
                    let handle = {
                        let mut gc = self.gc.lock().unwrap();
                        gc.allocate_object(HashMap::new())
                    };
                    self.push_stack(Value::Object(handle))?;
                    frame.pc += 1;
                }
                
                Instruction::NewArray(size) => {
                    let mut elements = Vec::with_capacity(*size);
                    for _ in 0..*size {
                        if let Value::Object(handle) = self.pop_stack()? {
                            elements.push(handle);
                        } else {
                            return Err(RuntimeError::TypeError("Array elements must be objects".to_string()));
                        }
                    }
                    elements.reverse(); // Stack is LIFO
                    
                    let handle = {
                        let mut gc = self.gc.lock().unwrap();
                        gc.allocate_array(elements)
                    };
                    self.push_stack(Value::Object(handle))?;
                    frame.pc += 1;
                }
                
                Instruction::Pop => {
                    self.pop_stack()?;
                    frame.pc += 1;
                }
                
                Instruction::Duplicate => {
                    let value = self.peek_stack(0)?;
                    self.push_stack(value)?;
                    frame.pc += 1;
                }
                
                Instruction::Halt => {
                    return Ok(self.stack.pop().unwrap_or(Value::Undefined));
                }
                
                _ => {
                    return Err(RuntimeError::InvalidBytecode(
                        format!("Unimplemented instruction: {:?}", instruction)
                    ));
                }
            }
            
            // Check for stack overflow
            if self.stack.len() > self.max_stack_size {
                return Err(RuntimeError::StackOverflow);
            }
            
            // Check for call depth overflow
            if self.call_stack.len() > self.max_call_depth {
                return Err(RuntimeError::StackOverflow);
            }
        }
    }

    fn constant_to_value(&mut self, constant: &Constant) -> RuntimeResult<Value> {
        match constant {
            Constant::Number(n) => Ok(Value::Number(*n)),
            Constant::String(s) => Ok(Value::String(s.clone())),
            Constant::Boolean(b) => Ok(Value::Boolean(*b)),
            Constant::Null => Ok(Value::Null),
            Constant::Undefined => Ok(Value::Undefined),
            Constant::Function { name, bytecode, .. } => {
                let handle = {
                    let mut gc = self.gc.lock().unwrap();
                    gc.allocate_function(
                        name.clone(),
                        vec![], // Simplified for now
                        HashMap::new(),
                    )
                };
                Ok(Value::Object(handle))
            }
        }
    }

    fn handle_function_call(&mut self, arg_count: usize) -> RuntimeResult<()> {
        // Pop arguments from stack
        let mut args = Vec::with_capacity(arg_count);
        for _ in 0..arg_count {
            args.push(self.pop_stack()?);
        }
        args.reverse(); // Stack is LIFO
        
        // Pop function from stack
        let function = self.pop_stack()?;
        
        match function {
            Value::Object(handle) => {
                // Check if it's a function object
                let gc = self.gc.lock().unwrap();
                if let Some(GcObjectType::Function { bytecode, .. }) = gc.get_object_type(handle) {
                    // Create new call frame for function execution
                    // This is simplified - a full implementation would handle closures, 'this', etc.
                    return Err(RuntimeError::InvalidOperation("Function calls not fully implemented".to_string()));
                } else {
                    return Err(RuntimeError::TypeError("Not a function".to_string()));
                }
            }
            _ => {
                return Err(RuntimeError::TypeError("Not a function".to_string()));
            }
        }
    }

    fn push_stack(&mut self, value: Value) -> RuntimeResult<()> {
        if self.stack.len() >= self.max_stack_size {
            Err(RuntimeError::StackOverflow)
        } else {
            self.stack.push(value);
            Ok(())
        }
    }

    fn pop_stack(&mut self) -> RuntimeResult<Value> {
        self.stack.pop().ok_or_else(|| {
            RuntimeError::InvalidOperation("Stack underflow".to_string())
        })
    }

    fn peek_stack(&self, offset: usize) -> RuntimeResult<Value> {
        let index = self.stack.len().saturating_sub(offset + 1);
        self.stack.get(index).cloned().ok_or_else(|| {
            RuntimeError::InvalidOperation("Stack underflow".to_string())
        })
    }

    pub fn get_global(&self, name: &str) -> Option<&Value> {
        self.globals.get(name)
    }

    pub fn set_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }

    pub fn stack_size(&self) -> usize {
        self.stack.len()
    }

    pub fn call_depth(&self) -> usize {
        self.call_stack.len()
    }
}