pub mod ram;
pub mod cache;
pub mod memory {
    pub use crate::ram::ram::RAM;
    pub use crate::cache::cache::Cache;
    use std::convert::TryFrom;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum PipelineStage {
        Fetch,
        Decode,
        Execute,
        Memory,
        Writeback,
    }

#[derive(Debug)]
    pub enum MemoryValue<'a> {
        Value(usize),
        Line(&'a Vec<usize>),
    }

    #[derive(Clone, Copy, Debug)]
    pub struct MemoryAccess {
        pub cycles_to_completion: i32,
        pub stage: Option<PipelineStage>,
    }

    pub trait MemoryType {
        fn size(&self) -> usize;
        fn word_size(&self) -> usize;
        fn block_size(&self) -> usize;
        fn access(&self) -> MemoryAccess;
        fn latency(&self) -> usize;
        fn set_access(&mut self, cycles_to_completion: Option<i32>, stage: Option<PipelineStage>);
        fn reset_stage(&mut self);

        fn align(&self, addr: usize) -> usize {
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
        fn read(&mut self, addr: usize, stage: PipelineStage, line: bool) -> Option<MemoryValue>;
        fn write(&mut self, addr: usize, value: MemoryValue, stage: PipelineStage) -> Option<()>;
    }

  
}
