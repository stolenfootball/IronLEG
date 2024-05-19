
pub mod memory {
    use std::convert::TryFrom;
    pub struct Memory {
        size: u32,
        block_size: u32,
        word_size: u32,
        contents: Vec<Vec<u32>>,
    }

    impl Memory {
        pub fn new(size: u32, block_size: u32, word_size: u32) -> Self {
            Self {
                size: size,
                block_size: block_size,
                word_size: word_size,
                contents: vec![vec![0, block_size]; size.try_into().unwrap()],
            }
        }

        fn align(&self, addr: u32) -> u32 {
            ((addr % self.size) / self.word_size) * self.word_size
        }

        fn addr_to_offset(&self, addr: u32) -> (usize, usize) {
            let addr = self.align(addr);
            (usize::try_from((addr % self.size) / self.block_size).unwrap(), usize::try_from(addr % self.block_size).unwrap())
        }

        pub fn read(&self, addr: u32) -> Option<u32> {
            let addr = self.addr_to_offset(addr);
            Some(self.contents[addr.0][addr.1])
        }

        pub fn write(&mut self, addr: u32, value: u32) -> Option<u32> {
            let addr = self.addr_to_offset(addr);
            self.contents[addr.0][addr.1] = value;
            Some(value)
        }
    }
}
