pub mod memory;


#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PipelineStage {
    Fetch,
    Decode,
    Execute,
    Memory,
    Writeback,
}