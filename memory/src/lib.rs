pub mod ram;
pub mod cache;
pub mod memory {
    pub use crate::ram::ram::RAM;
    pub use crate::cache::cache::Cache;

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

    pub trait Memory {
        fn read(&mut self, addr: usize, stage: PipelineStage, line: bool) -> Option<MemoryValue>;
        fn write(&mut self, addr: usize, value: MemoryValue, stage: PipelineStage) -> Option<()>;
    }

  
}
