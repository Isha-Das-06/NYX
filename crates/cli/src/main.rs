use clap::{Parser, Subcommand};
use anyhow::{Result, Context};
use std::fs;
use std::io::{self, Write};
use std::path::Path;

use nyx_lexer::Lexer;
use nyx_parser::parse;
use nyx_type_system::TypeChecker;
use nyx_vm::{VirtualMachine, Chunk, OpCode, Value};

#[derive(Parser)]
#[command(name = "nyx")]
#[command(about = "A modern, statically typed programming language")]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a Nyx source file
    Run {
        /// The source file to run
        file: String,
        /// Enable debug output
        #[arg(short, long)]
        debug: bool,
    },
    /// Compile a Nyx source file to bytecode
    Build {
        /// The source file to compile
        file: String,
        /// Output file for bytecode
        #[arg(short, long)]
        output: Option<String>,
        /// Show disassembled bytecode
        #[arg(short, long)]
        disassemble: bool,
    },
    /// Start an interactive REPL
    Repl {
        /// Enable debug output
        #[arg(short, long)]
        debug: bool,
    },
    /// Check a source file for type errors
    Check {
        /// The source file to check
        file: String,
        /// Show detailed type information
        #[arg(short, long)]
        verbose: bool,
    },
    /// Show version information
    Version,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run { file, debug } => {
            run_file(&file, debug)
        }
        Commands::Build { file, output, disassemble } => {
            build_file(&file, output.as_deref(), disassemble)
        }
        Commands::Repl { debug } => {
            start_repl(debug)
        }
        Commands::Check { file, verbose } => {
            check_file(&file, verbose)
        }
        Commands::Version => {
            print_version();
            Ok(())
        }
    }
}

fn run_file(file: &str, debug: bool) -> Result<()> {
    let source = fs::read_to_string(file)
        .with_context(|| format!("Could not read file: {}", file))?;

    if debug {
        eprintln!("Running file: {}", file);
        eprintln!("Source code:");
        eprintln!("{}", source);
        eprintln!("---");
    }

    // Lexical analysis
    let lexer = Lexer::new(source.clone());
    let tokens = lexer.tokenize()
        .with_context(|| "Lexical analysis failed")?;

    if debug {
        eprintln!("Tokens:");
        for token in &tokens {
            eprintln!("  {:?}", token);
        }
        eprintln!("---");
    }

    // Parsing
    let program = parse(&source)
        .with_context(|| "Parsing failed")?;

    if debug {
        eprintln!("AST:");
        eprintln!("{:#?}", program);
        eprintln!("---");
    }

    // Type checking
    let mut type_checker = TypeChecker::new();
    type_checker.check(&program)
        .with_context(|| "Type checking failed")?;

    if debug {
        eprintln!("Type checking passed");
        eprintln!("---");
    }

    // Compilation to bytecode
    let mut compiler = Compiler::new();
    let chunk = compiler.compile(&program)
        .with_context(|| "Compilation failed")?;

    if debug {
        eprintln!("Bytecode:");
        chunk.disassemble("main");
        eprintln!("---");
    }

    // Execution
    let mut vm = VirtualMachine::new();
    let result = vm.interpret(chunk)
        .with_context(|| "Execution failed")?;

    if debug {
        eprintln!("Execution result: {:?}", result);
    }

    // Print result if not null
    match result {
        Value::Null => {}
        _ => println!("{:?}", result),
    }

    Ok(())
}

fn build_file(file: &str, output: Option<&str>, disassemble: bool) -> Result<()> {
    let source = fs::read_to_string(file)
        .with_context(|| format!("Could not read file: {}", file))?;

    // Lexical analysis
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize()
        .with_context(|| "Lexical analysis failed")?;

    // Parsing
    let program = parse(&source)
        .with_context(|| "Parsing failed")?;

    // Type checking
    let mut type_checker = TypeChecker::new();
    type_checker.check(&program)
        .with_context(|| "Type checking failed")?;

    // Compilation to bytecode
    let mut compiler = Compiler::new();
    let chunk = compiler.compile(&program)
        .with_context(|| "Compilation failed")?;

    // Disassemble if requested
    if disassemble {
        chunk.disassemble("main");
    }

    // Write bytecode to file
    let output_file = output.unwrap_or(&format!("{}.nyxc", file));
    let bytecode = serialize_chunk(&chunk)?;
    fs::write(output_file, bytecode)
        .with_context(|| format!("Could not write bytecode to file: {}", output_file))?;

    println!("Compiled {} to {}", file, output_file);
    Ok(())
}

