use std::cell::RefCell;
use std::rc::Rc;

use self::pipeline::StageType;

pub use super::Context;

pub mod instruction;
pub mod registers;
pub mod pipeline;



pub fn new(context: Rc<RefCell<Box<Context>>>) -> Box<pipeline::Stage> {
    let fetch_stage = Box::new(pipeline::Stage::create(Rc::clone(&context), StageType::Fetch, None));
    let decode_stage = Box::new(pipeline::Stage::create(Rc::clone(&context), StageType::Decode, Some(fetch_stage)));
    let execute_stage = Box::new(pipeline::Stage::create(Rc::clone(&context), StageType::Execute, Some(decode_stage)));
    let memory_stage = Box::new(pipeline::Stage::create(Rc::clone(&context), StageType::Memory, Some(execute_stage)));
    let writeback_stage = Box::new(pipeline::Stage::create(Rc::clone(&context), StageType::Writeback, Some(memory_stage)));

    writeback_stage
}