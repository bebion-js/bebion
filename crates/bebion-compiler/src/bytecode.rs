//! Bytecode definitions and operations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
    // Stack operations
    LoadConstant(usize),    // Load constant from constant pool
    LoadGlobal(usize),      // Load global variable
    StoreGlobal(usize),     // Store to global variable
    LoadLocal(usize),       // Load local variable
    StoreLocal(usize),      // Store to local variable
    
    // Arithmetic operations
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Power,
    
    // Comparison operations
    Equal,
    NotEqual,
    StrictEqual,
    StrictNotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    
    // Logical operations
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    
    // Bitwise operations
    BitwiseAnd,
    BitwiseOr,
    BitwiseXor,
    BitwiseNot,
    LeftShift,
    RightShift,
    UnsignedRightShift,
    
    // Unary operations
    UnaryPlus,
    UnaryMinus,
    TypeOf,
    
    // Control flow
    Jump(isize),            // Unconditional jump
    JumpIfFalse(isize),     // Jump if top of stack is falsy
    JumpIfTrue(isize),      // Jump if top of stack is truthy
    
    // Function operations
    Call(usize),            // Call function with n arguments
    Return,                 // Return from function
    
    // Object operations
    NewObject,              // Create new object
    GetProperty,            // Get property from object
    SetProperty,            // Set property on object
    GetElement,             // Get array element
    SetElement,             // Set array element
    
    // Array operations
    NewArray(usize),        // Create new array with n elements
    
    // Variable operations
    DeclareVar(usize),      // Declare variable
    DeclareLet(usize),      // Declare let variable
    DeclareConst(usize),    // Declare const variable
    
    // Stack manipulation
    Pop,                    // Remove top of stack
    Duplicate,              // Duplicate top of stack
    Swap,                   // Swap top two stack items
    
    // Special operations
    Nop,                    // No operation
    Halt,                   // Stop execution
    
    // Async operations
    Await,                  // Await async operation
    
    // Exception handling
    Throw,                  // Throw exception
    TryBegin(usize),        // Begin try block
    TryEnd,                 // End try block
    CatchBegin,             // Begin catch block
    CatchEnd,               // End catch block
    FinallyBegin,           // Begin finally block
    FinallyEnd,             // End finally block
    
    // Module operations
    Import(usize),          // Import module
    Export(usize),          // Export value
    
    // Debug operations
    DebugInfo(usize, usize), // Line and column info
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Constant {
    Number(f64),
    String(String),
    Boolean(bool),
    Null,
    Undefined,
    Function {
        name: Option<String>,
        param_count: usize,
        bytecode: Bytecode,
        is_async: bool,
        is_generator: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub constants: Vec<Constant>,
    pub names: Vec<String>,        // Variable/property names
    pub source_map: HashMap<usize, (usize, usize)>, // instruction index -> (line, column)
}

impl Bytecode {
    pub fn new() -> Self {
        Self {
            instructions: Vec::new(),
            constants: Vec::new(),
            names: Vec::new(),
            source_map: HashMap::new(),
        }
    }

    pub fn emit(&mut self, instruction: Instruction) -> usize {
        let index = self.instructions.len();
        self.instructions.push(instruction);
        index
    }

    pub fn emit_at(&mut self, index: usize, instruction: Instruction) {
        if index < self.instructions.len() {
            self.instructions[index] = instruction;
        }
    }

    pub fn add_constant(&mut self, constant: Constant) -> usize {
        let index = self.constants.len();
        self.constants.push(constant);
        index
    }

    pub fn add_name(&mut self, name: String) -> usize {
        if let Some(index) = self.names.iter().position(|n| n == &name) {
            index
        } else {
            let index = self.names.len();
            self.names.push(name);
            index
        }
    }

    pub fn add_source_location(&mut self, instruction_index: usize, line: usize, column: usize) {
        self.source_map.insert(instruction_index, (line, column));
    }

    pub fn len(&self) -> usize {
        self.instructions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.instructions.is_empty()
    }

    pub fn patch_jump(&mut self, jump_index: usize, target_index: usize) {
        let offset = target_index as isize - jump_index as isize - 1;
        match &mut self.instructions[jump_index] {
            Instruction::Jump(ref mut offset_ref) |
            Instruction::JumpIfFalse(ref mut offset_ref) |
            Instruction::JumpIfTrue(ref mut offset_ref) => {
                *offset_ref = offset;
            }
            _ => panic!("Attempted to patch non-jump instruction"),
        }
    }

    pub fn optimize(&mut self) {
        // Simple peephole optimizations
        let mut i = 0;
        while i < self.instructions.len() {
            match self.instructions.get(i..i + 2) {
                // Remove redundant load/pop sequences
                Some([Instruction::LoadConstant(_), Instruction::Pop]) => {
                    self.instructions.drain(i..i + 2);
                    continue;
                }
                // Convert load constant + return to direct return constant
                Some([Instruction::LoadConstant(idx), Instruction::Return]) => {
                    let idx = *idx;
                    self.instructions[i] = Instruction::LoadConstant(idx);
                    self.instructions[i + 1] = Instruction::Return;
                }
                _ => {}
            }
            i += 1;
        }
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self::new()
    }
}