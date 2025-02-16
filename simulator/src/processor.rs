// use std::rc::Rc;
// use std::cell::RefCell;

use crate::memory::memory::Memory;
use self::registers::Registers;
// use crate::processor::pipeline::{Stage, StageType};

pub mod instruction;
pub mod registers;
pub mod pipeline;

pub struct Context<'a> {
    pub registers: Box<&'a mut Registers>,
    pub memory: &'a mut dyn Memory,
}

// pub struct Processor<'a, 'b> {
//     fetch_stage: Stage<'a, 'b>,
//     decode_stage: Stage<'a, 'b>,
//     execute_stage: Stage<'a, 'b>,
//     memory_stage: Stage<'a, 'b>,
//     writeback_stage: Stage<'a, 'b>,
//     context: Box<Rc<RefCell<&'a mut Context<'a>>>>,
// }

// impl<'a, 'b> Processor<'a, 'b> {
//     fn new(memory: &'a mut impl Memory) -> Processor<'a, 'b> {

//         let mut registers = Registers::new();
//         let mut context = Context {
//             memory: memory,
//             registers: Box::new(&mut registers),
//         };
//         let mut context = Box::new(Rc::new(RefCell::new(&mut context)));

//         let mut proc = Processor {
//             context,
//             fetch_stage: Stage::create(Rc::clone(&mut context), StageType::Fetch),
//             decode_stage: Stage::create(Rc::clone(&mut context), StageType::Decode),
//             execute_stage: Stage::create(Rc::clone(&mut context), StageType::Execute),
//             memory_stage: Stage::create(Rc::clone(&mut context), StageType::Memory),
//             writeback_stage: Stage::create(Rc::clone(&mut context), StageType::Writeback),
//         };

//         proc.decode_stage.set_prev(&mut proc.fetch_stage);
//         proc.execute_stage.set_prev(&mut proc.decode_stage);
//         proc.memory_stage.set_prev(&mut proc.execute_stage);
//         proc.writeback_stage.set_prev(&mut proc.memory_stage);


//         proc
//     }
// }