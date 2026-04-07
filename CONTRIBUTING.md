# Contributing to Nyx

Thank you for your interest in contributing to Nyx! This document provides guidelines for contributors.

## Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/yourusername/nyx-lang
   cd nyx-lang
   ```

2. **Install Rust**
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

3. **Build the project**
   ```bash
   cargo build --release
   ```

4. **Run tests**
   ```bash
   cargo test
   ```

## Code Style

- Use `rustfmt` for code formatting
- Use `clippy` for linting
- Follow Rust naming conventions
- Write comprehensive tests for new features

## Project Structure

```
nyx-lang/
 crates/
   lexer/        # Tokenization and lexical analysis
   parser/       # Syntax parsing and AST generation
   ast/          # Abstract syntax tree definitions
   type-system/  # Type checking and inference
   vm/           # Virtual machine and bytecode execution
   gc/           # Garbage collector
   cli/          # Command-line interface
 examples/       # Example programs
 tests/          # Integration tests
 benches/        # Performance benchmarks
 docs/           # Documentation
```

## Contributing Workflow

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes
4. Add tests for your changes
5. Run the test suite: `cargo test`
6. Format your code: `cargo fmt`
7. Run linter: `cargo clippy`
8. Commit your changes: `git commit -am 'Add feature'`
9. Push to your branch: `git push origin feature-name`
10. Create a pull request

## Areas for Contribution

### Language Features
- [ ] Additional built-in functions
- [ ] Pattern matching enhancements
- [ ] Module system
- [ ] Foreign function interface (FFI)
- [ ] Macros system

### Compiler Improvements
- [ ] Error recovery in parser
- [ ] Better error messages
- [ ] Optimization passes
- [ ] LLVM backend

### Standard Library
- [ ] String manipulation functions
- [ ] Math functions
- [ ] I/O operations
- [ ] Networking support

### Tools
- [ ] Language server protocol (LSP)
- [ ] IDE plugins
- [ ] Debugger
- [ ] Package manager

## Testing

### Unit Tests
Each crate should have comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_feature() {
        // Test implementation
    }
}
```

### Integration Tests
Add integration tests in the `tests/` directory:

```rust
use nyx_lang::*;

#[test]
fn test_full_compilation() {
    // Integration test
}
```

### Benchmarks
Add performance benchmarks in the `benches/` directory:

```rust
use criterion::*;

fn bench_feature(c: &mut Criterion) {
    c.bench_function("feature", |b| {
        b.iter(|| {
            // Benchmark code
        })
    });
}
```

## Documentation

- Update README.md for user-facing changes
- Add inline documentation (`///`) for public APIs
- Update this CONTRIBUTING.md as needed
- Add examples for new features

## Code Review Process

1. All changes require review
2. Maintain backward compatibility when possible
3. Follow semantic versioning
4. Update documentation as needed

## Release Process

1. Update version numbers
2. Update CHANGELOG.md
3. Create git tag
4. Publish to crates.io
5. Update GitHub releases

## Getting Help

- Open an issue for bugs or feature requests
- Join our Discord community
- Check the documentation
- Look at existing issues and pull requests

Thank you for contributing to Nyx!
