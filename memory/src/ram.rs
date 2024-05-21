pub mod ram {
    use crate::memory;
    use crate::memory::MemoryType;
    pub struct RAM {
        size: u32,
        block_size: u32,
        word_size: u32,
        latency: u32,
        access: memory::MemoryAccess,
        contents: Vec<Vec<u32>>,
    }

    impl RAM {
        pub fn new(size: u32, block_size: u32, word_size: u32, latency: u32) -> Self {
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

        fn addr_to_offset(&self, addr: u32) -> (usize, usize) {
            let addr = self.align(addr);
            (usize::try_from((addr % self.size()) / self.block_size()).unwrap(), 
             usize::try_from(addr % self.block_size()).unwrap())
        }
    }

    impl memory::MemoryType for RAM {
        fn size(&self) -> u32 { self.size }
        fn word_size(&self) -> u32 { self.word_size }
        fn block_size(&self) -> u32 { self.block_size }
        fn access(&self) -> memory::MemoryAccess { self.access }
        fn latency(&self) -> u32 { self.latency }
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
        fn read(&mut self, addr: u32, stage: memory::PipelineStage) -> Option<u32> {
            if !self.attempt_access(stage) { return None; }
            let addr = self.addr_to_offset(addr);
            self.reset_access_state();
            Some(self.contents[addr.0][addr.1])
        }

        fn write(&mut self, addr: u32, value: u32, stage: memory::PipelineStage) -> Option<u32> {
            if !self.attempt_access(stage) { return None; }
            let addr = self.addr_to_offset(addr);
            self.contents[addr.0][addr.1] = value;
            self.reset_access_state();
            Some(value)
        }
    }
}