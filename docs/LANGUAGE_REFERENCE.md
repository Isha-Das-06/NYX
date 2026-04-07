# Nyx Language Reference

This is the complete language reference for Nyx, a modern, statically typed programming language.

## Table of Contents

1. [Overview](#overview)
2. [Lexical Structure](#lexical-structure)
3. [Types](#types)
4. [Expressions](#expressions)
5. [Statements](#statements)
6. [Functions](#functions)
7. [Generics](#generics)
8. [Modules](#modules)
9. [Standard Library](#standard-library)
10. [Error Handling](#error-handling)

## Overview

Nyx is a statically typed programming language with:
- Type inference
- Generics
- Garbage collection
- Modern syntax
- Efficient bytecode execution

### Hello World

```nyx
fn main() {
    println("Hello, World!");
}
```

## Lexical Structure

### Comments

```nyx
// This is a line comment

/* This is a
   multi-line comment */
```

### Identifiers

Identifiers start with a letter or underscore, followed by letters, digits, or underscores:

```nyx
let x = 42;
let _private = "secret";
let camelCase = true;
let snake_case = false;
```

### Keywords

```
fn, let, if, else, for, while, return, break, continue,
match, struct, impl, true, false, Int, Float, String, Bool,
List, Map
```

### Literals

```nyx
42                    // Integer
3.14                  // Float
"Hello, World!"      // String
'c'                   // Character
true                  // Boolean
false                 // Boolean
```

## Types

### Primitive Types

```nyx
let integer: Int = 42;
let float: Float = 3.14;
let string: String = "Hello";
let boolean: Bool = true;
```

### Collection Types

```nyx
// Lists
let numbers: List<Int> = [1, 2, 3, 4, 5];
let strings: List<String> = ["hello", "world"];

// Maps
let person: Map<String, Value> = {
    "name": "Alice",
    "age": 30,
    "city": "New York"
};

// Tuples (planned)
let point: (Int, Int) = (10, 20);
```

### Function Types

```nyx
let add: fn(Int, Int) -> Int = fn(a, b) { a + b };
```

### Custom Types

```nyx
struct Point {
    x: Float,
    y: Float
}

struct Container<T> {
    value: T
}
```

## Expressions

### Literals

```nyx
42
3.14
"Hello, World!"
true
false
```

### Arithmetic

```nyx
let sum = 1 + 2;
let difference = 10 - 5;
let product = 3 * 4;
let quotient = 15 / 3;
let remainder = 17 % 5;
let negated = -x;
```

### Comparison

```nyx
let equal = a == b;
let not_equal = a != b;
let less_than = a < b;
let greater_than = a > b;
let less_or_equal = a <= b;
let greater_or_equal = a >= b;
```

### Logical

```nyx
let and_result = true && false;
let or_result = true || false;
let not_result = !true;
```

### String Operations

```nyx
let greeting = "Hello" + ", " + "World";
```

### List Operations

```nyx
let numbers = [1, 2, 3, 4, 5];
let first = numbers[0];
let empty = [];
```

### Map Operations

```nyx
let person = {"name": "Alice", "age": 30};
let name = person["name"];
let empty = {};
```

### Function Calls

```nyx
let result = add(1, 2);
println("Hello, {}", name);
```

### Conditional Expression

```nyx
let max = if a > b { a } else { b };
```

## Statements

### Variable Declarations

```nyx
// With type annotation
let x: Int = 42;

// With type inference
let y = 3.14;

// Mutable variable
let mut z = 0;
z = 1;
```

### Function Declarations

```nyx
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn greet(name: String) {
    println("Hello, {}!", name);
}
```

### If Statements

```nyx
if age >= 18 {
    println("Adult");
} else if age >= 13 {
    println("Teenager");
} else {
    println("Child");
}
```

### While Loops

```nyx
let mut i = 0;
while i < 10 {
    println("i = {}", i);
    i = i + 1;
}
```

### For Loops

```nyx
let numbers = [1, 2, 3, 4, 5];
for num in numbers {
    println("num = {}", num);
}

for i in 0..10 {
    println("i = {}", i);
}
```

### Match Statements

```nyx
let day = "Monday";
match day {
    "Monday" => println("Start of week"),
    "Friday" => println("Almost weekend"),
    "Saturday" | "Sunday" => println("Weekend!"),
    _ => println("Midweek")
}
```

### Return Statements

```nyx
fn factorial(n: Int) -> Int {
    if n <= 1 {
        return 1;
    }
    n * factorial(n - 1)
}
```

### Break and Continue

```nyx
let mut i = 0;
while i < 10 {
    if i == 3 {
        i = i + 1;
        continue;
    }
    if i == 7 {
        break;
    }
    println("i = {}", i);
    i = i + 1;
}
```

## Functions

### Basic Functions

```nyx
fn add(a: Int, b: Int) -> Int {
    a + b
}

fn greet(name: String) {
    println("Hello, {}!", name);
}
```

### Higher-Order Functions

```nyx
fn apply_twice<T>(f: fn(T) -> T, x: T) -> T {
    f(f(x))
}

fn increment(x: Int) -> Int {
    x + 1
}

let result = apply_twice(increment, 5); // 7
```

### Closures

```nyx
let add = fn(a, b) { a + b };
let result = add(1, 2);

let multiplier = fn(n) { fn(x) { x * n } };
let double = multiplier(2);
let result = double(5); // 10
```

### Recursive Functions

```nyx
fn factorial(n: Int) -> Int {
    if n <= 1 {
        1
    } else {
        n * factorial(n - 1)
    }
}

fn fibonacci(n: Int) -> Int {
    if n <= 1 {
        n
    } else {
        fibonacci(n - 1) + fibonacci(n - 2)
    }
}
```

## Generics

### Generic Functions

```nyx
fn identity<T>(value: T) -> T {
    value
}

fn swap<T>(a: T, b: T) -> (T, T) {
    (b, a)
}

let int_result = identity(42);
let string_result = identity("hello");
let (swapped_a, swapped_b) = swap("first", "second");
```

### Generic Structs

```nyx
struct Container<T> {
    value: T
}

impl<T> Container<T> {
    fn new(value: T) -> Container<T> {
        Container { value }
    }
    
    fn get(&self) -> T {
        self.value
    }
    
    fn set(&mut self, value: T) {
        self.value = value;
    }
}

let int_container = Container::new(42);
let string_container = Container::new("hello");
```

### Type Constraints (Planned)

```nyx
// Future feature for constrained generics
fn print_debug<T: Debug>(item: T) {
    println("Debug: {:?}", item);
}
```

## Modules

### Module Declaration

```nyx
// math.nyx
pub fn add(a: Int, b: Int) -> Int {
    a + b
}

pub fn multiply(a: Int, b: Int) -> Int {
    a * b
}

// Private function
fn internal_helper() {
    // ...
}
```

### Module Usage

```nyx
import math;

let result = math.add(1, 2);
let product = math.multiply(3, 4);

// Import specific items
import math::{add, multiply};

// Import with alias
import math as m;
let result = m.add(1, 2);
```

## Standard Library

### String Functions

```nyx
let text = "Hello, World!";
let length = text.length();
let upper = text.to_uppercase();
let contains = text.contains("World");
let parts = text.split(",");
```

### Math Functions

```nyx
import math;

let absolute = math.abs(-5);
let power = math.pow(2, 3);
let square_root = math.sqrt(16);
let sine = math.sin(3.14159);
```

### List Functions

```nyx
let numbers = [1, 2, 3, 4, 5];
let length = numbers.length();
let first = numbers.first();
let last = numbers.last();
let sum = numbers.fold(0, fn(acc, x) { acc + x });
let filtered = numbers.filter(fn(x) { x > 3 });
let mapped = numbers.map(fn(x) { x * 2 });
```

### I/O Operations

```nyx
import io;

let input = io.read_line();
io.print_line("Hello, World!");
io.print("Value: {}", 42);
```

## Error Handling

### Result Type

```nyx
fn divide(a: Float, b: Float) -> Result<Float, String> {
    if b == 0.0 {
        Result::Error("Division by zero".to_string())
    } else {
        Result::Ok(a / b)
    }
}

let result = divide(10.0, 2.0);
match result {
    Result::Ok(value) => println("Result: {}", value),
    Result::Error(error) => println("Error: {}", error)
}
```

### Option Type

```nyx
fn safe_divide(a: Float, b: Float) -> Option<Float> {
    if b == 0.0 {
        Option::None
    } else {
        Option::Some(a / b)
    }
}

let result = safe_divide(10.0, 2.0);
match result {
    Option::Some(value) => println("Result: {}", value),
    Option::None => println("Division by zero")
}
```

### Panic

```nyx
fn process(value: Int) {
    if value < 0 {
        panic!("Value must be non-negative");
    }
    // Process value
}
```

---

## Language Specification

### Grammar (Simplified)

```
program     ::= statement*
statement   ::= variable_decl | function_decl | if_stmt | while_stmt | for_stmt | return_stmt | expr_stmt
expression  ::= literal | identifier | binary_expr | unary_expr | call_expr | list_expr | map_expr
type        ::= primitive_type | list_type | map_type | function_type | struct_type
```

### Type System

Nyx uses a static type system with:
- **Strong typing**: No implicit conversions between unrelated types
- **Type inference**: Types can be inferred from usage
- **Generics**: Parameterized types and functions
- **Subtyping**: (Planned feature)

### Memory Management

Nyx uses automatic memory management with:
- **Garbage Collection**: Mark-and-sweep algorithm
- **Reference Counting**: For immediate cleanup
- **Stack Allocation**: For local variables
- **Heap Allocation**: For objects and collections

---

This reference covers the core features of Nyx. For more detailed information, see the source code and examples.
