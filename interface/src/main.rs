use std::fs;

use simulator;
use simulator::assembler;

// use axum;


// #[tokio::main]
// async fn main() {
//     // Build our application with a single route.
//     let app = axum::Router::new().route("/",
//         axum::routing::get(|| async { "Hello, World!" }));

//     // Run our application as a hyper server on http://localhost:3000.
//     let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
//     axum::serve(listener, app).await.unwrap();
// }

// fn main() {
//     test_processor();
// }


fn main() {
    let asm = fs::read_to_string("./test/test.leg").unwrap();
    let program = assembler::assemble(&asm);

    let mut simulator = simulator::Simulator::new();
    simulator.flash(&program);

    for _ in 0..20 {
        simulator.processor.cycle();
        println!("{:?}", simulator.processor.peek_pipeline_instrs());
        println!("{:?}", simulator.processor.peek_pipeline_status());
        println!("{:?}\n", simulator.peek_regs());
    }
}