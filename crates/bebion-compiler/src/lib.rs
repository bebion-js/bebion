//! Bebion Bytecode Compiler
//! 
//! Compiles JavaScript AST to bytecode for execution.

pub mod bytecode;
pub mod compiler;

pub use compiler::Compiler;
pub use bytecode::{Instruction, Bytecode};

use std::fmt;

#[derive(Debug, Clone)]
pub enum CompileError {
    UnsupportedFeature(String),
    InternalError(String),
    InvalidSyntax(String),
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompileError::UnsupportedFeature(feature) => {
                write!(f, "Unsupported feature: {}", feature)
            }
            CompileError::InternalError(msg) => {
                write!(f, "Internal compiler error: {}", msg)
            }
            CompileError::InvalidSyntax(msg) => {
                write!(f, "Invalid syntax: {}", msg)
            }
        }
    }
}

impl std::error::Error for CompileError {}

pub type CompileResult<T> = Result<T, CompileError>;