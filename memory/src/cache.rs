pub mod cache {
    use crate::memory::{self, Memory, MemoryType, MemoryValue, MemoryAccess, PipelineStage};

    #[derive(Debug)]
    struct CacheLocation {
        offset: usize,
        index: usize,
        tag: usize,
    }
    #[derive(Clone, Debug)]
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
        access: MemoryAccess,
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
                access: MemoryAccess {
                    cycles_to_completion: i32::try_from(latency).unwrap(),
                    stage: None,
                },
                contents: vec![CacheLine {
                    valid: false,
                    dirty: false,
                    tag: 0,
                    contents: vec![0; block_size],
                }; size],
            }
        }

        pub fn set_lower_level(&mut self, mem_type:&'static mut dyn Memory) {
            self.lower_level = Some(mem_type);
        }

        fn cache_location(&self, addr: usize) -> CacheLocation {
            let addr = self.align(addr);
            let offset = (addr & 2_usize.pow(u32::try_from(self.word_size).unwrap()) - 1) / self.word_size;
            CacheLocation {
                offset: offset,
                index: ((addr / self.word_size) >> offset) % self.size / self.associativity * self.associativity,
                tag:   ((addr / self.word_size) >> offset) / self.size,
            }
        }

        fn get_way(&mut self, addr: usize, is_write: bool) -> Option<&mut CacheLine> {
            let location = self.cache_location(self.align(addr));
            let mut cache_content = &self.contents[location.index];
            for i in (location.index)..(location.index + self.associativity) {
                if (cache_content.valid && cache_content.tag == location.tag) || (!cache_content.valid && is_write) {
                    self.reset_access_state();
                    return Some(&mut self.contents[i]); 
                }
                cache_content = &self.contents[i];
            }
            None
        }
    }

    impl MemoryType for Cache {
        fn size(&self) -> usize { self.size }
        fn word_size(&self) -> usize { self.word_size }
        fn block_size(&self) -> usize { self.block_size }
        fn access(&self) -> MemoryAccess { self.access }
        fn latency(&self) -> usize { self.latency }
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

    impl memory::Memory for Cache {
        fn read(&mut self, addr: usize, stage: PipelineStage, line: bool) -> Option<MemoryValue> {
            if !self.attempt_access(stage) { return None; }

            let location = self.cache_location(addr);
            match self.get_way(addr, false) {
                Some(content) =>  {
                    println!("{:?}", content);
                    if line {
                        Some(MemoryValue::Line(&content.contents))
                    } else {
                        Some(MemoryValue::Value(content.contents[location.offset]))
                    }
                },
                None => None
            }
        }

        fn write(&mut self, addr: usize, value: MemoryValue, stage: PipelineStage) -> Option<()> {
            if !self.attempt_access(stage) { return None; }

            let location = self.cache_location(addr);
            match self.get_way(addr, true) {
                Some(content) => {
                    content.dirty = true;
                    content.valid = true;
                    content.tag = location.tag;
                    match value {
                        MemoryValue::Value(val) => content.contents[location.offset] = val,
                        MemoryValue::Line(val) => content.contents = val.to_vec() // Not efficient, come back later
                    };
                    println!("{:?}", content);
                    Some(())
                },
                None => None  // No room in cache.  Will be fixed later
            }
        }
    }
}