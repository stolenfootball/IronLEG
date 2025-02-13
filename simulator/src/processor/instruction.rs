use crate::processor::registers::Register;

#[derive(Clone, Copy, Debug)]
pub enum InstrType {
    ALU(ALUType),
    Memory(MemoryType),
    Control(ControlType),
    Interrupt(InterruptType),
}

#[derive(Clone, Copy, Debug)]
pub enum AddrMode {
    Reg,
    Imm,
}

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub enum MemoryType {
    LDR,
    STR,
}

#[derive(Clone, Copy, Debug)]
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

#[derive(Clone, Copy, Debug)]
pub enum InterruptType {
    NOP,
    HLT,
}

#[derive(Debug)]
pub struct InstrMeta {
    pub writeback: bool,
    pub squashed: bool,
}

#[derive(Debug)]
pub struct Instruction {
    pub instr_type: Option<InstrType>,
    pub addr_mode: Option<AddrMode>,
    pub src: Option<Register>,
    pub dest: Option<Register>,
    pub imm: Option<u32>,
    pub meta: InstrMeta,
}

impl Instruction {
    pub fn new() -> Self {
        Self {
            instr_type: None,
            addr_mode: None,
            src: None,
            dest: None,
            imm: None,
            meta: InstrMeta {
                writeback: false,
                squashed: false,
            },
        }
    }
}