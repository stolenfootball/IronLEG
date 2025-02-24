use std::sync::{Arc, Mutex};
use serde::Serialize;

use super::instruction::Instruction;
use super::registers::Registers;
use super::stages;
use crate::memory::Memory;

pub use super::stages::StageType;

#[derive(Serialize, Copy, Clone, Debug, PartialEq)]
pub enum StageResult {
    DONE,
    WAIT,
    SQUASH,
    COMPLETE,
    HALT,
}

pub struct Stage {
    status: StageResult,
    pipeline_on: bool,
    instruction: Option<Instruction>,
    mem: Arc<Mutex<Box<dyn Memory>>>,
    regs: Arc<Mutex<Registers>>,
    prev_stage: Option<Box<Stage>>,
    process: fn(Arc<Mutex<Box<dyn Memory>>>, Arc<Mutex<Registers>>, &mut Instruction) -> StageResult,
}

impl Stage {
    pub fn create(mem: Arc<Mutex<Box<dyn Memory>>>, regs: Arc<Mutex<Registers>>, stage_type: StageType, prev_stage: Option<Box<Stage>>) -> Stage {
        Stage {
            status: StageResult::DONE,
            pipeline_on: true,
            instruction: None,
            mem: mem,
            regs: regs,
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
        self.status = StageResult::WAIT;
        self.instruction.take()
    }

    fn load(&mut self) {
        match &mut self.prev_stage {
            Some(prev) => {
                if self.instruction.is_none() && prev.status == StageResult::DONE {
                    self.instruction = prev.transfer();
                }
            },
            None => {
                if self.instruction.is_none() && self.pipeline_on {
                    self.instruction = Some(Instruction::new())
                }
            }
        };
    }

    pub fn squash(&mut self) {
        if let Some(instr) = &mut self.instruction {
            if instr.meta.initialized {
                instr.meta.squashed = true;
            }
        }
        if let Some(prev_stage) = &mut self.prev_stage {
            prev_stage.squash();
        }
    }

    pub fn cycle(&mut self) {
        if self.status == StageResult::HALT { return; }

        self.load();
        if let Some(instr) = &mut self.instruction {
            self.status = (self.process)(Arc::clone(&self.mem), Arc::clone(&self.regs), instr);

            if self.status == StageResult::SQUASH { self.squash(); }
            if self.status == StageResult::SQUASH || self.status == StageResult::COMPLETE { 
                self.instruction = None; 
                self.status = StageResult::DONE 
            }
        }
        if let Some(prev) = &mut self.prev_stage {
            prev.cycle();
        }
    }
} 

// Functions for external visibility separated out for clarity
impl Stage {
    pub fn view_pipeline_instrs(&self) -> Vec<&Option<Instruction>> {
        let mut instrs = match &self.prev_stage {
            Some(prev) => prev.view_pipeline_instrs(),
            None => vec![]
        };
        instrs.push(&self.instruction);
        instrs
    }

    pub fn view_pipeline_status(&self) -> Vec<StageResult> {
        let mut instrs = match &self.prev_stage {
            Some(prev) => prev.view_pipeline_status(),
            None => vec![]
        };
        instrs.push(self.status);
        instrs
    }

    pub fn view_registers(&self) -> [i32; 16] {
        self.regs.lock().unwrap().registers
    }
}