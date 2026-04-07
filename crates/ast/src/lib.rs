use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use nyx_lexer::Token;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(LiteralValue),
    Identifier(String),
    Binary {
        left: Box<Expression>,
        operator: BinaryOperator,
        right: Box<Expression>,
    },
    Unary {
        operator: UnaryOperator,
        expression: Box<Expression>,
    },
    Call {
        callee: Box<Expression>,
        arguments: Vec<Expression>,
    },
    Lambda {
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Vec<Statement>,
    },
    List {
        elements: Vec<Expression>,
        element_type: Option<Box<Type>>,
    },
    Map {
        entries: HashMap<String, Expression>,
        value_type: Option<Box<Type>>,
    },
    Access {
        object: Box<Expression>,
        property: String,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    Conditional {
        condition: Box<Expression>,
        then_branch: Box<Expression>,
        else_branch: Option<Box<Expression>>,
    },
    TypeCast {
        expression: Box<Expression>,
        target_type: Type,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    VariableDeclaration {
        name: String,
        type_annotation: Option<Type>,
        initializer: Option<Expression>,
        is_mutable: bool,
    },
    FunctionDeclaration {
        name: String,
        parameters: Vec<Parameter>,
        return_type: Option<Type>,
        body: Vec<Statement>,
        is_generic: bool,
        type_parameters: Vec<String>,
    },
    StructDeclaration {
        name: String,
        fields: Vec<Field>,
        methods: Vec<FunctionDeclaration>,
        is_generic: bool,
        type_parameters: Vec<String>,
    },
    Implementation {
        type_name: Type,
        methods: Vec<FunctionDeclaration>,
    },
    ExpressionStatement(Expression),
    Return {
        value: Option<Expression>,
    },
    If {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<ElseBranch>,
    },
    While {
        condition: Expression,
        body: Vec<Statement>,
    },
    For {
        variable: String,
        iterable: Expression,
        body: Vec<Statement>,
    },
    Match {
        expression: Expression,
        arms: Vec<MatchArm>,
    },
    Break,
    Continue,
    Block(Vec<Statement>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ElseBranch {
    Else(Vec<Statement>),
    ElseIf {
        condition: Expression,
        then_branch: Vec<Statement>,
        else_branch: Option<Box<ElseBranch>>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Pattern {
    Literal(LiteralValue),
    Identifier(String),
    Wildcard,
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
    },
    List(Vec<Pattern>),
    Tuple(Vec<Pattern>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionDeclaration {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<Type>,
    pub body: Vec<Statement>,
    pub is_generic: bool,
    pub type_parameters: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<Type>,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub type_annotation: Type,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Type {
    Primitive(PrimitiveType),
    Function {
        parameters: Vec<Type>,
        return_type: Box<Type>,
    },
    List(Box<Type>),
    Map {
        key: Box<Type>,
        value: Box<Type>,
    },
    Struct {
        name: String,
        type_arguments: Vec<Type>,
    },
    Generic(String),
    Tuple(Vec<Type>),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PrimitiveType {
    Int,
    Float,
    String,
    Bool,
    Void,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LiteralValue {
    Integer(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Negate,
    Not,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Statement>,
    pub imports: Vec<Import>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub path: String,
    pub alias: Option<String>,
    pub items: Vec<String>,
}

impl Program {
    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
            imports: Vec::new(),
        }
    }
}

impl Expression {
    pub fn literal_int(value: i64) -> Self {
        Self::Literal(LiteralValue::Integer(value))
    }

    pub fn literal_float(value: f64) -> Self {
        Self::Literal(LiteralValue::Float(value))
    }

    pub fn literal_string(value: String) -> Self {
        Self::Literal(LiteralValue::String(value))
    }

    pub fn literal_bool(value: bool) -> Self {
        Self::Literal(LiteralValue::Bool(value))
    }

    pub fn identifier(name: String) -> Self {
        Self::Identifier(name)
    }

    pub fn binary(left: Expression, operator: BinaryOperator, right: Expression) -> Self {
        Self::Binary {
            left: Box::new(left),
            operator,
            right: Box::new(right),
        }
    }

    pub fn unary(operator: UnaryOperator, expression: Expression) -> Self {
        Self::Unary {
            operator,
            expression: Box::new(expression),
        }
    }

    pub fn call(callee: Expression, arguments: Vec<Expression>) -> Self {
        Self::Call {
            callee: Box::new(callee),
            arguments,
        }
    }
}

impl Statement {
    pub fn variable_declaration(name: String, type_annotation: Option<Type>, initializer: Option<Expression>) -> Self {
        Self::VariableDeclaration {
            name,
            type_annotation,
            initializer,
            is_mutable: false,
        }
    }

    pub fn mutable_variable_declaration(name: String, type_annotation: Option<Type>, initializer: Option<Expression>) -> Self {
        Self::VariableDeclaration {
            name,
            type_annotation,
            initializer,
            is_mutable: true,
        }
    }

    pub fn expression_statement(expression: Expression) -> Self {
        Self::ExpressionStatement(expression)
    }

    pub fn return_statement(value: Option<Expression>) -> Self {
        Self::Return { value }
    }
}

impl Type {
    pub fn int() -> Self {
        Self::Primitive(PrimitiveType::Int)
    }

    pub fn float() -> Self {
        Self::Primitive(PrimitiveType::Float)
    }

    pub fn string() -> Self {
        Self::Primitive(PrimitiveType::String)
    }

    pub fn bool() -> Self {
        Self::Primitive(PrimitiveType::Bool)
    }

    pub fn void() -> Self {
        Self::Primitive(PrimitiveType::Void)
    }

    pub fn list(element_type: Type) -> Self {
        Self::List(Box::new(element_type))
    }

    pub fn map(key: Type, value: Type) -> Self {
        Self::Map {
            key: Box::new(key),
            value: Box::new(value),
        }
    }

    pub fn function(parameters: Vec<Type>, return_type: Type) -> Self {
        Self::Function {
            parameters,
            return_type: Box::new(return_type),
        }
    }

    pub fn generic(name: String) -> Self {
        Self::Generic(name)
    }

    pub fn tuple(types: Vec<Type>) -> Self {
        Self::Tuple(types)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expression_creation() {
        let expr = Expression::binary(
            Expression::literal_int(1),
            BinaryOperator::Add,
            Expression::literal_int(2),
        );
        
        if let Expression::Binary { left, operator, right } = expr {
            assert_eq!(operator, BinaryOperator::Add);
            assert_eq!(*left, Expression::literal_int(1));
            assert_eq!(*right, Expression::literal_int(2));
        } else {
            panic!("Expected binary expression");
        }
    }

    #[test]
    fn test_type_creation() {
        let list_type = Type::list(Type::int());
        assert_eq!(list_type, Type::List(Box::new(Type::int())));
    }
}
