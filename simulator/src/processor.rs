use std::rc::Rc;
use std::cell::RefCell;

use crate::memory::memory::Memory;
use crate::processor::registers::Registers;
use crate::processor::pipeline::{Stage, StageType};

pub mod instruction;
pub mod registers;
pub mod pipeline;

pub struct Context<'a> {
    pub registers: Box<&'a mut Registers>,
    pub memory: &'a mut dyn Memory,
}

// pub struct Processor<'a, 'b> {
//     pipeline: Box<Stage<'a, 'b>>,
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
    
//         let mut fetch_stage = Box::new(Stage::create(Rc::clone(&mut context), StageType::Fetch, None));
//         let mut decode_stage = Box::new(Stage::create(Rc::clone(&mut context), StageType::Decode, Some(Box::new(fetch_stage))));
//         let mut execute_stage = Box::new(Stage::create(Rc::clone(&mut context), StageType::Execute, Some(decode_stage)));
//         let mut memory_stage = Box::new(Stage::create(Rc::clone(&mut context), StageType::Memory, Some(execute_stage)));
//         let writeback_stage = Box::new(Stage::create(Rc::clone(&mut context), StageType::Writeback, Some(memory_stage)));

//         Processor {
//             context: context,
//             pipeline: writeback_stage,
//         }

//     }
// }