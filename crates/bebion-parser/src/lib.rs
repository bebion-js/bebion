//! Bebion JavaScript Parser
//! 
//! ECMAScript 2024 compliant parser with full AST generation.

pub mod ast;
pub mod lexer;
pub mod parser;

pub use parser::Parser;
pub use ast::{AstNode, Program};

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParseError {
    UnexpectedToken {
        expected: String,
        found: String,
        line: usize,
        column: usize,
    },
    SyntaxError {
        message: String,
        line: usize,
        column: usize,
    },
    LexicalError {
        message: String,
        line: usize,
        column: usize,
    },
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::UnexpectedToken { expected, found, line, column } => {
                write!(f, "Unexpected token '{}' at {}:{}, expected '{}'", found, line, column, expected)
            }
            ParseError::SyntaxError { message, line, column } => {
                write!(f, "Syntax error at {}:{}: {}", line, column, message)
            }
            ParseError::LexicalError { message, line, column } => {
                write!(f, "Lexical error at {}:{}: {}", line, column, message)
            }
        }
    }
}

impl std::error::Error for ParseError {}

pub type ParseResult<T> = Result<T, ParseError>;