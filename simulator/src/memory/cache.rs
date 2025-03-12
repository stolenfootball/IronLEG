use super::{Memory, Transparency};
use super::{MemoryValue, MemoryAccess};    
use crate::processor::pipeline::StageType;

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
    uses: usize,
    contents: Vec<usize>,
}

pub struct Cache {
    size: usize,
    num_lines: usize,
    block_size: usize,
    word_size: usize,
    associativity: usize,
    pub lower_level: Box<dyn Memory>,
    access: MemoryAccess,
    contents: Vec<CacheLine>,
}

impl Cache {
    pub fn new(size: usize, block_size: usize, word_size: usize, latency: i32, associativity: usize, lower_level: Box<dyn Memory>) -> Self {
        let num_lines = size / word_size / block_size;
        Self {
            size,
            num_lines,
            block_size,
            word_size,
            associativity,
            lower_level,
            access: MemoryAccess::new(latency, None),
            contents: Cache::create_blank_cache_contents(block_size, num_lines),
        }
    }

    fn create_blank_cache_contents(block_size: usize, num_lines: usize) -> Vec<CacheLine> {
        vec![CacheLine {
            addr: 0,
            valid: false,
            dirty: false,
            tag: 0,
            uses: 0,
            contents: vec![0; block_size],
        }; num_lines]
    }

    fn align(&self, addr: usize) -> usize {
        addr / self.word_size * self.word_size
    }

    fn cache_location(&self, addr: usize) -> CacheLocation {
        let addr = self.align(addr);
        CacheLocation {
            offset: (addr / self.word_size) % self.block_size,
            index:  (addr / (self.word_size * self.block_size)) * self.associativity % self.num_lines,
            tag:    (addr / (self.size / self.associativity)),
        }
    }

    fn find_line_in_cache(&self, location: &CacheLocation) -> Option<usize> {
        ((location.index)..(location.index + self.associativity)).find(|&i| self.contents[i].valid && self.contents[i].tag == location.tag)
    }

    fn find_line_to_replace(&mut self, location: &CacheLocation) -> usize {
        // Ideally, evict a line that isn't valid. Only possible for multi-cache processor
        for i in (location.index)..(location.index + self.associativity) {
            if !self.contents[i].valid {
                return i;
            }
        }

        // Otherwise, apply replacement policy
        self.get_replacement(location)
    }

    fn get_replacement(&mut self, location: &CacheLocation) -> usize {
        let mut least_used_index: usize = location.index;

        for i in (location.index)..(location.index + self.associativity) {

            // Add decay so no line gets "stuck" from uses a long time ago
            self.contents[i].uses = if self.contents[i].uses > 4 { 
                self.contents[i].uses.ilog2() as usize 
            } else { 
                self.contents[i].uses 
            };

            // Return the least used index after decay is applied
            if self.contents[i].uses < self.contents[least_used_index].uses {
                least_used_index = i;
            }
        }
        least_used_index
    }

    fn write_to_lower_level(&mut self, index: usize, stage: StageType) -> bool {
        let cloned_value = MemoryValue::Line(self.contents[index].contents.clone());
        if !self.lower_level.write(self.contents[index].addr, &cloned_value, stage) {
            return false;
        }
        self.contents[index].dirty = false;
        self.contents[index].uses = 0;
        true
    }

    fn insert_value_into_cache(&mut self, addr: usize, index: usize, location: &CacheLocation, value: &MemoryValue) {
        let cache_line = &mut self.contents[index];
        match value {
            MemoryValue::Line(val) => cache_line.contents = val.clone(),
            MemoryValue::Value(val) => cache_line.contents[location.offset] = *val,
        }
        cache_line.addr = addr;
        cache_line.valid = true;
        cache_line.dirty = true;
        cache_line.tag = location.tag;
        cache_line.uses += 1;
    }

}

impl Memory for Cache {
    fn read(&mut self, addr: usize, stage: StageType, line: bool) -> Option<MemoryValue> {

        // Apply a delay to simulate the time it would take to access a real cache
        if !self.access.attempt_access(stage) { return None; }

        // Location is the tag / offset / line number that the address would occupy if 
        // it is currently in the cache
        let location = self.cache_location(addr);

        // First, try to find the requested value in the cache. If it's there, we're done
        if let Some(cache_line_index) = self.find_line_in_cache(&location) {
            self.access.reset_access_state();
            self.contents[cache_line_index].uses += 1;
            return match line {
                true => Some(MemoryValue::Line(self.contents[cache_line_index].contents.clone())),
                false => Some(MemoryValue::Value(self.contents[cache_line_index].contents[location.offset])),
            }
        } 

        // Data not found in the cache, we'll have to grab it from the lower level of memory 
        let index_to_replace = self.find_line_to_replace(&location);

        // Replacement line was previously written to.  It will need to be written down to the lower
        // level before we can replace it.
        if self.contents[index_to_replace].dirty && !self.write_to_lower_level(index_to_replace, stage) {
                return None;
        }

        // Now that the data has been replaced, write the new data into it from the lower level
        if let Some(value) = &self.lower_level.read(addr, stage, true) {
            self.insert_value_into_cache(addr, index_to_replace, &location, value);

            self.access.reset_access_state();
            return match line {
                true => Some(MemoryValue::Line(self.contents[index_to_replace].contents.clone())),
                false => Some(MemoryValue::Value(self.contents[index_to_replace].contents[location.offset])),
            }
        }
        None
    }

    fn write(&mut self, addr: usize, value: &MemoryValue, stage: StageType) -> bool {
        if !self.access.attempt_access(stage) { return false; }

        // Location is the tag / offset / line number that the address would occupy if 
        // it is currently in the cache
        let location = self.cache_location(addr);

        // Get a place to write to
        let cache_line_index = match self.find_line_in_cache(&location) {
            Some(location) => location,
            None => self.find_line_to_replace(&location),
        };

        // Cache line is dirty and not the same as our current line.  Needs to be 
        // written to lower level before being overwritten 
        if self.contents[cache_line_index].dirty && 
           self.contents[cache_line_index].tag != location.tag &&
           !self.write_to_lower_level(cache_line_index, stage) {
                return false;
        }

        // Retrieve the most recent data from the lower level and put it into the cache
        if self.contents[cache_line_index].tag != location.tag {
            if let Some(value) = &self.lower_level.read(addr, stage, true) {
                self.insert_value_into_cache(addr, cache_line_index, &location, value);
            } else {
                return false;
            }
        }

        // Put the new data into the now free cache line
        self.insert_value_into_cache(addr, cache_line_index, &location, value);

        self.access.reset_access_state();
        true
    }

    fn reset_state(&mut self) {
        self.lower_level.reset_state();
        self.access.reset_access_state();
    }

    fn flash(&mut self, addr: usize, program: &[usize]) {
        self.lower_level.flash(addr, program);
    }

    fn reset(&mut self) {
        self.contents = Cache::create_blank_cache_contents(self.block_size, self.num_lines);
        self.lower_level.reset();
    }
}

impl Transparency for Cache {
    fn view_line(&self, line_num: usize) -> Vec<&Vec<usize>> {
        let mut contents = self.lower_level.view_line(line_num);
        if line_num < self.size {
            contents.push(&self.contents[line_num].contents);
        } else {
            contents.push(&self.contents[0].contents)
        }
        contents
    }

    fn view_access(&self) -> Vec<&MemoryAccess> {
        let mut access = self.lower_level.view_access();
        access.push(&self.access);
        access
    }

    fn view_size(&self) -> Vec<usize> {
        let mut size = self.lower_level.view_size();
        size.push(self.size);
        size
    }
}
