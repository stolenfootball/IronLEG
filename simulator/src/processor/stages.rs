use std::sync::{Arc, Mutex};

use crate::memory::{Memory, MemoryValue};

use super::registers::{Register, Registers};
use super::instruction::{Instruction, ALUType, AddrMode, ControlType, InstrType, InterruptType, MemoryType};
use super::pipeline::StageResult;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum StageType {
    Fetch,
    Decode,
    Execute,
    Memory,
    Writeback,
}


pub fn fetch<'a>(mem: Arc<Mutex<Box<dyn Memory>>>, regs: Arc<Mutex<Registers>>, instr: &mut Instruction) -> StageResult {
    let instr_addr = regs.lock().unwrap().get_reg(Register::PC) as usize;
    if let Some(MemoryValue::Value(value)) = mem.lock().unwrap().read(instr_addr, StageType::Fetch, false) {
        instr.instr_raw = value as i32;
        instr.meta.initialized = true;
        regs.lock().unwrap().set_reg(Register::PC, (instr_addr + 4) as i32);
        return StageResult::DONE;
    }
    StageResult::WAIT
}

pub fn decode<'a>(_mem: Arc<Mutex<Box<dyn Memory>>>, regs: Arc<Mutex<Registers>>, instr: &mut Instruction) -> StageResult {
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


    let mut regs = regs.lock().unwrap();
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

pub fn execute<'a>(_mem: Arc<Mutex<Box<dyn Memory>>>, regs: Arc<Mutex<Registers>>, instr: &mut Instruction) -> StageResult {
    let regs = regs.lock().unwrap();
    match instr.instr_type {
        InstrType::ALU(opcode) => {
            instr.meta.result = match opcode {
                ALUType::MOV  => instr.get_arg_2(&regs) + instr.imm,
                ALUType::ADD  => instr.get_arg_1(&regs) + (instr.get_arg_2(&regs) + instr.imm),
                ALUType::SUB  => instr.get_arg_1(&regs) - (instr.get_arg_2(&regs) + instr.imm),
                ALUType::IMUL => instr.get_arg_1(&regs) * (instr.get_arg_2(&regs) + instr.imm),
                ALUType::IDIV => instr.get_arg_1(&regs) / (instr.get_arg_2(&regs) + instr.imm),
                ALUType::AND  => instr.get_arg_1(&regs) & (instr.get_arg_2(&regs) + instr.imm),
                ALUType::OR   => instr.get_arg_1(&regs) | (instr.get_arg_2(&regs) + instr.imm),
                ALUType::XOR  => instr.get_arg_1(&regs) ^ (instr.get_arg_2(&regs) + instr.imm),
                ALUType::CMP  => instr.get_arg_1(&regs) - (instr.get_arg_2(&regs) + instr.imm),
                ALUType::MOD  => instr.get_arg_1(&regs) % (instr.get_arg_2(&regs) + instr.imm),
                ALUType::NOT  => !(instr.get_arg_1(&regs) + instr.imm),
                ALUType::LSL  => instr.get_arg_1(&regs) << (instr.get_arg_2(&regs) + instr.imm),
                ALUType::LSR  => instr.get_arg_1(&regs) >> (instr.get_arg_2(&regs) + instr.imm),
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
                instr.meta.result = instr.get_arg_1(&regs) + instr.imm
            } else { 
                instr.meta.writeback = false 
            }
            StageResult::DONE
        },
        InstrType::Memory(_opcode) => StageResult::DONE,
        InstrType::Interrupt(_opcode) => StageResult::DONE,
    }
}

pub fn memory<'a>(mem: Arc<Mutex<Box<dyn Memory>>>, regs: Arc<Mutex<Registers>>, instr: &mut Instruction) -> StageResult {
    let regs = regs.lock().unwrap();
    let mut mem = mem.lock().unwrap();

    if let InstrType::Memory(mem_type) = instr.instr_type  {
        let mem_addr = instr.get_arg_2(&regs) as usize;
        return match mem_type {
            MemoryType::LDR => {
                if let Some(MemoryValue::Value(response)) = mem.read(mem_addr, StageType::Memory, false) {
                    instr.meta.result = response as i32;
                }
                StageResult::WAIT
            },
            MemoryType::STR => {
                let val_to_store = instr.get_arg_1(&regs) as usize;
                if mem.write(mem_addr, &MemoryValue::Value(val_to_store), StageType::Memory) {
                    instr.meta.writeback = false;
                    return StageResult::DONE;
                }
                StageResult::WAIT
            },
        }
    }
    StageResult::DONE
}

pub fn writeback<'a>(mem: Arc<Mutex<Box<dyn Memory>>>, regs: Arc<Mutex<Registers>>, instr: &mut Instruction) -> StageResult {
    if instr.meta.squashed { return StageResult::DONE }

    let mut regs = regs.lock().unwrap();
    if instr.meta.writeback {
        regs.set_reg(instr.dest, instr.meta.result);
    }
    regs.set_in_use(instr.dest, false);

    if instr.meta.writeback {
        if let InstrType::Control(_) = instr.instr_type {
            regs.clear_in_use();
            mem.lock().unwrap().reset_state();
            return StageResult::SQUASH;
        }
    }

    if let InstrType::Interrupt(InterruptType::HLT) = instr.instr_type {
        return StageResult::HALT;
    }

    StageResult::DONE
}