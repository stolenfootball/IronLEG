use memory::memory;

fn main() {
    let mut mem = memory::Memory::new(65536, 16, 4, 5);
    // let result = mem.read(0).unwrap();
    // println!("Read result: {result}");

    for _ in 0..5 {
        mem.write(0, 50, memory::PipelineStage::Memory);
    }
    let mut result: Option<u32> = None;
    for _ in 0..5 {
        result = mem.read(0, memory::PipelineStage::Fetch);
    }
    println!("Read result: {}", result.unwrap());
}
