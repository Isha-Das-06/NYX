#!/bin/bash

echo "=== Nyx Language Verification ==="

echo "1. Checking project structure..."
if [ -d "crates" ] && [ -d "examples" ] && [ -d "tests" ]; then
    echo "   [OK] Project structure is complete"
else
    echo "   [ERROR] Missing directories"
fi

echo "2. Checking core crates..."
crates=("lexer" "parser" "ast" "type-system" "vm" "gc" "cli")
for crate in "${crates[@]}"; do
    if [ -f "crates/$crate/Cargo.toml" ] && [ -f "crates/$crate/src/lib.rs" ]; then
        echo "   [OK] $crate crate is complete"
    else
        echo "   [ERROR] $crate crate is incomplete"
    fi
done

echo "3. Checking examples..."
examples=("arithmetic.nyx" "functions.nyx" "collections.nyx" "control_flow.nyx" "generics.nyx")
for example in "${examples[@]}"; do
    if [ -f "examples/$example" ]; then
        echo "   [OK] $example exists"
    else
        echo "   [ERROR] $example missing"
    fi
done

echo "4. Checking documentation..."
if [ -f "README.md" ] && [ -f "LICENSE" ] && [ -f "CONTRIBUTING.md" ]; then
    echo "   [OK] Documentation is complete"
else
    echo "   [ERROR] Missing documentation files"
fi

echo "5. Checking tests..."
if [ -d "tests" ] && [ "$(ls tests/*.rs | wc -l)" -ge 3 ]; then
    echo "   [OK] Test suite is present"
else
    echo "   [ERROR] Test suite is incomplete"
fi

echo "=== Verification Complete ==="
