use std::fs;

use simulator;
use simulator::assembler;


use actix_web::{get, web, App, HttpResponse, HttpServer, Responder, Result};
use actix_files;
// use serde::Serialize;
use std::sync::Mutex;

struct SimulatorState {
    sim: Mutex<simulator::Simulator>,
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

#[get("/memory/size")]
async fn get_size(data: web::Data<SimulatorState>) -> Result<impl Responder> {
    let simulator = data.sim.lock().unwrap();
    let size = simulator.memory.lock().unwrap().view_size();

    Ok(web::Json(size))
}

#[get("/memory/line/{line_num}")]
async fn get_line(path: web::Path<usize>, data: web::Data<SimulatorState>) -> Result<impl Responder> {
    let line_num = path.into_inner();

    let simulator = data.sim.lock().unwrap();
    let mem = simulator.memory.lock().unwrap();
    let lines = mem.view_line(line_num);

    let lines: Vec<Vec<usize>> = lines.into_iter().map(|x| x.clone()).collect();
    Ok(web::Json(lines))
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
            .service(get_regs)
            .service(step)
            .service(get_size)
            .service(get_line)
            .service(actix_files::Files::new("/", "./interface/static").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}