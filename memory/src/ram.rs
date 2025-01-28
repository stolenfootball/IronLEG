pub mod ram {
    use crate::memory::{self, MemoryValue, PipelineStage, MemoryAccess};

    pub struct RAM {
        size: usize,
        block_size: usize,
        word_size: usize,
        access: MemoryAccess,
        contents: Vec<Vec<usize>>,
    }

    impl RAM {
        pub fn new(size: usize, block_size: usize, word_size: usize, latency: i32) -> Self {
            Self {
                size: size,
                block_size: block_size,
                word_size: word_size,
                access: MemoryAccess::new(latency, None),
                contents: vec![vec![0; block_size]; size],
            }
        }

        fn align(&self, addr: usize) -> usize {
            ((addr % self.size) / self.word_size) * self.word_size
        }

        fn addr_to_offset(&self, addr: usize) -> (usize, usize) {
            let addr = self.align(addr);
            ((addr / self.word_size) % self.size / self.block_size, (addr / self.word_size) % self.block_size)
        }

    }

    impl memory::Memory for RAM {
        fn read(&mut self, addr: usize, stage: PipelineStage, line: bool) -> Option<MemoryValue> {
            if !self.access.attempt_access(stage) { return None; }
            self.access.reset_access_state();

            let addr = self.addr_to_offset(addr);
            match line {
                true => Some(memory::MemoryValue::Line(self.contents[addr.0].clone())),
                false => Some(memory::MemoryValue::Value(self.contents[addr.0][addr.1])),
            }
        }

        fn write(&mut self, addr: usize, value: MemoryValue, stage: PipelineStage) -> Option<()> {
            if !self.access.attempt_access(stage) { return None; }
            self.access.reset_access_state();

            let addr = self.addr_to_offset(addr);
            match value {
                MemoryValue::Line(val) => self.contents[addr.0] = val,
                MemoryValue::Value(val) => self.contents[addr.0][addr.1] = val,
            }
            Some(())
        }
    }
}