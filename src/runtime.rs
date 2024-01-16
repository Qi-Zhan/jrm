use core::fmt;
use std::collections::HashMap;

use crate::{bytecode::ByteCode, class_file::ConstantInfo};

pub struct Frame<'a> {
    pub pc: usize,
    pub name: String,
    pub codes: Vec<u8>,
    pub operand_stack: Vec<Value>,
    pub locals: Vec<Value>,
    pub constant_pool: &'a Vec<ConstantInfo>,
}

impl<'a> Frame<'a> {
    pub fn new(
        name: &str,
        codes: &[u8],
        constant_pool: &'a Vec<ConstantInfo>,
        max_locals: u16,
        max_stack: u16,
    ) -> Self {
        let operand_stack = (0..max_stack).map(|_| Value::Int(0)).collect::<Vec<_>>();
        let locals = (0..max_locals).map(|_| Value::Int(0)).collect::<Vec<_>>();
        Self {
            pc: 0,
            name: name.to_string(),
            codes: codes.to_vec(),
            operand_stack,
            locals,
            constant_pool,
        }
    }

    pub fn fetch(&mut self) -> ByteCode {
        let (pc, bc) = ByteCode::parse(self.pc, &self.codes);
        self.pc = pc;
        bc
    }
}

pub struct Heap {
    /// Option<Instantce> is used to allow for null values and garbage collection
    pub instances: Vec<Option<Instantce>>,
}

impl Heap {
    pub fn new() -> Self {
        Self {
            instances: Vec::new(),
        }
    }

    pub fn malloc_instance(&mut self, class: usize) -> Reference {
        let index = self.instances.len();
        let instance = Instantce::new(class, index);
        let index = self.instances.len();
        self.instances.push(Some(instance));
        Reference::Object(index)
    }

    pub fn get(&self, reference: &Reference) -> &Instantce {
        match reference {
            Reference::Object(index) => self.instances[*index].as_ref().unwrap(),
            _ => panic!("Not implemented"),
        }
    }

    pub fn get_mut(&mut self, reference: &Reference) -> &mut Instantce {
        match reference {
            Reference::Object(index) => self.instances[*index].as_mut().unwrap(),
            _ => panic!("Not implemented"),
        }
    }

    /// Garbage collection
    pub fn gc(&mut self) {
        eprintln!("Garbage collection not implemented but we do not want to panic!")
    }
}

#[derive(Clone, Debug)]
pub enum Value {
    Boolean(bool),
    Byte(i8),
    Char(char),
    Short(i16),
    Int(i32),
    Float(f32),
    String(String),
    Reference(Reference),
    ReturnAddress(usize),
}

impl Value {
    pub fn as_reference(&self) -> Option<Reference> {
        match self {
            Value::Reference(reference) => Some(reference.clone()),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(value) => write!(f, "{}", value),
            Value::Byte(value) => write!(f, "{}", value),
            Value::Char(value) => write!(f, "{}", value),
            Value::Short(value) => write!(f, "{}", value),
            Value::Int(value) => write!(f, "{}", value),
            Value::Float(value) => write!(f, "{}", value),
            Value::Reference(reference) => write!(f, "{:?}", reference),
            Value::ReturnAddress(value) => write!(f, "{}", value),
            Value::String(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Clone, Debug)]
pub enum Reference {
    Class(usize),
    Array(usize),
    /// Instance of a class
    ///
    /// The usize is the index of the instance in the heap
    Object(usize),
}

impl Reference {
    pub fn as_object(&self) -> Option<usize> {
        match self {
            Reference::Object(index) => Some(*index),
            _ => None,
        }
    }
}

pub struct Instantce {
    pub class: usize,
    pub index: usize,
    pub fields: HashMap<String, Value>,
}

impl Instantce {
    pub fn new(class: usize, index: usize) -> Self {
        Self {
            class,
            index,
            fields: HashMap::new(),
        }
    }

    pub fn get_field(&self, name: &str) -> &Value {
        self.fields.get(name).unwrap()
    }

    pub fn put_field(&mut self, name: &str, value: Value) {
        self.fields.insert(name.to_string(), value);
    }
}
