use memory::memory;
use memory::{Memory, PipelineStage, MemoryValue};

fn main() {

    let mut mem = memory::RAM::new(65536, 16, 4, 5);

    let mut cache = memory::Cache::new(2048, 16, 4, 5, 2);
    cache.lower_level = Some(&mut mem);

    write_through(&mut cache, 0, MemoryValue::Line(vec![17,18,19,20,21,22,23,24,25,26,27,28,29,30,31,32]), PipelineStage::Memory);
    write_through(&mut cache, 200000, MemoryValue::Line(vec![33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48]), PipelineStage::Memory);
    write_through(&mut cache, 32768, MemoryValue::Line(vec![33,34,35,36,37,38,39,40,41,42,43,44,45,46,47,48]), PipelineStage::Memory);
    write_through(&mut cache, 65536, MemoryValue::Line(vec![49,50,51,52,53,54,55,56,57,58,59,60,61,62,63,64]), PipelineStage::Memory);

    print_through(&mut cache, 0, PipelineStage::Fetch, true);
    print_through(&mut cache, 200000, PipelineStage::Fetch, true);
    print_through(&mut cache, 32768, PipelineStage::Fetch, true);
    print_through(&mut cache, 65536, PipelineStage::Fetch, true);

    print_through(&mut mem, 32768, PipelineStage::Fetch, true);

}


fn read_through(mem: &mut impl Memory, addr: usize, stage: PipelineStage, line: bool) -> (usize, Option<MemoryValue>) {
    let mut count = 1;
    let mut result = mem.read(addr, stage, line);
    while result.is_none() {
        result = mem.read(addr, stage, line);
        count += 1;
    }
    (count, result)
}

fn print_through(mem: &mut impl Memory, addr: usize, stage: PipelineStage, line: bool) {
    let result = read_through(mem, addr, stage, line);
    println!("Addr: {} | Cycles: {} | Result: {:?}", addr, result.0, result.1.unwrap());
}

fn write_through(mem: &mut impl Memory, addr: usize, value: MemoryValue, stage: PipelineStage) -> bool {
    let mut result = mem.write(addr, value.clone(), stage);
    while !result {
        result = mem.write(addr, value.clone(), stage);
    }
    result
}