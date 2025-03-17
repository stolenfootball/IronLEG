use std::collections::VecDeque;

const NUM_REGS: usize = 16;

pub const SP: usize = 12;
pub const BF: usize = 13;
pub const LR: usize = 14;
pub const PC: usize = 16;

#[derive(Clone, Copy, Debug)]
pub enum Register {
    Value(usize),
    Station(usize),
}

impl Default for Register {
    fn default() -> Self {
        Register::Value(0)
    }
}

#[derive(Default)]
pub struct Registers {
    pub registers: [Register; NUM_REGS],
    pub instr_queue: VecDeque<usize>,
    pub flags: i32,
}

