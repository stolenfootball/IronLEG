
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

    pub struct RAM {
        size: u32,
        block_size: u32,
        word_size: u32,
        latency: u32,
        access: MemoryAccess,
        contents: Vec<Vec<u32>>,
    }


    trait MemoryType {
        fn size(&self) -> u32;
        fn word_size(&self) -> u32;
        fn block_size(&self) -> u32;
        fn access(&self) -> MemoryAccess;
        fn latency(&self) -> u32;
        fn set_access(&mut self, cycles_to_completion: Option<i32>, stage: Option<PipelineStage>);
        fn reset_stage(&mut self);

        fn align(&self, addr: u32) -> u32 {
            ((addr % self.size()) / self.word_size()) * self.word_size()
        }

        fn attempt_access(&mut self, attempt_stage: PipelineStage) -> bool {
            match self.access().stage {
                Some(current_stage) => {
                    if current_stage != attempt_stage { 
                        return false; 
                    }
                    self.set_access(Some(self.access().cycles_to_completion - 1), None);
                    return self.access().cycles_to_completion <= 1;
                },
                None => self.set_access(None, Some(attempt_stage))
            }
            false
        }

        fn reset_access_state(&mut self) {
            self.set_access(Some(i32::try_from(self.latency()).unwrap()), None);
            self.reset_stage();
        }
    }

    pub trait Memory {
        fn read(&mut self, addr: u32, stage: PipelineStage) -> Option<u32>;
        fn write(&mut self, addr: u32, value: u32, stage: PipelineStage) -> Option<u32>;
    }

    impl RAM {
        pub fn new(size: u32, block_size: u32, word_size: u32, latency: u32) -> Self {
            Self {
                size: size,
                block_size: block_size,
                word_size: word_size,
                latency: latency,
                access: MemoryAccess {
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

    impl MemoryType for RAM {
        fn size(&self) -> u32 { self.size }
        fn word_size(&self) -> u32 { self.word_size }
        fn block_size(&self) -> u32 { self.block_size }
        fn access(&self) -> MemoryAccess { self.access }
        fn latency(&self) -> u32 { self.latency }
        fn set_access(&mut self, cycles_to_completion: Option<i32>, stage: Option<PipelineStage>) {
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

    impl Memory for RAM {
        fn read(&mut self, addr: u32, stage: PipelineStage) -> Option<u32> {
            if !self.attempt_access(stage) { return None; }
            let addr = self.addr_to_offset(addr);
            self.reset_access_state();
            Some(self.contents[addr.0][addr.1])
        }

        fn write(&mut self, addr: u32, value: u32, stage: PipelineStage) -> Option<u32> {
            if !self.attempt_access(stage) { return None; }
            let addr = self.addr_to_offset(addr);
            self.contents[addr.0][addr.1] = value;
            self.reset_access_state();
            Some(value)
        }
    }

    struct CacheLocation {
        offset: u32,
        index: u32,
        tag: u32,
    }
    #[derive(Clone)]
    struct CacheLine {
        valid: bool,
        dirty: bool,
        tag: u32,
        contents: Vec<u32>,
    }

    pub struct Cache {
        size: u32,
        block_size: u32,
        word_size: u32,
        latency: u32,
        associativity: u32,
        access: MemoryAccess,
        contents: Vec<CacheLine>,
    }

    impl Cache {
        pub fn new(size: u32, block_size: u32, word_size: u32, latency: u32, associativity: u32) -> Self {
            Self {
                size: size,
                block_size: block_size,
                word_size: word_size,
                latency: latency,
                associativity: associativity,
                access: MemoryAccess {
                    cycles_to_completion: i32::try_from(latency).unwrap(),
                    stage: None,
                },
                contents: vec![CacheLine {
                    valid: false,
                    dirty: false,
                    tag: 0,
                    contents: vec![0, block_size],
                }; size.try_into().unwrap()],
            }
        }

        fn cache_location(&self, addr: u32) -> CacheLocation {
            let offset = addr & 2_u32.pow(self.word_size()) - 1;
            CacheLocation {
                offset: offset,
                index: (addr / self.word_size() >> offset % self.size()) / self.associativity * self.associativity,
                tag:   addr / self.word_size() >> offset / self.size(),
            }
        }

        fn get_way(&mut self, addr: u32, is_write: bool) -> Option<&mut CacheLine> {
            let location = self.cache_location(self.align(addr));
            let mut cache_content = &self.contents[usize::try_from(location.index).unwrap()];
            for i in (location.index + 1)..(location.index + self.associativity) {
                if (cache_content.valid && cache_content.tag == location.tag) || (!cache_content.valid && is_write) {
                    let content =  &mut self.contents[usize::try_from(location.index).unwrap()];
                    return Some(content); 
                }
                cache_content = &mut self.contents[usize::try_from(i).unwrap()];
            }
            None
        }
    }

    impl MemoryType for Cache {
        fn size(&self) -> u32 { self.size }
        fn word_size(&self) -> u32 { self.word_size }
        fn block_size(&self) -> u32 { self.block_size }
        fn access(&self) -> MemoryAccess { self.access }
        fn latency(&self) -> u32 { self.latency }
        fn set_access(&mut self, cycles_to_completion: Option<i32>, stage: Option<PipelineStage>) {
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

    impl Memory for Cache {
        fn read(&mut self, addr: u32, stage: PipelineStage) -> Option<u32> {
            if !self.attempt_access(stage) { return None; }

            let location = self.cache_location(addr);
            match self.get_way(addr, false) {
                Some(content) => Some(content.contents[usize::try_from(location.offset).unwrap()]),
                None => None // Cache miss, will be implemented later
            }
        }

        fn write(&mut self, addr: u32, value: u32, stage: PipelineStage) -> Option<u32> {
            if !self.attempt_access(stage) { return None; }

            let location = self.cache_location(addr);
            match self.get_way(addr, true) {
                Some(content) => {
                    content.contents[usize::try_from(location.offset).unwrap()] = value;
                    content.dirty = true;
                    content.valid = true;
                    return Some(value);
                },
                None => None  // No room in cache.  Will be fixed later
            }?;
            None
        }

    }
}
