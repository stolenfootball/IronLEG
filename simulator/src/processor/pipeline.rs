
use std::cell::RefCell;
use std::rc::Rc;

use crate::memory::MemoryValue;

use super::instruction::Instruction;
use super::Context;
use super::registers::Register;
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
                StageType::Fetch => fetch as fn(Rc<RefCell<Box<Context>>>, &mut Instruction) -> StageResult,
                StageType::Decode => decode as fn(Rc<RefCell<Box<Context>>>, &mut Instruction) -> StageResult,
                StageType::Execute => execute as fn(Rc<RefCell<Box<Context>>>, &mut Instruction) -> StageResult,
                StageType::Memory => memory as fn(Rc<RefCell<Box<Context>>>, &mut Instruction) -> StageResult,
                StageType::Writeback => writeback as fn(Rc<RefCell<Box<Context>>>, &mut Instruction) -> StageResult,
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

fn fetch<'a>(context: Rc<RefCell<Box<Context>>>, instr: &mut Instruction) -> StageResult {
    let mut ctx = context.borrow_mut();
    let instr_addr = ctx.registers.get_reg(Register::SP) as usize;
    if let Some(MemoryValue::Value(value)) = ctx.memory.read(instr_addr, StageType::Fetch, false) {
        instr.instr_raw = value as i32;
        ctx.registers.set_reg(Register::SP, (instr_addr + 4) as i32);
        return StageResult::DONE;
    }
    StageResult::WAIT
}

fn decode<'a>(context: Rc<RefCell<Box<Context>>>, instr: &mut Instruction) -> StageResult {
    let raw = instr.instr_raw;

    let opcode = (raw >> 25) & 0xF;
    instr.instr_type = match raw >> 29 {
        0b000 => InstrType::ALU(ALUType::from_i32(opcode)),
        0b001 => InstrType::Memory(MemoryType::from_i32(opcode)),
        0b010 => InstrType::Control(ControlType::from_i32(opcode)),
        0b011 => InstrType::Interrupt(InterruptType::from_i32(opcode)),
        _ => panic!("balls")
    };
    
    instr.addr_mode = AddrMode::from_i32((raw >> 22) & 0x7);


    let regs = &mut context.borrow_mut().registers;
    match instr.addr_mode {
        AddrMode::RegReg => {
            instr.reg_1 = Register::from_i32((raw >> 18) & 0xF);
            instr.reg_2 = Register::from_i32((raw >> 14) & 0xF);
            instr.dest = instr.reg_1;

            if regs.is_in_use(instr.reg_1) || regs.is_in_use(instr.reg_2) {
                return StageResult::WAIT;
            }
        },
        AddrMode::RegRegOff => {
            instr.reg_1 = Register::from_i32((raw >> 18) & 0xF);
            instr.reg_2 = Register::from_i32((raw >> 14) & 0xF);
            instr.imm = raw & 0xFFFF;
            instr.dest = instr.reg_1;

            if regs.is_in_use(instr.reg_1) || regs.is_in_use(instr.reg_2) {
                return StageResult::WAIT;
            }
        },
        AddrMode::RegImm => {
            instr.reg_1 = Register::from_i32((raw >> 18) & 0xF);
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
            instr.reg_1 = Register::from_i32((raw >> 18) & 0xF);
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

fn execute<'a>(context: Rc<RefCell<Box<Context>>>, instr: &mut Instruction) -> StageResult {
    let regs = &mut context.borrow_mut().registers;
    match instr.instr_type {
        InstrType::ALU(opcode) => {
            match opcode {
                ALUType::MOV  => instr.meta.result = instr.get_arg_2(regs) + instr.imm,
                ALUType::ADD  => instr.meta.result = instr.get_arg_1(regs) + instr.get_arg_2(regs) + instr.imm,
                ALUType::SUB  => instr.meta.result = instr.get_arg_1(regs) - instr.get_arg_2(regs) + instr.imm,
                ALUType::IMUL => instr.meta.result = instr.get_arg_1(regs) * instr.get_arg_2(regs) + instr.imm,
                ALUType::IDIV => instr.meta.result = instr.get_arg_1(regs) / instr.get_arg_2(regs) + instr.imm,
                ALUType::AND  => instr.meta.result = instr.get_arg_1(regs) & instr.get_arg_2(regs) + instr.imm,
                ALUType::OR   => instr.meta.result = instr.get_arg_1(regs) | instr.get_arg_2(regs) + instr.imm,
                ALUType::XOR  => instr.meta.result = instr.get_arg_1(regs) ^ instr.get_arg_2(regs) + instr.imm,
                ALUType::CMP  => instr.meta.result = instr.get_arg_1(regs) - (instr.get_arg_2(regs) + instr.imm),
                ALUType::MOD  => instr.meta.result = instr.get_arg_1(regs) % instr.get_arg_2(regs) + instr.imm,
                ALUType::NOT  => instr.meta.result = !(instr.get_arg_1(regs) + instr.imm),
                ALUType::LSL  => instr.meta.result = instr.get_arg_1(regs) << instr.get_arg_2(regs) + instr.imm,
                ALUType::LSR  => instr.meta.result = instr.get_arg_1(regs) >> instr.get_arg_2(regs) + instr.imm,
            };
            StageResult::DONE
        },
        InstrType::Control(opcode) => {
            if match opcode {
                ControlType::BEQ  => regs.get_reg(Register::BF) == 0,
                ControlType::BLT  => regs.get_reg(Register::BF) <  0,
                ControlType::BGT  => regs.get_reg(Register::BF) >  0,
                ControlType::BNE  => regs.get_reg(Register::BF) != 0,
                ControlType::B    => true,
                ControlType::BGE  => regs.get_reg(Register::BF) >= 0,
                ControlType::BLE  => regs.get_reg(Register::BF) <= 0,
            } { 
                instr.meta.result = instr.get_arg_1(regs) 
            } else { 
                instr.meta.writeback = false 
            }
            StageResult::DONE
        },
        InstrType::Memory(_opcode) => StageResult::DONE,
        InstrType::Interrupt(_opcode) => StageResult::DONE,
    }
}

fn memory<'a>(context: Rc<RefCell<Box<Context>>>, instr: &mut Instruction) -> StageResult {
    let mut ctx = context.borrow_mut();
    if let InstrType::Memory(mem_type) = instr.instr_type  {
        let mem_addr = instr.get_arg_2(&ctx.registers) as usize;
        return match mem_type {
            MemoryType::LDR => {
                if let Some(MemoryValue::Value(response)) = ctx.memory.read(mem_addr, StageType::Memory, false) {
                    instr.meta.result = response as i32;
                }
                StageResult::WAIT
            },
            MemoryType::STR => {
                let val_to_store = instr.get_arg_1(&ctx.registers) as usize;
                if ctx.memory.write(mem_addr, &MemoryValue::Value(val_to_store), StageType::Memory) {
                    instr.meta.writeback = false;
                    return StageResult::DONE;
                }
                StageResult::WAIT
            },
        }
    }
    StageResult::DONE
}

fn writeback<'a>(context: Rc<RefCell<Box<Context>>>, instr: &mut Instruction) -> StageResult {
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