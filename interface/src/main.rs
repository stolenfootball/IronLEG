use std::fs;

use simulator;
use simulator::assembler;


use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use std::sync::Mutex;

struct SimulatorState {
    sim: Mutex<simulator::Simulator>,
}

#[get("/")]
async fn index() -> impl Responder {
    "Hello world!"
}


#[get("/registers")]
async fn get_regs(data: web::Data<SimulatorState>) -> impl Responder {
    let simulator = data.sim.lock().unwrap();
    let regs = simulator.processor.peek_registers();

    format!("{:?}", regs)
}

#[get("/step")]
async fn step(data: web::Data<SimulatorState>) -> HttpResponse {
    let mut simulator = data.sim.lock().unwrap();
    simulator.processor.cycle();
    
    HttpResponse::Ok().body("1")
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let asm = fs::read_to_string("./test/test.leg").unwrap();
    let program = assembler::assemble(&asm);

    let mut simulator = simulator::Simulator::new();
    simulator.flash(&program);

    let sim = web::Data::new(SimulatorState {
        sim: Mutex::new(simulator),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(sim.clone())
            .service(index)
            .service(get_regs)
            .service(step)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}