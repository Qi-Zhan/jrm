//! This file contains the definition of the class file format.
//!
//! Each class file contains the definition of a single class or interface.

use std::fmt::Display;

use anyhow::{bail, Ok, Result};

type U1 = u8;
type U2 = u16;
type U4 = u32;

trait Read: Sized {
    fn read(bytes: &[u8], index: usize) -> Result<(usize, Self)>;
}
fn bound_check(bytes: &[u8], index: usize, len: usize) -> Result<()> {
    if index + len > bytes.len() {
        bail!(format!("index out of range: {}", index));
    }
    Ok(())
}

#[rustfmt::skip]
#[derive(Debug)]
pub struct ClassFile {
    /// The magic number identifying the class file format
    ///
    /// 0xCAFEBABE
    pub magic:                  U4,
    pub minor_version:          U2,
    pub major_version:          U2,
    /// The value of the constant_pool_count item is equal to
    /// the number of entries in the constant_pool table plus one.
    pub constant_pool_count:    U2,
    pub constant_pool:          Vec<ConstantInfo>,
    /// Denote the access permissions of the class or interface
    pub access_flags:           U2,
    /// The value of the this_class item must be a valid index into the constant_pool table.
    pub this_class:             U2,
    /// Zero or
    pub super_class:            U2,
    pub interfaces_count:       U2,
    pub interfaces:             Vec<U2>,
    pub fields_count:           U2,
    pub fields:                 Vec<FieldInfo>,
    pub methods_count:          U2,
    pub methods:                Vec<MethodInfo>,
    pub attributes_count:       U2,
    pub attributes:             Vec<AttributeInfo>,
}

#[derive(Debug)]
pub enum ConstantInfo {
    MethodRef {
        class_index: U2,
        name_and_type_index: U2,
    },
    FieldRef {
        class_index: U2,
        name_and_type_index: U2,
    },
    InterfaceMethodRef {
        class_index: U2,
        name_and_type_index: U2,
    },
    Class {
        name_index: U2,
    },
    NameAndType {
        name_index: U2,
        descriptor_index: U2,
    },
    Utf8(String),
    String(U2),
}

#[rustfmt::skip]
enum ConstantPoolTag {
    Utf8                = 1,
    Integer             = 3,
    Float               = 4,
    Long                = 5,
    Double              = 6,
    Class               = 7,
    String              = 8,
    FieldRef            = 9,
    MethodRef           = 10,
    InterfaceMethodRef  = 11,
    NameAndType         = 12,
    MethodHandle        = 15,
    MethodType          = 16,
    Dynamic             = 17,
    InvokeDynamic       = 18,
    Module              = 19,
    Package             = 20,
}

#[derive(Debug)]
pub struct FieldInfo {
    pub access_flags: U2,
    pub name_index: U2,
    pub descriptor_index: U2,
    pub attributes_count: U2,
    pub attributes: Vec<AttributeInfo>,
}
#[derive(Debug)]

#[rustfmt::skip]
pub struct MethodInfo {
    pub access_flags:       U2,
    pub name_index:         U2,
    pub descriptor_index:   U2,
    pub attributes_count:   U2,
    pub attributes:         Vec<AttributeInfo>,
}
#[derive(Debug, Clone)]
pub struct AttributeInfo {
    attribute_name_index: U2,
    attribute_length: U4,
    info: Vec<U1>,
}

#[derive(Debug, Clone)]
pub struct ExceptionTableEntry {
    pub start_pc: U2,
    pub end_pc: U2,
    pub handler_pc: U2,
    pub catch_type: U2,
}

#[derive(Debug, Clone)]
pub struct CodeAttribute {
    pub max_stack: U2,
    pub max_locals: U2,
    pub code_length: U4,
    pub code: Vec<U1>,
    pub exception_table_length: U2,
    pub exception_table: Vec<ExceptionTableEntry>,
    pub attributes_count: U2,
    pub attributes: Vec<AttributeInfo>,
}

