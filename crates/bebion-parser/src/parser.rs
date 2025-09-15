//! JavaScript parser implementation

use crate::ast::*;
use crate::lexer::{Lexer, Token, TokenType};
use crate::{ParseError, ParseResult};
use tracing::debug;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            tokens: Vec::new(),
            current: 0,
        }
    }

    pub fn parse(&mut self, source: &str) -> ParseResult<Program> {
        debug!("Parsing source: {} characters", source.len());
        
        let mut lexer = Lexer::new(source);
        self.tokens = lexer.tokenize()?;
        self.current = 0;
        
        debug!("Tokenized {} tokens", self.tokens.len());
        
        self.program()
    }

    fn program(&mut self) -> ParseResult<Program> {
        let mut body = Vec::new();
        
        while !self.is_at_end() {
            if let Ok(stmt) = self.statement() {
                body.push(stmt);
            } else {
                // Skip invalid tokens and continue
                self.advance();
            }
        }
        
        Ok(Program {
            body,
            source_type: SourceType::Script,
        })
    }

    fn statement(&mut self) -> ParseResult<AstNode> {
        match self.peek().token_type {
            TokenType::Var | TokenType::Let | TokenType::Const => self.variable_declaration(),
            TokenType::Function => self.function_declaration(),
            TokenType::If => self.if_statement(),
            TokenType::While => self.while_statement(),
            TokenType::For => self.for_statement(),
            TokenType::Return => self.return_statement(),
            TokenType::Break => self.break_statement(),
            TokenType::Continue => self.continue_statement(),
            TokenType::Throw => self.throw_statement(),
            TokenType::Try => self.try_statement(),
            TokenType::LeftBrace => self.block_statement(),
            _ => self.expression_statement(),
        }
    }

    fn variable_declaration(&mut self) -> ParseResult<AstNode> {
        let kind_token = self.advance().clone();
        let kind = match kind_token.token_type {
            TokenType::Var => VarKind::Var,
            TokenType::Let => VarKind::Let,
            TokenType::Const => VarKind::Const,
            _ => unreachable!(),
        };

        let mut declarations = Vec::new();
        
        loop {
            let id = self.expect_identifier()?;
            let init = if self.matches(&[TokenType::Assign]) {
                self.advance();
                Some(Box::new(self.expression()?))
            } else {
                None
            };
            
            declarations.push(AstNode::VariableDeclarator {
                id: Box::new(id),
                init,
                loc: None,
            });
            
            if !self.matches(&[TokenType::Comma]) {
                break;
            }
            self.advance();
        }
        
        self.consume_semicolon();
        
        Ok(AstNode::VariableDeclaration {
            declarations,
            kind,
            loc: None,
        })
    }

    fn function_declaration(&mut self) -> ParseResult<AstNode> {
        self.advance(); // consume 'function'
        
        let is_async = false;
        let is_generator = false;
        let id = Some(Box::new(self.expect_identifier()?));
        
        self.expect(&TokenType::LeftParen)?;
        let params = self.parameter_list()?;
        self.expect(&TokenType::RightParen)?;
        
        let body = Box::new(self.block_statement()?);
        
        Ok(AstNode::FunctionDeclaration {
            id,
            params,
            body,
            is_async,
            is_generator,
            loc: None,
        })
    }

    fn if_statement(&mut self) -> ParseResult<AstNode> {
        self.advance(); // consume 'if'
        
        self.expect(&TokenType::LeftParen)?;
        let test = Box::new(self.expression()?);
        self.expect(&TokenType::RightParen)?;
        
        let consequent = Box::new(self.statement()?);
        
        let alternate = if self.matches(&[TokenType::Else]) {
            self.advance();
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        
        Ok(AstNode::IfStatement {
            test,
            consequent,
            alternate,
            loc: None,
        })
    }

    fn while_statement(&mut self) -> ParseResult<AstNode> {
        self.advance(); // consume 'while'
        
        self.expect(&TokenType::LeftParen)?;
        let test = Box::new(self.expression()?);
        self.expect(&TokenType::RightParen)?;
        
        let body = Box::new(self.statement()?);
        
        Ok(AstNode::WhileStatement {
            test,
            body,
            loc: None,
        })
    }

    fn for_statement(&mut self) -> ParseResult<AstNode> {
        self.advance(); // consume 'for'
        
        self.expect(&TokenType::LeftParen)?;
        
        let init = if self.matches(&[TokenType::Semicolon]) {
            None
        } else if self.matches(&[TokenType::Var, TokenType::Let, TokenType::Const]) {
            Some(Box::new(self.variable_declaration()?))
        } else {
            Some(Box::new(self.expression()?))
        };
        
        if init.is_some() && !self.previous().token_type.eq(&TokenType::Semicolon) {
            self.expect(&TokenType::Semicolon)?;
        }
        
        let test = if self.matches(&[TokenType::Semicolon]) {
            None
        } else {
            let expr = self.expression()?;
            self.expect(&TokenType::Semicolon)?;
            Some(Box::new(expr))
        };
        
        if test.is_none() {
            self.advance(); // consume semicolon
        }
        
        let update = if self.matches(&[TokenType::RightParen]) {
            None
        } else {
            Some(Box::new(self.expression()?))
        };
        
        self.expect(&TokenType::RightParen)?;
        
        let body = Box::new(self.statement()?);
        
        Ok(AstNode::ForStatement {
            init,
            test,
            update,
            body,
            loc: None,
        })
    }

    fn return_statement(&mut self) -> ParseResult<AstNode> {
        self.advance(); // consume 'return'
        
        let argument = if self.matches(&[TokenType::Semicolon, TokenType::EOF]) || self.check(&TokenType::RightBrace) {
            None
        } else {
            Some(Box::new(self.expression()?))
        };
        
        self.consume_semicolon();
        
        Ok(AstNode::ReturnStatement {
            argument,
            loc: None,
        })
    }

    fn break_statement(&mut self) -> ParseResult<AstNode> {
        self.advance(); // consume 'break'
        
        let label = None;
        
        self.consume_semicolon();
        
        Ok(AstNode::BreakStatement {
            label,
            loc: None,
        })
    }

    fn continue_statement(&mut self) -> ParseResult<AstNode> {
        self.advance(); // consume 'continue'
        
        let label = None;
        
        self.consume_semicolon();
        
        Ok(AstNode::ContinueStatement {
            label,
            loc: None,
        })
    }

    fn throw_statement(&mut self) -> ParseResult<AstNode> {
        self.advance(); // consume 'throw'
        
        let argument = Box::new(self.expression()?);
        
        self.consume_semicolon();
        
        Ok(AstNode::ThrowStatement {
            argument,
            loc: None,
        })
    }

    fn try_statement(&mut self) -> ParseResult<AstNode> {
        self.advance(); // consume 'try'
        
        let block = Box::new(self.block_statement()?);
        
        let handler = if self.matches(&[TokenType::Catch]) {
            self.advance();
            
            let param = if self.matches(&[TokenType::LeftParen]) {
                self.advance();
                let p = Some(Box::new(self.expect_identifier()?));
                self.expect(&TokenType::RightParen)?;
                p
            } else {
                None
            };
            
            let body = Box::new(self.block_statement()?);
            
            Some(Box::new(AstNode::CatchClause {
                param,
                body,
                loc: None,
            }))
        } else {
            None
        };
        
        let finalizer = if self.matches(&[TokenType::Finally]) {
            self.advance();
            Some(Box::new(self.block_statement()?))
        } else {
            None
        };
        
        Ok(AstNode::TryStatement {
            block,
            handler,
            finalizer,
            loc: None,
        })
    }

    fn block_statement(&mut self) -> ParseResult<AstNode> {
        self.expect(&TokenType::LeftBrace)?;
        
        let mut body = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            body.push(self.statement()?);
        }
        
        self.expect(&TokenType::RightBrace)?;
        
        Ok(AstNode::BlockStatement {
            body,
            loc: None,
        })
    }

    fn expression_statement(&mut self) -> ParseResult<AstNode> {
        let expression = Box::new(self.expression()?);
        self.consume_semicolon();
        
        Ok(AstNode::ExpressionStatement {
            expression,
            loc: None,
        })
    }

    fn expression(&mut self) -> ParseResult<AstNode> {
        self.assignment()
    }

    fn assignment(&mut self) -> ParseResult<AstNode> {
        let expr = self.conditional()?;
        
        if self.matches(&[
            TokenType::Assign,
            TokenType::PlusAssign,
            TokenType::MinusAssign,
            TokenType::MultiplyAssign,
            TokenType::DivideAssign,
            TokenType::ModuloAssign,
            TokenType::PowerAssign,
        ]) {
            let operator_token = self.previous().clone();
            let operator = match operator_token.token_type {
                TokenType::Assign => AssignmentOperator::Assign,
                TokenType::PlusAssign => AssignmentOperator::AddAssign,
                TokenType::MinusAssign => AssignmentOperator::SubAssign,
                TokenType::MultiplyAssign => AssignmentOperator::MulAssign,
                TokenType::DivideAssign => AssignmentOperator::DivAssign,
                TokenType::ModuloAssign => AssignmentOperator::ModAssign,
                TokenType::PowerAssign => AssignmentOperator::PowAssign,
                _ => unreachable!(),
            };
            
            let right = Box::new(self.assignment()?);
            
            return Ok(AstNode::AssignmentExpression {
                operator,
                left: Box::new(expr),
                right,
                loc: None,
            });
        }
        
        Ok(expr)
    }

    fn conditional(&mut self) -> ParseResult<AstNode> {
        let expr = self.logical_or()?;
        
        if self.matches(&[TokenType::QuestionMark]) {
            self.advance();
            let consequent = Box::new(self.expression()?);
            self.expect(&TokenType::Colon)?;
            let alternate = Box::new(self.conditional()?);
            
            return Ok(AstNode::ConditionalExpression {
                test: Box::new(expr),
                consequent,
                alternate,
                loc: None,
            });
        }
        
        Ok(expr)
    }

    fn logical_or(&mut self) -> ParseResult<AstNode> {
        let mut expr = self.logical_and()?;
        
        while self.matches(&[TokenType::LogicalOr, TokenType::NullishCoalescing]) {
            let operator_token = self.previous().clone();
            let operator = match operator_token.token_type {
                TokenType::LogicalOr => BinaryOperator::LogicalOr,
                TokenType::NullishCoalescing => BinaryOperator::NullishCoalescing,
                _ => unreachable!(),
            };
            
            let right = Box::new(self.logical_and()?);
            
            expr = AstNode::BinaryExpression {
                operator,
                left: Box::new(expr),
                right,
                loc: None,
            };
        }
        
        Ok(expr)
    }

    fn logical_and(&mut self) -> ParseResult<AstNode> {
        let mut expr = self.equality()?;
        
        while self.matches(&[TokenType::LogicalAnd]) {
            let operator = BinaryOperator::LogicalAnd;
            let right = Box::new(self.equality()?);
            
            expr = AstNode::BinaryExpression {
                operator,
                left: Box::new(expr),
                right,
                loc: None,
            };
        }
        
        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<AstNode> {
        let mut expr = self.comparison()?;
        
        while self.matches(&[
            TokenType::Equal,
            TokenType::NotEqual,
            TokenType::StrictEqual,
            TokenType::StrictNotEqual,
        ]) {
            let operator_token = self.previous().clone();
            let operator = match operator_token.token_type {
                TokenType::Equal => BinaryOperator::Equal,
                TokenType::NotEqual => BinaryOperator::NotEqual,
                TokenType::StrictEqual => BinaryOperator::StrictEqual,
                TokenType::StrictNotEqual => BinaryOperator::StrictNotEqual,
                _ => unreachable!(),
            };
            
            let right = Box::new(self.comparison()?);
            
            expr = AstNode::BinaryExpression {
                operator,
                left: Box::new(expr),
                right,
                loc: None,
            };
        }
        
        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<AstNode> {
        let mut expr = self.term()?;
        
        while self.matches(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
            TokenType::In,
            TokenType::InstanceOf,
        ]) {
            let operator_token = self.previous().clone();
            let operator = match operator_token.token_type {
                TokenType::Greater => BinaryOperator::Greater,
                TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
                TokenType::Less => BinaryOperator::Less,
                TokenType::LessEqual => BinaryOperator::LessEqual,
                TokenType::In => BinaryOperator::In,
                TokenType::InstanceOf => BinaryOperator::InstanceOf,
                _ => unreachable!(),
            };
            
            let right = Box::new(self.term()?);
            
            expr = AstNode::BinaryExpression {
                operator,
                left: Box::new(expr),
                right,
                loc: None,
            };
        }
        
        Ok(expr)
    }

    fn term(&mut self) -> ParseResult<AstNode> {
        let mut expr = self.factor()?;
        
        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let operator_token = self.previous().clone();
            let operator = match operator_token.token_type {
                TokenType::Minus => BinaryOperator::Sub,
                TokenType::Plus => BinaryOperator::Add,
                _ => unreachable!(),
            };
            
            let right = Box::new(self.factor()?);
            
            expr = AstNode::BinaryExpression {
                operator,
                left: Box::new(expr),
                right,
                loc: None,
            };
        }
        
        Ok(expr)
    }

    fn factor(&mut self) -> ParseResult<AstNode> {
        let mut expr = self.unary()?;
        
        while self.matches(&[TokenType::Divide, TokenType::Multiply, TokenType::Modulo, TokenType::Power]) {
            let operator_token = self.previous().clone();
            let operator = match operator_token.token_type {
                TokenType::Divide => BinaryOperator::Div,
                TokenType::Multiply => BinaryOperator::Mul,
                TokenType::Modulo => BinaryOperator::Mod,
                TokenType::Power => BinaryOperator::Pow,
                _ => unreachable!(),
            };
            
            let right = Box::new(self.unary()?);
            
            expr = AstNode::BinaryExpression {
                operator,
                left: Box::new(expr),
                right,
                loc: None,
            };
        }
        
        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<AstNode> {
        if self.matches(&[
            TokenType::LogicalNot,
            TokenType::Minus,
            TokenType::Plus,
            TokenType::BitwiseNot,
            TokenType::TypeOf,
            TokenType::Void,
            TokenType::Delete,
        ]) {
            let operator_token = self.previous().clone();
            let operator = match operator_token.token_type {
                TokenType::LogicalNot => UnaryOperator::Not,
                TokenType::Minus => UnaryOperator::Minus,
                TokenType::Plus => UnaryOperator::Plus,
                TokenType::BitwiseNot => UnaryOperator::BitwiseNot,
                TokenType::TypeOf => UnaryOperator::TypeOf,
                TokenType::Void => UnaryOperator::Void,
                TokenType::Delete => UnaryOperator::Delete,
                _ => unreachable!(),
            };
            
            let argument = Box::new(self.unary()?);
            
            return Ok(AstNode::UnaryExpression {
                operator,
                argument,
                prefix: true,
                loc: None,
            });
        }
        
        self.postfix()
    }

    fn postfix(&mut self) -> ParseResult<AstNode> {
        let mut expr = self.call()?;
        
        if self.matches(&[TokenType::Increment, TokenType::Decrement]) {
            let operator_token = self.previous().clone();
            let operator = match operator_token.token_type {
                TokenType::Increment => UpdateOperator::Increment,
                TokenType::Decrement => UpdateOperator::Decrement,
                _ => unreachable!(),
            };
            
            expr = AstNode::UpdateExpression {
                operator,
                argument: Box::new(expr),
                prefix: false,
                loc: None,
            };
        }
        
        Ok(expr)
    }

    fn call(&mut self) -> ParseResult<AstNode> {
        let mut expr = self.primary()?;
        
        loop {
            if self.matches(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.matches(&[TokenType::Dot]) {
                let property = Box::new(self.expect_identifier()?);
                expr = AstNode::MemberExpression {
                    object: Box::new(expr),
                    property,
                    computed: false,
                    loc: None,
                };
            } else if self.matches(&[TokenType::LeftBracket]) {
                let property = Box::new(self.expression()?);
                self.expect(&TokenType::RightBracket)?;
                expr = AstNode::MemberExpression {
                    object: Box::new(expr),
                    property,
                    computed: true,
                    loc: None,
                };
            } else {
                break;
            }
        }
        
        Ok(expr)
    }

    fn finish_call(&mut self, callee: AstNode) -> ParseResult<AstNode> {
        let mut arguments = Vec::new();
        
        if !self.check(&TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
                self.advance();
            }
        }
        
        self.expect(&TokenType::RightParen)?;
        
        Ok(AstNode::CallExpression {
            callee: Box::new(callee),
            arguments,
            loc: None,
        })
    }

    fn primary(&mut self) -> ParseResult<AstNode> {
        match &self.peek().token_type {
            TokenType::BooleanLiteral(value) => {
                let value = *value;
                self.advance();
                Ok(AstNode::Literal {
                    value: LiteralValue::Boolean(value),
                    raw: value.to_string(),
                    loc: None,
                })
            }
            TokenType::NullLiteral => {
                self.advance();
                Ok(AstNode::Literal {
                    value: LiteralValue::Null,
                    raw: "null".to_string(),
                    loc: None,
                })
            }
            TokenType::UndefinedLiteral => {
                self.advance();
                Ok(AstNode::Literal {
                    value: LiteralValue::Undefined,
                    raw: "undefined".to_string(),
                    loc: None,
                })
            }
            TokenType::NumericLiteral(value) => {
                let value = *value;
                let raw = self.advance().lexeme.clone();
                Ok(AstNode::Literal {
                    value: LiteralValue::Number(value),
                    raw,
                    loc: None,
                })
            }
            TokenType::StringLiteral(value) => {
                let value = value.clone();
                let raw = self.advance().lexeme.clone();
                Ok(AstNode::Literal {
                    value: LiteralValue::String(value),
                    raw,
                    loc: None,
                })
            }
            TokenType::Identifier(name) => {
                let name = name.clone();
                self.advance();
                Ok(AstNode::Identifier {
                    name,
                    loc: None,
                })
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.expect(&TokenType::RightParen)?;
                Ok(expr)
            }
            TokenType::LeftBracket => self.array_expression(),
            TokenType::LeftBrace => self.object_expression(),
            TokenType::Function => self.function_expression(),
            TokenType::This => {
                self.advance();
                Ok(AstNode::Identifier {
                    name: "this".to_string(),
                    loc: None,
                })
            }
            _ => Err(ParseError::UnexpectedToken {
                expected: "expression".to_string(),
                found: self.peek().lexeme.clone(),
                line: self.peek().line,
                column: self.peek().column,
            }),
        }
    }

    fn array_expression(&mut self) -> ParseResult<AstNode> {
        self.advance(); // consume '['
        
        let mut elements = Vec::new();
        
        while !self.check(&TokenType::RightBracket) && !self.is_at_end() {
            if self.matches(&[TokenType::Comma]) {
                elements.push(None); // Hole in sparse array
                self.advance();
            } else {
                elements.push(Some(self.expression()?));
                if !self.check(&TokenType::RightBracket) {
                    self.expect(&TokenType::Comma)?;
                }
            }
        }
        
        self.expect(&TokenType::RightBracket)?;
        
        Ok(AstNode::ArrayExpression {
            elements,
            loc: None,
        })
    }

    fn object_expression(&mut self) -> ParseResult<AstNode> {
        self.advance(); // consume '{'
        
        let mut properties = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            properties.push(self.property()?);
            
            if !self.check(&TokenType::RightBrace) {
                self.expect(&TokenType::Comma)?;
            }
        }
        
        self.expect(&TokenType::RightBrace)?;
        
        Ok(AstNode::ObjectExpression {
            properties,
            loc: None,
        })
    }

    fn property(&mut self) -> ParseResult<AstNode> {
        let key = if self.check_identifier() {
            Box::new(self.expect_identifier()?)
        } else if matches!(self.peek().token_type, TokenType::StringLiteral(_) | TokenType::NumericLiteral(_)) {
            Box::new(self.primary()?)
        } else if self.matches(&[TokenType::LeftBracket]) {
            self.advance();
            let key = Box::new(self.expression()?);
            self.expect(&TokenType::RightBracket)?;
            key
        } else {
            return Err(ParseError::UnexpectedToken {
                expected: "property key".to_string(),
                found: self.peek().lexeme.clone(),
                line: self.peek().line,
                column: self.peek().column,
            });
        };
        
        self.expect(&TokenType::Colon)?;
        let value = Box::new(self.expression()?);
        
        Ok(AstNode::Property {
            key,
            value,
            kind: PropertyKind::Init,
            method: false,
            shorthand: false,
            computed: false,
            loc: None,
        })
    }

    fn function_expression(&mut self) -> ParseResult<AstNode> {
        self.advance(); // consume 'function'
        
        let id = if self.check_identifier() {
            Some(Box::new(self.expect_identifier()?))
        } else {
            None
        };
        
        self.expect(&TokenType::LeftParen)?;
        let params = self.parameter_list()?;
        self.expect(&TokenType::RightParen)?;
        
        let body = Box::new(self.block_statement()?);
        
        Ok(AstNode::FunctionExpression {
            id,
            params,
            body,
            is_async: false,
            is_generator: false,
            loc: None,
        })
    }

    fn parameter_list(&mut self) -> ParseResult<Vec<AstNode>> {
        let mut params = Vec::new();
        
        if !self.check(&TokenType::RightParen) {
            loop {
                params.push(self.expect_identifier()?);
                if !self.matches(&[TokenType::Comma]) {
                    break;
                }
                self.advance();
            }
        }
        
        Ok(params)
    }

    // Helper methods
    
    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            std::mem::discriminant(&self.peek().token_type) == std::mem::discriminant(token_type)
        }
    }

    fn matches(&self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                return true;
            }
        }
        false
    }

    fn expect(&mut self, token_type: &TokenType) -> ParseResult<&Token> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", token_type),
                found: self.peek().lexeme.clone(),
                line: self.peek().line,
                column: self.peek().column,
            })
        }
    }

    fn expect_identifier(&mut self) -> ParseResult<AstNode> {
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            Ok(AstNode::Identifier {
                name,
                loc: None,
            })
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: self.peek().lexeme.clone(),
                line: self.peek().line,
                column: self.peek().column,
            })
        }
    }

    fn check_identifier(&self) -> bool {
        matches!(self.peek().token_type, TokenType::Identifier(_))
    }

    fn consume_semicolon(&mut self) {
        if self.matches(&[TokenType::Semicolon]) {
            self.advance();
        }
    }
}
