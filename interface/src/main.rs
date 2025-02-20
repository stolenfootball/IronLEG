use std::fs;

use simulator;
use simulator::assembler;


// #[macro_use] extern crate rocket;
// use rocket::fs::FileServer;

// #[launch]
// fn rocket() -> _ {
//     rocket::build().mount("/", FileServer::from("./interface/static"))
// }

fn main() {
    test_processor();
}


fn test_processor() {
    let asm = fs::read_to_string("./test/test.leg").unwrap();
    let program = assembler::assemble(&asm);

    let mut simulator = simulator::Simulator::new();
    simulator.flash(&program);

    for _ in 0..20 {
        simulator.processor.cycle();
        println!("{:?}", simulator.processor.peek_pipeline_instrs());
        println!("{:?}", simulator.processor.peek_pipeline_status());
        println!("{:?}", simulator.peek_regs());
    }
}