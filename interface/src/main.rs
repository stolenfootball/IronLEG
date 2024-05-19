use memory::memory;

fn main() {
    let mut mem = memory::Memory::new(65536, 16, 4);
    let result = mem.read(0).unwrap();
    println!("Read result: {result}");

    mem.write(0, 50);
    let result = mem.read(0).unwrap();
    println!("Read result: {result}");
}
