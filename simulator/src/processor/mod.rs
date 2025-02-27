use std::sync::{Arc, Mutex};

use self::pipeline::StageType;
use self::registers::Registers;

use crate::memory::Memory;

pub mod instruction;
pub mod registers;
pub mod pipeline;
pub mod stages;


pub fn new(mem: Arc<Mutex<Box<dyn Memory>>>) -> Box<pipeline::Stage> {
    let regs = Arc::new(Mutex::new(Registers::new()));

    let fetch_stage = Box::new(pipeline::Stage::create(Arc::clone(&mem), Arc::clone(&regs), StageType::Fetch, None, false));
    let decode_stage = Box::new(pipeline::Stage::create(Arc::clone(&mem), Arc::clone(&regs), StageType::Decode, Some(fetch_stage), false));
    let execute_stage = Box::new(pipeline::Stage::create(Arc::clone(&mem), Arc::clone(&regs), StageType::Execute, Some(decode_stage), false));
    let memory_stage = Box::new(pipeline::Stage::create(Arc::clone(&mem), Arc::clone(&regs), StageType::Memory, Some(execute_stage), false));
    let writeback_stage = Box::new(pipeline::Stage::create(Arc::clone(&mem), Arc::clone(&regs), StageType::Writeback, Some(memory_stage), true));

    writeback_stage
}