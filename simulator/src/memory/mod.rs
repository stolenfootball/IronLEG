pub mod ram;
pub mod cache;


pub use self::ram::RAM;
pub use self::cache::Cache;
pub use crate::processor::pipeline::StageType;

#[derive(Debug, Clone)]
pub enum MemoryValue {
    Value(usize),
    Line(Vec<usize>),
}

#[derive(Clone, Copy, Debug)]
pub struct MemoryAccess {
    pub latency: i32,
    pub cycles_to_completion: i32,
    pub stage: Option<StageType>,
}

impl MemoryAccess {
    pub fn new(latency: i32, stage: Option<StageType>) -> Self {
        Self {
            latency,
            cycles_to_completion: latency,
            stage,
        }
    }

    pub fn attempt_access(&mut self, attempt_stage: StageType) -> bool {
        match self.stage {
            Some(current_stage) => {
                if current_stage != attempt_stage { 
                    return false; 
                }
            },
            None => self.stage = Some(attempt_stage)
        }
        self.cycles_to_completion -= 1;
        self.cycles_to_completion <= 0
    }

    pub fn reset_access_state(&mut self) {
        self.cycles_to_completion = self.latency;
        self.stage = None;
    }
}

pub trait Transparency {
    fn view_line(&self, line_num: usize) -> Vec<&Vec<usize>>;
    fn view_access(&self) -> Vec<&MemoryAccess>;
    fn view_size(&self) -> Vec<usize>;
}

pub trait Memory: Transparency + Send {
    fn read(&mut self, addr: usize, stage: StageType, line: bool) -> Option<MemoryValue>;
    fn write(&mut self, addr: usize, value: &MemoryValue, stage: StageType) -> bool;
    fn flash(&mut self, addr: usize, program: &[usize]);
    fn reset_state(&mut self);
    fn reset(&mut self);
}
