
pub mod memory {
    use std::convert::TryFrom;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum PipelineStage {
        Fetch,
        Decode,
        Execute,
        Memory,
        Writeback,
    }

    #[derive(Clone, Copy, Debug)]
    pub struct MemoryAccess {
        cycles_to_completion: i32,
        stage: Option<PipelineStage>,
    }

    pub struct Memory {
        size: u32,
        block_size: u32,
        word_size: u32,
        latency: i32,
        access: MemoryAccess,
        contents: Vec<Vec<u32>>,
    }

    impl Memory {
        pub fn new(size: u32, block_size: u32, word_size: u32, latency: i32) -> Self {
            Self {
                size: size,
                block_size: block_size,
                word_size: word_size,
                latency: latency,
                access: MemoryAccess {
                    cycles_to_completion: latency,
                    stage: None,
                },
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

        fn attempt_access(&mut self, attempt_stage: PipelineStage) -> bool {
            match self.access.stage {
                Some(current_stage) => {
                    if current_stage != attempt_stage { 
                        return false; 
                    }
                    self.access.cycles_to_completion -= 1;
                    return self.access.cycles_to_completion <= 1;
                },
                None => self.access.stage = Some(attempt_stage)
            }
            false
        }

        fn reset_access_state(&mut self) {
            self.access.cycles_to_completion = self.latency;
            self.access.stage = None;
        }

        pub fn read(&mut self, addr: u32, stage: PipelineStage) -> Option<u32> {
            if !self.attempt_access(stage) { return None; }
            let addr = self.addr_to_offset(addr);
            self.reset_access_state();
            Some(self.contents[addr.0][addr.1])
        }

        pub fn write(&mut self, addr: u32, value: u32, stage: PipelineStage) -> Option<u32> {
            if !self.attempt_access(stage) { return None; }
            let addr = self.addr_to_offset(addr);
            self.contents[addr.0][addr.1] = value;
            self.reset_access_state();
            Some(value)
        }
    }
}
