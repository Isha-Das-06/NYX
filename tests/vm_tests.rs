use nyx_vm::{VirtualMachine, Chunk, OpCode, Value};

#[test]
fn test_vm_arithmetic_operations() {
    let mut vm = VirtualMachine::new();
    let mut chunk = Chunk::new();
    
    // Test addition: 5 + 3 = 8
    chunk.write(OpCode::LoadInt.to_u8(), 1);
    chunk.code.extend_from_slice(&5i64.to_le_bytes());
    
    chunk.write(OpCode::LoadInt.to_u8(), 2);
    chunk.code.extend_from_slice(&3i64.to_le_bytes());
    
    chunk.write(OpCode::Add.to_u8(), 3);
    chunk.write(OpCode::Return.to_u8(), 4);
    
    let result = vm.interpret(chunk).unwrap();
    assert_eq!(result, Value::Int(8));
}

#[test]
fn test_vm_float_arithmetic() {
    let mut vm = VirtualMachine::new();
    let mut chunk = Chunk::new();
    
    // Test float addition: 3.14 + 2.86 = 6.0
    chunk.write(OpCode::LoadFloat.to_u8(), 1);
    chunk.code.extend_from_slice(&3.14f64.to_le_bytes());
    
    chunk.write(OpCode::LoadFloat.to_u8(), 2);
    chunk.code.extend_from_slice(&2.86f64.to_le_bytes());
    
    chunk.write(OpCode::Add.to_u8(), 3);
    chunk.write(OpCode::Return.to_u8(), 4);
    
    let result = vm.interpret(chunk).unwrap();
    assert_eq!(result, Value::Float(6.0));
}

#[test]
fn test_vm_mixed_type_arithmetic() {
    let mut vm = VirtualMachine::new();
    let mut chunk = Chunk::new();
    
    // Test int + float: 5 + 3.14 = 8.14
    chunk.write(OpCode::LoadInt.to_u8(), 1);
    chunk.code.extend_from_slice(&5i64.to_le_bytes());
    
    chunk.write(OpCode::LoadFloat.to_u8(), 2);
    chunk.code.extend_from_slice(&3.14f64.to_le_bytes());
    
    chunk.write(OpCode::Add.to_u8(), 3);
    chunk.write(OpCode::Return.to_u8(), 4);
    
    let result = vm.interpret(chunk).unwrap();
    assert_eq!(result, Value::Float(8.14));
}

#[test]
fn test_vm_string_operations() {
    let mut vm = VirtualMachine::new();
    let mut chunk = Chunk::new();
    
    // Test string concatenation: "Hello" + " World" = "Hello World"
    let hello_const = chunk.add_constant(Value::String("Hello".to_string()));
    let world_const = chunk.add_constant(Value::String(" World".to_string()));
    
    chunk.write(OpCode::LoadString.to_u8(), 1);
    chunk.write(hello_const as u8, 1);
    
    chunk.write(OpCode::LoadString.to_u8(), 2);
    chunk.write(world_const as u8, 2);
    
    chunk.write(OpCode::Add.to_u8(), 3);
    chunk.write(OpCode::Return.to_u8(), 4);
    
    let result = vm.interpret(chunk).unwrap();
    assert_eq!(result, Value::String("Hello World".to_string()));
}

#[test]
fn test_vm_comparison_operations() {
    let mut vm = VirtualMachine::new();
    let mut chunk = Chunk::new();
    
    // Test 5 < 10 = true
    chunk.write(OpCode::LoadInt.to_u8(), 1);
    chunk.code.extend_from_slice(&5i64.to_le_bytes());
    
    chunk.write(OpCode::LoadInt.to_u8(), 2);
    chunk.code.extend_from_slice(&10i64.to_le_bytes());
    
    chunk.write(OpCode::LessThan.to_u8(), 3);
    chunk.write(OpCode::Return.to_u8(), 4);
    
    let result = vm.interpret(chunk).unwrap();
    assert_eq!(result, Value::Bool(true));
}

#[test]
fn test_vm_logical_operations() {
    let mut vm = VirtualMachine::new();
    let mut chunk = Chunk::new();
    
    // Test true && false = false
    chunk.write(OpCode::LoadBool.to_u8(), 1);
    chunk.write(1, 1); // true
    
    chunk.write(OpCode::LoadBool.to_u8(), 2);
    chunk.write(0, 2); // false
    
    chunk.write(OpCode::And.to_u8(), 3);
    chunk.write(OpCode::Return.to_u8(), 4);
    
    let result = vm.interpret(chunk).unwrap();
    assert_eq!(result, Value::Bool(false));
}

#[test]
fn test_vm_unary_operations() {
    let mut vm = VirtualMachine::new();
    let mut chunk = Chunk::new();
    
    // Test -42 = -42
    chunk.write(OpCode::LoadInt.to_u8(), 1);
    chunk.code.extend_from_slice(&42i64.to_le_bytes());
    
    chunk.write(OpCode::Negate.to_u8(), 2);
    chunk.write(OpCode::Return.to_u8(), 3);
    
    let result = vm.interpret(chunk).unwrap();
    assert_eq!(result, Value::Int(-42));
}

