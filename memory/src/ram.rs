pub mod ram {
    use crate::memory;
    use crate::memory::MemoryType;
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
                contents: vec![vec![0, block_size]; size.try_into().unwrap()],
            }
        }

        fn addr_to_offset(&self, addr: usize) -> (usize, usize) {
            let addr = self.align(addr);
            (addr % self.size / self.block_size, addr % self.block_size)
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
        fn read(&mut self, addr: usize, stage: memory::PipelineStage) -> Option<usize> {
            if !self.attempt_access(stage) { return None; }
            let addr = self.addr_to_offset(addr);
            self.reset_access_state();
            Some(self.contents[addr.0][addr.1])
        }

        fn write(&mut self, addr: usize, value: usize, stage: memory::PipelineStage) -> Option<usize> {
            if !self.attempt_access(stage) { return None; }
            let addr = self.addr_to_offset(addr);
            self.contents[addr.0][addr.1] = value;
            self.reset_access_state();
            Some(value)
        }
    }
}