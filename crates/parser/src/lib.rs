use nyx_ast::*;
use nyx_lexer::{Lexer, Token, TokenKind};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected:?}, got {actual:?} at line {line}, column {column}")]
    UnexpectedToken {
        expected: TokenKind,
        actual: Token,
        line: usize,
        column: usize,
    },
    
    #[error("Unexpected end of input")]
    UnexpectedEof,
    
    #[error("Invalid expression at line {line}, column {column}")]
    InvalidExpression { line: usize, column: usize },
    
    #[error("Invalid statement at line {line}, column {column}")]
    InvalidStatement { line: usize, column: usize },
    
    #[error("Lexer error: {0}")]
    LexerError(#[from] nyx_lexer::LexerError),
}

pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    pub fn parse(&mut self) -> Result<Program, ParseError> {
        let mut program = Program::new();
        
        while !self.is_at_end() {
            if self.current_token().kind == TokenKind::Eof {
                break;
            }
            
            match self.current_token().kind {
                TokenKind::Newline => self.advance(),
                _ => {
                    let statement = self.parse_statement()?;
                    program.statements.push(statement);
                }
            }
        }
        
        Ok(program)
    }

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.current_token().kind {
            TokenKind::Let => self.parse_variable_declaration(),
            TokenKind::Fn => self.parse_function_declaration(),
            TokenKind::Struct => self.parse_struct_declaration(),
            TokenKind::Impl => self.parse_implementation(),
            TokenKind::Return => self.parse_return_statement(),
            TokenKind::If => self.parse_if_statement(),
            TokenKind::While => self.parse_while_statement(),
            TokenKind::For => self.parse_for_statement(),
            TokenKind::Match => self.parse_match_statement(),
            TokenKind::Break => self.parse_break_statement(),
            TokenKind::Continue => self.parse_continue_statement(),
            TokenKind::LeftBrace => self.parse_block_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    fn parse_variable_declaration(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenKind::Let)?;
        let name = self.consume_identifier()?;
        
        let type_annotation = if self.match_token(TokenKind::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        let initializer = if self.match_token(TokenKind::Assign) {
            Some(self.parse_expression()?)
        } else {
            None
        };
        
        self.consume_semicolon()?;
        
        Ok(Statement::VariableDeclaration {
            name,
            type_annotation,
            initializer,
            is_mutable: false,
        })
    }

    fn parse_function_declaration(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenKind::Fn)?;
        let name = self.consume_identifier()?;
        
        let type_parameters = if self.match_token(TokenKind::LessThan) {
            self.parse_type_parameter_list()?
        } else {
            Vec::new()
        };
        
        self.consume(TokenKind::LeftParen)?;
        let parameters = self.parse_parameter_list()?;
        self.consume(TokenKind::RightParen)?;
        
        let return_type = if self.match_token(TokenKind::Arrow) {
            Some(self.parse_type()?)
        } else {
            None
        };
        
        self.consume(TokenKind::LeftBrace)?;
        let body = self.parse_statement_block()?;
        self.consume(TokenKind::RightBrace)?;
        
        Ok(Statement::FunctionDeclaration {
            name,
            parameters,
            return_type,
            body,
            is_generic: !type_parameters.is_empty(),
            type_parameters,
        })
    }

    fn parse_struct_declaration(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenKind::Struct)?;
        let name = self.consume_identifier()?;
        
        let type_parameters = if self.match_token(TokenKind::LessThan) {
            self.parse_type_parameter_list()?
        } else {
            Vec::new()
        };
        
        self.consume(TokenKind::LeftBrace)?;
        let fields = self.parse_field_list()?;
        self.consume(TokenKind::RightBrace)?;
        
        Ok(Statement::StructDeclaration {
            name,
            fields,
            methods: Vec::new(),
            is_generic: !type_parameters.is_empty(),
            type_parameters,
        })
    }

    fn parse_implementation(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenKind::Impl)?;
        let type_name = self.parse_type()?;
        
        self.consume(TokenKind::LeftBrace)?;
        let mut methods = Vec::new();
        
        while !self.is_at_end() && self.current_token().kind != TokenKind::RightBrace {
            if let Statement::FunctionDeclaration {
                name,
                parameters,
                return_type,
                body,
                is_generic,
                type_parameters,
            } = self.parse_function_declaration()? {
                methods.push(FunctionDeclaration {
                    name,
                    parameters,
                    return_type,
                    body,
                    is_generic,
                    type_parameters,
                });
            }
        }
        
        self.consume(TokenKind::RightBrace)?;
        
        Ok(Statement::Implementation {
            type_name,
            methods,
        })
    }

    fn parse_return_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenKind::Return)?;
        let value = if !self.match_token(TokenKind::Semicolon) && !self.is_at_end() {
            Some(self.parse_expression()?)
        } else {
            None
        };
        self.consume_semicolon()?;
        
        Ok(Statement::Return { value })
    }

    fn parse_if_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenKind::If)?;
        let condition = self.parse_expression()?;
        self.consume(TokenKind::LeftBrace)?;
        let then_branch = self.parse_statement_block()?;
        self.consume(TokenKind::RightBrace)?;
        
        let else_branch = if self.match_token(TokenKind::Else) {
            if self.match_token(TokenKind::If) {
                Some(ElseBranch::ElseIf {
                    condition: self.parse_expression()?,
                    then_branch: {
                        self.consume(TokenKind::LeftBrace)?;
                        let block = self.parse_statement_block()?;
                        self.consume(TokenKind::RightBrace)?;
                        block
                    },
                    else_branch: None,
                })
            } else {
                self.consume(TokenKind::LeftBrace)?;
                let block = self.parse_statement_block()?;
                self.consume(TokenKind::RightBrace)?;
                Some(ElseBranch::Else(block))
            }
        } else {
            None
        };
        
        Ok(Statement::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn parse_while_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenKind::While)?;
        let condition = self.parse_expression()?;
        self.consume(TokenKind::LeftBrace)?;
        let body = self.parse_statement_block()?;
        self.consume(TokenKind::RightBrace)?;
        
        Ok(Statement::While { condition, body })
    }

    fn parse_for_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenKind::For)?;
        let variable = self.consume_identifier()?;
        self.consume(TokenKind::In)?;
        let iterable = self.parse_expression()?;
        self.consume(TokenKind::LeftBrace)?;
        let body = self.parse_statement_block()?;
        self.consume(TokenKind::RightBrace)?;
        
        Ok(Statement::For {
            variable,
            iterable,
            body,
        })
    }

    fn parse_match_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenKind::Match)?;
        let expression = self.parse_expression()?;
        self.consume(TokenKind::LeftBrace)?;
        
        let mut arms = Vec::new();
        while !self.is_at_end() && self.current_token().kind != TokenKind::RightBrace {
            let pattern = self.parse_pattern()?;
            self.consume(TokenKind::Arrow)?;
            self.consume(TokenKind::LeftBrace)?;
            let body = self.parse_statement_block()?;
            self.consume(TokenKind::RightBrace)?;
            
            arms.push(MatchArm { pattern, body });
        }
        
        self.consume(TokenKind::RightBrace)?;
        
        Ok(Statement::Match { expression, arms })
    }

    fn parse_break_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenKind::Break)?;
        self.consume_semicolon()?;
        Ok(Statement::Break)
    }

    fn parse_continue_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenKind::Continue)?;
        self.consume_semicolon()?;
        Ok(Statement::Continue)
    }

    fn parse_block_statement(&mut self) -> Result<Statement, ParseError> {
        self.consume(TokenKind::LeftBrace)?;
        let statements = self.parse_statement_block()?;
        self.consume(TokenKind::RightBrace)?;
        Ok(Statement::Block(statements))
    }

    fn parse_expression_statement(&mut self) -> Result<Statement, ParseError> {
        let expression = self.parse_expression()?;
        self.consume_semicolon()?;
        Ok(Statement::ExpressionStatement(expression))
    }

    fn parse_expression(&mut self) -> Result<Expression, ParseError> {
        self.parse_assignment()
    }

    fn parse_assignment(&mut self) -> Result<Expression, ParseError> {
        let left = self.parse_logical_or()?;
        
        if self.match_token(TokenKind::Assign) {
            let right = self.parse_assignment()?;
            Ok(Expression::Binary {
                left: Box::new(left),
                operator: BinaryOperator::Add, // This should be a separate assignment operator
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn parse_logical_or(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_logical_and()?;
        
        while self.match_token(TokenKind::Or) {
            let right = self.parse_logical_and()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator: BinaryOperator::Or,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_logical_and(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_equality()?;
        
        while self.match_token(TokenKind::And) {
            let right = self.parse_equality()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator: BinaryOperator::And,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_equality(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_comparison()?;
        
        while self.match_token(TokenKind::Equals) || self.match_token(TokenKind::NotEquals) {
            let operator = if self.previous_token().kind == TokenKind::Equals {
                BinaryOperator::Equal
            } else {
                BinaryOperator::NotEqual
            };
            let right = self.parse_comparison()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_comparison(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_term()?;
        
        while self.match_token(TokenKind::LessThan) || self.match_token(TokenKind::GreaterThan)
            || self.match_token(TokenKind::LessThanOrEqual) || self.match_token(TokenKind::GreaterThanOrEqual)
        {
            let operator = match self.previous_token().kind {
                TokenKind::LessThan => BinaryOperator::LessThan,
                TokenKind::GreaterThan => BinaryOperator::GreaterThan,
                TokenKind::LessThanOrEqual => BinaryOperator::LessThanOrEqual,
                TokenKind::GreaterThanOrEqual => BinaryOperator::GreaterThanOrEqual,
                _ => unreachable!(),
            };
            let right = self.parse_term()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_factor()?;
        
        while self.match_token(TokenKind::Plus) || self.match_token(TokenKind::Minus) {
            let operator = if self.previous_token().kind == TokenKind::Plus {
                BinaryOperator::Add
            } else {
                BinaryOperator::Subtract
            };
            let right = self.parse_factor()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_factor(&mut self) -> Result<Expression, ParseError> {
        let mut left = self.parse_unary()?;
        
        while self.match_token(TokenKind::Multiply) || self.match_token(TokenKind::Divide) || self.match_token(TokenKind::Modulo) {
            let operator = match self.previous_token().kind {
                TokenKind::Multiply => BinaryOperator::Multiply,
                TokenKind::Divide => BinaryOperator::Divide,
                TokenKind::Modulo => BinaryOperator::Modulo,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;
            left = Expression::Binary {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, ParseError> {
        if self.match_token(TokenKind::Minus) || self.match_token(TokenKind::Not) {
            let operator = if self.previous_token().kind == TokenKind::Minus {
                UnaryOperator::Negate
            } else {
                UnaryOperator::Not
            };
            let expression = self.parse_unary()?;
            Ok(Expression::Unary {
                operator,
                expression: Box::new(expression),
            })
        } else {
            self.parse_primary()
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, ParseError> {
        match self.current_token().kind {
            TokenKind::Integer => {
                let value = self.current_token().lexeme.parse().unwrap();
                self.advance();
                Ok(Expression::Literal(LiteralValue::Integer(value)))
            }
            TokenKind::Float => {
                let value = self.current_token().lexeme.parse().unwrap();
                self.advance();
                Ok(Expression::Literal(LiteralValue::Float(value)))
            }
            TokenKind::String => {
                let value = self.current_token().lexeme.clone();
                self.advance();
                Ok(Expression::Literal(LiteralValue::String(value)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Expression::Literal(LiteralValue::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Expression::Literal(LiteralValue::Bool(false)))
            }
            TokenKind::Identifier => {
                let name = self.current_token().lexeme.clone();
                self.advance();
                self.parse_identifier_expression(name)
            }
            TokenKind::LeftParen => {
                self.advance();
                let expression = self.parse_expression()?;
                self.consume(TokenKind::RightParen)?;
                Ok(expression)
            }
            TokenKind::LeftBracket => self.parse_list_literal(),
            _ => Err(ParseError::InvalidExpression {
                line: self.current_token().line,
                column: self.current_token().column,
            }),
        }
    }

    fn parse_identifier_expression(&mut self, name: String) -> Result<Expression, ParseError> {
        if self.match_token(TokenKind::LeftParen) {
            // Function call
            let mut arguments = Vec::new();
            if !self.match_token(TokenKind::RightParen) {
                arguments.push(self.parse_expression()?);
                while self.match_token(TokenKind::Comma) {
                    arguments.push(self.parse_expression()?);
                }
                self.consume(TokenKind::RightParen)?;
            }
            Ok(Expression::Call {
                callee: Box::new(Expression::Identifier(name)),
                arguments,
            })
        } else {
            Ok(Expression::Identifier(name))
        }
    }

    fn parse_list_literal(&mut self) -> Result<Expression, ParseError> {
        self.consume(TokenKind::LeftBracket)?;
        let mut elements = Vec::new();
        
        if !self.match_token(TokenKind::RightBracket) {
            elements.push(self.parse_expression()?);
            while self.match_token(TokenKind::Comma) {
                elements.push(self.parse_expression()?);
            }
            self.consume(TokenKind::RightBracket)?;
        }
        
        Ok(Expression::List {
            elements,
            element_type: None,
        })
    }

    fn parse_type(&mut self) -> Result<Type, ParseError> {
        match self.current_token().kind {
            TokenKind::Int => {
                self.advance();
                Ok(Type::int())
            }
            TokenKind::FloatType => {
                self.advance();
                Ok(Type::float())
            }
            TokenKind::StringType => {
                self.advance();
                Ok(Type::string())
            }
            TokenKind::BoolType => {
                self.advance();
                Ok(Type::bool())
            }
            TokenKind::List => {
                self.advance();
                self.consume(TokenKind::LessThan)?;
                let element_type = self.parse_type()?;
                self.consume(TokenKind::GreaterThan)?;
                Ok(Type::list(element_type))
            }
            TokenKind::Identifier => {
                let name = self.current_token().lexeme.clone();
                self.advance();
                Ok(Type::Struct {
                    name,
                    type_arguments: Vec::new(),
                })
            }
            _ => Err(ParseError::InvalidExpression {
                line: self.current_token().line,
                column: self.current_token().column,
            }),
        }
    }

    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> {
        match self.current_token().kind {
            TokenKind::Integer => {
                let value = self.current_token().lexeme.parse().unwrap();
                self.advance();
                Ok(Pattern::Literal(LiteralValue::Integer(value)))
            }
            TokenKind::String => {
                let value = self.current_token().lexeme.clone();
                self.advance();
                Ok(Pattern::Literal(LiteralValue::String(value)))
            }
            TokenKind::True => {
                self.advance();
                Ok(Pattern::Literal(LiteralValue::Bool(true)))
            }
            TokenKind::False => {
                self.advance();
                Ok(Pattern::Literal(LiteralValue::Bool(false)))
            }
            TokenKind::Identifier => {
                let name = self.current_token().lexeme.clone();
                self.advance();
                Ok(Pattern::Identifier(name))
            }
            _ => Err(ParseError::InvalidExpression {
                line: self.current_token().line,
                column: self.current_token().column,
            }),
        }
    }

    fn parse_parameter_list(&mut self) -> Result<Vec<Parameter>, ParseError> {
        let mut parameters = Vec::new();
        
        if !self.match_token(TokenKind::RightParen) {
            loop {
                let name = self.consume_identifier()?;
                let type_annotation = if self.match_token(TokenKind::Colon) {
                    Some(self.parse_type()?)
                } else {
                    None
                };
                
                parameters.push(Parameter {
                    name,
                    type_annotation,
                    is_mutable: false,
                });
                
                if !self.match_token(TokenKind::Comma) {
                    break;
                }
            }
        }
        
        Ok(parameters)
    }

    fn parse_field_list(&mut self) -> Result<Vec<Field>, ParseError> {
        let mut fields = Vec::new();
        
        while !self.is_at_end() && self.current_token().kind != TokenKind::RightBrace {
            let name = self.consume_identifier()?;
            self.consume(TokenKind::Colon)?;
            let type_annotation = self.parse_type()?;
            self.consume_semicolon()?;
            
            fields.push(Field {
                name,
                type_annotation,
                is_mutable: false,
            });
        }
        
        Ok(fields)
    }

    fn parse_type_parameter_list(&mut self) -> Result<Vec<String>, ParseError> {
        let mut type_parameters = Vec::new();
        
        loop {
            let name = self.consume_identifier()?;
            type_parameters.push(name);
            
            if !self.match_token(TokenKind::Comma) {
                break;
            }
        }
        
        self.consume(TokenKind::GreaterThan)?;
        Ok(type_parameters)
    }

    fn parse_statement_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut statements = Vec::new();
        
        while !self.is_at_end() && self.current_token().kind != TokenKind::RightBrace {
            if self.current_token().kind == TokenKind::Newline {
                self.advance();
                continue;
            }
            
            let statement = self.parse_statement()?;
            statements.push(statement);
        }
        
        Ok(statements)
    }

    fn consume_identifier(&mut self) -> Result<String, ParseError> {
        if self.current_token().kind == TokenKind::Identifier {
            let name = self.current_token().lexeme.clone();
            self.advance();
            Ok(name)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: TokenKind::Identifier,
                actual: self.current_token().clone(),
                line: self.current_token().line,
                column: self.current_token().column,
            })
        }
    }

    fn consume(&mut self, kind: TokenKind) -> Result<&Token, ParseError> {
        if self.current_token().kind == kind {
            let token = &self.tokens[self.position];
            self.advance();
            Ok(token)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: kind,
                actual: self.current_token().clone(),
                line: self.current_token().line,
                column: self.current_token().column,
            })
        }
    }

    fn consume_semicolon(&mut self) -> Result<(), ParseError> {
        if self.current_token().kind == TokenKind::Semicolon {
            self.advance();
            Ok(())
        } else if self.current_token().kind == TokenKind::Newline {
            // Allow newlines as statement terminators
            Ok(())
        } else {
            Err(ParseError::UnexpectedToken {
                expected: TokenKind::Semicolon,
                actual: self.current_token().clone(),
                line: self.current_token().line,
                column: self.current_token().column,
            })
        }
    }

    fn match_token(&mut self, kind: TokenKind) -> bool {
        if self.current_token().kind == kind {
            self.advance();
            true
        } else {
            false
        }
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn current_token(&self) -> &Token {
        &self.tokens[self.position]
    }

    fn previous_token(&self) -> &Token {
        &self.tokens[self.position - 1]
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() || self.current_token().kind == TokenKind::Eof
    }
}

pub fn parse(source: &str) -> Result<Program, ParseError> {
    let lexer = Lexer::new(source.to_string());
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_expression() {
        let source = "1 + 2 * 3";
        let program = parse(source).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_variable_declaration() {
        let source = "let x = 42;";
        let program = parse(source).unwrap();
        assert_eq!(program.statements.len(), 1);
    }

    #[test]
    fn test_function_declaration() {
        let source = "fn add(a: Int, b: Int) -> Int { a + b }";
        let program = parse(source).unwrap();
        assert_eq!(program.statements.len(), 1);
    }
}
