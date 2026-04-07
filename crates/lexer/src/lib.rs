use std::fmt;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenKind {
    // Literals
    Integer,
    Float,
    String,
    Bool,
    
    // Identifiers
    Identifier,
    
    // Keywords
    Fn,
    Let,
    If,
    Else,
    For,
    While,
    Return,
    Break,
    Continue,
    Match,
    Struct,
    Impl,
    True,
    False,
    
    // Types
    Int,
    FloatType,
    StringType,
    BoolType,
    List,
    Map,
    
    // Operators
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,
    Assign,
    Equals,
    NotEquals,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    And,
    Or,
    Not,
    
    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Colon,
    Semicolon,
    Arrow,
    
    // Special
    Newline,
    Indent,
    Dedent,
    Eof,
    Illegal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize, column: usize) -> Self {
        Self {
            kind,
            lexeme,
            line,
            column,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}('{}') at {}:{}", self.kind, self.lexeme, self.line, self.column)
    }
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(input: String) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
        }
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, LexerError> {
        while !self.is_at_end() {
            let start_pos = self.position;
            
            match self.current_char() {
                ' ' | '\t' | '\r' => self.advance(),
                '\n' => {
                    self.add_token(TokenKind::Newline, "\n".to_string());
                    self.advance();
                    self.line += 1;
                    self.column = 1;
                }
                '"' => self.string_literal()?,
                '\'' => self.char_literal()?,
                '0'..='9' => self.number()?,
                'a'..='z' | 'A'..='Z' | '_' => self.identifier_or_keyword()?,
                '+' => {
                    if self.match_char('=') {
                        self.add_token(TokenKind::Assign, "+=".to_string());
                    } else {
                        self.add_token(TokenKind::Plus, "+".to_string());
                    }
                }
                '-' => {
                    if self.match_char('>') {
                        self.add_token(TokenKind::Arrow, "->".to_string());
                    } else if self.match_char('=') {
                        self.add_token(TokenKind::Assign, "-=".to_string());
                    } else {
                        self.add_token(TokenKind::Minus, "-".to_string());
                    }
                }
                '*' => {
                    if self.match_char('=') {
                        self.add_token(TokenKind::Assign, "*=".to_string());
                    } else {
                        self.add_token(TokenKind::Multiply, "*".to_string());
                    }
                }
                '/' => {
                    if self.match_char('/') {
                        self.line_comment();
                    } else if self.match_char('*') {
                        self.block_comment()?;
                    } else if self.match_char('=') {
                        self.add_token(TokenKind::Assign, "/=".to_string());
                    } else {
                        self.add_token(TokenKind::Divide, "/".to_string());
                    }
                }
                '%' => {
                    if self.match_char('=') {
                        self.add_token(TokenKind::Assign, "%=".to_string());
                    } else {
                        self.add_token(TokenKind::Modulo, "%".to_string());
                    }
                }
                '=' => {
                    if self.match_char('=') {
                        self.add_token(TokenKind::Equals, "==".to_string());
                    } else {
                        self.add_token(TokenKind::Assign, "=".to_string());
                    }
                }
                '!' => {
                    if self.match_char('=') {
                        self.add_token(TokenKind::NotEquals, "!=".to_string());
                    } else {
                        self.add_token(TokenKind::Not, "!".to_string());
                    }
                }
                '<' => {
                    if self.match_char('=') {
                        self.add_token(TokenKind::LessThanOrEqual, "<=".to_string());
                    } else {
                        self.add_token(TokenKind::LessThan, "<".to_string());
                    }
                }
                '>' => {
                    if self.match_char('=') {
                        self.add_token(TokenKind::GreaterThanOrEqual, ">=".to_string());
                    } else {
                        self.add_token(TokenKind::GreaterThan, ">".to_string());
                    }
                }
                '&' => {
                    if self.match_char('&') {
                        self.add_token(TokenKind::And, "&&".to_string());
                    } else {
                        self.add_token(TokenKind::Illegal, "&".to_string());
                    }
                }
                '|' => {
                    if self.match_char('|') {
                        self.add_token(TokenKind::Or, "||".to_string());
                    } else {
                        self.add_token(TokenKind::Illegal, "|".to_string());
                    }
                }
                '(' => self.add_single_char_token(TokenKind::LeftParen),
                ')' => self.add_single_char_token(TokenKind::RightParen),
                '{' => self.add_single_char_token(TokenKind::LeftBrace),
                '}' => self.add_single_char_token(TokenKind::RightBrace),
                '[' => self.add_single_char_token(TokenKind::LeftBracket),
                ']' => self.add_single_char_token(TokenKind::RightBracket),
                ',' => self.add_single_char_token(TokenKind::Comma),
                '.' => self.add_single_char_token(TokenKind::Dot),
                ':' => self.add_single_char_token(TokenKind::Colon),
                ';' => self.add_single_char_token(TokenKind::Semicolon),
                _ => {
                    self.add_token(TokenKind::Illegal, self.current_char().to_string());
                    self.advance();
                }
            }
        }

        self.add_token(TokenKind::Eof, "".to_string());
        Ok(self.tokens)
    }

    fn string_literal(&mut self) -> Result<(), LexerError> {
        self.advance(); // Skip opening quote
        let mut value = String::new();
        
        while !self.is_at_end() && self.current_char() != '"' {
            if self.current_char() == '\\' {
                self.advance();
                match self.current_char() {
                    'n' => value.push('\n'),
                    't' => value.push('\t'),
                    'r' => value.push('\r'),
                    '\\' => value.push('\\'),
                    '"' => value.push('"'),
                    '\'' => value.push('\''),
                    _ => {
                        value.push(self.current_char());
                    }
                }
            } else {
                value.push(self.current_char());
            }
            self.advance();
        }
        
        if self.is_at_end() {
            return Err(LexerError::UnterminatedString(self.line, self.column));
        }
        
        self.advance(); // Skip closing quote
        self.add_token(TokenKind::String, value);
        Ok(())
    }

    fn char_literal(&mut self) -> Result<(), LexerError> {
        self.advance(); // Skip opening quote
        let mut value = String::new();
        
        if self.current_char() == '\\' {
            self.advance();
            match self.current_char() {
                'n' => value.push('\n'),
                't' => value.push('\t'),
                'r' => value.push('\r'),
                '\\' => value.push('\\'),
                '"' => value.push('"'),
                '\'' => value.push('\''),
                _ => value.push(self.current_char()),
            }
        } else {
            value.push(self.current_char());
        }
        
        self.advance(); // Skip character
        
        if self.current_char() != '\'' {
            return Err(LexerError::UnterminatedChar(self.line, self.column));
        }
        
        self.advance(); // Skip closing quote
        self.add_token(TokenKind::String, value);
        Ok(())
    }

    fn number(&mut self) -> Result<(), LexerError> {
        let start = self.position;
        
        while self.current_char().is_ascii_digit() {
            self.advance();
        }
        
        if self.current_char() == '.' && self.peek_char().is_ascii_digit() {
            self.advance();
            while self.current_char().is_ascii_digit() {
                self.advance();
            }
            
            let lexeme: String = self.input[start..self.position].iter().collect();
            self.add_token(TokenKind::Float, lexeme);
        } else {
            let lexeme: String = self.input[start..self.position].iter().collect();
            self.add_token(TokenKind::Integer, lexeme);
        }
        
        Ok(())
    }

    fn identifier_or_keyword(&mut self) -> Result<(), LexerError> {
        let start = self.position;
        
        while self.current_char().is_alphanumeric() || self.current_char() == '_' {
            self.advance();
        }
        
        let lexeme: String = self.input[start..self.position].iter().collect();
        let kind = match lexeme.as_str() {
            "fn" => TokenKind::Fn,
            "let" => TokenKind::Let,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "for" => TokenKind::For,
            "while" => TokenKind::While,
            "return" => TokenKind::Return,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "match" => TokenKind::Match,
            "struct" => TokenKind::Struct,
            "impl" => TokenKind::Impl,
            "true" => TokenKind::True,
            "false" => TokenKind::False,
            "Int" => TokenKind::Int,
            "Float" => TokenKind::FloatType,
            "String" => TokenKind::StringType,
            "Bool" => TokenKind::BoolType,
            "List" => TokenKind::List,
            "Map" => TokenKind::Map,
            _ => TokenKind::Identifier,
        };
        
        self.add_token(kind, lexeme);
        Ok(())
    }

    fn line_comment(&mut self) {
        while !self.is_at_end() && self.current_char() != '\n' {
            self.advance();
        }
    }

    fn block_comment(&mut self) -> Result<(), LexerError> {
        self.advance(); // Skip '*'
        
        while !self.is_at_end() && !(self.current_char() == '*' && self.peek_char() == '/') {
            if self.current_char() == '\n' {
                self.line += 1;
                self.column = 1;
            }
            self.advance();
        }
        
        if self.is_at_end() {
            return Err(LexerError::UnterminatedComment(self.line, self.column));
        }
        
        self.advance(); // Skip '*'
        self.advance(); // Skip '/'
        Ok(())
    }

    fn add_single_char_token(&mut self, kind: TokenKind) {
        self.add_token(kind, self.current_char().to_string());
        self.advance();
    }

    fn add_token(&mut self, kind: TokenKind, lexeme: String) {
        let token = Token::new(kind, lexeme.clone(), self.line, self.column);
        self.tokens.push(token);
        self.column += lexeme.len();
    }

    fn current_char(&self) -> char {
        if self.position >= self.input.len() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn peek_char(&self) -> char {
        if self.position + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.position + 1]
        }
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.current_char() == expected {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LexerError {
    #[error("Unterminated string at line {0}, column {1}")]
    UnterminatedString(usize, usize),
    
    #[error("Unterminated character at line {0}, column {1}")]
    UnterminatedChar(usize, usize),
    
    #[error("Unterminated comment at line {0}, column {1}")]
    UnterminatedComment(usize, usize),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let input = "let x = 42;";
        let lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier);
        assert_eq!(tokens[2].kind, TokenKind::Assign);
        assert_eq!(tokens[3].kind, TokenKind::Integer);
        assert_eq!(tokens[4].kind, TokenKind::Semicolon);
    }

    #[test]
    fn test_string_literals() {
        let input = "\"hello\" 'a'";
        let lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].kind, TokenKind::String);
        assert_eq!(tokens[0].lexeme, "hello");
        assert_eq!(tokens[1].kind, TokenKind::String);
        assert_eq!(tokens[1].lexeme, "a");
    }

    #[test]
    fn test_numbers() {
        let input = "42 3.14";
        let lexer = Lexer::new(input.to_string());
        let tokens = lexer.tokenize().unwrap();
        
        assert_eq!(tokens[0].kind, TokenKind::Integer);
        assert_eq!(tokens[0].lexeme, "42");
        assert_eq!(tokens[1].kind, TokenKind::Float);
        assert_eq!(tokens[1].lexeme, "3.14");
    }
}