impl From<&[U1]> for CodeAttribute {
    fn from(info: &[U1]) -> Self {
        let index = 0;
        let (index, max_stack) = U2::read(info, index).unwrap();
        let (index, max_locals) = U2::read(info, index).unwrap();
        let (mut index, code_length) = U4::read(info, index).unwrap();
        let code = info[index..(index + code_length as usize)].to_vec();
        index += code_length as usize;
        let (index, exception_table_length) = U2::read(info, index).unwrap();
        let exception_table = vec![];
        // for _ in 0..exception_table_length {
        //     let (index, start_pc) = U2::read(&attribute.info, index).unwrap();
        //     let (index, end_pc) = U2::read(&attribute.info, index).unwrap();
        //     let (index, handler_pc) = U2::read(&attribute.info, index).unwrap();
        //     let (index, catch_type) = U2::read(&attribute.info, index).unwrap();
        //     exception_table.push(ExceptionTableEntry {
        //         start_pc,
        //         end_pc,
        //         handler_pc,
        //         catch_type,
        //     });
        // }
        let (mut index, attributes_count) = U2::read(info, index).unwrap();
        let mut attributes = vec![];
        for _ in 0..attributes_count {
            let attribute = AttributeInfo::read(info, index).unwrap();
            index = attribute.0;
            attributes.push(attribute.1);
        }
        Self {
            max_stack,
            max_locals,
            code_length,
            code,
            exception_table_length,
            exception_table,
            attributes_count,
            attributes,
        }
    }
}

impl ClassFile {
    pub fn parse(path: &str) -> Result<Self> {
        let bytes = std::fs::read(path)?;
        // print!("raw bytes: ");
        // for byte in &bytes {
        //     print!("{:02X} ", byte);
        // }
        let (index, class) = Self::read(&bytes, 0)?;
        assert_eq!(index, bytes.len());
        Ok(class)
    }

    pub fn name(&self) -> &str {
        let class_index = self.constant_pool[self.this_class as usize]
            .as_class()
            .unwrap();
        self.constant_pool[class_index as usize].as_utf8().unwrap()
    }

    pub fn find_method(&self, target: &str) -> Option<&MethodInfo> {
        for method in &self.methods {
            let name = method.name(&self.constant_pool);
            if name == target {
                return Some(method);
            }
        }
        None
    }

    pub fn find_main_method(&self) -> Option<&MethodInfo> {
        self.find_method("main")
    }
}

enum AccessFlag {
    Public = 0x0001,
    Final = 0x0010,
    Super = 0x0020,
    Interface = 0x0200,
    Abstract = 0x0400,
    Synthetic = 0x1000,
    Annotation = 0x2000,
    Enum = 0x4000,
}

impl Read for ClassFile {
    fn read(bytes: &[u8], index: usize) -> Result<(usize, Self)> {
        let (index, magic) = U4::read(bytes, index)?;
        assert_eq!(magic, 0xCAFEBABE);
        let (index, minor_version) = U2::read(bytes, index)?;
        let (index, major_version) = U2::read(bytes, index)?;
        let (mut index, constant_pool_count) = U2::read(bytes, index)?;

        // constant pool index starts from 1
        let mut constant_pool = vec![ConstantInfo::Utf8("".to_string())];
        for _ in 1..constant_pool_count {
            let constant = ConstantInfo::read(bytes, index)?;
            index = constant.0;
            constant_pool.push(constant.1);
        }

        let (index, access_flags) = U2::read(bytes, index)?;
        let (index, this_class) = U2::read(bytes, index)?;
        let (index, super_class) = U2::read(bytes, index)?;

        let (mut index, interfaces_count) = U2::read(bytes, index)?;
        let mut interfaces = vec![];
        for _ in 0..interfaces_count {
            let interface = U2::read(bytes, index)?;
            index = interface.0;
            interfaces.push(interface.1);
        }

        let (mut index, fields_count) = U2::read(bytes, index)?;
        let mut fields = vec![];
        for _ in 0..fields_count {
            let field = FieldInfo::read(bytes, index)?;
            index = field.0;
            fields.push(field.1);
        }

        let (mut index, methods_count) = U2::read(bytes, index)?;
        let mut methods = vec![];
        for _ in 0..methods_count {
            let method = MethodInfo::read(bytes, index)?;
            index = method.0;
            methods.push(method.1);
        }

        let (mut index, attributes_count) = U2::read(bytes, index)?;
        let mut attributes = vec![];
        for _ in 0..attributes_count {
            let attribute = AttributeInfo::read(bytes, index)?;
            index = attribute.0;
            attributes.push(attribute.1);
        }

        Ok((
            index,
            Self {
                magic,
                minor_version,
                major_version,
                constant_pool_count,
                constant_pool,
                access_flags,
                this_class,
                super_class,
                interfaces_count,
                interfaces,
                fields_count,
                fields,
                methods_count,
                methods,
                attributes_count,
                attributes,
            },
        ))
    }
}

impl Read for U1 {
    fn read(bytes: &[u8], index: usize) -> Result<(usize, Self)> {
        bound_check(bytes, index, 1)?;
        let value = bytes[index];
        Ok((index + 1, value))
    }
}

