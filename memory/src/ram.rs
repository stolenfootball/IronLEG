pub mod ram {
    use crate::memory::{self, MemoryValue, PipelineStage, MemoryAccess};
    pub struct RAM {
        size: usize,
        block_size: usize,
        word_size: usize,
        latency: usize,
        access: MemoryAccess,
        contents: Vec<Vec<usize>>,
    }

    impl RAM {
        pub fn new(size: usize, block_size: usize, word_size: usize, latency: usize) -> Self {
            Self {
                size: size,
                block_size: block_size,
                word_size: word_size,
                latency: latency,
                access: MemoryAccess {
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

        fn align(&self, addr: usize) -> usize {
            ((addr % self.size) / self.word_size) * self.word_size
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
            self.access.cycles_to_completion = i32::try_from(self.latency).unwrap();
            self.access.stage = None;
        }
    }

    impl memory::Memory for RAM {
        fn read(&mut self, addr: usize, stage: PipelineStage, line: bool) -> Option<MemoryValue> {
            if !self.attempt_access(stage) { return None; }
            self.reset_access_state();

            let addr = self.addr_to_offset(addr);
            if line {
                Some(memory::MemoryValue::Line(&self.contents[addr.0]))
            } else {
                Some(memory::MemoryValue::Value(self.contents[addr.0][addr.1]))
            }
        }

        fn write(&mut self, addr: usize, value: MemoryValue, stage: PipelineStage) -> Option<()> {
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