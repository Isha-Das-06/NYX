use nyx_ast::*;
use nyx_gc::{GarbageCollector, GcPtr, GcTrace, GcVisitor};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpCode {
    // Constants
    LoadConstant,
    LoadInt,
    LoadFloat,
    LoadString,
    LoadBool,
    
    // Variables
    LoadLocal,
    StoreLocal,
    LoadGlobal,
    StoreGlobal,
    
    // Stack operations
    Pop,
    Duplicate,
    
    // Arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Negate,
    
    // Comparison
    Equal,
    NotEqual,
    LessThan,
    GreaterThan,
    LessThanOrEqual,
    GreaterThanOrEqual,
    
    // Logical
    And,
    Or,
    Not,
    
    // Control flow
    Jump,
    JumpIfFalse,
    JumpIfTrue,
    Return,
    
    // Functions
    Call,
    DefineFunction,
    
    // Collections
    NewList,
    NewMap,
    IndexGet,
    IndexSet,
    
    // Objects
    NewObject,
    GetField,
    SetField,
    
    // Special
    Halt,
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    List(GcPtr<ListValue>),
    Map(GcPtr<MapValue>),
    Object(GcPtr<ObjectValue>),
    Function(GcPtr<FunctionValue>),
    Null,
}

impl GcTrace for Value {
    fn trace(&self, visitor: &mut GcVisitor) {
        match self {
            Value::List(ptr) => {
                unsafe {
                    let gc = visitor.marked.get_index(0).map(|(_, &p)| p).unwrap_or(0);
                    // This is a simplified trace - in practice, we'd need proper GC integration
                }
            }
            Value::Map(ptr) => {
                // Similar trace for maps
            }
            Value::Object(ptr) => {
                // Similar trace for objects
            }
            Value::Function(ptr) => {
                // Similar trace for functions
            }
            _ => {}
        }
    }

    fn size(&self) -> usize {
        match self {
            Value::Int(_) => std::mem::size_of::<i64>(),
            Value::Float(_) => std::mem::size_of::<f64>(),
            Value::String(s) => std::mem::size_of::<String>() + s.len(),
            Value::Bool(_) => std::mem::size_of::<bool>(),
            Value::List(_) => std::mem::size_of::<GcPtr<ListValue>>(),
            Value::Map(_) => std::mem::size_of::<GcPtr<MapValue>>(),
            Value::Object(_) => std::mem::size_of::<GcPtr<ObjectValue>>(),
            Value::Function(_) => std::mem::size_of::<GcPtr<FunctionValue>>(),
            Value::Null => 0,
        }
    }
}

#[derive(Debug)]
pub struct ListValue {
    pub elements: Vec<Value>,
}

impl GcTrace for ListValue {
    fn trace(&self, visitor: &mut GcVisitor) {
        for element in &self.elements {
            element.trace(visitor);
        }
    }

    fn size(&self) -> usize {
        std::mem::size_of::<ListValue>() + 
            self.elements.len() * std::mem::size_of::<Value>()
    }
}

#[derive(Debug)]
pub struct MapValue {
    pub entries: HashMap<String, Value>,
}

impl GcTrace for MapValue {
    fn trace(&self, visitor: &mut GcVisitor) {
        for value in self.entries.values() {
            value.trace(visitor);
        }
    }

    fn size(&self) -> usize {
        std::mem::size_of::<MapValue>() + 
            self.entries.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>())
    }
}

#[derive(Debug)]
pub struct ObjectValue {
    pub class: String,
    pub fields: HashMap<String, Value>,
}

impl GcTrace for ObjectValue {
    fn trace(&self, visitor: &mut GcVisitor) {
        for value in self.fields.values() {
            value.trace(visitor);
        }
    }

    fn size(&self) -> usize {
        std::mem::size_of::<ObjectValue>() + 
            self.fields.len() * (std::mem::size_of::<String>() + std::mem::size_of::<Value>())
    }
}

#[derive(Debug)]
pub struct FunctionValue {
    pub name: String,
    pub arity: usize,
    pub chunk: Chunk,
    pub closure: Vec<Value>,
}

impl GcTrace for FunctionValue {
    fn trace(&self, visitor: &mut GcVisitor) {
        for value in &self.closure {
            value.trace(visitor);
        }
    }

