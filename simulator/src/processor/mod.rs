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

    let fetch_stage = Box::new(pipeline::Stage::create(Arc::clone(&mem), Arc::clone(&regs), StageType::Fetch, None));
    let decode_stage = Box::new(pipeline::Stage::create(Arc::clone(&mem), Arc::clone(&regs), StageType::Decode, Some(fetch_stage)));
    let execute_stage = Box::new(pipeline::Stage::create(Arc::clone(&mem), Arc::clone(&regs), StageType::Execute, Some(decode_stage)));
    let memory_stage = Box::new(pipeline::Stage::create(Arc::clone(&mem), Arc::clone(&regs), StageType::Memory, Some(execute_stage)));
    let writeback_stage = Box::new(pipeline::Stage::create(Arc::clone(&mem), Arc::clone(&regs), StageType::Writeback, Some(memory_stage)));

    writeback_stage
}