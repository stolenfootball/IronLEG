
use crate::processor::registers::{Register, Registers};

#[derive(Clone, Copy, Debug)]
pub enum InstrType {
    ALU(ALUType),
    Memory(MemoryType),
    Control(ControlType),
    Interrupt(InterruptType),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AddrMode {
    RegReg,
    RegRegOff,
    RegImm,
    Imm,
    Reg,
}

impl AddrMode {
    pub fn from_u32(mode: u32) -> AddrMode {
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

#[derive(Clone, Copy, Debug, PartialEq)]
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
    pub fn from_u32(alu_type: u32) -> ALUType {
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum MemoryType {
    LDR,
    STR,
}

impl MemoryType {
    pub fn from_u32(mem_type: u32) -> MemoryType {
        match mem_type {
            0b0000 => MemoryType::LDR,
            0b0001 => MemoryType::STR,
            _ => panic!("MemoryType convert failure: {}", mem_type)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ControlType {
    BEQ,
    BLT,
    BGT,
    BNE,
    B,
    CALL,
    RET,
    BGE,
    BLE,
}
impl ControlType {
    pub fn from_u32(ctrl_type: u32) -> ControlType {
        match ctrl_type {
            0b0000 => ControlType::BEQ,
            0b0001 => ControlType::BLT,
            0b0010 => ControlType::BGT,
            0b0011 => ControlType::BNE,
            0b0100 => ControlType::B,
            0b0101 => ControlType::CALL,
            0b0110 => ControlType::RET,
            0b0111 => ControlType::BGE,
            0b1000 => ControlType::BLE,
            _ => panic!("ControlType convert failure: {}", ctrl_type)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum InterruptType {
    NOP,
    HLT,
}
impl InterruptType {
    pub fn from_u32(int_type: u32) -> InterruptType {
        match int_type {
            0b0000 => InterruptType::NOP,
            0b0001 => InterruptType::HLT,
            _ => panic!("InterruptType convert failure: {}", int_type)
        }
    }
}


#[derive(Debug)]
pub struct InstrMeta {
    pub writeback: bool,
    pub squashed: bool,
    pub result: u32,
}

#[derive(Debug)]
pub struct Instruction {
    pub instr_raw: u32,
    pub instr_type: Option<InstrType>,
    pub addr_mode: Option<AddrMode>,
    pub reg_1: Option<Register>,
    pub reg_2: Option<Register>,
    pub dest: Option<Register>,
    pub imm: Option<u32>,
    pub meta: InstrMeta,
}

impl Instruction {
    pub fn new() -> Self {
        Self {
            instr_raw: 0,
            instr_type: None,
            addr_mode: None,
            reg_1: None,
            reg_2: None,
            dest: None,
            imm: None,
            meta: InstrMeta {
                writeback: false,
                squashed: false,
                result: 0,
            },
        }
    }

    pub fn get_arg_1(&self, regs: &Registers) -> u32 {
        match self.addr_mode.unwrap() {
            AddrMode::RegReg => regs.get_reg(self.reg_1.unwrap()),
            AddrMode::RegRegOff => regs.get_reg(self.reg_1.unwrap()),
            AddrMode::RegImm => regs.get_reg(self.reg_1.unwrap()),
            AddrMode::Imm => self.imm.unwrap(),
            AddrMode::Reg => regs.get_reg(self.reg_1.unwrap()),
        }
    }

    pub fn get_arg_2(&self, regs: &Registers) -> u32 {
        match self.addr_mode.unwrap() {
            AddrMode::RegReg => regs.get_reg(self.reg_2.unwrap()),
            AddrMode::RegRegOff => regs.get_reg(self.reg_2.unwrap()),
            AddrMode::RegImm => self.imm.unwrap(),
            AddrMode::Imm => 0,
            AddrMode::Reg => regs.get_reg(self.reg_1.unwrap()),
        }
    }
}