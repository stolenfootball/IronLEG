use std::sync::{Arc, Mutex};

use crate::processor::pipeline;
use crate::memory::Memory;

pub mod memory;
pub mod assembler;
pub mod processor;

pub struct Simulator {
    pub processor: Box<pipeline::Stage>,
    pub memory: Arc<Mutex<Box<dyn Memory>>>,
}

impl Simulator {
    pub fn new() -> Simulator {
        let ram = Box::new(memory::RAM::new(65536, 16, 4, 5));
        let cache = Box::new(memory::Cache::new(16384, 16, 4, 1, 2, ram));
        let memory: Arc<Mutex<Box<dyn Memory>>> = Arc::new(Mutex::new(cache));

        Simulator {
            processor: processor::new(Arc::clone(&memory)),
            memory: memory,
        }
    }

    pub fn flash(&mut self, addr: usize, program: &Vec<u32>) {
        let program: Vec<usize> = program.into_iter().map(|x| *x as usize).collect();
        self.memory.lock().unwrap().flash(addr, &program);
    }

    pub fn reset(&mut self) {
        self.processor.reset();
        self.memory.lock().unwrap().reset();
    }
}
