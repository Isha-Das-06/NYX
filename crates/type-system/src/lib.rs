use nyx_ast::*;
use thiserror::Error;
use std::collections::HashMap;
use indexmap::IndexMap;
use petgraph::{Graph, Direction};

#[derive(Debug, Error)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, got {actual} at {location}")]
    TypeMismatch {
        expected: Type,
        actual: Type,
        location: String,
    },
    
    #[error("Undefined variable '{name}' at {location}")]
    UndefinedVariable { name: String, location: String },
    
    #[error("Undefined function '{name}' at {location}")]
    UndefinedFunction { name: String, location: String },
    
    #[error("Undefined type '{name}' at {location}")]
    UndefinedType { name: String, location: String },
    
    #[error("Wrong number of arguments: expected {expected}, got {actual} for function '{name}' at {location}")]
    WrongArgumentCount {
        name: String,
        expected: usize,
        actual: usize,
        location: String,
    },
    
    #[error("Cannot assign to immutable variable '{name}' at {location}")]
    CannotAssignImmutable { name: String, location: String },
    
    #[error("Break statement outside of loop at {location}")]
    BreakOutsideLoop { location: String },
    
    #[error("Continue statement outside of loop at {location}")]
    ContinueOutsideLoop { location: String },
    
    #[error("Return statement outside of function at {location}")]
    ReturnOutsideFunction { location: String },
    
    #[error("Non-exhaustive match at {location}")]
    NonExhaustiveMatch { location: String },
    
    #[error("Generic constraint violation: {message} at {location}")]
    GenericConstraintViolation { message: String, location: String },
    
    #[error("Recursive type definition: {type_name} at {location}")]
    RecursiveTypeDefinition { type_name: String, location: String },
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub type_info: Type,
    pub is_mutable: bool,
    pub kind: SymbolKind,
    pub scope_level: usize,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Variable,
    Function {
        parameters: Vec<Type>,
        return_type: Type,
        is_generic: bool,
        type_parameters: Vec<String>,
    },
    Struct {
        fields: HashMap<String, Type>,
        methods: HashMap<String, Symbol>,
        is_generic: bool,
        type_parameters: Vec<String>,
    },
    TypeParameter,
}

#[derive(Debug)]
pub struct Scope {
    pub symbols: IndexMap<String, Symbol>,
    pub parent: Option<usize>,
    pub level: usize,
    pub is_function_scope: bool,
    pub is_loop_scope: bool,
}

impl Scope {
    pub fn new(level: usize, parent: Option<usize>) -> Self {
        Self {
            symbols: IndexMap::new(),
            parent,
            level,
            is_function_scope: false,
            is_loop_scope: false,
        }
    }

    pub fn with_flags(level: usize, parent: Option<usize>, is_function_scope: bool, is_loop_scope: bool) -> Self {
        Self {
            symbols: IndexMap::new(),
            parent,
            level,
            is_function_scope,
            is_loop_scope,
        }
    }
}

#[derive(Debug)]
pub struct TypeChecker {
    scopes: Vec<Scope>,
    current_scope: usize,
    type_graph: Graph<Type, ()>,
    type_constraints: HashMap<String, Type>,
    generic_instances: HashMap<String, HashMap<Vec<Type>, Type>>,
    errors: Vec<TypeError>,
}

impl TypeChecker {
    pub fn new() -> Self {
        let mut type_checker = Self {
            scopes: Vec::new(),
            current_scope: 0,
            type_graph: Graph::new(),
            type_constraints: HashMap::new(),
            generic_instances: HashMap::new(),
            errors: Vec::new(),
        };
        
        // Create global scope
        type_checker.scopes.push(Scope::new(0, None));
        type_checker.add_builtin_types();
        type_checker
    }