    fn size(&self) -> usize {
        std::mem::size_of::<FunctionValue>() + 
            self.closure.len() * std::mem::size_of::<Value>() +
            self.chunk.code.len() * std::mem::size_of::<u8>() +
            self.chunk.constants.len() * std::mem::size_of::<Value>()
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub constants: Vec<Value>,
    pub lines: Vec<usize>,
}

impl Chunk {
    pub fn new() -> Self {
        Self {
            code: Vec::new(),
            constants: Vec::new(),
            lines: Vec::new(),
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Value) -> usize {
        self.constants.push(value);
        self.constants.len() - 1
    }

    pub fn disassemble(&self, name: &str) {
        println!("== {} chunk ==", name);
        let mut offset = 0;
        while offset < self.code.len() {
            offset = self.disassemble_instruction(offset);
        }
    }

    fn disassemble_instruction(&self, offset: usize) -> usize {
        print!("{:04} ", offset);
        if offset > 0 && self.lines[offset] == self.lines[offset - 1] {
            print!("   | ");
        } else {
            print!("{:4} ", self.lines[offset]);
        }

        let instruction = self.code[offset];
        match OpCode::from_u8(instruction) {
            Some(op) => match op {
                OpCode::LoadConstant => {
                    let constant = self.code[offset + 1];
                    println!("{:16} {:4}", "OP_CONSTANT", constant);
                    println!("{:16} '{}'", "", self.constants[constant as usize]);
                    offset + 2
                }
                OpCode::LoadInt => {
                    let value = i64::from_le_bytes([
                        self.code[offset + 1],
                        self.code[offset + 2],
                        self.code[offset + 3],
                        self.code[offset + 4],
                        self.code[offset + 5],
                        self.code[offset + 6],
                        self.code[offset + 7],
                        self.code[offset + 8],
                    ]);
                    println!("{:16} {}", "OP_LOAD_INT", value);
                    offset + 9
                }
                OpCode::LoadFloat => {
                    let value = f64::from_le_bytes([
                        self.code[offset + 1],
                        self.code[offset + 2],
                        self.code[offset + 3],
                        self.code[offset + 4],
                        self.code[offset + 5],
                        self.code[offset + 6],
                        self.code[offset + 7],
                        self.code[offset + 8],
                    ]);
                    println!("{:16} {}", "OP_LOAD_FLOAT", value);
                    offset + 9
                }
                OpCode::LoadString => {
                    let constant = self.code[offset + 1];
                    println!("{:16} {:4}", "OP_LOAD_STRING", constant);
                    println!("{:16} '{}'", "", self.constants[constant as usize]);
                    offset + 2
                }
                OpCode::LoadBool => {
                    let value = self.code[offset + 1] != 0;
                    println!("{:16} {}", "OP_LOAD_BOOL", value);
                    offset + 2
                }
                OpCode::LoadLocal => {
                    let slot = self.code[offset + 1];
                    println!("{:16} {:4}", "OP_LOAD_LOCAL", slot);
                    offset + 2
                }
                OpCode::StoreLocal => {
                    let slot = self.code[offset + 1];
                    println!("{:16} {:4}", "OP_STORE_LOCAL", slot);
                    offset + 2
                }
                OpCode::LoadGlobal => {
                    let constant = self.code[offset + 1];
                    println!("{:16} {:4}", "OP_LOAD_GLOBAL", constant);
                    offset + 2
                }
                OpCode::StoreGlobal => {
                    let constant = self.code[offset + 1];
                    println!("{:16} {:4}", "OP_STORE_GLOBAL", constant);
                    offset + 2
                }
                OpCode::Pop => {
                    println!("{:16}", "OP_POP");
                    offset + 1
                }
                OpCode::Duplicate => {
                    println!("{:16}", "OP_DUPLICATE");
                    offset + 1
                }
                OpCode::Add => println!("{:16}", "OP_ADD"),
                OpCode::Subtract => println!("{:16}", "OP_SUBTRACT"),
                OpCode::Multiply => println!("{:16}", "OP_MULTIPLY"),
                OpCode::Divide => println!("{:16}", "OP_DIVIDE"),
                OpCode::Modulo => println!("{:16}", "OP_MODULO"),
                OpCode::Negate => println!("{:16}", "OP_NEGATE"),
                OpCode::Equal => println!("{:16}", "OP_EQUAL"),
                OpCode::NotEqual => println!("{:16}", "OP_NOT_EQUAL"),
                OpCode::LessThan => println!("{:16}", "OP_LESS_THAN"),
                OpCode::GreaterThan => println!("{:16}", "OP_GREATER_THAN"),
                OpCode::LessThanOrEqual => println!("{:16}", "OP_LESS_THAN_EQUAL"),
                OpCode::GreaterThanOrEqual => println!("{:16}", "OP_GREATER_THAN_EQUAL"),
                OpCode::And => println!("{:16}", "OP_AND"),
                OpCode::Or => println!("{:16}", "OP_OR"),
                OpCode::Not => println!("{:16}", "OP_NOT"),
                OpCode::Jump => {
                    let offset = u16::from_le_bytes([self.code[offset + 1], self.code[offset + 2]]);
                    println!("{:16} {:4} -> {}", "OP_JUMP", offset, offset + 3 + offset as usize);
                    offset + 3
                }
                OpCode::JumpIfFalse => {
                    let jump_offset = u16::from_le_bytes([self.code[offset + 1], self.code[offset + 2]]);
                    println!("{:16} {:4} -> {}", "OP_JUMP_IF_FALSE", jump_offset, offset + 3 + jump_offset as usize);
                    offset + 3
                }
                OpCode::JumpIfTrue => {
                    let jump_offset = u16::from_le_bytes([self.code[offset + 1], self.code[offset + 2]]);
                    println!("{:16} {:4} -> {}", "OP_JUMP_IF_TRUE", jump_offset, offset + 3 + jump_offset as usize);
                    offset + 3
                }
                OpCode::Return => println!("{:16}", "OP_RETURN"),
                OpCode::Call => {
                    let arg_count = self.code[offset + 1];
                    println!("{:16} {:4}", "OP_CALL", arg_count);
                    offset + 2
                }
                OpCode::DefineFunction => {
                    let constant = self.code[offset + 1];
                    println!("{:16} {:4}", "OP_DEFINE_FUNCTION", constant);
                    offset + 2
                }
                OpCode::NewList => {
                    let element_count = self.code[offset + 1];
                    println!("{:16} {:4}", "OP_NEW_LIST", element_count);
                    offset + 2
                }
                OpCode::NewMap => println!("{:16}", "OP_NEW_MAP"),
                OpCode::IndexGet => println!("{:16}", "OP_INDEX_GET"),
                OpCode::IndexSet => println!("{:16}", "OP_INDEX_SET"),
                OpCode::NewObject => {
                    let constant = self.code[offset + 1];
                    println!("{:16} {:4}", "OP_NEW_OBJECT", constant);
                    offset + 2
                }
                OpCode::GetField => {
                    let constant = self.code[offset + 1];
                    println!("{:16} {:4}", "OP_GET_FIELD", constant);
                    offset + 2
                }
                OpCode::SetField => {
                    let constant = self.code[offset + 1];
                    println!("{:16} {:4}", "OP_SET_FIELD", constant);
                    offset + 2
                }
                OpCode::Halt => println!("{:16}", "OP_HALT"),
            }
            None => {
                println!("UNKNOWN OP: {}", instruction);
                offset + 1
            }
        }
    }
}

impl OpCode {
    pub fn to_u8(self) -> u8 {
        self as u8
    }

    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Self::LoadConstant),
            1 => Some(Self::LoadInt),
            2 => Some(Self::LoadFloat),
            3 => Some(Self::LoadString),
            4 => Some(Self::LoadBool),
            5 => Some(Self::LoadLocal),
            6 => Some(Self::StoreLocal),
            7 => Some(Self::LoadGlobal),
            8 => Some(Self::StoreGlobal),
            9 => Some(Self::Pop),
            10 => Some(Self::Duplicate),
            11 => Some(Self::Add),
            12 => Some(Self::Subtract),
            13 => Some(Self::Multiply),
            14 => Some(Self::Divide),
            15 => Some(Self::Modulo),
            16 => Some(Self::Negate),
             17 => Some(Self::Equal),
            18 => Some(Self::NotEqual),
            19 => Some(Self::LessThan),
            20 => Some(Self::GreaterThan),
            21 => Some(Self::LessThanOrEqual),
            22 => Some(Self::GreaterThanOrEqual),
            23 => Some(Self::And),
            24 => Some(Self::Or),
            25 => Some(Self::Not),
            26 => Some(Self::Jump),
            27 => Some(Self::JumpIfFalse),
            28 => Some(Self::JumpIfTrue),
            29 => Some(Self::Return),
            30 => Some(Self::Call),
            31 => Some(Self::DefineFunction),
            32 => Some(Self::NewList),
            33 => Some(Self::NewMap),
            34 => Some(Self::IndexGet),
            35 => Some(Self::IndexSet),
            36 => Some(Self::NewObject),
            37 => Some(Self::GetField),
            38 => Some(Self::SetField),
            39 => Some(Self::Halt),
            _ => None,
        }
    }
}

