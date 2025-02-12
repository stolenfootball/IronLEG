pub mod memory;
pub mod assembler;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PipelineStage {
    Fetch,
    Decode,
    Execute,
    Memory,
    Writeback,
}