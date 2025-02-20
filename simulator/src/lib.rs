use std::cell::RefCell;
use std::rc::Rc;

use crate::processor::registers::Registers;
use crate::processor::pipeline;

pub mod memory;
pub mod assembler;
pub mod processor;

pub struct Context {
    pub registers: Box<Registers>,
    pub memory: Box<dyn memory::Memory>,
}

impl Context {
    pub fn new() -> Context {
        let ram = Box::new(memory::RAM::new(65536, 16, 4, 5));
        let cache = Box::new(memory::Cache::new(65536, 16, 4, 5, 2, ram));
        
        Context {
            registers: Box::new(Registers::new()),
            memory: cache,
        }
    }
}

pub struct Simulator {
    pub processor: Box<pipeline::Stage>,
    pub context: Rc<RefCell<Box<Context>>>,
}

impl Simulator {
    pub fn new() -> Simulator {
        let context = Rc::new(RefCell::new(Box::new(Context::new())));
        Simulator {
            processor: processor::new(Rc::clone(&context)),
            context: context,
        }
    }

    pub fn flash(&mut self, program: &Vec<u32>) {
        let program: Vec<usize> = program.into_iter().map(|x| *x as usize).collect();
        self.context.borrow_mut().memory.flash(&program);
    }
}


// Functions for external visibility separated out for clarity
impl Simulator {
    pub fn peek_regs(&self) -> [i32; 16] {
        self.context.borrow().registers.registers
    }
}