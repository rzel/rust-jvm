use std::cell::RefCell;
use std::num::Wrapping;
use std::rc::Rc;

use model::class_file::access_flags::class_access_flags;

use vm::{sig, symref};
use vm::bytecode::opcode;
use vm::class::{Class, Method, MethodCode};
use vm::class_loader::ClassLoader;
use vm::constant_pool::RuntimeConstantPoolEntry;
use vm::sig::Type;
use vm::value::{Array, Scalar, Value};

/// A frame is used to store data and partial results, as well as to perform dynamic linking,
/// return values for methods, and dispatch exceptions.
#[derive(Debug)]
pub struct Frame<'a> {
    /// A reference to the class containing the currently executing method.
    current_class: &'a Class,
    /// Contains the bytecode currently executing in this frame, along with related structures.
    method_code: &'a MethodCode,
    /// The current program counter.
    pc: u16,
    /// The local variables of the current method.
    /// Values that occupy two indices (`long` and `double`) are stored in one slot followed by a
    /// `None` value in the subsequent index.
    local_variables: Vec<Option<Value>>,
    /// The operand stack manipulated by the instructions of the current method.
    operand_stack: Vec<Value>,
}

impl<'a> Frame<'a> {
    pub fn new(current_class: &'a Class, method_code: &'a MethodCode,
               local_variables: Vec<Option<Value>>) -> Self {
        Frame {
            current_class: current_class,
            method_code: method_code,
            pc: 0,
            local_variables: local_variables,
            operand_stack: vec![],
        }
    }

    fn read_next_byte(&mut self) -> u8 {
        let result = self.method_code.code[self.pc as usize];
        self.pc += 1;
        result
    }

    fn read_next_short(&mut self) -> u16 {
        ((self.read_next_byte() as u16) << 8) | (self.read_next_byte() as u16)
    }

    fn pop_as_locals(&mut self, count: usize) -> Vec<Option<Value>> {
        let mut result = vec![];
        let start_index = self.operand_stack.len() - count;
        for value in self.operand_stack.split_at(start_index).1 {
            result.push(Some(value.clone()));
            match *value {
                Value::Long(_) | Value::Double(_) => result.push(None),
                _ => (),
            }
        }
        result
    }

    fn invoke(&mut self, class_loader: &mut ClassLoader, class: &Class, method: &Method,
              args: Vec<Option<Value>>) {
        match method.method_code {
            None => (),
            Some(ref method_code) => {
                let frame = Frame::new(class, method_code, args);
                match frame.run(class_loader) {
                    None => (),
                    Some(return_value) => self.operand_stack.push(return_value)
                }
            }
        }
    }

