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
    pub status: StageResult,
    is_head: bool,
    pipeline_on: bool,
    cycles: u128,
    pub instruction: Option<Instruction>,
    mem: Arc<Mutex<Box<dyn Memory>>>,
    regs: Arc<Mutex<Registers>>,
    prev_stage: Option<Box<Stage>>,
    process: fn(Arc<Mutex<Box<dyn Memory>>>, Arc<Mutex<Registers>>, &mut Instruction) -> StageResult,
}

impl Stage {
    pub fn create(mem: Arc<Mutex<Box<dyn Memory>>>, regs: Arc<Mutex<Registers>>, stage_type: StageType, prev_stage: Option<Box<Stage>>, is_head: bool) -> Stage {
        Stage {
            status: StageResult::DONE,
            pipeline_on: true,
            is_head,
            cycles: 0,
            instruction: None,
            mem,
            regs,
            prev_stage,
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

    pub async fn cycle(&mut self) -> bool {
        if self.status == StageResult::HALT { return false; }
        
        self.load();
        if let Some(instr) = &mut self.instruction {
            if instr.meta.squashed { self.status =  StageResult::DONE }
            if self.status !=  StageResult::DONE || self.is_head {
                self.status = (self.process)(Arc::clone(&self.mem), Arc::clone(&self.regs), instr);
            }
            if self.status == StageResult::SQUASH { self.squash(); self.status = StageResult::DONE }
            if self.status ==  StageResult::DONE && self.is_head { self.instruction = None }
        }
        if let Some(prev) = &mut self.prev_stage {
            Box::pin(prev.cycle()).await;
        }
        
        self.cycles += 1;
        true
    }

    pub fn reset(&mut self) {
        self.cycles = 0;
        self.instruction = None;
        self.status = StageResult::DONE;
        self.regs.lock().unwrap().reset();
        if let Some(prev_stage) = &mut self.prev_stage {
            prev_stage.reset();
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

    pub async fn view_cycles(&self) -> u128 {
        self.cycles
    }

    pub fn view_register_status(&self) -> [bool; 16] {
        self.regs.lock().unwrap().in_use
    }
}