#[test]
fn test_vm_list_operations() {
    let mut vm = VirtualMachine::new();
    let mut chunk = Chunk::new();
    
    // Create list [1, 2, 3]
    chunk.write(OpCode::LoadInt.to_u8(), 1);
    chunk.code.extend_from_slice(&1i64.to_le_bytes());
    
    chunk.write(OpCode::LoadInt.to_u8(), 2);
    chunk.code.extend_from_slice(&2i64.to_le_bytes());
    
    chunk.write(OpCode::LoadInt.to_u8(), 3);
    chunk.code.extend_from_slice(&3i64.to_le_bytes());
    
    chunk.write(OpCode::NewList.to_u8(), 4);
    chunk.write(3, 4); // 3 elements
    
    chunk.write(OpCode::Return.to_u8(), 5);
    
    let result = vm.interpret(chunk).unwrap();
    
    if let Value::List(_) = result {
        // List created successfully
    } else {
        panic!("Expected list value");
    }
}

#[test]
fn test_vm_global_variables() {
    let mut vm = VirtualMachine::new();
    let mut chunk = Chunk::new();
    
    // Store global variable x = 42
    chunk.write(OpCode::LoadInt.to_u8(), 1);
    chunk.code.extend_from_slice(&42i64.to_le_bytes());
    
    let x_const = chunk.add_constant(Value::String("x".to_string()));
    chunk.write(OpCode::StoreGlobal.to_u8(), 2);
    chunk.write(x_const as u8, 2);
    
    // Load global variable x
    chunk.write(OpCode::LoadGlobal.to_u8(), 3);
    chunk.write(x_const as u8, 3);
    
    chunk.write(OpCode::Return.to_u8(), 4);
    
    let result = vm.interpret(chunk).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_vm_complex_expression() {
    let mut vm = VirtualMachine::new();
    let mut chunk = Chunk::new();
    
    // Test (5 + 3) * 2 = 16
    chunk.write(OpCode::LoadInt.to_u8(), 1);
    chunk.code.extend_from_slice(&5i64.to_le_bytes());
    
    chunk.write(OpCode::LoadInt.to_u8(), 2);
    chunk.code.extend_from_slice(&3i64.to_le_bytes());
    
    chunk.write(OpCode::Add.to_u8(), 3); // 5 + 3 = 8
    
    chunk.write(OpCode::LoadInt.to_u8(), 4);
    chunk.code.extend_from_slice(&2i64.to_le_bytes());
    
    chunk.write(OpCode::Multiply.to_u8(), 5); // 8 * 2 = 16
    
    chunk.write(OpCode::Return.to_u8(), 6);
    
    let result = vm.interpret(chunk).unwrap();
    assert_eq!(result, Value::Int(16));
}

#[test]
fn test_vm_stack_operations() {
    let mut vm = VirtualMachine::new();
    let mut chunk = Chunk::new();
    
    // Test duplicate and pop
    chunk.write(OpCode::LoadInt.to_u8(), 1);
    chunk.code.extend_from_slice(&42i64.to_le_bytes());
    
    chunk.write(OpCode::Duplicate.to_u8(), 2); // Duplicate 42
    chunk.write(OpCode::Pop.to_u8(), 3); // Pop one 42
    chunk.write(OpCode::Return.to_u8(), 4); // Return the other 42
    
    let result = vm.interpret(chunk).unwrap();
    assert_eq!(result, Value::Int(42));
}

#[test]
fn test_vm_division_by_zero() {
    let mut vm = VirtualMachine::new();
    let mut chunk = Chunk::new();
    
    // Test 5 / 0 = error
    chunk.write(OpCode::LoadInt.to_u8(), 1);
    chunk.code.extend_from_slice(&5i64.to_le_bytes());
    
    chunk.write(OpCode::LoadInt.to_u8(), 2);
    chunk.code.extend_from_slice(&0i64.to_le_bytes());
    
    chunk.write(OpCode::Divide.to_u8(), 3);
    chunk.write(OpCode::Return.to_u8(), 4);
    
    let result = vm.interpret(chunk);
    assert!(result.is_err());
}

#[test]
fn test_vm_modulo_operation() {
    let mut vm = VirtualMachine::new();
    let mut chunk = Chunk::new();
    
    // Test 10 % 3 = 1
    chunk.write(OpCode::LoadInt.to_u8(), 1);
    chunk.code.extend_from_slice(&10i64.to_le_bytes());
    
    chunk.write(OpCode::LoadInt.to_u8(), 2);
    chunk.code.extend_from_slice(&3i64.to_le_bytes());
    
    chunk.write(OpCode::Modulo.to_u8(), 3);
    chunk.write(OpCode::Return.to_u8(), 4);
    
    let result = vm.interpret(chunk).unwrap();
    assert_eq!(result, Value::Int(1));
}
