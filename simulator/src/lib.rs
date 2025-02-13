pub mod memory;
pub mod assembler;
pub mod processor;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PipelineStage {
    Fetch,
    Decode,
    Execute,
    Memory,
    Writeback,
}