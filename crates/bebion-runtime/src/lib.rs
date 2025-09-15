//! Bebion Runtime Engine
//! 
//! Executes bytecode with async/await support and event loop integration.

pub mod event_loop;
pub mod runtime;
pub mod vm;
pub mod value;

pub use event_loop::EventLoop;
pub use runtime::Runtime;
pub use vm::VirtualMachine;
pub use value::Value;

use std::fmt;

#[derive(Debug, Clone)]
pub enum RuntimeError {
    TypeError(String),
    ReferenceError(String),
    SyntaxError(String),
    RangeError(String),
    StackOverflow,
    OutOfMemory,
    InvalidBytecode(String),
    InvalidOperation(String),
    AsyncError(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::TypeError(msg) => write!(f, "TypeError: {}", msg),
            RuntimeError::ReferenceError(msg) => write!(f, "ReferenceError: {}", msg),
            RuntimeError::SyntaxError(msg) => write!(f, "SyntaxError: {}", msg),
            RuntimeError::RangeError(msg) => write!(f, "RangeError: {}", msg),
            RuntimeError::StackOverflow => write!(f, "RangeError: Maximum call stack size exceeded"),
            RuntimeError::OutOfMemory => write!(f, "RangeError: Out of memory"),
            RuntimeError::InvalidBytecode(msg) => write!(f, "Internal Error: Invalid bytecode - {}", msg),
            RuntimeError::InvalidOperation(msg) => write!(f, "Internal Error: Invalid operation - {}", msg),
            RuntimeError::AsyncError(msg) => write!(f, "Async Error: {}", msg),
        }
    }
}

impl std::error::Error for RuntimeError {}

pub type RuntimeResult<T> = Result<T, RuntimeError>;