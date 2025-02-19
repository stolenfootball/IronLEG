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

fn _test_asm() {
    let asm = fs::read_to_string("./test/test.leg").unwrap();
    let parsed = assembler::assemble(&asm);
    println!("{:?}", parsed);
}


fn test_processor() {
    let mut simulator = simulator::Simulator::new();
    for _ in 0..100 {
        simulator.pipeline.cycle();
    }
}