impl Read for U2 {
    fn read(bytes: &[u8], index: usize) -> Result<(usize, Self)> {
        bound_check(bytes, index, 2)?;
        let value = ((bytes[index] as U2) << 8) | (bytes[index + 1] as U2);
        Ok((index + 2, value))
    }
}

impl Read for U4 {
    fn read(bytes: &[u8], index: usize) -> Result<(usize, Self)> {
        bound_check(bytes, index, 4)?;
        let value = ((bytes[index] as U4) << 24)
            | ((bytes[index + 1] as U4) << 16)
            | ((bytes[index + 2] as U4) << 8)
            | (bytes[index + 3] as U4);
        Ok((index + 4, value))
    }
}

impl Read for ConstantPoolTag {
    fn read(bytes: &[u8], index: usize) -> Result<(usize, Self)> {
        use ConstantPoolTag::*;
        let (index, tag) = U1::read(bytes, index)?;
        let tag = match tag {
            1 => Utf8,
            3 => Integer,
            4 => Float,
            5 => Long,
            6 => Double,
            7 => Class,
            8 => String,
            9 => FieldRef,
            10 => MethodRef,
            11 => InterfaceMethodRef,
            12 => NameAndType,
            15 => MethodHandle,
            16 => MethodType,
            17 => Dynamic,
            18 => InvokeDynamic,
            19 => Module,
            20 => Package,
            _ => bail!(format!("invalid constant pool tag: {}", tag)),
        };
        Ok((index, tag))
    }
}

impl Read for ConstantInfo {
    fn read(bytes: &[u8], index: usize) -> Result<(usize, Self)> {
        let (index, tag) = ConstantPoolTag::read(bytes, index)?;
        Ok(match tag {
            ConstantPoolTag::Class => {
                let (index, name_index) = U2::read(bytes, index)?;
                (index, ConstantInfo::Class { name_index })
            }
            ConstantPoolTag::FieldRef => {
                let (index, class_index) = U2::read(bytes, index)?;
                let (index, name_and_type_index) = U2::read(bytes, index)?;
                (
                    index,
                    ConstantInfo::FieldRef {
                        class_index,
                        name_and_type_index,
                    },
                )
            }
            ConstantPoolTag::MethodRef => {
                let (index, class_index) = U2::read(bytes, index)?;
                let (index, name_and_type_index) = U2::read(bytes, index)?;
                (
                    index,
                    ConstantInfo::MethodRef {
                        class_index,
                        name_and_type_index,
                    },
                )
            }
            ConstantPoolTag::InterfaceMethodRef => {
                let (index, class_index) = U2::read(bytes, index)?;
                let (index, name_and_type_index) = U2::read(bytes, index)?;
                (
                    index,
                    ConstantInfo::InterfaceMethodRef {
                        class_index,
                        name_and_type_index,
                    },
                )
            }
            ConstantPoolTag::String => {
                let (index, string_index) = U2::read(bytes, index)?;
                (index, ConstantInfo::String(string_index))
            }
            ConstantPoolTag::Integer => todo!("integer"),
            ConstantPoolTag::Float => todo!("float"),
            ConstantPoolTag::Long => todo!("long"),
            ConstantPoolTag::Double => todo!("double"),
            ConstantPoolTag::NameAndType => {
                let (index, name_index) = U2::read(bytes, index)?;
                let (index, descriptor_index) = U2::read(bytes, index)?;
                (
                    index,
                    ConstantInfo::NameAndType {
                        name_index,
                        descriptor_index,
                    },
                )
            }
            ConstantPoolTag::Utf8 => {
                let (index, length) = U2::read(bytes, index)?;
                let length = length as usize;
                let string = String::from_utf8_lossy(&bytes[index..(index + length)]).to_string();
                (index + length, ConstantInfo::Utf8(string))
            }
            ConstantPoolTag::MethodHandle => todo!("method handle"),
            ConstantPoolTag::MethodType => todo!("method type"),
            ConstantPoolTag::Dynamic => todo!("dynamic"),
            ConstantPoolTag::InvokeDynamic => todo!("invoke dynamic"),
            ConstantPoolTag::Module => todo!("module"),
            ConstantPoolTag::Package => todo!("package"),
        })
    }
}

impl Read for FieldInfo {
    fn read(bytes: &[u8], index: usize) -> Result<(usize, Self)> {
        let (index, access_flags) = U2::read(bytes, index)?;
        let (index, name_index) = U2::read(bytes, index)?;
        let (index, descriptor_index) = U2::read(bytes, index)?;
        let (mut index, attributes_count) = U2::read(bytes, index)?;
        let mut attributes = vec![];
        for _ in 0..attributes_count {
            let attribute = AttributeInfo::read(bytes, index)?;
            index = attribute.0;
            attributes.push(attribute.1);
        }
        Ok((
            index,
            Self {
                access_flags,
                name_index,
                descriptor_index,
                attributes_count,
                attributes,
            },
        ))
    }
}