    pub fn run(mut self, class_loader: &mut ClassLoader) -> Option<Value> {
        macro_rules! with {
            ($read_next_action: ident, $k: ident) => ({
                let value = self.$read_next_action() as u16;
                $k!(value);
            })
        }

        macro_rules! do_ipush {
            ($value: ident) => (self.operand_stack.push(Value::Int(Wrapping($value as i32))))
        }

        macro_rules! do_ldc {
            ($index: ident) => ({
                // TODO: should use resolve_literal
                let value = self.current_class.get_constant_pool()
                                .resolve_literal($index, class_loader).unwrap();
                self.operand_stack.push(value);
            });
        }

        macro_rules! do_load {
            ($index: expr) => ({
                let value = self.local_variables[$index as usize].clone().unwrap();
                self.operand_stack.push(value);
            })
        }

        macro_rules! do_store {
            ($index: expr) => ({
                let value = self.operand_stack.pop().unwrap();
                // invalidate the slot after this one if we're storing a category 2 operand
                match value {
                    Value::Int(_) | Value::Float(_) | Value::ScalarReference(_)
                            | Value::ArrayReference(_) | Value::NullReference => (),
                    Value::Long(_) | Value::Double(_) => {
                        self.local_variables[($index + 1) as usize] = None;
                    },
                }
                // actually store the local variable
                self.local_variables[$index as usize] = Some(value);
                // invalidate the slot before this one if it was formerly storing a category 2
                // operand
                let prev_index = $index - 1;
                if prev_index > 0 {
                    match self.local_variables[prev_index as usize] {
                        None | Some(Value::Int(_)) | Some(Value::Float(_))
                                | Some(Value::ScalarReference(_)) | Some(Value::ArrayReference(_))
                                | Some(Value::NullReference) => (),
                        Some(Value::Long(_)) | Some(Value::Double(_)) => {
                            self.local_variables[prev_index as usize] = None;
                        },
                    }
                }
            })
        }

        loop {
            match self.read_next_byte() {
                opcode::NOP => (),
                opcode::ACONST_NULL => self.operand_stack.push(Value::NullReference),
                opcode::ICONST_M1 => self.operand_stack.push(Value::Int(Wrapping(-1))),
                opcode::ICONST_0 => self.operand_stack.push(Value::Int(Wrapping(0))),
                opcode::ICONST_1 => self.operand_stack.push(Value::Int(Wrapping(1))),
                opcode::ICONST_2 => self.operand_stack.push(Value::Int(Wrapping(2))),
                opcode::ICONST_3 => self.operand_stack.push(Value::Int(Wrapping(3))),
                opcode::ICONST_4 => self.operand_stack.push(Value::Int(Wrapping(4))),
                opcode::ICONST_5 => self.operand_stack.push(Value::Int(Wrapping(5))),
                opcode::LCONST_0 => self.operand_stack.push(Value::Long(Wrapping(0))),
                opcode::LCONST_1 => self.operand_stack.push(Value::Long(Wrapping(1))),
                opcode::FCONST_0 => self.operand_stack.push(Value::Float(0.0)),
                opcode::FCONST_1 => self.operand_stack.push(Value::Float(1.0)),
                opcode::FCONST_2 => self.operand_stack.push(Value::Float(2.0)),
                opcode::DCONST_0 => self.operand_stack.push(Value::Double(0.0)),
                opcode::DCONST_1 => self.operand_stack.push(Value::Double(1.0)),
                opcode::BIPUSH => with!(read_next_byte, do_ipush),
                opcode::SIPUSH => with!(read_next_short, do_ipush),
                opcode::LDC => with!(read_next_byte, do_ldc),
                opcode::LDC_W | opcode::LDC2_W => with!(read_next_short, do_ldc),

                // these are a little out of order, since we combine identical cases
                opcode::ILOAD | opcode::LLOAD | opcode::FLOAD | opcode::DLOAD | opcode::ALOAD =>
                    with!(read_next_byte, do_load),
                opcode::ILOAD_0 | opcode::LLOAD_0 | opcode::FLOAD_0 | opcode::DLOAD_0
                        | opcode::ALOAD_0 =>
                    do_load!(0),
                opcode::ILOAD_1 | opcode::LLOAD_1 | opcode::FLOAD_1 | opcode::DLOAD_1
                        | opcode::ALOAD_1 =>
                    do_load!(1),
                opcode::ILOAD_2 | opcode::LLOAD_2 | opcode::FLOAD_2 | opcode::DLOAD_2
                        | opcode::ALOAD_2 =>
                    do_load!(2),
                opcode::ILOAD_3 | opcode::LLOAD_3 | opcode::FLOAD_3 | opcode::DLOAD_3
                        | opcode::ALOAD_3 =>
                    do_load!(3),
                opcode::IALOAD | opcode::LALOAD | opcode::FALOAD | opcode::DALOAD
                        | opcode::AALOAD | opcode::BALOAD | opcode::CALOAD | opcode::SALOAD => {
                    let index_value = self.operand_stack.pop().unwrap();
                    if let Value::Int(Wrapping(index)) = index_value {
                        let array_value = self.operand_stack.pop().unwrap();
                        match array_value {
                            Value::ArrayReference(array_rc) => {
                                let component = array_rc.borrow().get(index);
                                self.operand_stack.push(component);
                            },
                            Value::NullReference => panic!("NullPointerException"),
                            _ => panic!("xaload instruction on non-array value"),
                        }
                    } else {
                        panic!("xaload instruction on non-integer index");
                    }
                },

                // same thing here
                opcode::ISTORE | opcode::LSTORE | opcode::FSTORE | opcode::DSTORE | opcode::ASTORE =>
                    with!(read_next_byte, do_store),
                opcode::ISTORE_0 | opcode::LSTORE_0 | opcode::FSTORE_0 | opcode::DSTORE_0
                        | opcode::ASTORE_0 =>
                    do_store!(0),
                opcode::ISTORE_1 | opcode::LSTORE_1 | opcode::FSTORE_1 | opcode::DSTORE_1
                        | opcode::ASTORE_1 =>
                    do_store!(1),
                opcode::ISTORE_2 | opcode::LSTORE_2 | opcode::FSTORE_2 | opcode::DSTORE_2
                        | opcode::ASTORE_2 =>
                    do_store!(2),
                opcode::ISTORE_3 | opcode::LSTORE_3 | opcode::FSTORE_3 | opcode::DSTORE_3
                        | opcode::ASTORE_3 =>
                    do_store!(3),
                opcode::IASTORE | opcode::LASTORE | opcode::FASTORE | opcode::DASTORE
                        | opcode::AASTORE | opcode::BASTORE | opcode::CASTORE | opcode::SASTORE => {
                    let value = self.operand_stack.pop().unwrap();
                    let index_value = self.operand_stack.pop().unwrap();
                    if let Value::Int(Wrapping(index)) = index_value {
                        let array_value = self.operand_stack.pop().unwrap();
                        match array_value {
                            Value::ArrayReference(array_rc) => {
                                array_rc.borrow_mut().put(index, value);
                            },
                            Value::NullReference => panic!("NullPointerException"),
                            _ => panic!("xastore instruction on non-array value"),
                        }
                    } else {
                        panic!("xastore instruction on non-integer index");
                    }
                },

                opcode::POP => {
                    self.operand_stack.pop();
                },
                opcode::POP2 => {
                    match self.operand_stack.pop() {
                        Some(Value::Long(_)) | Some(Value::Double(_)) => (),
                        _ => {
                            self.operand_stack.pop();
                        },
                    }
                },
                opcode::DUP => {
                    let value = self.operand_stack.last().unwrap().clone();
                    self.operand_stack.push(value);
                },
                opcode::DUP_X1 => {
                    let value1 = self.operand_stack.pop().unwrap();
                    let value2 = self.operand_stack.pop().unwrap();
                    self.operand_stack.extend_from_slice(&[value1.clone(), value2, value1]);
                },
                opcode::DUP_X2 => {
                    let value1 = self.operand_stack.pop().unwrap();
                    let value2 = self.operand_stack.pop().unwrap();
                    match value2 {
                        Value::Long(_) | Value::Double(_) => {
                            self.operand_stack.extend_from_slice(&[value1.clone(), value2, value1]);
                        },
                        _ => {
                            let value3 = self.operand_stack.pop().unwrap();
                            self.operand_stack.extend_from_slice(
                                &[value1.clone(), value3, value2, value1]);
                        },
                    }
                },
                opcode::DUP2 => {
                    let value1 = self.operand_stack.pop().unwrap();
                    match value1 {
                        Value::Long(_) | Value::Double(_) => {
                            self.operand_stack.extend_from_slice(&[value1.clone(), value1]);
                        },
                        _ => {
                            let value2 = self.operand_stack.pop().unwrap();
                            self.operand_stack.extend_from_slice(
                                &[value2.clone(), value1.clone(), value2, value1]);
                        },
                    }
                },

                opcode::RETURN => return None,

                opcode::GETSTATIC => {
                    let index = self.read_next_short();
                    if let Some(RuntimeConstantPoolEntry::FieldRef(ref symref)) =
                            self.current_class.get_constant_pool()[index] {
                        let resolved_class = class_loader.resolve_class(&symref.class).unwrap();
                        let value = resolved_class.resolve_and_get_field(symref, class_loader);
                        self.operand_stack.push(value)
                    } else {
                        panic!("getstatic refers to non-field in constant pool");
                    }
                },

                opcode::PUTSTATIC => {
                    let index = self.read_next_short();
                    if let Some(RuntimeConstantPoolEntry::FieldRef(ref symref)) =
                            self.current_class.get_constant_pool()[index] {
                        let resolved_class = class_loader.resolve_class(&symref.class).unwrap();
                        let new_value = self.operand_stack.pop().unwrap();
                        resolved_class.resolve_and_put_field(symref, new_value, class_loader);
                    } else {
                        panic!("putstatic refers to non-field in constant pool");
                    }
                },

                opcode::GETFIELD => {
                    let index = self.read_next_short();
                    if let Some(RuntimeConstantPoolEntry::FieldRef(ref symref)) =
                            self.current_class.get_constant_pool()[index] {
                        match self.operand_stack.pop().unwrap() {
                            Value::ScalarReference(object_rc) => {
                                let value = object_rc.borrow().get_field(&symref.sig).clone();
                                self.operand_stack.push(value);
                            },
                            Value::ArrayReference(_) => panic!("getfield called on array"),
                            Value::NullReference => panic!("NullPointerException"),
                            _ => panic!("getfield called on a primitive value"),
                        }
                    } else {
                        panic!("getfield refers to non-field in constant pool");
                    }
                },

                opcode::PUTFIELD => {
                    let index = self.read_next_short();
                    let value = self.operand_stack.pop().unwrap();
                    if let Some(RuntimeConstantPoolEntry::FieldRef(ref symref)) =
                            self.current_class.get_constant_pool()[index] {
                        match self.operand_stack.pop().unwrap() {
                            Value::ScalarReference(object_rc) => {
                                object_rc.borrow_mut().put_field(symref.sig.clone(), value);
                            },
                            Value::ArrayReference(_) => panic!("putfield called on array"),
                            Value::NullReference => panic!("NullPointerException"),
                            _ => panic!("putfield called on a primitive value"),
                        }
                    } else {
                        panic!("putfield refers to non-field in constant pool");
                    }
                },

                opcode::INVOKEVIRTUAL => {
                    let index = self.read_next_short();
                    if let Some(RuntimeConstantPoolEntry::MethodRef(ref symref)) =
                            self.current_class.get_constant_pool()[index] {
                        // TODO: this should throw Java exceptions instead of unwrapping
                        let resolved_class = class_loader.resolve_class(&symref.class).unwrap();
                        let resolved_method = resolved_class.resolve_method(symref);
                        // TODO: check for <clinit> and <init>
                        // TODO: check protected accesses
                        let num_args = symref.sig.params.len();
                        let args = self.pop_as_locals(num_args + 1);
                        let object_class = {
                            let object_value = &args[0];
                            match *object_value {
                                Some(Value::ScalarReference(ref scalar_rc)) =>
                                    scalar_rc.borrow().get_class().clone(),
                                Some(Value::ArrayReference(ref array_rc)) =>
                                    array_rc.borrow().get_class().clone(),
                                Some(Value::NullReference) => panic!("NullPointerException"),
                                _ => panic!("invokevirtual on a primitive type"),
                            }
                        };
                        match object_class.dispatch_method(resolved_method) {
                            None => panic!("AbstractMethodError"),
                            Some((actual_class, actual_method)) =>
                                self.invoke(class_loader, actual_class, actual_method, args),
                        }
                    } else {
                        panic!("invokevirtual refers to non-method in constant pool");
                    }
                },

                opcode::INVOKESPECIAL => {
                    let index = self.read_next_short();
                    if let Some(RuntimeConstantPoolEntry::MethodRef(ref symref)) =
                            self.current_class.get_constant_pool()[index] {
                        // TODO: this should throw Java exceptions instead of unwrapping
                        let resolved_class = class_loader.resolve_class(&symref.class).unwrap();
                        let resolved_method = resolved_class.resolve_method(symref);
                        // TODO: check protected accesses
                        // TODO: lots of other checks here too
                        let num_args = symref.sig.params.len();
                        let args = self.pop_as_locals(num_args + 1);

                        // check the three conditions from the spec
                        let actual_method = {
                            if resolved_class.access_flags & class_access_flags::ACC_SUPER == 0
                                    || !self.current_class.is_descendant(resolved_class.as_ref())
                                    || resolved_method.symref.sig.name == "<init>" {
                                resolved_method
                            } else {
                                self.current_class.superclass.as_ref().and_then(|superclass| {
                                    superclass.find_method(&symref.sig)
                                }).expect("AbstractMethodError")
                            }
                        };
                        let actual_class = class_loader.resolve_class(&actual_method.symref.class).unwrap();
                        self.invoke(class_loader, actual_class.as_ref(), actual_method, args)
                    } else {
                        panic!("invokespecial refers to non-method in constant pool");
                    }
                },

                opcode::INVOKESTATIC => {
                    let index = self.read_next_short();
                    if let Some(RuntimeConstantPoolEntry::MethodRef(ref symref)) =
                            self.current_class.get_constant_pool()[index] {
                        // TODO: this should throw Java exceptions instead of unwrapping
                        let resolved_class = class_loader.resolve_class(&symref.class).unwrap();
                        let resolved_method = resolved_class.resolve_method(symref);
                        // TODO: check protected accesses
                        // TODO: lots of other checks here too
                        let num_args = symref.sig.params.len();
                        let args = self.pop_as_locals(num_args);
                        self.invoke(class_loader, resolved_class.as_ref(), resolved_method, args);
                    } else {
                        panic!("invokestatic refers to non-method in constant pool");
                    }
                },

                opcode::NEW => {
                    let index = self.read_next_short();
                    if let Some(RuntimeConstantPoolEntry::ClassRef(ref symref)) =
                            self.current_class.get_constant_pool()[index] {
                        // TODO proper error checking
                        let resolved_class = class_loader.resolve_class(symref).unwrap();
                        let object = Scalar::new(resolved_class);
                        let object_rc = Rc::new(RefCell::new(object));
                        self.operand_stack.push(Value::ScalarReference(object_rc));
                    } else {
                        panic!("new refers to non-class in constant pool");
                    }
                },

                opcode::NEWARRAY => {
                    let type_tag = self.read_next_byte();
                    let component_ty = match type_tag {
                        4 => Type::Boolean,
                        5 => Type::Char,
                        6 => Type::Float,
                        7 => Type::Double,
                        8 => Type::Byte,
                        9 => Type::Short,
                        10 => Type::Int,
                        11 => Type::Long,
                        _ => panic!("newarray: bad type tag"),
                    };
                    let class_sig = sig::Class::Array(Box::new(component_ty));
                    let class_symref = symref::Class { sig: class_sig };
                    let class = class_loader.resolve_class(&class_symref).unwrap();

                    match self.operand_stack.pop().unwrap() {
                        Value::Int(Wrapping(length)) => {
                            let array = Array::new(class, length);
                            let array_rc = Rc::new(RefCell::new(array));
                            self.operand_stack.push(Value::ArrayReference(array_rc));
                        },
                        _ => panic!("newarray called with non-int length"),
                    }
                },

                opcode::ARRAYLENGTH => {
                    match self.operand_stack.pop().unwrap() {
                        Value::ArrayReference(array_rc) => {
                            let len = array_rc.borrow().len();
                            self.operand_stack.push(Value::Int(Wrapping(len)));
                        },
                        Value::NullReference => panic!("NullPointerException"),
                        _ => panic!("arraylength called on non-array"),
                    }
                },

                _ => {
                    println!("{}", self.method_code.code[(self.pc as usize) - 1]);
                    panic!("undefined or reserved opcode")
                },
            }
        }
    }
}
