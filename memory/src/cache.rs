pub mod cache {
    use crate::memory::{self, Memory};
    use crate::memory::MemoryType;

    struct CacheLocation {
        offset: usize,
        index: usize,
        tag: usize,
    }
    #[derive(Clone)]
    struct CacheLine {
        valid: bool,
        dirty: bool,
        tag: usize,
        contents: Vec<usize>,
    }

    pub struct Cache {
        size: usize,
        block_size: usize,
        word_size: usize,
        latency: usize,
        associativity: usize,
        lower_level: Option<&'static mut dyn Memory>,
        access: memory::MemoryAccess,
        contents: Vec<CacheLine>,
    }

    impl Cache {
        pub fn new(size: usize, block_size: usize, word_size: usize, latency: usize, associativity: usize) -> Self {
            Self {
                size: size,
                block_size: block_size,
                word_size: word_size,
                latency: latency,
                associativity: associativity,
                lower_level: None,
                access: memory::MemoryAccess {
                    cycles_to_completion: i32::try_from(latency).unwrap(),
                    stage: None,
                },
                contents: vec![CacheLine {
                    valid: false,
                    dirty: false,
                    tag: 0,
                    contents: vec![0, block_size],
                }; size],
            }
        }

        pub fn set_lower_level(&mut self, mem_type:&'static mut dyn Memory) {
            self.lower_level = Some(mem_type);
        }

        fn cache_location(&self, addr: usize) -> CacheLocation {
            let offset = addr & 2_usize.pow(u32::try_from(self.word_size()).unwrap()) - 1;
            CacheLocation {
                offset: offset,
                index: (addr / self.word_size() >> offset % self.size()) / self.associativity * self.associativity,
                tag:   addr / self.word_size() >> offset / self.size(),
            }
        }

        fn get_way(&mut self, addr: usize, is_write: bool) -> Option<&mut CacheLine> {
            let location = self.cache_location(self.align(addr));
            let mut cache_content = &self.contents[location.index];
            for i in (location.index + 1)..(location.index + self.associativity) {
                if (cache_content.valid && cache_content.tag == location.tag) || (!cache_content.valid && is_write) {
                    let content =  &mut self.contents[location.index];
                    return Some(content); 
                }
                cache_content = &mut self.contents[i];
            }
            None
        }
    }

    impl MemoryType for Cache {
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

    impl memory::Memory for Cache {
        fn read(&mut self, addr: usize, stage: memory::PipelineStage) -> Option<usize> {
            if !self.attempt_access(stage) { return None; }

            let location = self.cache_location(addr);
            let result = match self.get_way(addr, false) {
                Some(content) => Some(content.contents[location.offset]),
                None => None
            };

            if let Some(_) = result { self.reset_access_state() };
            result
        }

        fn write(&mut self, addr: usize, value: usize, stage: memory::PipelineStage) -> Option<usize> {
            if !self.attempt_access(stage) { return None; }

            let location = self.cache_location(addr);
            let result = match self.get_way(addr, true) {
                Some(content) => {
                    content.contents[location.offset] = value;
                    content.dirty = true;
                    content.valid = true;
                    Some(value)
                },
                None => None  // No room in cache.  Will be fixed later
            };

            if let Some(_) = result { self.reset_access_state() };
            result
        }
    }
}