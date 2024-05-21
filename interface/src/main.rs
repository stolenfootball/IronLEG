use memory::memory;
use memory::{Memory, PipelineStage, MemoryValue};

fn main() {
    // let mut mem = memory::RAM::new(65536, 16, 4, 5);

    // for _ in 0..5 {
    //     mem.write(0, 50, memory::PipelineStage::Memory);
    // }
    // let mut result: Option<usize> = None;
    // for _ in 0..5 {
    //     result = mem.read(0, memory::PipelineStage::Fetch);
    // }
    // println!("Read result: {}", result.unwrap());

    let mut cache = memory::Cache::new(2048, 16, 4, 5, 2);
    for _ in 0..5 {
        cache.write(0, MemoryValue::Line(&vec![20; 16]), PipelineStage::Memory);
    }
    let mut result: Option<MemoryValue> = None;
    for _ in 0..5 {
        result = cache.read(0, PipelineStage::Fetch, false);
    }
    print!("Cache result: {:?}", result.unwrap());
}
