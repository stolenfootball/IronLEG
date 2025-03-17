use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

#[derive(Clone, Copy, Debug)]
pub enum Operation {
    ALU(ALUType),
    Memory(MemoryType),
    Control(ControlType),
    Interrupt(InterruptType),
}

impl Operation {
    pub fn decode(instr_type: usize, opcode: usize) -> Operation {
        match instr_type {
            0 => Operation::ALU(ALUType::from_usize(opcode)),
            1 => Operation::Memory(MemoryType::from_usize(opcode)),
            2 => Operation::Control(ControlType::from_usize(opcode)),
            3 => Operation::Interrupt(InterruptType::from_usize(opcode)),
            _ => panic!("balls"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
pub enum AddrMode {
    RegReg,
    RegRegOff,
    RegImm,
    Imm,
    Reg,
}

impl AddrMode {
    pub fn from_usize(num: usize) -> AddrMode {
        <AddrMode as FromPrimitive>::from_usize(num).unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
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
    pub fn from_usize(num: usize) -> ALUType {
        <ALUType as FromPrimitive>::from_usize(num).unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
pub enum MemoryType {
    LDR,
    STR,
}

impl MemoryType {
    pub fn from_usize(num: usize) -> MemoryType {
        <MemoryType as FromPrimitive>::from_usize(num).unwrap()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]

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
    pub fn from_usize(num: usize) -> ControlType {
        <ControlType as FromPrimitive>::from_usize(num).unwrap()
    }
}


#[derive(Clone, Copy, Debug, PartialEq, FromPrimitive)]
pub enum InterruptType {
    NOP,
    HLT,
}

impl InterruptType {
    pub fn from_usize(num: usize) -> InterruptType {
        <InterruptType as FromPrimitive>::from_usize(num).unwrap()
    }
}
