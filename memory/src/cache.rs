pub mod cache {
    use crate::memory::{Memory, MemoryValue, MemoryAccess, PipelineStage};
    use rand::Rng;

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

    pub struct Cache<'a> {
        size: usize,
        block_size: usize,
        word_size: usize,
        associativity: usize,
        lower_level: Option<&'a mut dyn Memory>,
        access: MemoryAccess,
        contents: Vec<CacheLine>,
    }

    impl <'a> Cache<'a> {
        pub fn new(size: usize, block_size: usize, word_size: usize, latency: i32, associativity: usize) -> Self {
            Self {
                size: size,
                block_size: block_size,
                word_size: word_size,
                associativity: associativity,
                lower_level: None,
                access: MemoryAccess::new(latency, None),
                contents: vec![CacheLine {
                    valid: false,
                    dirty: false,
                    tag: 0,
                    contents: vec![0; block_size],
                }; size],
            }
        }

        fn align(&self, addr: usize) -> usize {
            ((addr % self.size) / self.word_size) * self.word_size
        }

        fn cache_location(&self, addr: usize) -> CacheLocation {
            let addr = self.align(addr) / self.word_size;
            CacheLocation {
                offset: addr & self.block_size - 1,
                index: (addr >> usize::ilog2(self.block_size)) * self.associativity % self.size,
                tag:   (addr >> usize::ilog2(self.block_size)) / self.size,
            }
        }

        fn get_read_line(&self, location: &CacheLocation) -> Option<&CacheLine> {
            for i in (location.index)..(location.index + self.associativity) {
                if self.contents[i].valid && self.contents[i].tag == location.tag {
                    return Some(&self.contents[i]); 
                }
            }
            None
        }

        fn get_write_line(&mut self, location: &CacheLocation) -> &mut CacheLine {
            for i in (location.index)..(location.index + self.associativity) {
                if !self.contents[i].valid {
                    return &mut self.contents[i];
                }
            }
            self.get_replacement(location)
        }

        fn get_replacement(&mut self, location: &CacheLocation) -> &mut CacheLine {
            &mut self.contents[location.index + rand::thread_rng().gen_range(0..self.associativity)]
        }
    }

    impl <'a> Memory for Cache<'a> {
        fn read(&mut self, addr: usize, stage: PipelineStage, line: bool) -> Option<MemoryValue> {
            if self.access.attempt_access(stage) {
                self.access.reset_access_state();

                let location = self.cache_location(addr);
                if let Some(cache_line) = self.get_read_line(&location) {
                    return match line {
                        true => Some(MemoryValue::Line(cache_line.contents.clone())),
                        false => Some(MemoryValue::Value(cache_line.contents[location.offset])),
                    }
                } 
                // TODO: Implement lower level read
            }
            None
        }

        fn write(&mut self, addr: usize, value: MemoryValue, stage: PipelineStage) -> Option<()> {
            if self.access.attempt_access(stage) {
                self.access.reset_access_state();

                let location = self.cache_location(addr);
                let cache_line = self.get_write_line(&location);
                match value {
                    MemoryValue::Line(val) => cache_line.contents = val,
                    MemoryValue::Value(val) => cache_line.contents[location.offset] = val,
                }
                cache_line.valid = true;
                cache_line.dirty = true;
                cache_line.tag = location.tag;
                return Some(());

                // TODO: Implement lower level write
            }
            None
        }
    }

}