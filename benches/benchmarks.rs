use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nyx_lexer::Lexer;
use nyx_parser::parse;
use nyx_type_system::TypeChecker;
use nyx_vm::{VirtualMachine, Chunk, OpCode, Value};

fn bench_lexer(c: &mut Criterion) {
    let source = r#"
        let x = 42;
        let y = 3.14;
        let message = "Hello, World!";
        let numbers = [1, 2, 3, 4, 5];
        let person = {"name": "Alice", "age": 30};
        
        fn add(a: Int, b: Int) -> Int {
            a + b
        }
        
        fn factorial(n: Int) -> Int {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
        
        let result = add(10, 20);
        let fact = factorial(5);
    "#;

    c.bench_function("lexer", |b| {
        b.iter(|| {
            let lexer = Lexer::new(source.to_string());
            black_box(lexer.tokenize().unwrap())
        })
    });
}

fn bench_parser(c: &mut Criterion) {
    let source = r#"
        let x = 42;
        let y = 3.14;
        let message = "Hello, World!";
        let numbers = [1, 2, 3, 4, 5];
        let person = {"name": "Alice", "age": 30};
        
        fn add(a: Int, b: Int) -> Int {
            a + b
        }
        
        fn factorial(n: Int) -> Int {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
        
        let result = add(10, 20);
        let fact = factorial(5);
    "#;

    c.bench_function("parser", |b| {
        b.iter(|| {
            black_box(parse(source).unwrap())
        })
    });
}

fn bench_type_checker(c: &mut Criterion) {
    let source = r#"
        let x = 42;
        let y = 3.14;
        let message = "Hello, World!";
        let numbers = [1, 2, 3, 4, 5];
        let person = {"name": "Alice", "age": 30};
        
        fn add(a: Int, b: Int) -> Int {
            a + b
        }
        
        fn factorial(n: Int) -> Int {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
        
        let result = add(10, 20);
        let fact = factorial(5);
    "#;

    let program = parse(source).unwrap();

    c.bench_function("type_checker", |b| {
        b.iter(|| {
            let mut type_checker = TypeChecker::new();
            black_box(type_checker.check(&program).unwrap())
        })
    });
}

fn bench_vm_execution(c: &mut Criterion) {
    let mut chunk = Chunk::new();
    
    // Create a simple arithmetic expression: (5 + 3) * 2 - 4
    chunk.write(OpCode::LoadInt.to_u8(), 1);
    chunk.code.extend_from_slice(&5i64.to_le_bytes());
    
    chunk.write(OpCode::LoadInt.to_u8(), 2);
    chunk.code.extend_from_slice(&3i64.to_le_bytes());
    
    chunk.write(OpCode::Add.to_u8(), 3);
    
    chunk.write(OpCode::LoadInt.to_u8(), 4);
    chunk.code.extend_from_slice(&2i64.to_le_bytes());
    
    chunk.write(OpCode::Multiply.to_u8(), 5);
    
    chunk.write(OpCode::LoadInt.to_u8(), 6);
    chunk.code.extend_from_slice(&4i64.to_le_bytes());
    
    chunk.write(OpCode::Subtract.to_u8(), 7);
    chunk.write(OpCode::Return.to_u8(), 8);

    c.bench_function("vm_execution", |b| {
        b.iter(|| {
            let mut vm = VirtualMachine::new();
            black_box(vm.interpret(chunk.clone()).unwrap())
        })
    });
}

fn bench_full_compilation(c: &mut Criterion) {
    let source = r#"
        let x = 42;
        let y = 3.14;
        let message = "Hello, World!";
        let numbers = [1, 2, 3, 4, 5];
        let person = {"name": "Alice", "age": 30};
        
        fn add(a: Int, b: Int) -> Int {
            a + b
        }
        
        fn factorial(n: Int) -> Int {
            if n <= 1 {
                1
            } else {
                n * factorial(n - 1)
            }
        }
        
        let result = add(10, 20);
        let fact = factorial(5);
    "#;

    c.bench_function("full_compilation", |b| {
        b.iter(|| {
            // Lexical analysis
            let lexer = Lexer::new(source.to_string());
            let tokens = black_box(lexer.tokenize().unwrap());
            
            // Parsing
            let program = black_box(parse(&tokens).unwrap());
            
            // Type checking
            let mut type_checker = TypeChecker::new();
            black_box(type_checker.check(&program).unwrap());
        })
    });
}

criterion_group!(
    benches,
    bench_lexer,
    bench_parser,
    bench_type_checker,
    bench_vm_execution,
    bench_full_compilation
);

criterion_main!(benches);