impl Read for MethodInfo {
    fn read(bytes: &[u8], index: usize) -> Result<(usize, Self)> {
        let (index, access_flags) = U2::read(bytes, index)?;
        let (index, name_index) = U2::read(bytes, index)?;
        let (index, descriptor_index) = U2::read(bytes, index)?;
        let (mut index, attributes_count) = U2::read(bytes, index)?;
        let mut attributes = vec![];
        for _ in 0..attributes_count {
            let attribute = AttributeInfo::read(bytes, index)?;
            index = attribute.0;
            attributes.push(attribute.1);
        }
        Ok((
            index,
            Self {
                access_flags,
                name_index,
                descriptor_index,
                attributes_count,
                attributes,
            },
        ))
    }
}

impl Read for AttributeInfo {
    fn read(bytes: &[u8], index: usize) -> Result<(usize, Self)> {
        let (index, attribute_name_index) = U2::read(bytes, index)?;
        let (index, attribute_length) = U4::read(bytes, index)?;
        let info = bytes[index..(index + attribute_length as usize)].to_vec();
        Ok((
            index + attribute_length as usize,
            Self {
                attribute_name_index,
                attribute_length,
                info,
            },
        ))
    }
}

impl Display for ConstantInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ConstantInfo::*;
        match self {
            Class { name_index } => write!(f, "Class #{}", name_index),
            FieldRef {
                class_index,
                name_and_type_index,
            } => write!(f, "Fieldref #{}.#{}", class_index, name_and_type_index),
            MethodRef {
                class_index,
                name_and_type_index,
            } => write!(f, "Methodref #{}.#{}", class_index, name_and_type_index),
            InterfaceMethodRef {
                class_index,
                name_and_type_index,
            } => write!(
                f,
                "InterfaceMethodref #{}.#{}",
                class_index, name_and_type_index
            ),
            String(string_index) => write!(f, "String #{}", string_index),
            Integer => write!(f, "Integer"),
            Float => write!(f, "Float"),
            Long => write!(f, "Long"),
            Double => write!(f, "Double"),
            NameAndType {
                name_index,
                descriptor_index,
            } => write!(f, "NameAndType #{}:#{}", name_index, descriptor_index),
            Utf8(string) => write!(f, "Utf8 \"{}\"", string),
            MethodHandle => write!(f, "MethodHandle"),
            MethodType => write!(f, "MethodType"),
            Dynamic => write!(f, "Dynamic"),
            InvokeDynamic => write!(f, "InvokeDynamic"),
            Module => write!(f, "Module"),
            Package => write!(f, "Package"),
        }
    }
}

impl ConstantInfo {
    pub fn as_utf8(&self) -> Option<&str> {
        match self {
            ConstantInfo::Utf8(string) => Some(string),
            _ => None,
        }
    }

    pub fn as_class(&self) -> Option<u16> {
        match self {
            ConstantInfo::Class { name_index } => Some(*name_index),
            _ => None,
        }
    }

    pub fn as_name_and_type(&self) -> Option<(u16, u16)> {
        match self {
            ConstantInfo::NameAndType {
                name_index,
                descriptor_index,
            } => Some((*name_index, *descriptor_index)),
            _ => None,
        }
    }

    pub fn as_field_ref(&self) -> Option<(u16, u16)> {
        match self {
            ConstantInfo::FieldRef {
                class_index,
                name_and_type_index,
            } => Some((*class_index, *name_and_type_index)),
            _ => None,
        }
    }

    pub fn as_method_ref(&self) -> Option<(u16, u16)> {
        match self {
            ConstantInfo::MethodRef {
                class_index,
                name_and_type_index,
            } => Some((*class_index, *name_and_type_index)),
            _ => None,
        }
    }
    
}

impl<'a> MethodInfo {
    pub fn name(&'a self, constant_pool: &'a [ConstantInfo]) -> &'a str {
        constant_pool[self.name_index as usize].as_utf8().unwrap()
    }

    pub fn code(&self, constant_pool: &[ConstantInfo]) -> CodeAttribute {
        for attribute in &self.attributes {
            if constant_pool[attribute.attribute_name_index as usize]
                .as_utf8()
                .unwrap()
                == "Code"
            {
                return (*attribute.info).into();
            }
        }
        panic!("no code attribute found")
    }
}
