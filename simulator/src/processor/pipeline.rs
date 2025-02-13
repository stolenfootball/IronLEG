
use std::cell::RefCell;
use std::rc::Rc;
use crate::processor::instruction::Instruction;
use crate::processor::Context;
use crate::processor::registers::Register;
use crate::memory::memory::MemoryValue;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StageType {
    Fetch,
    Decode,
    Execute,
    Memory,
    Writeback,
}

struct StageStatus {
    finished: bool,
    pipeline_on: bool,
}

pub struct Stage<'a, 'b> {
    status: StageStatus,
    instruction: Option<Instruction>,
    context: Rc<RefCell<&'a mut Context<'a>>>,
    prev_stage: Option<&'a mut Stage<'a, 'b>>,
    process: fn(Rc<RefCell<&'a mut Context<'a>>>, &mut Instruction) -> bool,
}

impl<'a, 'b> Stage<'a, 'b> {
    pub fn create(context: Rc<RefCell<&'a mut Context<'a>>>, stage_type: StageType, prev_stage: Option<&'a mut Stage<'a, 'b>>) -> Stage<'a, 'b> {
        match stage_type {
            StageType::Fetch => Stage::new(context, fetch, prev_stage),
            StageType::Decode => Stage::new(context, decode, prev_stage),
            StageType::Execute => Stage::new(context, execute, prev_stage),
            StageType::Memory => Stage::new(context, memory, prev_stage),
            StageType::Writeback => Stage::new(context, writeback, prev_stage),
        }
    }

    fn new(context: Rc<RefCell<&'a mut Context<'a>>>, 
           process: fn(Rc<RefCell<&'a mut Context<'a>>>, &mut Instruction) -> bool,
           prev_stage: Option<&'a mut Stage<'a, 'b>>) -> Stage<'a, 'b> {
        Stage {
            status: StageStatus {
                finished: false,
                pipeline_on: true,
            },
            instruction: None,
            context: context,
            prev_stage: prev_stage,
            process: process,
        }
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

    pub fn cycle(&mut self) {
        self.load();
        if let Some(instr) = &mut self.instruction {
            if instr.meta.squashed { self.status.finished = true; }
            if !self.status.finished { 
                self.status.finished = (self.process)(Rc::clone(&self.context), instr);
            }
        }
        if let Some(prev) = &mut self.prev_stage {
            prev.cycle();
        }
    }
} 

fn fetch<'a>(context: Rc<RefCell<&'a mut Context<'a>>>, instr: &mut Instruction) -> bool {
    let mut cons = context.borrow_mut();
    let instr_addr = cons.registers.get_reg(Register::SP) as usize;
    if let Some(MemoryValue::Value(value)) = cons.memory.read(instr_addr, StageType::Fetch, false) {
        instr.instr_raw = value as u32;
        cons.registers.set_reg(Register::SP, (instr_addr + 4) as u32);
        return true;
    }
    false
}

fn decode<'a>(_context: Rc<RefCell<&'a mut Context<'a>>>, _instr: &mut Instruction) -> bool {
    true
}

fn execute<'a>(_context: Rc<RefCell<&'a mut Context<'a>>>, _instr: &mut Instruction) -> bool {
    true
}

fn memory<'a>(_context: Rc<RefCell<&'a mut Context<'a>>>, _instr: &mut Instruction) -> bool {
    true
}

fn writeback<'a>(_context: Rc<RefCell<&'a mut Context<'a>>>, _instr: &mut Instruction) -> bool {
    true
}