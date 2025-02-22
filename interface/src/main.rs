// use std::fs;

use simulator;
// use simulator::assembler;


// fn main() {
//     let asm = fs::read_to_string("./test/test.leg").unwrap();
//     let program = assembler::assemble(&asm);

//     let mut simulator = simulator::Simulator::new();
//     simulator.flash(&program);

//     for _ in 0..20 {
//         simulator.processor.cycle();
//         println!("{:?}", simulator.processor.peek_pipeline_instrs());
//         println!("{:?}", simulator.processor.peek_pipeline_status());
//         println!("{:?}\n", simulator.peek_regs());
//     }
// }


use actix_web::{get, web, App, HttpServer, Responder};
use std::sync::Mutex;

struct SimulatorState {
    sim: Mutex<simulator::Simulator>,
}


#[get("/")]
async fn index() -> impl Responder {
    "Hello world!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let sim = web::Data::new(SimulatorState {
        sim: Mutex::new(simulator::Simulator::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(sim.clone())
            .service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}