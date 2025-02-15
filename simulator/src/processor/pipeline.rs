
use std::cell::RefCell;
use std::rc::Rc;
use crate::processor::instruction::Instruction;
use crate::processor::Context;
use crate::processor::registers::Register;
use crate::memory::memory::MemoryValue;

use super::instruction::{ALUType, AddrMode, ControlType, InstrType, InterruptType, MemoryType};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StageType {
    Fetch,
    Decode,
    Execute,
    Memory,
    Writeback,
}

enum StageResult {
    DONE,
    WAIT,
    SQUASH,
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
    process: fn(Rc<RefCell<&'a mut Context<'a>>>, &mut Instruction) -> StageResult,
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
           process: fn(Rc<RefCell<&'a mut Context<'a>>>, &mut Instruction) -> StageResult,
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

fn fetch<'a>(context: Rc<RefCell<&'a mut Context<'a>>>, instr: &mut Instruction) -> StageResult {
    let mut ctx = context.borrow_mut();
    let instr_addr = ctx.registers.get_reg(Register::SP) as usize;
    if let Some(MemoryValue::Value(value)) = ctx.memory.read(instr_addr, StageType::Fetch, false) {
        instr.instr_raw = value as u32;
        ctx.registers.set_reg(Register::SP, (instr_addr + 4) as u32);
        return StageResult::DONE;
    }
    StageResult::WAIT
}

fn decode<'a>(context: Rc<RefCell<&'a mut Context<'a>>>, instr: &mut Instruction) -> StageResult {
    let raw = instr.instr_raw;

    let opcode = (raw >> 25) & 0xF;
    instr.instr_type = match raw >> 29 {
        0b000 => InstrType::ALU(ALUType::from_u32(opcode)),
        0b001 => InstrType::Memory(MemoryType::from_u32(opcode)),
        0b010 => InstrType::Control(ControlType::from_u32(opcode)),
        0b011 => InstrType::Interrupt(InterruptType::from_u32(opcode)),
        _ => panic!("balls")
    };
    
    instr.addr_mode = AddrMode::from_u32((raw >> 22) & 0x7);


    let regs = &mut context.borrow_mut().registers;
    match instr.addr_mode {
        AddrMode::RegReg => {
            instr.reg_1 = Register::from_u32((raw >> 18) & 0xF);
            instr.reg_2 = Register::from_u32((raw >> 14) & 0xF);
            instr.dest = instr.reg_1;

            if regs.is_in_use(instr.reg_1) || regs.is_in_use(instr.reg_2) {
                return StageResult::WAIT;
            }
        },
        AddrMode::RegRegOff => {
            instr.reg_1 = Register::from_u32((raw >> 18) & 0xF);
            instr.reg_2 = Register::from_u32((raw >> 14) & 0xF);
            instr.imm = raw & 0xFFFF;
            instr.dest = instr.reg_1;

            if regs.is_in_use(instr.reg_1) || regs.is_in_use(instr.reg_2) {
                return StageResult::WAIT;
            }
        },
        AddrMode::RegImm => {
            instr.reg_1 = Register::from_u32((raw >> 18) & 0xF);
            instr.imm = raw & 0xFFF;
            instr.dest = instr.reg_1;

            if regs.is_in_use(instr.reg_1) {
                return StageResult::WAIT;
            }
        },
        AddrMode::Imm => {
            instr.imm = raw & 0x3FFFFF
        },
        AddrMode::Reg => {
            instr.reg_1 = Register::from_u32((raw >> 18) & 0xF);
            instr.dest = instr.reg_1;

            if regs.is_in_use(instr.reg_1) {
                return StageResult::WAIT;
            }
        },
    }

    if let InstrType::ALU(value) = instr.instr_type {
        if value == ALUType::CMP {
            instr.dest = Register::BF;
        }
    }

    if let InstrType::Control(_) = instr.instr_type {
        if regs.is_in_use(Register::BF) { return StageResult::WAIT; }
        instr.dest = Register::PC;
    }
            
    regs.set_in_use(instr.dest, true);
    StageResult::DONE
}

fn execute<'a>(_context: Rc<RefCell<&'a mut Context<'a>>>, instr: &mut Instruction) -> StageResult {
    match instr.instr_type {
        InstrType::ALU(_alu_type) => StageResult::DONE,
        InstrType::Control(_ctrl_type) => StageResult::DONE,
        InstrType::Memory(_mem_type) => StageResult::DONE,
        InstrType::Interrupt(_int_type) => StageResult::DONE,
    }
}

fn memory<'a>(context: Rc<RefCell<&'a mut Context<'a>>>, instr: &mut Instruction) -> StageResult {
    let mut ctx = context.borrow_mut();
    if let InstrType::Memory(mem_type) = instr.instr_type  {
        let mem_addr = instr.get_arg_2(&ctx.registers) as usize;
        return match mem_type {
            MemoryType::LDR => {
                if let Some(MemoryValue::Value(response)) = ctx.memory.read(mem_addr, StageType::Memory, false) {
                    instr.meta.result = response as u32;
                }
                StageResult::WAIT
            },
            MemoryType::STR => {
                let val_to_store = instr.get_arg_1(&ctx.registers) as usize;
                if ctx.memory.write(mem_addr, MemoryValue::Value(val_to_store), StageType::Memory) {
                    instr.meta.writeback = false;
                    return StageResult::DONE;
                }
                StageResult::WAIT
            },
        }
    }
    StageResult::DONE
}

fn writeback<'a>(context: Rc<RefCell<&'a mut Context<'a>>>, instr: &mut Instruction) -> StageResult {
    let mut ctx = context.borrow_mut();
    if instr.meta.writeback {
        ctx.registers.set_reg(instr.dest, instr.meta.result);
    }
    ctx.registers.set_in_use(instr.dest, false);

    if instr.meta.writeback {
        if let InstrType::Control(_) = instr.instr_type {
            ctx.registers.clear_in_use();
            ctx.memory.reset_state();
            return StageResult::SQUASH;
        }
    }   
    StageResult::DONE
}