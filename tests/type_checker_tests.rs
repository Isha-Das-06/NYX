use nyx_type_system::TypeChecker;
use nyx_parser::parse;

#[test]
fn test_type_checker_basic_types() {
    let source = "let x = 42;";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_function_declaration() {
    let source = "fn add(a: Int, b: Int) -> Int { a + b }";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_type_mismatch() {
    let source = "let x: Int = \"hello\";";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_err());
}

#[test]
fn test_type_checker_undefined_variable() {
    let source = "let y = x + 1;";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_err());
}

#[test]
fn test_type_checker_arithmetic_operations() {
    let source = "
        let a = 1 + 2;
        let b = 3.14 * 2.0;
        let c = a - b;
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_boolean_operations() {
    let source = "
        let a = true && false;
        let b = true || false;
        let c = !true;
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_comparison_operations() {
    let source = "
        let a = 1 < 2;
        let b = 3.14 > 2.0;
        let c = 1 == 2;
        let d = \"hello\" != \"world\";
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_function_call() {
    let source = "
        fn add(a: Int, b: Int) -> Int { a + b }
        let result = add(1, 2);
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_wrong_argument_count() {
    let source = "
        fn add(a: Int, b: Int) -> Int { a + b }
        let result = add(1);
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_err());
}

#[test]
fn test_type_checker_if_statement() {
    let source = "
        let x = 42;
        if x > 0 {
            let y = x * 2;
        } else {
            let y = x / 2;
        }
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_while_statement() {
    let source = "
        let mut i = 0;
        while i < 10 {
            i = i + 1;
        }
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_for_statement() {
    let source = "
        let numbers = [1, 2, 3, 4, 5];
        let mut sum = 0;
        for num in numbers {
            sum = sum + num;
        }
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_list_operations() {
    let source = "
        let numbers = [1, 2, 3, 4, 5];
        let first = numbers[0];
        let empty = [];
        let mixed = [1, \"hello\", 3.14];
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_map_operations() {
    let source = "
        let person = {\"name\": \"Alice\", \"age\": 30};
        let name = person[\"name\"];
        let empty = {};
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_struct_declaration() {
    let source = "
        struct Point {
            x: Float,
            y: Float
        }
        
        let p = Point { x: 1.0, y: 2.0 };
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_recursive_function() {
    let source = "
        fn factorial(n: Int) -> Int {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
        
        let result = factorial(5);
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_generic_function() {
    let source = "
        fn identity<T>(value: T) -> T {
            value
        }
        
        let int_result = identity(42);
        let string_result = identity(\"hello\");
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}

#[test]
fn test_type_checker_generic_struct() {
    let source = "
        struct Container<T> {
            value: T
        }
        
        impl<T> Container<T> {
            fn new(value: T) -> Container<T> {
                Container { value }
            }
        }
        
        let int_container = Container::new(42);
        let string_container = Container::new(\"hello\");
    ";
    let program = parse(source).unwrap();
    let mut type_checker = TypeChecker::new();
    let result = type_checker.check(&program);
    assert!(result.is_ok());
}
