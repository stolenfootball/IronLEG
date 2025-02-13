use std::rc::Rc;
use std::cell::RefCell;

use simulator::processor::pipeline::{Stage, StageType};
use simulator::processor::registers::Registers;
use simulator::processor;
use simulator::memory::memory;

fn main() {
    let mut mem = memory::RAM::new(4096, 16, 4, 5);
    let mut registers = Registers::new();

    let mut context = processor::Context {
        memory: &mut mem,
        registers: Box::new(&mut registers),
    };
    let mut context = Rc::new(RefCell::new(&mut context));

    let mut fetch_stage = Stage::create(Rc::clone(&mut context), StageType::Fetch, None);
    let mut decode_stage = Stage::create(Rc::clone(&mut context), StageType::Decode, Some(&mut fetch_stage));
    let mut execute_stage = Stage::create(Rc::clone(&mut context), StageType::Execute, Some(&mut decode_stage));
    let mut memory_stage = Stage::create(Rc::clone(&mut context), StageType::Memory, Some(&mut execute_stage));
    let mut writeback_stage = Stage::create(Rc::clone(&mut context), StageType::Writeback, Some(&mut memory_stage));

    for _ in 0..100 {
        writeback_stage.cycle();
    }
}