#[derive(Debug, Error)]
pub enum VmError {
    #[error("Stack overflow")]
    StackOverflow,
    
    #[error("Stack underflow")]
    StackUnderflow,
    
    #[error("Type error: {message}")]
    TypeError { message: String },
    
    #[error("Index out of bounds: {index}")]
    IndexOutOfBounds { index: usize },
    
    #[error("Undefined variable: {name}")]
    UndefinedVariable { name: String },
    
    #[error("Undefined function: {name}")]
    UndefinedFunction { name: String },
    
    #[error("Wrong number of arguments: expected {expected}, got {actual}")]
    WrongArgumentCount { expected: usize, actual: usize },
    
    #[error("Division by zero")]
    DivisionByZero,
    
    #[error("Runtime error: {message}")]
    RuntimeError { message: String },
}

pub struct VirtualMachine {
    stack: Vec<Value>,
    globals: HashMap<String, Value>,
    frames: Vec<CallFrame>,
    gc: GarbageCollector,
    chunk: Chunk,
    ip: usize,
}

#[derive(Debug)]
struct CallFrame {
    ip: usize,
    slot_start: usize,
}

impl VirtualMachine {
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            globals: HashMap::new(),
            frames: Vec::new(),
            gc: GarbageCollector::new(),
            chunk: Chunk::new(),
            ip: 0,
        }
    }

    pub fn interpret(&mut self, chunk: Chunk) -> Result<Value, VmError> {
        self.chunk = chunk;
        self.ip = 0;
        self.stack.clear();
        self.frames.clear();

        loop {
            if self.ip >= self.chunk.code.len() {
                break;
            }

            let instruction = self.chunk.code[self.ip];
            let op = OpCode::from_u8(instruction);

            match op {
                Some(OpCode::LoadConstant) => {
                    let constant = self.chunk.code[self.ip + 1];
                    self.ip += 1;
                    let value = self.chunk.constants[constant as usize].clone();
                    self.push(value);
                }
                Some(OpCode::LoadInt) => {
                    let bytes = [
                        self.chunk.code[self.ip + 1],
                        self.chunk.code[self.ip + 2],
                        self.chunk.code[self.ip + 3],
                        self.chunk.code[self.ip + 4],
                        self.chunk.code[self.ip + 5],
                        self.chunk.code[self.ip + 6],
                        self.chunk.code[self.ip + 7],
                        self.chunk.code[self.ip + 8],
                    ];
                    let value = i64::from_le_bytes(bytes);
                    self.ip += 8;
                    self.push(Value::Int(value));
                }
                Some(OpCode::LoadFloat) => {
                    let bytes = [
                        self.chunk.code[self.ip + 1],
                        self.chunk.code[self.ip + 2],
                        self.chunk.code[self.ip + 3],
                        self.chunk.code[self.ip + 4],
                        self.chunk.code[self.ip + 5],
                        self.chunk.code[self.ip + 6],
                        self.chunk.code[self.ip + 7],
                        self.chunk.code[self.ip + 8],
                    ];
                    let value = f64::from_le_bytes(bytes);
                    self.ip += 8;
                    self.push(Value::Float(value));
                }
                Some(OpCode::LoadString) => {
                    let constant = self.chunk.code[self.ip + 1];
                    self.ip += 1;
                    let value = self.chunk.constants[constant as usize].clone();
                    self.push(value);
                }
                Some(OpCode::LoadBool) => {
                    let value = self.chunk.code[self.ip + 1] != 0;
                    self.ip += 1;
                    self.push(Value::Bool(value));
                }
                Some(OpCode::LoadLocal) => {
                    let slot = self.chunk.code[self.ip + 1] as usize;
                    self.ip += 1;
                    let value = self.stack[self.stack.len() - slot - 1].clone();
                    self.push(value);
                }
                Some(OpCode::StoreLocal) => {
                    let slot = self.chunk.code[self.ip + 1] as usize;
                    self.ip += 1;
                    let value = self.pop()?;
                    self.stack[self.stack.len() - slot - 1] = value;
                }
                Some(OpCode::LoadGlobal) => {
                    let constant = self.chunk.code[self.ip + 1];
                    self.ip += 1;
                    if let Value::String(name) = &self.chunk.constants[constant as usize] {
                        if let Some(value) = self.globals.get(name) {
                            self.push(value.clone());
                        } else {
                            return Err(VmError::UndefinedVariable { name: name.clone() });
                        }
                    } else {
                        return Err(VmError::RuntimeError { message: "Global name must be a string".to_string() });
                    }
                }
                Some(OpCode::StoreGlobal) => {
                    let constant = self.chunk.code[self.ip + 1];
                    self.ip += 1;
                    let value = self.pop()?;
                    if let Value::String(name) = &self.chunk.constants[constant as usize] {
                        self.globals.insert(name.clone(), value);
                    } else {
                        return Err(VmError::RuntimeError { message: "Global name must be a string".to_string() });
                    }
                }
                Some(OpCode::Pop) => {
                    self.pop()?;
                }
                Some(OpCode::Duplicate) => {
                    let value = self.peek()?;
                    self.push(value);
                }
                Some(OpCode::Add) => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_add(a, b)?;
                    self.push(result);
                }
                Some(OpCode::Subtract) => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_subtract(a, b)?;
                    self.push(result);
                }
                Some(OpCode::Multiply) => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_multiply(a, b)?;
                    self.push(result);
                }
                Some(OpCode::Divide) => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_divide(a, b)?;
                    self.push(result);
                }
                Some(OpCode::Modulo) => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_modulo(a, b)?;
                    self.push(result);
                }
                Some(OpCode::Negate) => {
                    let value = self.pop()?;
                    let result = self.unary_negate(value)?;
                    self.push(result);
                }
                Some(OpCode::Equal) => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = Value::Bool(self.values_equal(&a, &b));
                    self.push(result);
                }
                Some(OpCode::NotEqual) => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = Value::Bool(!self.values_equal(&a, &b));
                    self.push(result);
                }
                Some(OpCode::LessThan) => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_less_than(a, b)?;
                    self.push(result);
                }
                Some(OpCode::GreaterThan) => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_greater_than(a, b)?;
                    self.push(result);
                }
                Some(OpCode::LessThanOrEqual) => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_less_than_equal(a, b)?;
                    self.push(result);
                }
                Some(OpCode::GreaterThanOrEqual) => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.binary_greater_than_equal(a, b)?;
                    self.push(result);
                }
                Some(OpCode::And) => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.logical_and(a, b)?;
                    self.push(result);
                }
                Some(OpCode::Or) => {
                    let b = self.pop()?;
                    let a = self.pop()?;
                    let result = self.logical_or(a, b)?;
                    self.push(result);
                }
                Some(OpCode::Not) => {
                    let value = self.pop()?;
                    let result = self.logical_not(value)?;
                    self.push(result);
                }
                Some(OpCode::Jump) => {
                    let offset = u16::from_le_bytes([self.chunk.code[self.ip + 1], self.chunk.code[self.ip + 2]]) as usize;
                    self.ip += 2;
                    self.ip += offset;
                    continue;
                }
                Some(OpCode::JumpIfFalse) => {
                    let offset = u16::from_le_bytes([self.chunk.code[self.ip + 1], self.chunk.code[self.ip + 2]]) as usize;
                    self.ip += 2;
                    if self.is_falsey(&self.peek()?) {
                        self.ip += offset;
                        continue;
                    }
                }
                Some(OpCode::JumpIfTrue) => {
                    let offset = u16::from_le_bytes([self.chunk.code[self.ip + 1], self.chunk.code[self.ip + 2]]) as usize;
                    self.ip += 2;
                    if !self.is_falsey(&self.peek()?) {
                        self.ip += offset;
                        continue;
                    }
                }
                Some(OpCode::Return) => {
                    return Ok(self.pop().unwrap_or(Value::Null));
                }
                Some(OpCode::Call) => {
                    let arg_count = self.chunk.code[self.ip + 1] as usize;
                    self.ip += 1;
                    self.call_value(arg_count)?;
                }
                Some(OpCode::NewList) => {
                    let element_count = self.chunk.code[self.ip + 1] as usize;
                    self.ip += 1;
                    let mut elements = Vec::new();
                    for _ in 0..element_count {
                        elements.push(self.pop()?);
                    }
                    elements.reverse();
                    let list_value = ListValue { elements };
                    let list_ptr = self.gc.allocate(list_value);
                    self.push(Value::List(list_ptr));
                }
                Some(OpCode::NewMap) => {
                    let map_value = MapValue { entries: HashMap::new() };
                    let map_ptr = self.gc.allocate(map_value);
                    self.push(Value::Map(map_ptr));
                }
                Some(OpCode::IndexGet) => {
                    let index = self.pop()?;
                    let collection = self.pop()?;
                    let result = self.index_get(collection, index)?;
                    self.push(result);
                }
                Some(OpCode::IndexSet) => {
                    let value = self.pop()?;
                    let index = self.pop()?;
                    let collection = self.pop()?;
                    self.index_set(collection, index, value)?;
                }
                Some(OpCode::Halt) => {
                    break;
                }
                _ => {
                    return Err(VmError::RuntimeError { message: format!("Unknown opcode: {:?}", instruction) });
                }
            }

            self.ip += 1;
        }

        Ok(self.pop().unwrap_or(Value::Null))
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Result<Value, VmError> {
        self.stack.pop().ok_or(VmError::StackUnderflow)
    }

    fn peek(&self) -> Result<Value, VmError> {
        self.stack.last().cloned().ok_or(VmError::StackUnderflow)
    }

    fn binary_add(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a + b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 + b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a + b as f64)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            _ => Err(VmError::TypeError { message: "Cannot add these types".to_string() }),
        }
    }

    fn binary_subtract(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a - b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 - b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a - b as f64)),
            _ => Err(VmError::TypeError { message: "Cannot subtract these types".to_string() }),
        }
    }

    fn binary_multiply(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a * b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 * b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a * b as f64)),
            _ => Err(VmError::TypeError { message: "Cannot multiply these types".to_string() }),
        }
    }

    fn binary_divide(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(_), Value::Int(0)) => Err(VmError::DivisionByZero),
            (Value::Float(_), Value::Float(0.0)) => Err(VmError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a / b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a / b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Float(a as f64 / b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Float(a / b as f64)),
            _ => Err(VmError::TypeError { message: "Cannot divide these types".to_string() }),
        }
    }

    fn binary_modulo(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(_), Value::Int(0)) => Err(VmError::DivisionByZero),
            (Value::Int(a), Value::Int(b)) => Ok(Value::Int(a % b)),
            _ => Err(VmError::TypeError { message: "Modulo only works on integers".to_string() }),
        }
    }

    fn unary_negate(&self, value: Value) -> Result<Value, VmError> {
        match value {
            Value::Int(v) => Ok(Value::Int(-v)),
            Value::Float(v) => Ok(Value::Float(-v)),
            _ => Err(VmError::TypeError { message: "Cannot negate non-numeric type".to_string() }),
        }
    }

    fn binary_less_than(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a < b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a < b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) < b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a < (b as f64))),
            _ => Err(VmError::TypeError { message: "Cannot compare these types".to_string() }),
        }
    }

    fn binary_greater_than(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a > b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a > b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) > b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a > (b as f64))),
            _ => Err(VmError::TypeError { message: "Cannot compare these types".to_string() }),
        }
    }

    fn binary_less_than_equal(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a <= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a <= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) <= b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a <= (b as f64))),
            _ => Err(VmError::TypeError { message: "Cannot compare these types".to_string() }),
        }
    }

    fn binary_greater_than_equal(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => Ok(Value::Bool(a >= b)),
            (Value::Float(a), Value::Float(b)) => Ok(Value::Bool(a >= b)),
            (Value::Int(a), Value::Float(b)) => Ok(Value::Bool((a as f64) >= b)),
            (Value::Float(a), Value::Int(b)) => Ok(Value::Bool(a >= (b as f64))),
            _ => Err(VmError::TypeError { message: "Cannot compare these types".to_string() }),
        }
    }

    fn logical_and(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
            _ => Err(VmError::TypeError { message: "Logical AND requires boolean operands".to_string() }),
        }
    }

    fn logical_or(&self, a: Value, b: Value) -> Result<Value, VmError> {
        match (a, b) {
            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
            _ => Err(VmError::TypeError { message: "Logical OR requires boolean operands".to_string() }),
        }
    }

    fn logical_not(&self, value: Value) -> Result<Value, VmError> {
        match value {
            Value::Bool(v) => Ok(Value::Bool(!v)),
            _ => Err(VmError::TypeError { message: "Logical NOT requires boolean operand".to_string() }),
        }
    }

    fn values_equal(&self, a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Int(a), Value::Int(b)) => a == b,
            (Value::Float(a), Value::Float(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Null, Value::Null) => true,
            _ => false,
        }
    }

    fn is_falsey(&self, value: &Value) -> bool {
        match value {
            Value::Bool(false) => true,
            Value::Null => true,
            Value::Int(0) => true,
            Value::Float(0.0) => true,
            Value::String(s) => s.is_empty(),
            _ => false,
        }
    }

    fn call_value(&mut self, arg_count: usize) -> Result<(), VmError> {
        // Simplified function calling - in a real implementation,
        // we'd handle different callable types
        let callable = self.stack[self.stack.len() - arg_count - 1].clone();
        
        match callable {
            Value::Function(function) => {
                if function.arity != arg_count {
                    return Err(VmError::WrongArgumentCount { expected: function.arity, actual: arg_count });
                }
                
                // Create new call frame
                let frame = CallFrame {
                    ip: self.ip,
                    slot_start: self.stack.len() - arg_count,
                };
                self.frames.push(frame);
                
                // Switch to function chunk
                self.chunk = function.chunk.clone();
                self.ip = 0;
                
                Ok(())
            }
            _ => Err(VmError::RuntimeError { message: "Can only call functions".to_string() }),
        }
    }

    fn index_get(&self, collection: Value, index: Value) -> Result<Value, VmError> {
        match (collection, index) {
            (Value::List(list_ptr), Value::Int(index)) => {
                let list = unsafe { &*list_ptr.as_raw() };
                if index < 0 || index >= list.elements.len() as i64 {
                    return Err(VmError::IndexOutOfBounds { index: index as usize });
                }
                Ok(list.elements[index as usize].clone())
            }
            _ => Err(VmError::TypeError { message: "Invalid index operation".to_string() }),
        }
    }

    fn index_set(&mut self, collection: Value, index: Value, value: Value) -> Result<(), VmError> {
        match (collection, index) {
            (Value::List(list_ptr), Value::Int(index)) => {
                let list = unsafe { &mut *list_ptr.as_raw() };
                if index < 0 || index >= list.elements.len() as i64 {
                    return Err(VmError::IndexOutOfBounds { index: index as usize });
                }
                list.elements[index as usize] = value;
                Ok(())
            }
            _ => Err(VmError::TypeError { message: "Invalid index operation".to_string() }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_arithmetic() {
        let mut vm = VirtualMachine::new();
        let mut chunk = Chunk::new();
        
        // Push 42
        chunk.write(OpCode::LoadInt.to_u8(), 1);
        chunk.code.extend_from_slice(&42i64.to_le_bytes());
        
        // Push 8
        chunk.write(OpCode::LoadInt.to_u8(), 2);
        chunk.code.extend_from_slice(&8i64.to_le_bytes());
        
        // Add
        chunk.write(OpCode::Add.to_u8(), 3);
        
        // Return
        chunk.write(OpCode::Return.to_u8(), 4);
        
        let result = vm.interpret(chunk).unwrap();
        assert_eq!(result, Value::Int(50));
    }

    #[test]
    fn test_string_concatenation() {
        let mut vm = VirtualMachine::new();
        let mut chunk = Chunk::new();
        
        let hello_const = chunk.add_constant(Value::String("Hello".to_string()));
        let world_const = chunk.add_constant(Value::String(", World".to_string()));
        
        chunk.write(OpCode::LoadString.to_u8(), 1);
        chunk.write(hello_const as u8, 1);
        
        chunk.write(OpCode::LoadString.to_u8(), 2);
        chunk.write(world_const as u8, 2);
        
        chunk.write(OpCode::Add.to_u8(), 3);
        chunk.write(OpCode::Return.to_u8(), 4);
        
        let result = vm.interpret(chunk).unwrap();
        assert_eq!(result, Value::String("Hello, World".to_string()));
    }
}
