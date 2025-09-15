//! Abstract Syntax Tree definitions for JavaScript

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub body: Vec<AstNode>,
    pub source_type: SourceType,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SourceType {
    Script,
    Module,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SourceLocation {
    pub start: Location,
    pub end: Location,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AstNode {
    Program(Program),
    
    // Statements
    ExpressionStatement { expression: Box<AstNode>, loc: Option<SourceLocation> },
    BlockStatement { body: Vec<AstNode>, loc: Option<SourceLocation> },
    VariableDeclaration { declarations: Vec<AstNode>, kind: VarKind, loc: Option<SourceLocation> },
    FunctionDeclaration { 
        id: Option<Box<AstNode>>, 
        params: Vec<AstNode>, 
        body: Box<AstNode>,
        is_async: bool,
        is_generator: bool,
        loc: Option<SourceLocation> 
    },
    ReturnStatement { argument: Option<Box<AstNode>>, loc: Option<SourceLocation> },
    IfStatement { 
        test: Box<AstNode>, 
        consequent: Box<AstNode>, 
        alternate: Option<Box<AstNode>>, 
        loc: Option<SourceLocation> 
    },
    WhileStatement { test: Box<AstNode>, body: Box<AstNode>, loc: Option<SourceLocation> },
    ForStatement { 
        init: Option<Box<AstNode>>, 
        test: Option<Box<AstNode>>, 
        update: Option<Box<AstNode>>, 
        body: Box<AstNode>, 
        loc: Option<SourceLocation> 
    },
    BreakStatement { label: Option<Box<AstNode>>, loc: Option<SourceLocation> },
    ContinueStatement { label: Option<Box<AstNode>>, loc: Option<SourceLocation> },
    ThrowStatement { argument: Box<AstNode>, loc: Option<SourceLocation> },
    TryStatement { 
        block: Box<AstNode>, 
        handler: Option<Box<AstNode>>, 
        finalizer: Option<Box<AstNode>>, 
        loc: Option<SourceLocation> 
    },
    
    // Expressions
    Identifier { name: String, loc: Option<SourceLocation> },
    Literal { value: LiteralValue, raw: String, loc: Option<SourceLocation> },
    ArrayExpression { elements: Vec<Option<AstNode>>, loc: Option<SourceLocation> },
    ObjectExpression { properties: Vec<AstNode>, loc: Option<SourceLocation> },
    FunctionExpression { 
        id: Option<Box<AstNode>>, 
        params: Vec<AstNode>, 
        body: Box<AstNode>,
        is_async: bool,
        is_generator: bool,
        loc: Option<SourceLocation> 
    },
    ArrowFunctionExpression { 
        params: Vec<AstNode>, 
        body: Box<AstNode>,
        is_async: bool,
        loc: Option<SourceLocation> 
    },
    CallExpression { 
        callee: Box<AstNode>, 
        arguments: Vec<AstNode>, 
        loc: Option<SourceLocation> 
    },
    MemberExpression { 
        object: Box<AstNode>, 
        property: Box<AstNode>, 
        computed: bool, 
        loc: Option<SourceLocation> 
    },
    BinaryExpression { 
        operator: BinaryOperator, 
        left: Box<AstNode>, 
        right: Box<AstNode>, 
        loc: Option<SourceLocation> 
    },
    UnaryExpression { 
        operator: UnaryOperator, 
        argument: Box<AstNode>, 
        prefix: bool, 
        loc: Option<SourceLocation> 
    },
    AssignmentExpression { 
        operator: AssignmentOperator, 
        left: Box<AstNode>, 
        right: Box<AstNode>, 
        loc: Option<SourceLocation> 
    },
    UpdateExpression { 
        operator: UpdateOperator, 
        argument: Box<AstNode>, 
        prefix: bool, 
        loc: Option<SourceLocation> 
    },
    ConditionalExpression { 
        test: Box<AstNode>, 
        consequent: Box<AstNode>, 
        alternate: Box<AstNode>, 
        loc: Option<SourceLocation> 
    },
    
    // ES2015+ Features
    TemplateLiteral { 
        quasis: Vec<AstNode>, 
        expressions: Vec<AstNode>, 
        loc: Option<SourceLocation> 
    },
    ClassDeclaration { 
        id: Option<Box<AstNode>>, 
        superclass: Option<Box<AstNode>>, 
        body: Box<AstNode>, 
        loc: Option<SourceLocation> 
    },
    ImportDeclaration { 
        specifiers: Vec<AstNode>, 
        source: Box<AstNode>, 
        loc: Option<SourceLocation> 
    },
    ExportDeclaration { 
        declaration: Option<Box<AstNode>>, 
        specifiers: Vec<AstNode>, 
        source: Option<Box<AstNode>>, 
        loc: Option<SourceLocation> 
    },
    
    // Async/Await
    AwaitExpression { argument: Box<AstNode>, loc: Option<SourceLocation> },
    
    // Other nodes
    VariableDeclarator { 
        id: Box<AstNode>, 
        init: Option<Box<AstNode>>, 
        loc: Option<SourceLocation> 
    },
    Property { 
        key: Box<AstNode>, 
        value: Box<AstNode>, 
        kind: PropertyKind, 
        method: bool, 
        shorthand: bool, 
        computed: bool, 
        loc: Option<SourceLocation> 
    },
    CatchClause { 
        param: Option<Box<AstNode>>, 
        body: Box<AstNode>, 
        loc: Option<SourceLocation> 
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
    Undefined,
    RegExp { pattern: String, flags: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VarKind {
    Var,
    Let,
    Const,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add, Sub, Mul, Div, Mod, Pow,
    Equal, NotEqual, StrictEqual, StrictNotEqual,
    Less, Greater, LessEqual, GreaterEqual,
    LeftShift, RightShift, UnsignedRightShift,
    BitwiseAnd, BitwiseOr, BitwiseXor,
    LogicalAnd, LogicalOr, NullishCoalescing,
    In, InstanceOf,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Plus, Minus, Not, BitwiseNot, TypeOf, Void, Delete,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssignmentOperator {
    Assign, AddAssign, SubAssign, MulAssign, DivAssign, ModAssign, PowAssign,
    LeftShiftAssign, RightShiftAssign, UnsignedRightShiftAssign,
    BitwiseAndAssign, BitwiseOrAssign, BitwiseXorAssign,
    LogicalAndAssign, LogicalOrAssign, NullishCoalescingAssign,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UpdateOperator {
    Increment, Decrement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyKind {
    Init, Get, Set, Method,
}

impl Program {
    pub fn new() -> Self {
        Self {
            body: Vec::new(),
            source_type: SourceType::Script,
        }
    }

    pub fn node_count(&self) -> usize {
        fn count_nodes(node: &AstNode) -> usize {
            match node {
                AstNode::Program(program) => {
                    1 + program.body.iter().map(count_nodes).sum::<usize>()
                }
                AstNode::BlockStatement { body, .. } => {
                    1 + body.iter().map(count_nodes).sum::<usize>()
                }
                AstNode::FunctionDeclaration { params, body, .. } => {
                    1 + params.iter().map(count_nodes).sum::<usize>() + count_nodes(body)
                }
                AstNode::BinaryExpression { left, right, .. } => {
                    1 + count_nodes(left) + count_nodes(right)
                }
                AstNode::CallExpression { callee, arguments, .. } => {
                    1 + count_nodes(callee) + arguments.iter().map(count_nodes).sum::<usize>()
                }
                _ => 1,
            }
        }
        
        count_nodes(&AstNode::Program(self.clone()))
    }
}
