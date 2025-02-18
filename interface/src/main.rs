use std::rc::Rc;
use std::cell::RefCell;
use std::fs;

use simulator::processor::pipeline::{Stage, StageType};
use simulator::processor::registers::Registers;
use simulator::processor;
use simulator::memory;

use simulator::assembler;


#[macro_use] extern crate rocket;
use rocket::fs::FileServer;

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", FileServer::from("./interface/static"))
}

fn _test_asm() {
    let asm = fs::read_to_string("./test/test.leg").unwrap();
    let parsed = assembler::assemble(&asm);
    println!("{:?}", parsed);
}



fn _test_sim() {
    let mut mem = memory::RAM::new(4096, 16, 4, 5);
    let mut registers = Registers::new();

    let mut context = processor::Context {
        memory: &mut mem,
        registers: Box::new(&mut registers),
    };
    let mut context = Rc::new(RefCell::new(&mut context));

    let mut fetch_stage = Stage::create(Rc::clone(&mut context), StageType::Fetch);
    let mut decode_stage = Stage::create(Rc::clone(&mut context), StageType::Decode);
    let mut execute_stage = Stage::create(Rc::clone(&mut context), StageType::Execute);
    let mut memory_stage = Stage::create(Rc::clone(&mut context), StageType::Memory);
    let mut writeback_stage = Stage::create(Rc::clone(&mut context), StageType::Writeback);

    decode_stage.set_prev(&mut fetch_stage);
    execute_stage.set_prev(&mut decode_stage);
    memory_stage.set_prev(&mut execute_stage);
    writeback_stage.set_prev(&mut memory_stage);

    for _ in 0..100 {
        writeback_stage.cycle();
    }
}