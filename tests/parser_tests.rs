use nyx_parser::parse;
use nyx_ast::*;

#[test]
fn test_parser_simple_expression() {
    let source = "1 + 2 * 3";
    let program = parse(source).unwrap();
    assert_eq!(program.statements.len(), 1);
    
    if let Statement::ExpressionStatement(expr) = &program.statements[0] {
        if let Expression::Binary { left, operator, right } = expr {
            assert_eq!(operator, &BinaryOperator::Add);
            assert_eq!(*left.as_ref(), Expression::Literal(LiteralValue::Integer(1)));
            
            if let Expression::Binary { left: inner_left, operator: inner_op, right: inner_right } = right.as_ref() {
                assert_eq!(inner_op, &BinaryOperator::Multiply);
                assert_eq!(*inner_left.as_ref(), Expression::Literal(LiteralValue::Integer(2)));
                assert_eq!(*inner_right.as_ref(), Expression::Literal(LiteralValue::Integer(3)));
            } else {
                panic!("Expected binary expression");
            }
        } else {
            panic!("Expected binary expression");
        }
    } else {
        panic!("Expected expression statement");
    }
}

#[test]
fn test_parser_variable_declaration() {
    let source = "let x = 42;";
    let program = parse(source).unwrap();
    assert_eq!(program.statements.len(), 1);
    
    if let Statement::VariableDeclaration { name, type_annotation, initializer, is_mutable } = &program.statements[0] {
        assert_eq!(name, "x");
        assert_eq!(type_annotation, &None);
        assert_eq!(is_mutable, &false);
        
        if let Some(initializer) = initializer {
            assert_eq!(initializer.as_ref(), &Expression::Literal(LiteralValue::Integer(42)));
        } else {
            panic!("Expected initializer");
        }
    } else {
        panic!("Expected variable declaration");
    }
}

#[test]
fn test_parser_function_declaration() {
    let source = "fn add(a: Int, b: Int) -> Int { a + b }";
    let program = parse(source).unwrap();
    assert_eq!(program.statements.len(), 1);
    
    if let Statement::FunctionDeclaration { name, parameters, return_type, body, is_generic, type_parameters } = &program.statements[0] {
        assert_eq!(name, "add");
        assert_eq!(parameters.len(), 2);
        assert_eq!(return_type, &Some(Type::int()));
        assert_eq!(is_generic, &false);
        assert_eq!(type_parameters, &vec![]);
        
        assert_eq!(parameters[0].name, "a");
        assert_eq!(parameters[0].type_annotation, Some(Type::int()));
        assert_eq!(parameters[1].name, "b");
        assert_eq!(parameters[1].type_annotation, Some(Type::int()));
        
        assert_eq!(body.len(), 1);
    } else {
        panic!("Expected function declaration");
    }
}

#[test]
fn test_parser_if_statement() {
    let source = "if x > 0 { println(\"positive\") } else { println(\"negative\") }";
    let program = parse(source).unwrap();
    assert_eq!(program.statements.len(), 1);
    
    if let Statement::If { condition, then_branch, else_branch } = &program.statements[0] {
        // Check condition
        if let Expression::Binary { left, operator, right } = condition {
            assert_eq!(operator, &BinaryOperator::GreaterThan);
        } else {
            panic!("Expected binary condition");
        }
        
        // Check then branch
        assert_eq!(then_branch.len(), 1);
        
        // Check else branch
        assert!(else_branch.is_some());
    } else {
        panic!("Expected if statement");
    }
}

#[test]
fn test_parser_while_statement() {
    let source = "while i < 10 { i = i + 1 }";
    let program = parse(source).unwrap();
    assert_eq!(program.statements.len(), 1);
    
    if let Statement::While { condition, body } = &program.statements[0] {
        // Check condition
        if let Expression::Binary { left, operator, right } = condition {
            assert_eq!(operator, &BinaryOperator::LessThan);
        } else {
            panic!("Expected binary condition");
        }
        
        // Check body
        assert_eq!(body.len(), 1);
    } else {
        panic!("Expected while statement");
    }
}

#[test]
fn test_parser_list_literal() {
    let source = "[1, 2, 3, 4, 5]";
    let program = parse(source).unwrap();
    assert_eq!(program.statements.len(), 1);
    
    if let Statement::ExpressionStatement(expr) = &program.statements[0] {
        if let Expression::List { elements, element_type } = expr {
            assert_eq!(elements.len(), 5);
            assert_eq!(element_type, &None);
            
            for (i, element) in elements.iter().enumerate() {
                if let Expression::Literal(LiteralValue::Integer(value)) = element {
                    assert_eq!(*value, (i + 1) as i64);
                } else {
                    panic!("Expected integer literal");
                }
            }
        } else {
            panic!("Expected list expression");
        }
    } else {
        panic!("Expected expression statement");
    }
}

#[test]
fn test_parser_map_literal() {
    let source = "{\"name\": \"Alice\", \"age\": 30}";
    let program = parse(source).unwrap();
    assert_eq!(program.statements.len(), 1);
    
    if let Statement::ExpressionStatement(expr) = &program.statements[0] {
        if let Expression::Map { entries, value_type } = expr {
            assert_eq!(entries.len(), 2);
            assert_eq!(value_type, &None);
            
            assert!(entries.contains_key("name"));
            assert!(entries.contains_key("age"));
            
            if let Some(Expression::Literal(LiteralValue::String(name))) = entries.get("name") {
                assert_eq!(name, "Alice");
            } else {
                panic!("Expected string literal for name");
            }
            
            if let Some(Expression::Literal(LiteralValue::Integer(age))) = entries.get("age") {
                assert_eq!(*age, 30);
            } else {
                panic!("Expected integer literal for age");
            }
        } else {
            panic!("Expected map expression");
        }
    } else {
        panic!("Expected expression statement");
    }
}

#[test]
fn test_parser_unary_expression() {
    let source = "-x";
    let program = parse(source).unwrap();
    assert_eq!(program.statements.len(), 1);
    
    if let Statement::ExpressionStatement(expr) = &program.statements[0] {
        if let Expression::Unary { operator, expression } = expr {
            assert_eq!(operator, &UnaryOperator::Negate);
            assert_eq!(expression.as_ref(), &Expression::Identifier("x".to_string()));
        } else {
            panic!("Expected unary expression");
        }
    } else {
        panic!("Expected expression statement");
    }
}

#[test]
fn test_parser_function_call() {
    let source = "add(1, 2)";
    let program = parse(source).unwrap();
    assert_eq!(program.statements.len(), 1);
    
    if let Statement::ExpressionStatement(expr) = &program.statements[0] {
        if let Expression::Call { callee, arguments } = expr {
            assert_eq!(callee.as_ref(), &Expression::Identifier("add".to_string()));
            assert_eq!(arguments.len(), 2);
            
            assert_eq!(arguments[0], &Expression::Literal(LiteralValue::Integer(1)));
            assert_eq!(arguments[1], &Expression::Literal(LiteralValue::Integer(2)));
        } else {
            panic!("Expected function call");
        }
    } else {
        panic!("Expected expression statement");
    }
}