fn start_repl(debug: bool) -> Result<()> {
    println!("Nyx REPL v0.1.0");
    println!("Type 'exit' to quit, 'help' for commands");
    println!();

    let mut vm = VirtualMachine::new();
    let mut line_number = 1;

    loop {
        print!("nyx:{}> ", line_number);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        match input {
            "exit" | "quit" => break,
            "help" => {
                print_help();
                continue;
            }
            "clear" => {
                // Clear screen (platform dependent)
                print!("\x1B[2J\x1B[1;1H");
                continue;
            }
            _ => {}
        }

        // Try to evaluate the input
        match evaluate_repl_input(input, &mut vm, debug) {
            Ok(result) => {
                match result {
                    Value::Null => {}
                    _ => println!("=> {:?}", result),
                }
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }

        line_number += 1;
    }

    Ok(())
}

fn check_file(file: &str, verbose: bool) -> Result<()> {
    let source = fs::read_to_string(file)
        .with_context(|| format!("Could not read file: {}", file))?;

    // Lexical analysis
    let lexer = Lexer::new(source);
    let tokens = lexer.tokenize()
        .with_context(|| "Lexical analysis failed")?;

    // Parsing
    let program = parse(&source)
        .with_context(|| "Parsing failed")?;

    // Type checking
    let mut type_checker = TypeChecker::new();
    match type_checker.check(&program) {
        Ok(_) => {
            println!("Type checking passed for {}", file);
            if verbose {
                println!("Symbols: {}", type_checker.get_symbol_count());
                println!("Scopes: {}", type_checker.get_scope_count());
            }
        }
        Err(errors) => {
            println!("Type errors found in {}:", file);
            for error in errors {
                println!("  {}", error);
            }
            return Err(anyhow::anyhow!("Type checking failed"));
        }
    }

    Ok(())
}

fn print_version() {
    println!("Nyx Programming Language v0.1.0");
    println!("A modern, statically typed programming language");
    println!("Features: Type inference, generics, garbage collection");
    println!("GitHub: https://github.com/yourusername/nyx-lang");
}

fn print_help() {
    println!("REPL Commands:");
    println!("  exit, quit  - Exit the REPL");
    println!("  help       - Show this help message");
    println!("  clear      - Clear the screen");
    println!();
    println!("Language Features:");
    println!("  let x = 42;           - Variable declaration");
    println!("  fn add(a, b) { a + b } - Function definition");
    println!("  [1, 2, 3]             - List literal");
    println!("  {\"key\": \"value\"}      - Map literal");
    println!();
    println!("Examples:");
    println!("  let x = 1 + 2 * 3");
    println!("  fn factorial(n) { if n <= 1 { 1 } else { n * factorial(n - 1) } }");
    println!("  factorial(5)");
}

fn evaluate_repl_input(input: &str, vm: &mut VirtualMachine, debug: bool) -> Result<Value> {
    // Lexical analysis
    let lexer = Lexer::new(input.to_string());
    let tokens = lexer.tokenize()
        .with_context(|| "Lexical analysis failed")?;

    // Parsing
    let program = parse(input)
        .with_context(|| "Parsing failed")?;

    // Type checking
    let mut type_checker = TypeChecker::new();
    type_checker.check(&program)
        .with_context(|| "Type checking failed")?;

    // Compilation
    let mut compiler = Compiler::new();
    let chunk = compiler.compile(&program)
        .with_context(|| "Compilation failed")?;

    if debug {
        chunk.disassemble("repl");
    }

    // Execution
    vm.interpret(chunk)
        .with_context(|| "Execution failed")
}

fn serialize_chunk(chunk: &Chunk) -> Result<Vec<u8>> {
    // Simple serialization format:
    // [code_length:4][code...][constant_count:4][constants...]
    let mut buffer = Vec::new();

    // Code length and code
    let code_len = chunk.code.len() as u32;
    buffer.extend_from_slice(&code_len.to_le_bytes());
    buffer.extend_from_slice(&chunk.code);

    // Constant count and constants
    let constant_count = chunk.constants.len() as u32;
    buffer.extend_from_slice(&constant_count.to_le_bytes());

    for constant in &chunk.constants {
        serialize_value(constant, &mut buffer)?;
    }

    Ok(buffer)
}

fn serialize_value(value: &Value, buffer: &mut Vec<u8>) -> Result<()> {
    match value {
        Value::Int(i) => {
            buffer.push(0); // Type tag for int
            buffer.extend_from_slice(&i.to_le_bytes());
        }
        Value::Float(f) => {
            buffer.push(1); // Type tag for float
            buffer.extend_from_slice(&f.to_le_bytes());
        }
        Value::String(s) => {
            buffer.push(2); // Type tag for string
            let len = s.len() as u32;
            buffer.extend_from_slice(&len.to_le_bytes());
            buffer.extend_from_slice(s.as_bytes());
        }
        Value::Bool(b) => {
            buffer.push(3); // Type tag for bool
            buffer.push(if *b { 1 } else { 0 });
        }
        Value::Null => {
            buffer.push(4); // Type tag for null
        }
        _ => {
            return Err(anyhow::anyhow!("Cannot serialize complex values yet"));
        }
    }
    Ok(())
}

// Simple compiler that converts AST to bytecode
struct Compiler {
    chunk: Chunk,
}

impl Compiler {
    fn new() -> Self {
        Self {
            chunk: Chunk::new(),
        }
    }

    fn compile(&mut self, program: &nyx_ast::Program) -> Result<Chunk> {
        for statement in &program.statements {
            self.compile_statement(statement)?;
        }
        
        // Add return at the end
        self.chunk.write(OpCode::Return.to_u8(), 0);
        
        Ok(Chunk {
            code: self.chunk.code.clone(),
            constants: self.chunk.constants.clone(),
            lines: self.chunk.lines.clone(),
        })
    }

    fn compile_statement(&mut self, statement: &nyx_ast::Statement) -> Result<()> {
        match statement {
            nyx_ast::Statement::ExpressionStatement(expr) => {
                self.compile_expression(expr)?;
                self.chunk.write(OpCode::Pop.to_u8(), 0);
            }
            nyx_ast::Statement::VariableDeclaration { name, type_annotation: _, initializer, is_mutable: _ } => {
                if let Some(initializer) = initializer {
                    self.compile_expression(initializer)?;
                } else {
                    // Default to null
                    self.chunk.write(OpCode::LoadConstant.to_u8(), 0);
                    let null_const = self.chunk.add_constant(Value::Null);
                    self.chunk.write(null_const as u8, 0);
                }
                self.chunk.write(OpCode::StoreGlobal.to_u8(), 0);
                let name_const = self.chunk.add_constant(Value::String(name.clone()));
                self.chunk.write(name_const as u8, 0);
            }
            nyx_ast::Statement::Return { value } => {
                if let Some(value) = value {
                    self.compile_expression(value)?;
                } else {
                    self.chunk.write(OpCode::LoadConstant.to_u8(), 0);
                    let null_const = self.chunk.add_constant(Value::Null);
                    self.chunk.write(null_const as u8, 0);
                }
                self.chunk.write(OpCode::Return.to_u8(), 0);
            }
            _ => {
                return Err(anyhow::anyhow!("Statement type not yet implemented"));
            }
        }
        Ok(())
    }

    fn compile_expression(&mut self, expression: &nyx_ast::Expression) -> Result<()> {
        match expression {
            nyx_ast::Expression::Literal(literal) => {
                let value = match literal {
                    nyx_ast::LiteralValue::Integer(i) => Value::Int(*i),
                    nyx_ast::LiteralValue::Float(f) => Value::Float(*f),
                    nyx_ast::LiteralValue::String(s) => Value::String(s.clone()),
                    nyx_ast::LiteralValue::Bool(b) => Value::Bool(*b),
                    nyx_ast::LiteralValue::Null => Value::Null,
                };
                let const_idx = self.chunk.add_constant(value);
                self.chunk.write(OpCode::LoadConstant.to_u8(), 0);
                self.chunk.write(const_idx as u8, 0);
            }
            nyx_ast::Expression::Identifier(name) => {
                self.chunk.write(OpCode::LoadGlobal.to_u8(), 0);
                let name_const = self.chunk.add_constant(Value::String(name.clone()));
                self.chunk.write(name_const as u8, 0);
            }
            nyx_ast::Expression::Binary { left, operator, right } => {
                self.compile_expression(left)?;
                self.compile_expression(right)?;
                
                let op = match operator {
                    nyx_ast::BinaryOperator::Add => OpCode::Add,
                    nyx_ast::BinaryOperator::Subtract => OpCode::Subtract,
                    nyx_ast::BinaryOperator::Multiply => OpCode::Multiply,
                    nyx_ast::BinaryOperator::Divide => OpCode::Divide,
                    nyx_ast::BinaryOperator::Modulo => OpCode::Modulo,
                    nyx_ast::BinaryOperator::Equal => OpCode::Equal,
                    nyx_ast::BinaryOperator::NotEqual => OpCode::NotEqual,
                    nyx_ast::BinaryOperator::LessThan => OpCode::LessThan,
                    nyx_ast::BinaryOperator::GreaterThan => OpCode::GreaterThan,
                    nyx_ast::BinaryOperator::LessThanOrEqual => OpCode::LessThanOrEqual,
                    nyx_ast::BinaryOperator::GreaterThanOrEqual => OpCode::GreaterThanOrEqual,
                    nyx_ast::BinaryOperator::And => OpCode::And,
                    nyx_ast::BinaryOperator::Or => OpCode::Or,
                };
                self.chunk.write(op.to_u8(), 0);
            }
            nyx_ast::Expression::Unary { operator, expression } => {
                self.compile_expression(expression)?;
                
                let op = match operator {
                    nyx_ast::UnaryOperator::Negate => OpCode::Negate,
                    nyx_ast::UnaryOperator::Not => OpCode::Not,
                };
                self.chunk.write(op.to_u8(), 0);
            }
            nyx_ast::Expression::List { elements, element_type: _ } => {
                for element in elements {
                    self.compile_expression(element)?;
                }
                self.chunk.write(OpCode::NewList.to_u8(), 0);
                self.chunk.write(elements.len() as u8, 0);
            }
            _ => {
                return Err(anyhow::anyhow!("Expression type not yet implemented"));
            }
        }
        Ok(())
    }
}

// Extension methods for TypeChecker
impl TypeChecker {
    fn get_symbol_count(&self) -> usize {
        self.scopes.iter().map(|s| s.symbols.len()).sum()
    }

    fn get_scope_count(&self) -> usize {
        self.scopes.len()
    }
}
