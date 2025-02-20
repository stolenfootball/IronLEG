use std::cell::RefCell;
use std::rc::Rc;

use super::instruction::Instruction;
use super::Context;
use super::stages;

pub use super::stages::StageType;

pub enum StageResult {
    DONE,
    WAIT,
    SQUASH,
}

struct StageStatus {
    finished: bool,
    pipeline_on: bool,
}

pub struct Stage {
    status: StageStatus,
    instruction: Option<Instruction>,
    context: Rc<RefCell<Box<Context>>>,
    prev_stage: Option<Box<Stage>>,
    process: fn(Rc<RefCell<Box<Context>>>, &mut Instruction) -> StageResult,
}

impl Stage {
    pub fn create(context: Rc<RefCell<Box<Context>>>, stage_type: StageType, prev_stage: Option<Box<Stage>>) -> Stage {
        Stage {
            status: StageStatus {
                finished: false,
                pipeline_on: true,
            },
            instruction: None,
            context: context,
            prev_stage: prev_stage,
            process: match stage_type {
                StageType::Fetch => stages::fetch, 
                StageType::Decode => stages::decode,
                StageType::Execute => stages::execute,
                StageType::Memory => stages::memory,
                StageType::Writeback => stages::writeback,
            },
        }
        
    }

    pub fn set_prev(&mut self, prev_stage: Box<Stage>) {
        self.prev_stage = Some(prev_stage)
    }

    fn transfer(&mut self) -> Option<Instruction> {
        self.status.finished = false;
        self.instruction.take()
    }

    fn load(&mut self) {
        match &mut self.prev_stage {
            Some(prev) => {
                if self.instruction.is_none() && prev.status.finished {
                    self.instruction = prev.transfer();
                }
            },
            None => {
                if self.instruction.is_none() && self.status.pipeline_on {
                    self.instruction = Some(Instruction::new())
                }
            }
        };
    }

    pub fn squash(&mut self) {
        if let Some(instr) = &mut self.instruction {
            instr.meta.squashed = true;
        }
        if let Some(prev_stage) = &mut self.prev_stage {
            prev_stage.squash();
        }
    }

    pub fn cycle(&mut self) {
        self.load();
        if let Some(instr) = &mut self.instruction {
            if instr.meta.squashed { self.status.finished = true; }
            if !self.status.finished { 
                self.status.finished = match (self.process)(Rc::clone(&self.context), instr) {
                    StageResult::DONE => true,
                    StageResult::WAIT => false,
                    StageResult::SQUASH => {
                        self.squash();
                        true
                    }
                }
            }
        }
        if let Some(prev) = &mut self.prev_stage {
            prev.cycle();
        }
    }
} 

// Functions for external visibility separated out for clarity
impl Stage {
    pub fn peek_pipeline_instrs(&self) -> Vec<&Option<Instruction>> {
        let mut instrs = match &self.prev_stage {
            Some(prev) => prev.peek_pipeline_instrs(),
            None => vec![]
        };
        instrs.push(&self.instruction);
        instrs
    }

    pub fn peek_pipeline_status(&self) -> Vec<bool> {
        let mut instrs = match &self.prev_stage {
            Some(prev) => prev.peek_pipeline_status(),
            None => vec![]
        };
        instrs.push(self.status.finished);
        instrs
    }
}