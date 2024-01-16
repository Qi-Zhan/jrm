#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum ByteCode {
    /// Get static field from class
    GetStatic(u16),
    /// Push item from run-time constant pool
    Ldc(u8),
    /// Invoke instance method; dispatch based on class
    InvokeVirtual(u16),
    /// Push byte
    Bipush(i8),
    /// Return void from method
    Return,
    /// Push int constant
    IConst(i32),
    /// Invoke a class (static) method
    InvokeStatic(u16),
    /// Store reference into local variable
    AStore(u8),
    /// Store int into local variable
    IStore(u8),
    /// Load reference from local variable
    ALoad(u8),
    /// Load int from local variable
    ILoad(u8),
    /// Add int
    IAdd,
    /// Create new object
    New(u16),
    /// Duplicate the top operand stack value
    Dup,
    /// Invoke instance method
    InvokeSpecial(u16),
    /// Fetch field from object
    GetField(u16),
    /// Set field in object
    PutField(u16),
    /// Return int from method
    IReturn,
}

const GETSTATIC: u8 = 0xb2;
const LDC: u8 = 0x12;
const INVOKEVIRTUAL: u8 = 0xb6;
const BIPUSH: u8 = 0x10;
const RETURN: u8 = 0xb1;
const INVOKESTATIC: u8 = 0xb8;
const IADD: u8 = 0x60;
const NEW: u8 = 0xbb;
const DUP: u8 = 0x59;
const INVOKESPECIAL: u8 = 0xb7;
const ISTORE: u8 = 0x36;
const ILOAD: u8 = 0x15;
const GETFIELD: u8 = 0xb4;
const PUTFIELD: u8 = 0xb5;
const IRETURN: u8 = 0xac;

impl ByteCode {
    pub fn parse(pc: usize, code: &[u8]) -> (usize, Self) {
        use ByteCode::*;
        let op = code[pc];
        match op {
            NEW => {
                let index = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                (pc + 3, New(index))
            }
            RETURN => (pc + 1, Return),
            BIPUSH => {
                let value = code[pc + 1] as i8;
                (pc + 2, Bipush(value))
            }
            GETSTATIC => {
                let index = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                (pc + 3, GetStatic(index))
            }
            INVOKEVIRTUAL => {
                let index = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                (pc + 3, InvokeVirtual(index))
            }
            INVOKESPECIAL => {
                let index = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                (pc + 3, InvokeSpecial(index))
            }
            LDC => {
                let index = code[pc + 1];
                (pc + 2, Ldc(index))
            }
            INVOKESTATIC => {
                let index = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                (pc + 3, InvokeStatic(index))
            }
            DUP => (pc + 1, Dup),
            IADD => (pc + 1, IAdd),
            // iconst_m1..iconst_5
            0x3..=0x8 => {
                let value = op - 0x3;
                (pc + 1, IConst(value as i32))
            }
            // astore_0..astore_3
            0x4b..=0x4e => {
                let value = op - 0x4b;
                (pc + 1, AStore(value))
            }
            // aload_0..aload_3
            0x2a..=0x2d => {
                let value = op - 0x2a;
                (pc + 1, ALoad(value))
            }
            // istore_0..istore_3
            0x3b..=0x3e => {
                let value = op - 0x3b;
                (pc + 1, IStore(value))
            }
            ISTORE => (pc + 2, IStore(code[pc + 1])),
            // iload_0..iload_3
            0x1a..=0x1d => {
                let value = op - 0x1a;
                (pc + 1, ILoad(value))
            }
            ILOAD => (pc + 2, ILoad(code[pc + 1])),
            GETFIELD => {
                let index = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                (pc + 3, GetField(index))
            }
            PUTFIELD => {
                let index = u16::from_be_bytes([code[pc + 1], code[pc + 2]]);
                (pc + 3, PutField(index))
            }
            IRETURN => (pc + 1, IReturn),
            _ => {
                panic!("Unknown byte code: 0x{:x}", op);
            }
        }
    }
}
