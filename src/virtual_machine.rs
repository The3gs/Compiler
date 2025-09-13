use std::rc::Rc;

#[derive(Debug)]
pub enum Operation {
    // Stack Operations
    Push(u32),
    Pop,
    Get(u32),
    Put(u32),
    // Heap Operations
    Store(u32),
    Load(u32),
    Allocate(u32),
    Free(u32),
    // Function Operations
    Call(u32),
    CallFnPointer,
    Return,
    // Arithmatic Operations
    AddImmediate(u32),
    Add,
    SubImmediate(u32),
    SubImmediateBy(u32),
    Sub,
    MulImmediate(u32),
    Mul,
    DivImmediate(u32),
    DivImmediateBy(u32),
    Div,
    ModImmediate(u32),
    ModImmediateBy(u32),
    Mod,
    // Control Flow
    Jump(u32),
    JumpIf(u32),
    JumpIfNot(u32),
    Goto,
    GotoIf,
    GotoIfNot,
}

enum FunctionData {
    Code(Vec<Operation>),
    Builtin(Rc<dyn Fn(&mut VirtualMachine)>),
}

pub struct Function {
    name: String,
    implementation: FunctionData,
}

impl Function {
    pub fn from_operations(name: impl Into<String>, operations: Vec<Operation>) -> Self {
        Self {
            name: name.into(),
            implementation: FunctionData::Code(operations),
        }
    }

    pub fn from_builtin(
        name: impl Into<String>,
        function: impl Fn(&mut VirtualMachine) + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            implementation: FunctionData::Builtin(Rc::new(function)),
        }
    }
}

pub struct VirtualMachine {
    function_id: u32,
    program_counter: u32,
    functions: Vec<Function>,
    stack: Vec<u32>,
    heap: Vec<u32>,
}

impl VirtualMachine {
    pub fn from_functions(functions: Vec<Function>) -> Self {
        let main_index = functions.iter().position(|s| s.name == "main").unwrap_or(0) as u32;
        Self {
            function_id: main_index,
            program_counter: 0,
            functions,
            stack: vec![0, 0, u32::MAX],
            heap: vec![],
        }
    }

