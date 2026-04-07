use nyx_lexer::{Lexer, TokenKind};

#[test]
fn test_lexer_basic_tokens() {
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
fn test_lexer_string_literals() {
    let input = "\"hello\" 'a'";
    let lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenize().unwrap();
    
    assert_eq!(tokens[0].kind, TokenKind::String);
    assert_eq!(tokens[0].lexeme, "hello");
    assert_eq!(tokens[1].kind, TokenKind::String);
    assert_eq!(tokens[1].lexeme, "a");
}

#[test]
fn test_lexer_numbers() {
    let input = "42 3.14";
    let lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenize().unwrap();
    
    assert_eq!(tokens[0].kind, TokenKind::Integer);
    assert_eq!(tokens[0].lexeme, "42");
    assert_eq!(tokens[1].kind, TokenKind::Float);
    assert_eq!(tokens[1].lexeme, "3.14");
}

#[test]
fn test_lexer_keywords() {
    let input = "fn if else while for return";
    let lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenize().unwrap();
    
    assert_eq!(tokens[0].kind, TokenKind::Fn);
    assert_eq!(tokens[1].kind, TokenKind::If);
    assert_eq!(tokens[2].kind, TokenKind::Else);
    assert_eq!(tokens[3].kind, TokenKind::While);
    assert_eq!(tokens[4].kind, TokenKind::For);
    assert_eq!(tokens[5].kind, TokenKind::Return);
}

#[test]
fn test_lexer_operators() {
    let input = "+ - * / % == != < > <= >=";
    let lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenize().unwrap();
    
    assert_eq!(tokens[0].kind, TokenKind::Plus);
    assert_eq!(tokens[1].kind, TokenKind::Minus);
    assert_eq!(tokens[2].kind, TokenKind::Multiply);
    assert_eq!(tokens[3].kind, TokenKind::Divide);
    assert_eq!(tokens[4].kind, TokenKind::Modulo);
    assert_eq!(tokens[5].kind, TokenKind::Equals);
    assert_eq!(tokens[6].kind, TokenKind::NotEquals);
    assert_eq!(tokens[7].kind, TokenKind::LessThan);
    assert_eq!(tokens[8].kind, TokenKind::GreaterThan);
    assert_eq!(tokens[9].kind, TokenKind::LessThanOrEqual);
    assert_eq!(tokens[10].kind, TokenKind::GreaterThanOrEqual);
}

#[test]
fn test_lexer_comments() {
    let input = "let x = 42; // This is a comment\nlet y = 43;";
    let lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenize().unwrap();
    
    assert_eq!(tokens[0].kind, TokenKind::Let);
    assert_eq!(tokens[1].kind, TokenKind::Identifier);
    assert_eq!(tokens[2].kind, TokenKind::Assign);
    assert_eq!(tokens[3].kind, TokenKind::Integer);
    assert_eq!(tokens[4].kind, TokenKind::Semicolon);
    assert_eq!(tokens[5].kind, TokenKind::Newline);
    assert_eq!(tokens[6].kind, TokenKind::Let);
    assert_eq!(tokens[7].kind, TokenKind::Identifier);
    assert_eq!(tokens[8].kind, TokenKind::Assign);
    assert_eq!(tokens[9].kind, TokenKind::Integer);
    assert_eq!(tokens[10].kind, TokenKind::Semicolon);
}
