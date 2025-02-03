pub mod cache {
    use crate::memory::{Memory, MemoryValue, MemoryAccess, PipelineStage};
    use xxhash_rust::xxh3::xxh3_64;

    #[derive(Debug)]
    struct CacheLocation {
        offset: usize,
        index: usize,
        tag: usize,
    }

    #[derive(Clone, Debug)]
    struct CacheLine {
        addr: usize,
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
        pub lower_level: Option<&'a mut dyn Memory>,
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
                    addr: 0,
                    valid: false,
                    dirty: false,
                    tag: 0,
                    contents: vec![0; block_size],
                }; size],
            }
        }

        fn align(&self, addr: usize) -> usize {
            addr / self.word_size * self.word_size
        }

        fn cache_location(&self, addr: usize) -> CacheLocation {
            let addr = self.align(addr);
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

        fn get_write_line_index(&mut self, location: &CacheLocation) -> usize {
            for i in (location.index)..(location.index + self.associativity) {
                if self.contents[i].valid && self.contents[i].tag == location.tag {
                    return i; 
                }
            }

            for i in (location.index)..(location.index + self.associativity) {
                if !self.contents[i].valid {
                    return i;
                }
            }
            self.get_replacement(location)
        }

        fn get_replacement(&mut self, location: &CacheLocation) -> usize {
            // "random" replacement policy.  It's helpful to get the same "random" number for each location.
            // will be made less ass later (hopefully)
            location.index + xxh3_64(&[(location.tag ^ location.index) as u8]) as usize % self.associativity
        }

    }

    impl <'a> Memory for Cache<'a> {
        fn read(&mut self, addr: usize, stage: PipelineStage, line: bool) -> Option<MemoryValue> {
            if !self.access.attempt_access(stage) { return None; }
            self.access.reset_access_state();

            let location = self.cache_location(addr);
            if let Some(cache_line) = self.get_read_line(&location) {
                return match line {
                    true => Some(MemoryValue::Line(cache_line.contents.clone())),
                    false => Some(MemoryValue::Value(cache_line.contents[location.offset])),
                }
            } 

            let mut retrieved: Option<MemoryValue> = None;
            if let Some(lower_level) = &mut self.lower_level {
                retrieved = lower_level.read(addr, stage, line);
            } 

            if let Some(data) = &retrieved {
                let cache_line_index = self.get_write_line_index(&location);
                let cache_line = &mut self.contents[cache_line_index];
                cache_line.contents = match data {
                    MemoryValue::Line(val) => val.clone(),
                    MemoryValue::Value(val) => vec![*val; self.block_size],
                };
                cache_line.addr = addr;
                cache_line.valid = true;
                cache_line.dirty = false;
                cache_line.tag = location.tag;
                
            }
            
            retrieved
        }

        fn write(&mut self, addr: usize, value: MemoryValue, stage: PipelineStage) -> bool {
            if !self.access.attempt_access(stage) { return false; }

            let location = self.cache_location(addr);
            let cache_line_index = self.get_write_line_index(&location);

            if self.contents[cache_line_index].dirty && self.contents[cache_line_index].tag != location.tag{
                if let Some(lower_level) = &mut self.lower_level {
                    let cloned_value = MemoryValue::Line(self.contents[cache_line_index].contents.clone());
                    if !lower_level.write(self.contents[cache_line_index].addr, cloned_value, stage) {
                        return false;
                    }
                    self.contents[cache_line_index].dirty = false;
                }
            }

            let cache_line = &mut self.contents[cache_line_index];
            match value {
                MemoryValue::Line(val) => cache_line.contents = val,
                MemoryValue::Value(val) => cache_line.contents[location.offset] = val,
            }
            cache_line.addr = addr;
            cache_line.valid = true;
            cache_line.dirty = true;
            cache_line.tag = location.tag;

            self.access.reset_access_state();
            true
        }
}

}