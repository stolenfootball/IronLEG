use serde::Serialize;

use super::registers::{Register, Registers};

#[derive(Clone, Copy, Debug, Serialize)]
pub enum InstrType {
    ALU(ALUType),
    Memory(MemoryType),
    Control(ControlType),
    Interrupt(InterruptType),
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum AddrMode {
    RegReg,
    RegRegOff,
    RegImm,
    Imm,
    Reg,
}

impl AddrMode {
    pub fn from_i32(mode: i32) -> AddrMode {
        match mode {
            0b000 => AddrMode::RegReg,
            0b001 => AddrMode::RegRegOff,
            0b010 => AddrMode::RegImm,
            0b011 => AddrMode::Imm,
            0b100 => AddrMode::Reg,
            _ => panic!("AddrMode convert failure: {}", mode)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum ALUType {
    MOV,
    ADD,
    SUB,
    IMUL,
    IDIV,
    AND,
    OR,
    XOR,
    CMP,
    MOD,
    NOT,
    LSL,
    LSR,
}
impl ALUType {
    pub fn from_i32(alu_type: i32) -> ALUType {
        match alu_type {
            0b0000 => ALUType::MOV,
            0b0001 => ALUType::ADD,
            0b0010 => ALUType::SUB,
            0b0011 => ALUType::IMUL,
            0b0100 => ALUType::IDIV,
            0b0101 => ALUType::AND,
            0b0110 => ALUType::OR,
            0b0111 => ALUType::XOR,
            0b1000 => ALUType::CMP,
            0b1001 => ALUType::MOD,
            0b1010 => ALUType::NOT,
            0b1011 => ALUType::LSL,
            0b1100 => ALUType::LSR,
            _ => panic!("ALUType convert failure: {}", alu_type)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum MemoryType {
    LDR,
    STR,
}

impl MemoryType {
    pub fn from_i32(mem_type: i32) -> MemoryType {
        match mem_type {
            0b0000 => MemoryType::LDR,
            0b0001 => MemoryType::STR,
            _ => panic!("MemoryType convert failure: {}", mem_type)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum ControlType {
    BEQ,
    BLT,
    BGT,
    BNE,
    B,
    BGE,
    BLE,
}
impl ControlType {
    pub fn from_i32(ctrl_type: i32) -> ControlType {
        match ctrl_type {
            0b0000 => ControlType::BEQ,
            0b0001 => ControlType::BLT,
            0b0010 => ControlType::BGT,
            0b0011 => ControlType::BNE,
            0b0100 => ControlType::B,
            0b0101 => ControlType::BGE,
            0b0110 => ControlType::BLE,
            _ => panic!("ControlType convert failure: {}", ctrl_type)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub enum InterruptType {
    NOP,
    HLT,
}
impl InterruptType {
    pub fn from_i32(int_type: i32) -> InterruptType {
        match int_type {
            0b0000 => InterruptType::NOP,
            0b0001 => InterruptType::HLT,
            _ => panic!("InterruptType convert failure: {}", int_type)
        }
    }
}


#[derive(Debug, Serialize, Clone)]
pub struct InstrMeta {
    pub writeback: bool,
    pub squashed: bool,
    pub result: i32,
    pub initialized: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct Instruction {
    pub instr_raw: i32,
    pub instr_type: InstrType,
    pub addr_mode: AddrMode,
    pub reg_1: Register,
    pub reg_2: Register,
    pub dest: Register,
    pub imm: i32,
    pub meta: InstrMeta,
}

impl Instruction {
    pub fn new() -> Self {
        Self {
            instr_raw: 0,
            instr_type: InstrType::ALU(ALUType::MOV),
            addr_mode: AddrMode::RegReg,
            reg_1: Register::R0,
            reg_2: Register::R0,
            dest: Register::R0,
            imm: 0,
            meta: InstrMeta {
                writeback: true,
                squashed: false,
                result: 0,
                initialized: false,
            },
        }
    }

    pub fn get_arg_1(&self, regs: &Registers) -> i32 {
        match self.addr_mode {
            AddrMode::RegReg => regs.get_reg(self.reg_1),
            AddrMode::RegRegOff => regs.get_reg(self.reg_1),
            AddrMode::RegImm => regs.get_reg(self.reg_1),
            AddrMode::Imm => 0,
            AddrMode::Reg => regs.get_reg(self.reg_1),
        }
    }

    pub fn get_arg_2(&self, regs: &Registers) -> i32 {
        match self.addr_mode {
            AddrMode::RegReg => regs.get_reg(self.reg_2),
            AddrMode::RegRegOff => regs.get_reg(self.reg_2),
            AddrMode::RegImm => 0,
            AddrMode::Imm => 0,
            AddrMode::Reg => 0,
        }
    }
}