    pub fn check(&mut self, program: &Program) -> Result<(), Vec<TypeError>> {
        self.errors.clear();
        
        // Check all top-level declarations
        for statement in &program.statements {
            self.check_statement(statement);
        }
        
        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn add_builtin_types(&mut self) {
        let builtins = vec![
            ("Int", Type::int()),
            ("Float", Type::float()),
            ("String", Type::string()),
            ("Bool", Type::bool()),
            ("Void", Type::void()),
        ];
        
        for (name, type_info) in builtins {
            let symbol = Symbol {
                name: name.to_string(),
                type_info: type_info.clone(),
                is_mutable: false,
                kind: SymbolKind::Struct {
                    fields: HashMap::new(),
                    methods: HashMap::new(),
                    is_generic: false,
                    type_parameters: Vec::new(),
                },
                scope_level: 0,
            };
            self.add_symbol_to_current_scope(symbol);
        }
    }

    fn check_statement(&mut self, statement: &Statement) {
        match statement {
            Statement::VariableDeclaration { name, type_annotation, initializer, is_mutable } => {
                self.check_variable_declaration(name, type_annotation, initializer, *is_mutable);
            }
            Statement::FunctionDeclaration { name, parameters, return_type, body, is_generic, type_parameters } => {
                self.check_function_declaration(name, parameters, return_type, body, *is_generic, type_parameters);
            }
            Statement::StructDeclaration { name, fields, methods, is_generic, type_parameters } => {
                self.check_struct_declaration(name, fields, methods, *is_generic, type_parameters);
            }
            Statement::Implementation { type_name, methods } => {
                self.check_implementation(type_name, methods);
            }
            Statement::ExpressionStatement(expression) => {
                self.check_expression(expression);
            }
            Statement::Return { value } => {
                self.check_return_statement(value);
            }
            Statement::If { condition, then_branch, else_branch } => {
                self.check_if_statement(condition, then_branch, else_branch);
            }
            Statement::While { condition, body } => {
                self.check_while_statement(condition, body);
            }
            Statement::For { variable, iterable, body } => {
                self.check_for_statement(variable, iterable, body);
            }
            Statement::Match { expression, arms } => {
                self.check_match_statement(expression, arms);
            }
            Statement::Break => {
                self.check_break_statement();
            }
            Statement::Continue => {
                self.check_continue_statement();
            }
            Statement::Block(statements) => {
                self.check_block(statements);
            }
        }
    }

    fn check_variable_declaration(&mut self, name: &str, type_annotation: &Option<Type>, initializer: &Option<Expression>, is_mutable: bool) {
        let initializer_type = if let Some(initializer) = initializer {
            Some(self.check_expression(initializer))
        } else {
            None
        };

        let final_type = if let Some(type_annotation) = type_annotation {
            if let Some(initializer_type) = initializer_type {
                if !self.types_equal(type_annotation, &initializer_type) {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: type_annotation.clone(),
                        actual: initializer_type,
                        location: format!("variable declaration for '{}'", name),
                    });
                }
            }
            type_annotation.clone()
        } else if let Some(initializer_type) = initializer_type {
            initializer_type
        } else {
            Type::Unknown
        };

        let symbol = Symbol {
            name: name.to_string(),
            type_info: final_type,
            is_mutable,
            kind: SymbolKind::Variable,
            scope_level: self.scopes[self.current_scope].level,
        };

