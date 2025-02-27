use simulator;
use simulator::assembler;


use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result};
use actix_files;
use serde::Deserialize;
use std::sync::Mutex;

struct SimulatorState {
    sim: Mutex<simulator::Simulator>,
}

#[get("/registers")]
async fn get_regs(data: web::Data<SimulatorState>) -> impl Responder {
    let simulator = data.sim.lock().unwrap();
    web::Json(simulator.processor.view_registers())
}

#[get("/registers/status")]
async fn get_regs_status(data: web::Data<SimulatorState>) -> impl Responder {
    let simulator = data.sim.lock().unwrap();
    web::Json(simulator.processor.view_register_status())
}

#[get("/step")]
async fn step(data: web::Data<SimulatorState>) -> HttpResponse {
    let mut simulator = data.sim.lock().unwrap();
    simulator.processor.cycle();
    
    HttpResponse::Ok().body("🦿")
}

#[get("/run")]
async fn run(data: web::Data<SimulatorState>) -> HttpResponse {
    let mut simulator = data.sim.lock().unwrap();
    
    while simulator.processor.cycle() {}
    HttpResponse::Ok().body("🦿")
}

#[get("/reset")]
async fn reset(data: web::Data<SimulatorState>) -> HttpResponse {
    data.sim.lock().unwrap().reset();
    HttpResponse::Ok().body("🦿")
}

#[derive(Deserialize, Debug)]
struct Program {
    program: String,
}

#[post("/flash")]
async fn flash(program: web::Json<Program>, data: web::Data<SimulatorState>) -> HttpResponse {
    let asm = assembler::assemble(&program.program);
    data.sim.lock().unwrap().flash(&asm);

    HttpResponse::Ok().body("🦿")
}


#[get("/processor/cycles")]
async fn get_cycles(data: web::Data<SimulatorState>) -> Result<impl Responder> {
    let simulator = data.sim.lock().unwrap();
    Ok(web::Json(simulator.processor.view_cycles()))
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

    let mut returnable: Vec<Vec<Vec<usize>>> = vec![];
    for i in line_num..line_num + 5 {
        let lines = mem.view_line(i);
        returnable.push(lines.into_iter().map(|x| x.clone()).collect());
    }

    Ok(web::Json(returnable))
}


#[get("/processor/pipeline")]
async fn get_pipeline(data: web::Data<SimulatorState>) -> Result<impl Responder> {
    let simulator = data.sim.lock().unwrap();
    let status = simulator.processor.view_pipeline_instrs();

    let status: Vec<Option<simulator::processor::instruction::Instruction>> = status.into_iter().map(|x| x.clone()).collect();
    Ok(web::Json(status))
}


#[get("/processor/pipeline/status")]
async fn get_pipeline_status(data: web::Data<SimulatorState>) -> Result<impl Responder> {
    let simulator = data.sim.lock().unwrap();
    Ok(web::Json(simulator.processor.view_pipeline_status()))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let sim = web::Data::new(SimulatorState {
        sim: Mutex::new(simulator::Simulator::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(sim.clone())
            .service(run)
            .service(step)
            .service(reset)
            .service(flash)
            .service(get_regs_status)
            .service(get_regs)
            .service(get_cycles)
            .service(get_size)
            .service(get_line)
            .service(get_pipeline_status)
            .service(get_pipeline)
            .service(actix_files::Files::new("/", "./interface/static").show_files_listing())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}