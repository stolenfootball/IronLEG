
use super::{Memory, Transparency};
use super::{MemoryValue, MemoryAccess};    
use crate::processor::pipeline::StageType;
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
    pub lower_level: &'a mut dyn Memory,
    access: MemoryAccess,
    contents: Vec<CacheLine>,
}

impl <'a> Cache<'a> {
    pub fn new(size: usize, block_size: usize, word_size: usize, latency: i32, associativity: usize, lower_level: &'a mut dyn Memory) -> Self {
        Self {
            size: size,
            block_size: block_size,
            word_size: word_size,
            associativity: associativity,
            lower_level: lower_level,
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

    fn get_read_line_index(&self, location: &CacheLocation) -> Option<usize> {
        for i in (location.index)..(location.index + self.associativity) {
            if self.contents[i].valid && self.contents[i].tag == location.tag {
                return Some(i); 
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
    fn read(&mut self, addr: usize, stage: StageType, line: bool) -> Option<MemoryValue> {
        if !self.access.attempt_access(stage) { return None; }

        let location = self.cache_location(addr);
        if let Some(cache_line_index) = self.get_read_line_index(&location) {
            self.access.reset_access_state();
            return match line {
                true => Some(MemoryValue::Line(self.contents[cache_line_index].contents.clone())),
                false => Some(MemoryValue::Value(self.contents[cache_line_index].contents[location.offset])),
            }
        } 

        let cache_line_index = self.get_write_line_index(&location);

        if self.contents[cache_line_index].dirty {
            let evicted_value = MemoryValue::Line(self.contents[cache_line_index].contents.clone());
            if !self.lower_level.write(self.contents[cache_line_index].addr, evicted_value, stage) {
                return None;
            }
            self.contents[cache_line_index].dirty = false;
        }

        let retrieved = self.lower_level.read(addr, stage, line);

        if let Some(data) = &retrieved {
            let cache_line = &mut self.contents[cache_line_index];
            cache_line.contents = match data {
                MemoryValue::Line(val) => val.clone(),
                MemoryValue::Value(val) => vec![*val; self.block_size],
            };
            cache_line.addr = addr;
            cache_line.valid = true;
            cache_line.dirty = false;
            cache_line.tag = location.tag;

            self.access.reset_access_state();
        }
        
        retrieved
    }

    fn write(&mut self, addr: usize, value: MemoryValue, stage: StageType) -> bool {
        if !self.access.attempt_access(stage) { return false; }

        let location = self.cache_location(addr);
        let cache_line_index = self.get_write_line_index(&location);

        if self.contents[cache_line_index].dirty && self.contents[cache_line_index].tag != location.tag{
            let cloned_value = MemoryValue::Line(self.contents[cache_line_index].contents.clone());
            if !self.lower_level.write(self.contents[cache_line_index].addr, cloned_value, stage) {
                return false;
            }
            self.contents[cache_line_index].dirty = false;
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

    fn reset_state(&mut self) {
        self.lower_level.reset_state();
        self.access.reset_access_state();
    }
}

impl Transparency for Cache<'_> {
    fn peek_line(&self, line_num: usize) -> &Vec<usize> {
        &self.contents[line_num].contents
    }

    fn peek_access(&self) -> &MemoryAccess {
        &self.access
    }
}