        self.add_symbol_to_current_scope(symbol);
    }

    fn check_function_declaration(&mut self, name: &str, parameters: &[Parameter], return_type: &Option<Type>, body: &[Statement], is_generic: bool, type_parameters: &[String]) {
        let param_types: Vec<Type> = parameters.iter()
            .map(|p| p.type_annotation.clone().unwrap_or(Type::Unknown))
            .collect();

        let return_type = return_type.clone().unwrap_or(Type::void());

        // Create new scope for function
        let function_scope = self.scopes.len();
        self.scopes.push(Scope::with_flags(
            self.scopes[self.current_scope].level + 1,
            Some(self.current_scope),
            true,
            false,
        ));

        // Add type parameters to scope if generic
        for type_param in type_parameters {
            let symbol = Symbol {
                name: type_param.clone(),
                type_info: Type::generic(type_param.clone()),
                is_mutable: false,
                kind: SymbolKind::TypeParameter,
                scope_level: self.scopes[function_scope].level,
            };
            self.add_symbol_to_scope(function_scope, symbol);
        }

        // Add parameters to scope
        for (i, param) in parameters.iter().enumerate() {
            let param_type = param.type_annotation.clone().unwrap_or(Type::Unknown);
            let symbol = Symbol {
                name: param.name.clone(),
                type_info: param_type,
                is_mutable: param.is_mutable,
                kind: SymbolKind::Variable,
                scope_level: self.scopes[function_scope].level,
            };
            self.add_symbol_to_scope(function_scope, symbol);
        }

        // Check function body
        let old_scope = self.current_scope;
        self.current_scope = function_scope;
        for statement in body {
            self.check_statement(statement);
        }
        self.current_scope = old_scope;

        // Add function symbol to current scope
        let symbol = Symbol {
            name: name.to_string(),
            type_info: Type::function(param_types, return_type),
            is_mutable: false,
            kind: SymbolKind::Function {
                parameters: param_types,
                return_type,
                is_generic,
                type_parameters: type_parameters.to_vec(),
            },
            scope_level: self.scopes[self.current_scope].level,
        };

        self.add_symbol_to_current_scope(symbol);
    }

    fn check_struct_declaration(&mut self, name: &str, fields: &[Field], methods: &[FunctionDeclaration], is_generic: bool, type_parameters: &[String]) {
        let mut field_types = HashMap::new();
        
        for field in fields {
            field_types.insert(field.name.clone(), field.type_annotation.clone());
        }

        // Create new scope for struct methods
        let struct_scope = self.scopes.len();
        self.scopes.push(Scope::new(
            self.scopes[self.current_scope].level + 1,
            Some(self.current_scope),
        ));

        // Add type parameters to scope if generic
        for type_param in type_parameters {
            let symbol = Symbol {
                name: type_param.clone(),
                type_info: Type::generic(type_param.clone()),
                is_mutable: false,
                kind: SymbolKind::TypeParameter,
                scope_level: self.scopes[struct_scope].level,
            };
            self.add_symbol_to_scope(struct_scope, symbol);
        }

        // Add struct symbol to current scope
        let symbol = Symbol {
            name: name.to_string(),
            type_info: Type::Struct {
                name: name.to_string(),
                type_arguments: Vec::new(),
            },
            is_mutable: false,
            kind: SymbolKind::Struct {
                fields: field_types,
                methods: HashMap::new(),
                is_generic,
                type_parameters: type_parameters.to_vec(),
            },
            scope_level: self.scopes[self.current_scope].level,
        };

        self.add_symbol_to_current_scope(symbol);
    }

    fn check_implementation(&mut self, type_name: &Type, methods: &[FunctionDeclaration]) {
        // Implementation checking would go here
        // For now, just check method bodies
        for method in methods {
            self.check_function_declaration(
                &method.name,
                &method.parameters,
                &method.return_type,
                &method.body,
                method.is_generic,
                &method.type_parameters,
            );
        }
    }

    fn check_return_statement(&mut self, value: &Option<Expression>) {
        if !self.scopes[self.current_scope].is_function_scope {
            self.errors.push(TypeError::ReturnOutsideFunction {
                location: "return statement".to_string(),
            });
            return;
        }

        if let Some(value) = value {
            self.check_expression(value);
        }
    }

    fn check_if_statement(&mut self, condition: &Expression, then_branch: &[Statement], else_branch: &Option<ElseBranch>) {
        let condition_type = self.check_expression(condition);
        if !self.is_bool_type(&condition_type) {
            self.errors.push(TypeError::TypeMismatch {
                expected: Type::bool(),
                actual: condition_type,
                location: "if condition".to_string(),
            });
        }

        // Check then branch
        let then_scope = self.create_new_scope();
        for statement in then_branch {
            self.check_statement(statement);
        }
        self.exit_scope(then_scope);

        // Check else branch
        if let Some(else_branch) = else_branch {
            match else_branch {
                ElseBranch::Else(statements) => {
                    let else_scope = self.create_new_scope();
                    for statement in statements {
                        self.check_statement(statement);
                    }
                    self.exit_scope(else_scope);
                }
                ElseBranch::ElseIf { condition, then_branch, else_branch } => {
                    let condition_type = self.check_expression(condition);
                    if !self.is_bool_type(&condition_type) {
                        self.errors.push(TypeError::TypeMismatch {
                            expected: Type::bool(),
                            actual: condition_type,
                            location: "else if condition".to_string(),
                        });
                    }

                    let elseif_scope = self.create_new_scope();
                    for statement in then_branch {
                        self.check_statement(statement);
                    }
                    self.exit_scope(elseif_scope);
                }
            }
        }
    }

    fn check_while_statement(&mut self, condition: &Expression, body: &[Statement]) {
        let condition_type = self.check_expression(condition);
        if !self.is_bool_type(&condition_type) {
            self.errors.push(TypeError::TypeMismatch {
                expected: Type::bool(),
                actual: condition_type,
                location: "while condition".to_string(),
            });
        }

        let loop_scope = self.scopes.len();
        self.scopes.push(Scope::with_flags(
            self.scopes[self.current_scope].level + 1,
            Some(self.current_scope),
            false,
            true,
        ));

        let old_scope = self.current_scope;
        self.current_scope = loop_scope;
        for statement in body {
            self.check_statement(statement);
        }
        self.current_scope = old_scope;
    }

    fn check_for_statement(&mut self, variable: &str, iterable: &Expression, body: &[Statement]) {
        let iterable_type = self.check_expression(iterable);
        
        // Check if iterable is a list
        let element_type = match &iterable_type {
            Type::List(element_type) => *element_type.clone(),
            _ => {
                self.errors.push(TypeError::TypeMismatch {
                    expected: Type::list(Type::Unknown),
                    actual: iterable_type,
                    location: "for loop iterable".to_string(),
                });
                Type::Unknown
            }
        };

        let loop_scope = self.scopes.len();
        self.scopes.push(Scope::with_flags(
            self.scopes[self.current_scope].level + 1,
            Some(self.current_scope),
            false,
            true,
        ));

        // Add loop variable to scope
        let symbol = Symbol {
            name: variable.to_string(),
            type_info: element_type,
            is_mutable: false,
            kind: SymbolKind::Variable,
            scope_level: self.scopes[loop_scope].level,
        };
        self.add_symbol_to_scope(loop_scope, symbol);

        let old_scope = self.current_scope;
        self.current_scope = loop_scope;
        for statement in body {
            self.check_statement(statement);
        }
        self.current_scope = old_scope;
    }

    fn check_match_statement(&mut self, expression: &Expression, arms: &[MatchArm]) {
        let expression_type = self.check_expression(expression);
        
        for arm in arms {
            self.check_pattern(&arm.pattern, &expression_type);
            
            let arm_scope = self.create_new_scope();
            for statement in &arm.body {
                self.check_statement(statement);
            }
            self.exit_scope(arm_scope);
        }
    }

    fn check_break_statement(&mut self) {
        if !self.scopes[self.current_scope].is_loop_scope {
            self.errors.push(TypeError::BreakOutsideLoop {
                location: "break statement".to_string(),
            });
        }
    }

    fn check_continue_statement(&mut self) {
        if !self.scopes[self.current_scope].is_loop_scope {
            self.errors.push(TypeError::ContinueOutsideLoop {
                location: "continue statement".to_string(),
            });
        }
    }

    fn check_block(&mut self, statements: &[Statement]) {
        let block_scope = self.create_new_scope();
        for statement in statements {
            self.check_statement(statement);
        }
        self.exit_scope(block_scope);
    }

    fn check_expression(&mut self, expression: &Expression) -> Type {
        match expression {
            Expression::Literal(literal) => self.check_literal(literal),
            Expression::Identifier(name) => self.check_identifier(name),
            Expression::Binary { left, operator, right } => {
                self.check_binary_expression(left, operator, right)
            }
            Expression::Unary { operator, expression } => {
                self.check_unary_expression(operator, expression)
            }
            Expression::Call { callee, arguments } => {
                self.check_function_call(callee, arguments)
            }
            Expression::List { elements, element_type } => {
                self.check_list_literal(elements, element_type)
            }
            Expression::Map { entries, value_type } => {
                self.check_map_literal(entries, value_type)
            }
            Expression::Access { object, property } => {
                self.check_property_access(object, property)
            }
            Expression::Index { object, index } => {
                self.check_index_access(object, index)
            }
            Expression::Conditional { condition, then_branch, else_branch } => {
                self.check_conditional_expression(condition, then_branch, else_branch)
            }
            Expression::Lambda { parameters, return_type, body } => {
                self.check_lambda(parameters, return_type, body)
            }
            Expression::TypeCast { expression, target_type } => {
                self.check_type_cast(expression, target_type)
            }
        }
    }

    fn check_literal(&self, literal: &LiteralValue) -> Type {
        match literal {
            LiteralValue::Integer(_) => Type::int(),
            LiteralValue::Float(_) => Type::float(),
            LiteralValue::String(_) => Type::string(),
            LiteralValue::Bool(_) => Type::bool(),
            LiteralValue::Null => Type::Unknown,
        }
    }

    fn check_identifier(&mut self, name: &str) -> Type {
        if let Some(symbol) = self.lookup_symbol(name) {
            symbol.type_info.clone()
        } else {
            self.errors.push(TypeError::UndefinedVariable {
                name: name.to_string(),
                location: format!("identifier '{}'", name),
            });
            Type::Unknown
        }
    }

    fn check_binary_expression(&mut self, left: &Expression, operator: &BinaryOperator, right: &Expression) -> Type {
        let left_type = self.check_expression(left);
        let right_type = self.check_expression(right);

        match operator {
            BinaryOperator::Add | BinaryOperator::Subtract | BinaryOperator::Multiply | BinaryOperator::Divide | BinaryOperator::Modulo => {
                if self.is_numeric_type(&left_type) && self.is_numeric_type(&right_type) {
                    // Promote to float if either is float
                    if self.is_float_type(&left_type) || self.is_float_type(&right_type) {
                        Type::float()
                    } else {
                        Type::int()
                    }
                } else {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: Type::int(),
                        actual: left_type,
                        location: "binary operation left operand".to_string(),
                    });
                    Type::Unknown
                }
            }
            BinaryOperator::Equal | BinaryOperator::NotEqual => {
                if self.types_equal(&left_type, &right_type) {
                    Type::bool()
                } else {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: left_type.clone(),
                        actual: right_type,
                        location: "equality comparison".to_string(),
                    });
                    Type::bool()
                }
            }
            BinaryOperator::LessThan | BinaryOperator::GreaterThan | BinaryOperator::LessThanOrEqual | BinaryOperator::GreaterThanOrEqual => {
                if self.is_numeric_type(&left_type) && self.is_numeric_type(&right_type) {
                    Type::bool()
                } else {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: Type::int(),
                        actual: left_type,
                        location: "comparison left operand".to_string(),
                    });
                    Type::Unknown
                }
            }
            BinaryOperator::And | BinaryOperator::Or => {
                if self.is_bool_type(&left_type) && self.is_bool_type(&right_type) {
                    Type::bool()
                } else {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: Type::bool(),
                        actual: left_type,
                        location: "logical operation left operand".to_string(),
                    });
                    Type::Unknown
                }
            }
        }
    }

    fn check_unary_expression(&mut self, operator: &UnaryOperator, expression: &Expression) -> Type {
        let expr_type = self.check_expression(expression);
        
        match operator {
            UnaryOperator::Negate => {
                if self.is_numeric_type(&expr_type) {
                    expr_type
                } else {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: Type::int(),
                        actual: expr_type,
                        location: "unary negation".to_string(),
                    });
                    Type::Unknown
                }
            }
            UnaryOperator::Not => {
                if self.is_bool_type(&expr_type) {
                    Type::bool()
                } else {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: Type::bool(),
                        actual: expr_type,
                        location: "logical not".to_string(),
                    });
                    Type::Unknown
                }
            }
        }
    }

    fn check_function_call(&mut self, callee: &Expression, arguments: &[Expression]) -> Type {
        let callee_type = self.check_expression(callee);
        
        match &callee_type {
            Type::Function { parameters, return_type } => {
                if parameters.len() != arguments.len() {
                    self.errors.push(TypeError::WrongArgumentCount {
                        name: "function".to_string(),
                        expected: parameters.len(),
                        actual: arguments.len(),
                        location: "function call".to_string(),
                    });
                }
                
                for (i, (param_type, arg)) in parameters.iter().zip(arguments.iter()).enumerate() {
                    let arg_type = self.check_expression(arg);
                    if !self.types_equal(param_type, &arg_type) {
                        self.errors.push(TypeError::TypeMismatch {
                            expected: param_type.clone(),
                            actual: arg_type,
                            location: format!("function call argument {}", i + 1),
                        });
                    }
                }
                
                *return_type.clone()
            }
            _ => {
                self.errors.push(TypeError::TypeMismatch {
                    expected: Type::function(Vec::new(), Type::void()),
                    actual: callee_type,
                    location: "function call".to_string(),
                });
                Type::Unknown
            }
        }
    }

    fn check_list_literal(&mut self, elements: &[Expression], element_type: &Option<Box<Type>>) -> Type {
        if elements.is_empty() {
            if let Some(element_type) = element_type {
                Type::list(*element_type.clone())
            } else {
                Type::list(Type::Unknown)
            }
        } else {
            let first_type = self.check_expression(&elements[0]);
            for (i, element) in elements.iter().enumerate().skip(1) {
                let elem_type = self.check_expression(element);
                if !self.types_equal(&first_type, &elem_type) {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: first_type.clone(),
                        actual: elem_type,
                        location: format!("list element {}", i + 1),
                    });
                }
            }
            Type::list(first_type)
        }
    }

    fn check_map_literal(&mut self, entries: &HashMap<String, Expression>, value_type: &Option<Box<Type>>) -> Type {
        let mut inferred_value_type = Type::Unknown;
        
        for (key, value) in entries {
            let value_expr_type = self.check_expression(value);
            if matches!(inferred_value_type, Type::Unknown) {
                inferred_value_type = value_expr_type;
            } else if !self.types_equal(&inferred_value_type, &value_expr_type) {
                self.errors.push(TypeError::TypeMismatch {
                    expected: inferred_value_type.clone(),
                    actual: value_expr_type,
                    location: format!("map value for key '{}'", key),
                });
            }
        }
        
        Type::map(Type::string(), inferred_value_type)
    }

    fn check_property_access(&mut self, object: &Expression, property: &str) -> Type {
        let object_type = self.check_expression(object);
        
        match &object_type {
            Type::Struct { name, type_arguments } => {
                if let Some(symbol) = self.lookup_symbol(name) {
                    if let SymbolKind::Struct { fields, .. } = &symbol.kind {
                        if let Some(field_type) = fields.get(property) {
                            field_type.clone()
                        } else {
                            self.errors.push(TypeError::UndefinedVariable {
                                name: format!("{}.{}", name, property),
                                location: "property access".to_string(),
                            });
                            Type::Unknown
                        }
                    } else {
                        Type::Unknown
                    }
                } else {
                    Type::Unknown
                }
            }
            _ => {
                self.errors.push(TypeError::TypeMismatch {
                    expected: Type::Struct { name: "struct".to_string(), type_arguments: Vec::new() },
                    actual: object_type,
                    location: "property access".to_string(),
                });
                Type::Unknown
            }
        }
    }

    fn check_index_access(&mut self, object: &Expression, index: &Expression) -> Type {
        let object_type = self.check_expression(object);
        let index_type = self.check_expression(index);
        
        if !self.is_int_type(&index_type) {
            self.errors.push(TypeError::TypeMismatch {
                expected: Type::int(),
                actual: index_type,
                location: "index access".to_string(),
            });
        }
        
        match &object_type {
            Type::List(element_type) => *element_type.clone(),
            _ => {
                self.errors.push(TypeError::TypeMismatch {
                    expected: Type::list(Type::Unknown),
                    actual: object_type,
                    location: "index access".to_string(),
                });
                Type::Unknown
            }
        }
    }

    fn check_conditional_expression(&mut self, condition: &Expression, then_branch: &Expression, else_branch: &Option<Box<Expression>>) -> Type {
        let condition_type = self.check_expression(condition);
        if !self.is_bool_type(&condition_type) {
            self.errors.push(TypeError::TypeMismatch {
                expected: Type::bool(),
                actual: condition_type,
                location: "conditional expression condition".to_string(),
            });
        }
        
        let then_type = self.check_expression(then_branch);
        
        if let Some(else_branch) = else_branch {
            let else_type = self.check_expression(else_branch);
            if self.types_equal(&then_type, &else_type) {
                then_type
            } else {
                self.errors.push(TypeError::TypeMismatch {
                    expected: then_type.clone(),
                    actual: else_type,
                    location: "conditional expression branches".to_string(),
                });
                Type::Unknown
            }
        } else {
            Type::void()
        }
    }

    fn check_lambda(&mut self, parameters: &[Parameter], return_type: &Option<Type>, body: &[Statement]) -> Type {
        let param_types: Vec<Type> = parameters.iter()
            .map(|p| p.type_annotation.clone().unwrap_or(Type::Unknown))
            .collect();

        let return_type = return_type.clone().unwrap_or(Type::void());

        // Create new scope for lambda
        let lambda_scope = self.scopes.len();
        self.scopes.push(Scope::with_flags(
            self.scopes[self.current_scope].level + 1,
            Some(self.current_scope),
            true,
            false,
        ));

        // Add parameters to scope
        for param in parameters {
            let param_type = param.type_annotation.clone().unwrap_or(Type::Unknown);
            let symbol = Symbol {
                name: param.name.clone(),
                type_info: param_type,
                is_mutable: param.is_mutable,
                kind: SymbolKind::Variable,
                scope_level: self.scopes[lambda_scope].level,
            };
            self.add_symbol_to_scope(lambda_scope, symbol);
        }

        // Check lambda body
        let old_scope = self.current_scope;
        self.current_scope = lambda_scope;
        for statement in body {
            self.check_statement(statement);
        }
        self.current_scope = old_scope;

        Type::function(param_types, return_type)
    }

    fn check_type_cast(&mut self, expression: &Expression, target_type: &Type) -> Type {
        let expr_type = self.check_expression(expression);
        
        // Basic type checking for casts
        if self.is_valid_cast(&expr_type, target_type) {
            target_type.clone()
        } else {
            self.errors.push(TypeError::TypeMismatch {
                expected: target_type.clone(),
                actual: expr_type,
                location: "type cast".to_string(),
            });
            Type::Unknown
        }
    }

    fn check_pattern(&mut self, pattern: &Pattern, expression_type: &Type) {
        match pattern {
            Pattern::Literal(literal) => {
                let literal_type = self.check_literal(literal);
                if !self.types_equal(&literal_type, expression_type) {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: expression_type.clone(),
                        actual: literal_type,
                        location: "pattern matching".to_string(),
                    });
                }
            }
            Pattern::Identifier(name) => {
                let symbol = Symbol {
                    name: name.clone(),
                    type_info: expression_type.clone(),
                    is_mutable: false,
                    kind: SymbolKind::Variable,
                    scope_level: self.scopes[self.current_scope].level,
                };
                self.add_symbol_to_current_scope(symbol);
            }
            Pattern::Wildcard => {
                // Wildcard matches anything
            }
            Pattern::Struct { name, fields } => {
                // Struct pattern matching
                if let Type::Struct { name: type_name, .. } = expression_type {
                    if name != type_name {
                        self.errors.push(TypeError::TypeMismatch {
                            expected: Type::Struct { name: name.clone(), type_arguments: Vec::new() },
                            actual: expression_type.clone(),
                            location: "struct pattern".to_string(),
                        });
                    }
                } else {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: Type::Struct { name: name.clone(), type_arguments: Vec::new() },
                        actual: expression_type.clone(),
                        location: "struct pattern".to_string(),
                    });
                }
            }
            Pattern::List(patterns) => {
                if let Type::List(element_type) = expression_type {
                    for pattern in patterns {
                        self.check_pattern(pattern, element_type);
                    }
                } else {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: Type::list(Type::Unknown),
                        actual: expression_type.clone(),
                        location: "list pattern".to_string(),
                    });
                }
            }
            Pattern::Tuple(patterns) => {
                if let Type::Tuple(types) = expression_type {
                    if patterns.len() != types.len() {
                        self.errors.push(TypeError::TypeMismatch {
                            expected: Type::tuple(types.clone()),
                            actual: Type::tuple(patterns.iter().map(|_| Type::Unknown).collect()),
                            location: "tuple pattern".to_string(),
                        });
                    } else {
                        for (pattern, type_info) in patterns.iter().zip(types.iter()) {
                            self.check_pattern(pattern, type_info);
                        }
                    }
                } else {
                    self.errors.push(TypeError::TypeMismatch {
                        expected: Type::tuple(vec![]),
                        actual: expression_type.clone(),
                        location: "tuple pattern".to_string(),
                    });
                }
            }
        }
    }

    // Helper methods
    fn create_new_scope(&mut self) -> usize {
        let new_scope = self.scopes.len();
        self.scopes.push(Scope::new(
            self.scopes[self.current_scope].level + 1,
            Some(self.current_scope),
        ));
        new_scope
    }

    fn exit_scope(&mut self, scope: usize) {
        if scope < self.scopes.len() {
            self.scopes.remove(scope);
            if self.current_scope >= self.scopes.len() {
                self.current_scope = self.scopes.len() - 1;
            }
        }
    }

    fn add_symbol_to_current_scope(&mut self, symbol: Symbol) {
        self.add_symbol_to_scope(self.current_scope, symbol);
    }

    fn add_symbol_to_scope(&mut self, scope_id: usize, symbol: Symbol) {
        if scope_id < self.scopes.len() {
            self.scopes[scope_id].symbols.insert(symbol.name.clone(), symbol);
        }
    }

    fn lookup_symbol(&self, name: &str) -> Option<&Symbol> {
        let mut current_scope = self.current_scope;
        
        while current_scope < self.scopes.len() {
            if let Some(symbol) = self.scopes[current_scope].symbols.get(name) {
                return Some(symbol);
            }
            
            if let Some(parent) = self.scopes[current_scope].parent {
                current_scope = parent;
            } else {
                break;
            }
        }
        
        None
    }

    fn types_equal(&self, type1: &Type, type2: &Type) -> bool {
        match (type1, type2) {
            (Type::Primitive(p1), Type::Primitive(p2)) => p1 == p2,
            (Type::Function { parameters: p1, return_type: r1 }, Type::Function { parameters: p2, return_type: r2 }) => {
                p1.len() == p2.len() && p1.iter().zip(p2.iter()).all(|(t1, t2)| self.types_equal(t1, t2)) 
                    && self.types_equal(r1, r2)
            }
            (Type::List(e1), Type::List(e2)) => self.types_equal(e1, e2),
            (Type::Map { key: k1, value: v1 }, Type::Map { key: k2, value: v2 }) => {
                self.types_equal(k1, k2) && self.types_equal(v1, v2)
            }
            (Type::Struct { name: n1, type_arguments: a1 }, Type::Struct { name: n2, type_arguments: a2 }) => {
                n1 == n2 && a1.len() == a2.len() && a1.iter().zip(a2.iter()).all(|(t1, t2)| self.types_equal(t1, t2))
            }
            (Type::Generic(g1), Type::Generic(g2)) => g1 == g2,
            (Type::Tuple(t1), Type::Tuple(t2)) => {
                t1.len() == t2.len() && t1.iter().zip(t2.iter()).all(|(type1, type2)| self.types_equal(type1, type2))
            }
            (Type::Unknown, _) | (_, Type::Unknown) => true,
            _ => false,
        }
    }

    fn is_numeric_type(&self, type_info: &Type) -> bool {
        matches!(type_info, Type::Primitive(PrimitiveType::Int | PrimitiveType::Float))
    }

    fn is_int_type(&self, type_info: &Type) -> bool {
        matches!(type_info, Type::Primitive(PrimitiveType::Int))
    }

    fn is_float_type(&self, type_info: &Type) -> bool {
        matches!(type_info, Type::Primitive(PrimitiveType::Float))
    }

    fn is_bool_type(&self, type_info: &Type) -> bool {
        matches!(type_info, Type::Primitive(PrimitiveType::Bool))
    }

    fn is_valid_cast(&self, from: &Type, to: &Type) -> bool {
        match (from, to) {
            (Type::Primitive(PrimitiveType::Int), Type::Primitive(PrimitiveType::Float)) => true,
            (Type::Primitive(PrimitiveType::Float), Type::Primitive(PrimitiveType::Int)) => true,
            (Type::Primitive(PrimitiveType::Int), Type::Primitive(PrimitiveType::String)) => true,
            (Type::Primitive(PrimitiveType::Float), Type::Primitive(PrimitiveType::String)) => true,
            (Type::Primitive(PrimitiveType::String), Type::Primitive(PrimitiveType::Int)) => true,
            (Type::Primitive(PrimitiveType::String), Type::Primitive(PrimitiveType::Float)) => true,
            _ => self.types_equal(from, to),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyx_parser::parse;

    #[test]
    fn test_basic_type_checking() {
        let source = "let x = 42;";
        let program = parse(source).unwrap();
        let mut type_checker = TypeChecker::new();
        let result = type_checker.check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_function_type_checking() {
        let source = "fn add(a: Int, b: Int) -> Int { a + b }";
        let program = parse(source).unwrap();
        let mut type_checker = TypeChecker::new();
        let result = type_checker.check(&program);
        assert!(result.is_ok());
    }

    #[test]
    fn test_type_mismatch() {
        let source = "let x: Int = \"hello\";";
        let program = parse(source).unwrap();
        let mut type_checker = TypeChecker::new();
        let result = type_checker.check(&program);
        assert!(result.is_err());
    }
}
