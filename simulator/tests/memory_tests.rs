use simulator::memory::{Memory, Cache, RAM, MemoryValue, Transparency};
use simulator::processor::pipeline::StageType;

fn new_mem() -> Box<Cache> {
    let ram = Box::new(RAM::new(4194304, 16, 4, 5));
    let cache = Box::new(Cache::new(2048, 16, 4, 1, 2, ram));
    cache
}

#[test]
fn cache_write_replacement() {
    let mut mem = new_mem();

    mem.write(0, &MemoryValue::Line(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]), StageType::Fetch);
    mem.write(2048, &MemoryValue::Line(vec![16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 20, 31]), StageType::Memory);

    mem.read(0, StageType::Fetch, true);
    for _ in 0..5 {
        mem.write(4096, &MemoryValue::Line(vec![32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47]), StageType::Fetch);
    }

    let x = mem.view_line(1);
    // println!("{:?}", mem.view_line(0));
    // println!("{:?}", x);
    assert_eq!(32, x[1][0])
}

#[test]
fn read_from_cache() {
    let mut mem = new_mem();

    mem.write(8, &MemoryValue::Value(30), StageType::Fetch);
    // println!("{:?}", mem.view_line(0));
    if let MemoryValue::Value(x) = mem.read(8, StageType::Fetch, false).unwrap() {
        assert_eq!(30, x);
    } else {
        panic!("No value returned")
    }
}


