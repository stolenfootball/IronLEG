pub mod ram;
pub mod cache;
mod transparency;

pub mod memory {
    pub use crate::memory::ram::ram::RAM;
    pub use crate::memory::cache::cache::Cache;

    use crate::memory::transparency::transparency::Transparency;

    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub enum PipelineStage {
        Fetch,
        Decode,
        Execute,
        Memory,
        Writeback,
    }

    #[derive(Debug, Clone)]
    pub enum MemoryValue {
        Value(usize),
        Line(Vec<usize>),
    }

    #[derive(Clone, Copy, Debug)]
    pub struct MemoryAccess {
        pub latency: i32,
        pub cycles_to_completion: i32,
        pub stage: Option<PipelineStage>,
    }

    impl MemoryAccess {
        pub fn new(latency: i32, stage: Option<PipelineStage>) -> Self {
            Self {
                latency,
                cycles_to_completion: latency,
                stage,
            }
        }

        pub fn attempt_access(&mut self, attempt_stage: PipelineStage) -> bool {
            match self.stage {
                Some(current_stage) => {
                    if current_stage != attempt_stage { 
                        return false; 
                    }
                    self.cycles_to_completion -= 1;
                    return self.cycles_to_completion <= 0;
                },
                None => self.stage = Some(attempt_stage)
            }
            false
        }

        pub fn reset_access_state(&mut self) {
            self.cycles_to_completion = self.latency;
            self.stage = None;
        }
    }

    pub trait Memory: Transparency {
        fn read(&mut self, addr: usize, stage: PipelineStage, line: bool) -> Option<MemoryValue>;
        fn write(&mut self, addr: usize, value: MemoryValue, stage: PipelineStage) -> bool;
    }
}