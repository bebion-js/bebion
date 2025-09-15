//! JavaScript lexer for tokenizing source code

use crate::{ParseError, ParseResult};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TokenType {
    // Literals
    Identifier(String),
    StringLiteral(String),
    NumericLiteral(f64),
    BooleanLiteral(bool),
    NullLiteral,
    UndefinedLiteral,
    RegExpLiteral { pattern: String, flags: String },
    
    // Keywords
    Break, Case, Catch, Class, Const, Continue, Debugger, Default, Delete,
    Do, Else, Export, Extends, Finally, For, Function, If, Import, In,
    InstanceOf, Let, New, Return, Super, Switch, This, Throw, Try, TypeOf,
    Var, Void, While, With, Yield, Async, Await, Static,
    
    // Operators
    Plus, Minus, Multiply, Divide, Modulo, Power,
    Assign, PlusAssign, MinusAssign, MultiplyAssign, DivideAssign, ModuloAssign, PowerAssign,
    Equal, NotEqual, StrictEqual, StrictNotEqual,
    Less, Greater, LessEqual, GreaterEqual,
    LogicalAnd, LogicalOr, LogicalNot, NullishCoalescing,
    BitwiseAnd, BitwiseOr, BitwiseXor, BitwiseNot,
    LeftShift, RightShift, UnsignedRightShift,
    Increment, Decrement,
    
    // Punctuation
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    LeftBracket, RightBracket,
    Semicolon, Comma, Dot, QuestionMark, Colon,
    Arrow, Spread,
    
    // Template literals
    TemplateHead, TemplateMiddle, TemplateTail, TemplateNoSubstitution,
    
    // Special
    EOF,
    Newline,
    Whitespace,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
    pub start: usize,
    pub end: usize,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}({})", self.token_type, self.lexeme)
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    keywords: std::collections::HashMap<String, TokenType>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut keywords = std::collections::HashMap::new();
        
        // Populate keywords
        keywords.insert("break".to_string(), TokenType::Break);
        keywords.insert("case".to_string(), TokenType::Case);
        keywords.insert("catch".to_string(), TokenType::Catch);
        keywords.insert("class".to_string(), TokenType::Class);
        keywords.insert("const".to_string(), TokenType::Const);
        keywords.insert("continue".to_string(), TokenType::Continue);
        keywords.insert("debugger".to_string(), TokenType::Debugger);
        keywords.insert("default".to_string(), TokenType::Default);
        keywords.insert("delete".to_string(), TokenType::Delete);
        keywords.insert("do".to_string(), TokenType::Do);
        keywords.insert("else".to_string(), TokenType::Else);
        keywords.insert("export".to_string(), TokenType::Export);
        keywords.insert("extends".to_string(), TokenType::Extends);
        keywords.insert("finally".to_string(), TokenType::Finally);
        keywords.insert("for".to_string(), TokenType::For);
        keywords.insert("function".to_string(), TokenType::Function);
        keywords.insert("if".to_string(), TokenType::If);
        keywords.insert("import".to_string(), TokenType::Import);
        keywords.insert("in".to_string(), TokenType::In);
        keywords.insert("instanceof".to_string(), TokenType::InstanceOf);
        keywords.insert("let".to_string(), TokenType::Let);
        keywords.insert("new".to_string(), TokenType::New);
        keywords.insert("return".to_string(), TokenType::Return);
        keywords.insert("super".to_string(), TokenType::Super);
        keywords.insert("switch".to_string(), TokenType::Switch);
        keywords.insert("this".to_string(), TokenType::This);
        keywords.insert("throw".to_string(), TokenType::Throw);
        keywords.insert("try".to_string(), TokenType::Try);
        keywords.insert("typeof".to_string(), TokenType::TypeOf);
        keywords.insert("var".to_string(), TokenType::Var);
        keywords.insert("void".to_string(), TokenType::Void);
        keywords.insert("while".to_string(), TokenType::While);
        keywords.insert("with".to_string(), TokenType::With);
        keywords.insert("yield".to_string(), TokenType::Yield);
        keywords.insert("async".to_string(), TokenType::Async);
        keywords.insert("await".to_string(), TokenType::Await);
        keywords.insert("static".to_string(), TokenType::Static);
        keywords.insert("true".to_string(), TokenType::BooleanLiteral(true));
        keywords.insert("false".to_string(), TokenType::BooleanLiteral(false));
        keywords.insert("null".to_string(), TokenType::NullLiteral);
        keywords.insert("undefined".to_string(), TokenType::UndefinedLiteral);
        
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            keywords,
        }
    }

    pub fn tokenize(&mut self) -> ParseResult<Vec<Token>> {
        let mut tokens = Vec::new();
        
        while !self.is_at_end() {
            let token = self.next_token()?;
            
            // Skip whitespace tokens for now
            if !matches!(token.token_type, TokenType::Whitespace | TokenType::Newline) {
                tokens.push(token);
            }
        }
        
        tokens.push(Token {
            token_type: TokenType::EOF,
            lexeme: "".to_string(),
            line: self.line,
            column: self.column,
            start: self.position,
            end: self.position,
        });
        
        Ok(tokens)
    }

    fn next_token(&mut self) -> ParseResult<Token> {
        let start_pos = self.position;
        let start_line = self.line;
        let start_column = self.column;
        
        let ch = self.advance();
        
        match ch {
            ' ' | '\t' | '\r' => {
                self.skip_whitespace();
                Ok(Token {
                    token_type: TokenType::Whitespace,
                    lexeme: " ".to_string(),
                    line: start_line,
                    column: start_column,
                    start: start_pos,
                    end: self.position,
                })
            }
            '\n' => {
                self.line += 1;
                self.column = 1;
                Ok(Token {
                    token_type: TokenType::Newline,
                    lexeme: "\n".to_string(),
                    line: start_line,
                    column: start_column,
                    start: start_pos,
                    end: self.position,
                })
            }
            '/' => {
                if self.peek() == '/' {
                    self.skip_line_comment();
                    self.next_token()
                } else if self.peek() == '*' {
                    self.skip_block_comment()?;
                    self.next_token()
                } else if self.peek() == '=' {
                    self.advance();
                    Ok(self.make_token(TokenType::DivideAssign, "/=", start_line, start_column, start_pos))
                } else {
                    Ok(self.make_token(TokenType::Divide, "/", start_line, start_column, start_pos))
                }
            }
            '+' => {
                if self.peek() == '+' {
                    self.advance();
                    Ok(self.make_token(TokenType::Increment, "++", start_line, start_column, start_pos))
                } else if self.peek() == '=' {
                    self.advance();
                    Ok(self.make_token(TokenType::PlusAssign, "+=", start_line, start_column, start_pos))
                } else {
                    Ok(self.make_token(TokenType::Plus, "+", start_line, start_column, start_pos))
                }
            }
            '-' => {
                if self.peek() == '-' {
                    self.advance();
                    Ok(self.make_token(TokenType::Decrement, "--", start_line, start_column, start_pos))
                } else if self.peek() == '=' {
                    self.advance();
                    Ok(self.make_token(TokenType::MinusAssign, "-=", start_line, start_column, start_pos))
                } else {
                    Ok(self.make_token(TokenType::Minus, "-", start_line, start_column, start_pos))
                }
            }
            '*' => {
                if self.peek() == '*' {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        Ok(self.make_token(TokenType::PowerAssign, "**=", start_line, start_column, start_pos))
                    } else {
                        Ok(self.make_token(TokenType::Power, "**", start_line, start_column, start_pos))
                    }
                } else if self.peek() == '=' {
                    self.advance();
                    Ok(self.make_token(TokenType::MultiplyAssign, "*=", start_line, start_column, start_pos))
                } else {
                    Ok(self.make_token(TokenType::Multiply, "*", start_line, start_column, start_pos))
                }
            }
            '=' => {
                if self.peek() == '=' {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        Ok(self.make_token(TokenType::StrictEqual, "===", start_line, start_column, start_pos))
                    } else {
                        Ok(self.make_token(TokenType::Equal, "==", start_line, start_column, start_pos))
                    }
                } else if self.peek() == '>' {
                    self.advance();
                    Ok(self.make_token(TokenType::Arrow, "=>", start_line, start_column, start_pos))
                } else {
                    Ok(self.make_token(TokenType::Assign, "=", start_line, start_column, start_pos))
                }
            }
            '!' => {
                if self.peek() == '=' {
                    self.advance();
                    if self.peek() == '=' {
                        self.advance();
                        Ok(self.make_token(TokenType::StrictNotEqual, "!==", start_line, start_column, start_pos))
                    } else {
                        Ok(self.make_token(TokenType::NotEqual, "!=", start_line, start_column, start_pos))
                    }
                } else {
                    Ok(self.make_token(TokenType::LogicalNot, "!", start_line, start_column, start_pos))
                }
            }
            '<' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(self.make_token(TokenType::LessEqual, "<=", start_line, start_column, start_pos))
                } else if self.peek() == '<' {
                    self.advance();
                    Ok(self.make_token(TokenType::LeftShift, "<<", start_line, start_column, start_pos))
                } else {
                    Ok(self.make_token(TokenType::Less, "<", start_line, start_column, start_pos))
                }
            }
            '>' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(self.make_token(TokenType::GreaterEqual, ">=", start_line, start_column, start_pos))
                } else if self.peek() == '>' {
                    self.advance();
                    if self.peek() == '>' {
                        self.advance();
                        Ok(self.make_token(TokenType::UnsignedRightShift, ">>>", start_line, start_column, start_pos))
                    } else {
                        Ok(self.make_token(TokenType::RightShift, ">>", start_line, start_column, start_pos))
                    }
                } else {
                    Ok(self.make_token(TokenType::Greater, ">", start_line, start_column, start_pos))
                }
            }
            '&' => {
                if self.peek() == '&' {
                    self.advance();
                    Ok(self.make_token(TokenType::LogicalAnd, "&&", start_line, start_column, start_pos))
                } else {
                    Ok(self.make_token(TokenType::BitwiseAnd, "&", start_line, start_column, start_pos))
                }
            }
            '|' => {
                if self.peek() == '|' {
                    self.advance();
                    Ok(self.make_token(TokenType::LogicalOr, "||", start_line, start_column, start_pos))
                } else {
                    Ok(self.make_token(TokenType::BitwiseOr, "|", start_line, start_column, start_pos))
                }
            }
            '?' => {
                if self.peek() == '?' {
                    self.advance();
                    Ok(self.make_token(TokenType::NullishCoalescing, "??", start_line, start_column, start_pos))
                } else {
                    Ok(self.make_token(TokenType::QuestionMark, "?", start_line, start_column, start_pos))
                }
            }
            '(' => Ok(self.make_token(TokenType::LeftParen, "(", start_line, start_column, start_pos)),
            ')' => Ok(self.make_token(TokenType::RightParen, ")", start_line, start_column, start_pos)),
            '{' => Ok(self.make_token(TokenType::LeftBrace, "{", start_line, start_column, start_pos)),
            '}' => Ok(self.make_token(TokenType::RightBrace, "}", start_line, start_column, start_pos)),
            '[' => Ok(self.make_token(TokenType::LeftBracket, "[", start_line, start_column, start_pos)),
            ']' => Ok(self.make_token(TokenType::RightBracket, "]", start_line, start_column, start_pos)),
            ';' => Ok(self.make_token(TokenType::Semicolon, ";", start_line, start_column, start_pos)),
            ',' => Ok(self.make_token(TokenType::Comma, ",", start_line, start_column, start_pos)),
            '.' => {
                if self.peek() == '.' && self.peek_ahead(1) == '.' {
                    self.advance();
                    self.advance();
                    Ok(self.make_token(TokenType::Spread, "...", start_line, start_column, start_pos))
                } else {
                    Ok(self.make_token(TokenType::Dot, ".", start_line, start_column, start_pos))
                }
            }
            ':' => Ok(self.make_token(TokenType::Colon, ":", start_line, start_column, start_pos)),
            '^' => Ok(self.make_token(TokenType::BitwiseXor, "^", start_line, start_column, start_pos)),
            '~' => Ok(self.make_token(TokenType::BitwiseNot, "~", start_line, start_column, start_pos)),
            '%' => {
                if self.peek() == '=' {
                    self.advance();
                    Ok(self.make_token(TokenType::ModuloAssign, "%=", start_line, start_column, start_pos))
                } else {
                    Ok(self.make_token(TokenType::Modulo, "%", start_line, start_column, start_pos))
                }
            }
            '"' | '\'' => self.string_literal(ch, start_line, start_column, start_pos),
            '`' => self.template_literal(start_line, start_column, start_pos),
            _ if ch.is_ascii_digit() => self.numeric_literal(start_line, start_column, start_pos),
            _ if ch.is_alphabetic() || ch == '_' || ch == '$' => {
                self.identifier_or_keyword(start_line, start_column, start_pos)
            }
            _ => Err(ParseError::LexicalError {
                message: format!("Unexpected character: '{}'", ch),
                line: start_line,
                column: start_column,
            }),
        }
    }

    fn advance(&mut self) -> char {
        if self.is_at_end() {
            return '\0';
        }
        
        let ch = self.input[self.position];
        self.position += 1;
        self.column += 1;
        ch
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn peek_ahead(&self, offset: usize) -> char {
        let pos = self.position + offset;
        if pos >= self.input.len() {
            '\0'
        } else {
            self.input[pos]
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn make_token(&self, token_type: TokenType, lexeme: &str, line: usize, column: usize, start: usize) -> Token {
        Token {
            token_type,
            lexeme: lexeme.to_string(),
            line,
            column,
            start,
            end: self.position,
        }
    }

    fn skip_whitespace(&mut self) {
        while !self.is_at_end() && matches!(self.peek(), ' ' | '\t' | '\r') {
            self.advance();
        }
    }

    fn skip_line_comment(&mut self) {
        while !self.is_at_end() && self.peek() != '\n' {
            self.advance();
        }
    }

    fn skip_block_comment(&mut self) -> ParseResult<()> {
        self.advance(); // consume '*'
        
        while !self.is_at_end() {
            if self.peek() == '*' && self.peek_ahead(1) == '/' {
                self.advance(); // consume '*'
                self.advance(); // consume '/'
                return Ok(());
            }
            
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 0;
            }
            
            self.advance();
        }
        
        Err(ParseError::LexicalError {
            message: "Unterminated block comment".to_string(),
            line: self.line,
            column: self.column,
        })
    }

    fn string_literal(&mut self, quote: char, start_line: usize, start_column: usize, start_pos: usize) -> ParseResult<Token> {
        let mut value = String::new();
        
        while !self.is_at_end() && self.peek() != quote {
            if self.peek() == '\n' {
                return Err(ParseError::LexicalError {
                    message: "Unterminated string literal".to_string(),
                    line: self.line,
                    column: self.column,
                });
            }
            
            if self.peek() == '\\' {
                self.advance(); // consume '\'
                let escaped = self.advance();
                match escaped {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '\'' => value.push('\''),
                    '"' => value.push('"'),
                    '0' => value.push('\0'),
                    _ => {
                        value.push('\\');
                        value.push(escaped);
                    }
                }
            } else {
                value.push(self.advance());
            }
        }
        
        if self.is_at_end() {
            return Err(ParseError::LexicalError {
                message: "Unterminated string literal".to_string(),
                line: start_line,
                column: start_column,
            });
        }
        
        self.advance(); // consume closing quote
        
        let lexeme = format!("{}{}{}", quote, value, quote);
        Ok(Token {
            token_type: TokenType::StringLiteral(value),
            lexeme,
            line: start_line,
            column: start_column,
            start: start_pos,
            end: self.position,
        })
    }

    fn template_literal(&mut self, start_line: usize, start_column: usize, start_pos: usize) -> ParseResult<Token> {
        // This is a simplified template literal lexer
        // A full implementation would need to handle substitutions
        let mut value = String::new();
        
        while !self.is_at_end() && self.peek() != '`' {
            if self.peek() == '\n' {
                self.line += 1;
                self.column = 0;
            }
            value.push(self.advance());
        }
        
        if self.is_at_end() {
            return Err(ParseError::LexicalError {
                message: "Unterminated template literal".to_string(),
                line: start_line,
                column: start_column,
            });
        }
        
        self.advance(); // consume closing '`'
        
        let lexeme = format!("`{}`", value);
        Ok(Token {
            token_type: TokenType::TemplateNoSubstitution,
            lexeme,
            line: start_line,
            column: start_column,
            start: start_pos,
            end: self.position,
        })
    }

    fn numeric_literal(&mut self, start_line: usize, start_column: usize, start_pos: usize) -> ParseResult<Token> {
        self.position -= 1; // Go back to include the first digit
        self.column -= 1;
        
        let mut lexeme = String::new();
        
        while !self.is_at_end() && (self.peek().is_ascii_digit() || self.peek() == '.') {
            lexeme.push(self.advance());
        }
        
        // Handle scientific notation
        if !self.is_at_end() && (self.peek() == 'e' || self.peek() == 'E') {
            lexeme.push(self.advance());
            if !self.is_at_end() && (self.peek() == '+' || self.peek() == '-') {
                lexeme.push(self.advance());
            }
            while !self.is_at_end() && self.peek().is_ascii_digit() {
                lexeme.push(self.advance());
            }
        }
        
        let value = lexeme.parse::<f64>().map_err(|_| ParseError::LexicalError {
            message: format!("Invalid numeric literal: {}", lexeme),
            line: start_line,
            column: start_column,
        })?;
        
        Ok(Token {
            token_type: TokenType::NumericLiteral(value),
            lexeme,
            line: start_line,
            column: start_column,
            start: start_pos,
            end: self.position,
        })
    }

    fn identifier_or_keyword(&mut self, start_line: usize, start_column: usize, start_pos: usize) -> ParseResult<Token> {
        self.position -= 1; // Go back to include the first character
        self.column -= 1;
        
        let mut lexeme = String::new();
        
        while !self.is_at_end() && (self.peek().is_alphanumeric() || self.peek() == '_' || self.peek() == '$') {
            lexeme.push(self.advance());
        }
        
        let token_type = self.keywords.get(&lexeme)
            .cloned()
            .unwrap_or_else(|| TokenType::Identifier(lexeme.clone()));
        
        Ok(Token {
            token_type,
            lexeme,
            line: start_line,
            column: start_column,
            start: start_pos,
            end: self.position,
        })
    }
}