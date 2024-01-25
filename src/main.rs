mod bytecode;
mod class_file;
mod runtime;

use bytecode::ByteCode;
use class_file::{ClassFile, ConstantInfo};
use runtime::{Frame, Heap, Value};

fn args_size(descriptor: &str) -> usize {
    let mut size = 0;
    let mut chars = descriptor.chars();
    while let Some(c) = chars.next() {
        match c {
            '(' => {}
            ')' => break,
            'I' => size += 1,
            'L' => {
                for c in chars.by_ref() {
                    if c == ';' {
                        break;
                    }
                }
                size += 1;
            }
            _ => unimplemented!("Not implemented args_size for {}", c),
        }
    }
    size
}

fn class_method(index: usize, constant_pool: &[ConstantInfo]) -> (&str, &str) {
    let (class_index, name_and_type_index) = constant_pool[index].as_method_ref().unwrap();
    let class_index = constant_pool[class_index as usize].as_class().unwrap();
    let class_name = constant_pool[class_index as usize].as_utf8().unwrap();
    let (name_index, type_index) = constant_pool[name_and_type_index as usize]
        .as_name_and_type()
        .unwrap();
    let (method_name, _type) = (
        constant_pool[name_index as usize].as_utf8().unwrap(),
        constant_pool[type_index as usize].as_utf8().unwrap(),
    );
    (class_name, method_name)
}

