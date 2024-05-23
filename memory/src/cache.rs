pub mod cache {
    
    use crate::memory::{self, Memory, MemoryValue, MemoryAccess, PipelineStage};
    use std::rc::Rc;
    use std::cell::RefCell;

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
        contents: Rc<RefCell<Vec<usize>>>,
    }

    pub struct Cache {
        size: usize,
        // block_size: usize,
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
                // block_size: block_size,
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
                    contents: Rc::new(RefCell::new(vec![0; block_size])),
                }; size],
            }
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

        pub fn set_lower_level(&mut self, mem_type: &'static mut dyn Memory) {
            self.lower_level = Some(mem_type);
        }

        fn cache_location(&self, addr: usize) -> CacheLocation {
            let addr = self.align(addr);
            let offset = (addr / self.word_size) & 2_usize.pow(u32::try_from(self.word_size).unwrap()) - 1;
            CacheLocation {
                offset: offset,
                index: ((addr / self.word_size) >> offset) % self.size / self.associativity * self.associativity,
                tag:   ((addr / self.word_size) >> offset) / self.size,
            }
        }

        fn get_way(&mut self, location: &CacheLocation, is_write: bool) -> Option<&mut CacheLine> {
            for i in (location.index)..(location.index + self.associativity) {
                if (self.contents[i].valid && self.contents[i].tag == location.tag) || (!self.contents[i].valid && is_write) {
                    self.reset_access_state();
                    return Some(&mut self.contents[i]); 
                }
            }
            None
        }
    }

    impl memory::Memory for Cache {
        fn read(&mut self, addr: usize, stage: PipelineStage, line: bool) -> Option<MemoryValue> {
            if !self.attempt_access(stage) { return None; }

            let location = self.cache_location(addr);
            match self.get_way(&location, false) {
                Some(content) =>  {
                    if line {
                        Some(MemoryValue::Line(Rc::clone(&content.contents)))
                    } else {
                        Some(MemoryValue::Value(content.contents.borrow()[location.offset]))
                    }
                },
                None => None
            }
        }

        fn write(&mut self, addr: usize, value: MemoryValue, stage: PipelineStage) -> Option<()> {
            if !self.attempt_access(stage) { return None; }

            let location = self.cache_location(addr);
            match self.get_way(&location, true) {
                Some(content) => {
                    content.dirty = true;
                    content.valid = true;
                    content.tag = location.tag;
                    match value {
                        MemoryValue::Value(val) => content.contents.borrow_mut()[location.offset] = val,
                        MemoryValue::Line(val) => content.contents = val 
                    };
                    Some(())
                },
                None => None  // No room in cache.  Will be fixed later
            }
        }
    }
}