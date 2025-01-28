use memory::memory;
use memory::{Memory, PipelineStage, MemoryValue};

fn main() {
    let mut mem = memory::RAM::new(65536, 16, 4, 5);

    for _ in 0..5 {
        mem.write(0, MemoryValue::Value(50), PipelineStage::Memory);
    }
    for _ in 0..5 {
        mem.write(4, MemoryValue::Value(25), PipelineStage::Memory);
    }
    let mut result: Option<MemoryValue> = None;
    for _ in 0..5 {
        result = mem.read(63, PipelineStage::Fetch, true);
    }
    println!("Read result: {:?}", result.unwrap());

    let mut cache = memory::Cache::new(2048, 16, 4, 5, 2);
    for _ in 0..5 {
        cache.write(0, MemoryValue::Line(vec![1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16]), PipelineStage::Memory);
    }
    let mut result: Option<MemoryValue> = None;
    for _ in 0..5 {
        result = cache.read(12, PipelineStage::Fetch, true);
    }
    print!("Cache result: {:?}", result.unwrap());
}
