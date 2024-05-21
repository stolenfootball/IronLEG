pub mod ram {
    use crate::memory::{self, MemoryValue, MemoryType};
    pub struct RAM {
        size: usize,
        block_size: usize,
        word_size: usize,
        latency: usize,
        access: memory::MemoryAccess,
        contents: Vec<Vec<usize>>,
    }

    impl RAM {
        pub fn new(size: usize, block_size: usize, word_size: usize, latency: usize) -> Self {
            Self {
                size: size,
                block_size: block_size,
                word_size: word_size,
                latency: latency,
                access: memory::MemoryAccess {
                    cycles_to_completion: i32::try_from(latency).unwrap(),
                    stage: None,
                },
                contents: vec![vec![0; block_size]; size.try_into().unwrap()],
            }
        }

        fn addr_to_offset(&self, addr: usize) -> (usize, usize) {
            let addr = self.align(addr);
            ((addr / self.word_size) % self.size / self.block_size, (addr / self.word_size) % self.block_size)
        }
    }

    impl memory::MemoryType for RAM {
        fn size(&self) -> usize { self.size }
        fn word_size(&self) -> usize { self.word_size }
        fn block_size(&self) -> usize { self.block_size }
        fn access(&self) -> memory::MemoryAccess { self.access }
        fn latency(&self) -> usize { self.latency }
        fn set_access(&mut self, cycles_to_completion: Option<i32>, stage: Option<memory::PipelineStage>) {
            if let Some(cycles) = cycles_to_completion {
                self.access.cycles_to_completion = cycles;
            }
            if let Some(pipeline_stage) = stage {
                self.access.stage = Some(pipeline_stage);
            }
        }
        fn reset_stage(&mut self) {
            self.access.stage = None;
        }
    }

    impl memory::Memory for RAM {
        fn read(&mut self, addr: usize, stage: memory::PipelineStage, line: bool) -> Option<memory::MemoryValue> {
            if !self.attempt_access(stage) { return None; }
            self.reset_access_state();

            let addr = self.addr_to_offset(addr);
            if line {
                Some(memory::MemoryValue::Line(&self.contents[addr.0]))
            } else {
                Some(memory::MemoryValue::Value(self.contents[addr.0][addr.1]))
            }
        }

        fn write(&mut self, addr: usize, value: MemoryValue, stage: memory::PipelineStage) -> Option<()> {
            if !self.attempt_access(stage) { return None; }
            self.reset_access_state();

            let addr = self.addr_to_offset(addr);
            match value {
                MemoryValue::Value(val) => self.contents[addr.0][addr.1] = val,
                MemoryValue::Line(val) => self.contents[addr.0] = val.to_vec() // Not efficient, come back later
            }
            Some(())
        }
    }
}