    pub fn run(&mut self) -> u32 {
        while self.function_id != u32::MAX {
            println!("stack: {:?}", self.stack);
            println!("function: {}", self.function_id);
            println!("pc: {}", self.program_counter);
            match &self.functions[self.function_id as usize].implementation {
                FunctionData::Builtin(f) => {
                    println!(
                        "Running builtin {:?}",
                        self.functions[self.function_id as usize].name
                    );
                    Rc::clone(f)(self);
                    self.function_id = self.stack.pop().unwrap();
                    self.program_counter = self.stack.pop().unwrap();
                }
                FunctionData::Code(operations) => {
                    println!("op: {:?}", operations[self.program_counter as usize]);
                    use Operation::*;
                    match operations[self.program_counter as usize] {
                        Push(n) => self.stack.push(n),
                        Pop => {
                            self.stack.pop();
                        }
                        Get(depth) => {
                            let n = self.stack[self.stack.len() - 1 - depth as usize];
                            self.stack.push(n);
                        }
                        Put(depth) => {
                            let v = self.stack.pop().unwrap();
                            let index = self.stack.len() - 1 - depth as usize;
                            self.stack[index] = v;
                        }
                        Store(address) => {
                            self.heap[address as usize] = self.stack.pop().unwrap();
                        }
                        Load(address) => self.stack.push(self.heap[address as usize]),
                        Allocate(size) => {
                            self.stack.push(self.heap.len() as u32);
                            self.heap.extend((0..size).map(|_| 0));
                        }
                        Free(_address) => {
                            self.stack.pop().unwrap();
                            // Will eventually free memory properly, but as alloc is a simpl bump allocator for now, we can't do much.
                        }
                        Call(function_id) => {
                            self.stack.push(self.program_counter);
                            self.stack.push(self.function_id);
                            self.function_id = function_id;
                            self.program_counter = u32::MAX;
                        }
                        CallFnPointer => {
                            let function_id = self.stack.pop().unwrap();
                            self.stack.push(self.program_counter);
                            self.stack.push(self.function_id);
                            self.function_id = function_id;
                            self.program_counter = u32::MAX;
                        }
                        Return => {
                            self.function_id = self.stack.pop().unwrap();
                            self.program_counter = self.stack.pop().unwrap();
                        }
                        AddImmediate(i) => {
                            let b = self.stack.pop().unwrap();
                            self.stack.push(b.wrapping_add(i));
                        }
                        Add => {
                            let b = self.stack.pop().unwrap();
                            let a = self.stack.pop().unwrap();
                            self.stack.push(a.wrapping_add(b));
                        }
                        SubImmediate(i) => {
                            let b = self.stack.pop().unwrap();
                            self.stack.push(b.wrapping_sub(i));
                        }
                        SubImmediateBy(i) => {
                            let b = self.stack.pop().unwrap();
                            self.stack.push(b.wrapping_sub(i));
                        }
                        Sub => {
                            let b = self.stack.pop().unwrap();
                            let a = self.stack.pop().unwrap();
                            self.stack.push(a.wrapping_sub(b));
                        }
                        MulImmediate(i) => {
                            let b = self.stack.pop().unwrap();
                            self.stack.push(b.wrapping_mul(i));
                        }
                        Mul => {
                            let b = self.stack.pop().unwrap();
                            let a = self.stack.pop().unwrap();
                            self.stack.push(a.wrapping_mul(b));
                        }
                        DivImmediate(i) => {
                            let b = self.stack.pop().unwrap();
                            self.stack.push(if i != 0 { b / i } else { 0 });
                        }
                        DivImmediateBy(i) => {
                            let b = self.stack.pop().unwrap();
                            self.stack.push(if b != 0 { b / i } else { 0 });
                        }
                        Div => {
                            let b = self.stack.pop().unwrap();
                            let a = self.stack.pop().unwrap();
                            self.stack.push(if b != 0 { a / b } else { 0 });
                        }
                        ModImmediate(i) => {
                            let b = self.stack.pop().unwrap();
                            self.stack.push(if i != 0 { b % i } else { 0 });
                        }
                        ModImmediateBy(i) => {
                            let b = self.stack.pop().unwrap();
                            self.stack.push(if b != 0 { i % b } else { 0 });
                        }
                        Mod => {
                            let b = self.stack.pop().unwrap();
                            let a = self.stack.pop().unwrap();
                            self.stack.push(if b != 0 { a % b } else { 0 });
                        }
                        Jump(location) => {
                            self.program_counter = location.wrapping_sub(1);
                        }
                        JumpIf(location) => {
                            if self.stack.pop().unwrap() != 0 {
                                self.program_counter = location.wrapping_sub(1);
                            }
                        }
                        JumpIfNot(location) => {
                            if self.stack.pop().unwrap() == 0 {
                                self.program_counter = location.wrapping_sub(1);
                            }
                        }
                        Goto => {
                            let location = self.stack.pop().unwrap();
                            self.program_counter = location.wrapping_sub(1);
                        }
                        GotoIf => {
                            let location = self.stack.pop().unwrap();
                            if self.stack.pop().unwrap() != 0 {
                                self.program_counter = location.wrapping_sub(1);
                            }
                        }
                        GotoIfNot => {
                            let location = self.stack.pop().unwrap();
                            if self.stack.pop().unwrap() == 0 {
                                self.program_counter = location.wrapping_sub(1);
                            }
                        }
                    }
                }
            }
            self.program_counter = self.program_counter.wrapping_add(1);
        }

        return self.stack.last().copied().unwrap_or(u32::MAX);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn fibonacci() {
        fn fib(n: u32) -> u32 {
            let mut a = 0;
            let mut b = 1;
            for _ in 0..n {
                std::mem::swap(&mut a, &mut b);
                b += a;
            }
            return b;
        }

        for i in 0..30 {
            use Operation::*;
            let mut program = VirtualMachine::from_functions(vec![Function::from_operations(
                "main",
                vec![
                    Push(0),         // 0
                    Push(1),         // 1
                    Push(i),         // 2
                    Get(0),          // 3
                    JumpIfNot(13),   // 4
                    Get(2),          // 5
                    Get(2),          // 6
                    Add,             // 7
                    Get(2),          // 8
                    Put(3),          // 9
                    Put(1),          // 10
                    SubImmediate(1), // 11
                    Jump(3),         // 12
                    Pop,             // 13
                    Put(3),          // 14
                    Pop,             // 15
                    Return,          // 16
                ],
            )]);
            assert_eq!(fib(i), program.run());
        }
    }
}