fn main() {
    let args = std::env::args().collect::<Vec<String>>();
    if args.len() < 2 {
        println!("Usage: {} <class file>", args[0]);
        return;
    }
    let path = &args[1];

    let class = ClassFile::parse(path).unwrap();
    let classes = [&class];
    let constant_pool = &class.constant_pool;
    let main_method = class.find_main_method().expect("No main method found");
    let code_attribute = main_method.code(constant_pool);

    let mut heap = Heap::new();
    let frame = Frame::new(
        main_method.name(constant_pool),
        &code_attribute.code,
        constant_pool,
        code_attribute.max_locals,
        code_attribute.max_stack,
    );
    let mut stack = vec![frame];
    let mut current_frame_index = 0;

    loop {
        let current_frame = &mut stack[current_frame_index];
        let func = current_frame.name.clone();
        let mut next_frame_index = current_frame_index;
        let bc = current_frame.fetch();
        match bc {
            ByteCode::Return => {
                if current_frame_index == 0 {
                    return;
                } else {
                    stack.pop();
                    next_frame_index = current_frame_index - 1;
                }
            }
            ByteCode::IReturn => {
                assert_ne!(current_frame_index, 0);
                let value = current_frame.operand_stack.pop().unwrap();
                stack.pop();
                next_frame_index = current_frame_index - 1;
                let current_frame = &mut stack[next_frame_index];
                current_frame.operand_stack.push(value);
            }
            ByteCode::IConst(value) => {
                current_frame.operand_stack.push(Value::Int(value));
            }
            ByteCode::Bipush(value) => {
                current_frame.operand_stack.push(Value::Int(value as i32));
            }
            ByteCode::IStore(index) => {
                let value = current_frame.operand_stack.pop().unwrap();
                assert!(matches!(value, Value::Int(_)));
                current_frame.locals[index as usize] = value;
            }
            ByteCode::ILoad(index) => {
                let value = current_frame.locals[index as usize].clone();
                assert!(matches!(value, Value::Int(_)));
                current_frame.operand_stack.push(value);
            }
            ByteCode::New(index) => {
                let class_index = constant_pool[index as usize].as_class().unwrap();
                let instance = heap.malloc_instance(class_index as usize);
                current_frame.operand_stack.push(Value::Reference(instance));
            }
            ByteCode::Dup => {
                let value = current_frame.operand_stack.pop().unwrap();
                current_frame.operand_stack.push(value.clone());
                current_frame.operand_stack.push(value);
            }
            ByteCode::GetField(index) => {
                let reference = &current_frame
                    .operand_stack
                    .pop()
                    .unwrap()
                    .as_reference()
                    .unwrap();
                let instance = heap.get(reference);
                let field_ref = &current_frame.constant_pool[index as usize]
                    .as_field_ref()
                    .unwrap();
                let name_index = current_frame.constant_pool[field_ref.1 as usize]
                    .as_name_and_type()
                    .unwrap()
                    .0;
                let field_name = current_frame.constant_pool[name_index as usize]
                    .as_utf8()
                    .unwrap();
                current_frame.operand_stack.push(instance.get_field(field_name).clone());
            }
            ByteCode::PutField(index) => {
                let value = current_frame.operand_stack.pop().unwrap();
                let reference = &current_frame
                    .operand_stack
                    .pop()
                    .unwrap()
                    .as_reference()
                    .unwrap();
                let instance = heap.get_mut(reference);
                let field_ref = &current_frame.constant_pool[index as usize]
                    .as_field_ref()
                    .unwrap();
                let name_index = current_frame.constant_pool[field_ref.1 as usize]
                    .as_name_and_type()
                    .unwrap()
                    .0;
                let field_name = current_frame.constant_pool[name_index as usize]
                    .as_utf8()
                    .unwrap();
                instance.put_field(field_name, value);
            }
            ByteCode::GetStatic(index) => {
                let value = &current_frame.constant_pool[index as usize];
                let (class_index, name_and_type_index) = value.as_field_ref().unwrap();
                let class_index = current_frame.constant_pool[class_index as usize]
                    .as_class()
                    .unwrap();
                let class_name = current_frame.constant_pool[class_index as usize]
                    .as_utf8()
                    .unwrap();
                let (name_index, type_index) = current_frame.constant_pool
                    [name_and_type_index as usize]
                    .as_name_and_type()
                    .unwrap();
                let (name, type_) = (
                    current_frame.constant_pool[name_index as usize]
                        .as_utf8()
                        .unwrap(),
                    current_frame.constant_pool[type_index as usize]
                        .as_utf8()
                        .unwrap(),
                );
                if class_name == "java/lang/System"
                    && name == "out"
                    && type_ == "Ljava/io/PrintStream;"
                {
                    let out = heap.malloc_instance(0);
                    current_frame.operand_stack.push(Value::Reference(out));
                } else {
                    todo!("Not implemented getstatic");
                }
            }
            ByteCode::IAdd => {
                let value1 = current_frame.operand_stack.pop().unwrap();
                let value2 = current_frame.operand_stack.pop().unwrap();
                match (&value1, &value2) {
                    (Value::Int(v1), Value::Int(v2)) => {
                        current_frame.operand_stack.push(Value::Int(v1 + v2))
                    }
                    _ => {
                        dbg!(&value1);
                        dbg!(&value2);
                        todo!("Not implemented iadd");
                    }
                }
            }
            ByteCode::ALoad(index) => {
                let value = current_frame.locals[index as usize].clone();
                assert!(matches!(value, Value::Reference(_)));
                current_frame.operand_stack.push(value);
            }
            ByteCode::AStore(index) => {
                let value = current_frame.operand_stack.pop().unwrap();
                assert!(matches!(value, Value::Reference(_)));
                current_frame.locals[index as usize] = value;
            }
            ByteCode::Ldc(index) => {
                let value = &current_frame.constant_pool[index as usize];
                match value {
                    ConstantInfo::String(value) => {
                        let value = current_frame.constant_pool[*value as usize]
                            .as_utf8()
                            .unwrap();
                        current_frame
                            .operand_stack
                            .push(Value::String(value.to_string()));
                    }
                    _ => todo!("Not implemented ldc"),
                }
            }
            ByteCode::InvokeSpecial(index) => {
                let (class_name, method_name) =
                    class_method(index as usize, current_frame.constant_pool);
                if class_name == "java/lang/Object" && method_name == "<init>" {
                    // consume the reference, do nothing
                    current_frame.operand_stack.pop().unwrap();
                    continue;
                }
                let class = classes
                    .iter()
                    .find(|c| c.name() == class_name)
                    .expect("Class not found");
                let method = class.find_method(method_name).expect("Method not found");
                let descriptor = &class.constant_pool[method.descriptor_index as usize]
                    .as_utf8()
                    .unwrap();
                let args_size = args_size(descriptor);
                let code = method.code(&class.constant_pool);
                let mut frame = Frame::new(
                    method.name(&class.constant_pool),
                    &code.code,
                    &class.constant_pool,
                    code.max_locals,
                    code.max_stack,
                );
                for i in 0..args_size + 1 {
                    frame.locals[args_size - i] = current_frame.operand_stack.pop().unwrap();
                }
                stack.push(frame);
                next_frame_index = current_frame_index + 1;
            }
            ByteCode::InvokeVirtual(index) => {
                let (class_name, method_name) =
                    class_method(index as usize, current_frame.constant_pool);
                // special case for println
                if class_name == "java/io/PrintStream" && method_name == "println" {
                    let value = current_frame.operand_stack.pop().unwrap();
                    println!("{}", value);
                    continue;
                }
                // common case
                let class = classes
                    .iter()
                    .find(|c| c.name() == class_name)
                    .expect("Class not found");
                let method = class.find_method(method_name).expect("Method not found");
                let descriptor = &class.constant_pool[method.descriptor_index as usize]
                    .as_utf8()
                    .unwrap();
                let args_size = args_size(descriptor);
                let code = method.code(&class.constant_pool);
                let mut frame = Frame::new(
                    method.name(&class.constant_pool),
                    &code.code,
                    &class.constant_pool,
                    code.max_locals,
                    code.max_stack,
                );
                // + 1 for `this`
                for i in 0..args_size + 1 {
                    frame.locals[args_size-i] = current_frame.operand_stack.pop().unwrap();
                }
                stack.push(frame);
                next_frame_index = current_frame_index + 1;
            }
            _ => {
                println!("Unimplemented: {:?}", bc);
            }
        }
        if next_frame_index < current_frame_index {
            // garbage collection when returning from a method
            heap.gc(&stack, &func);
        }
        current_frame_index = next_frame_index;
    }
}
