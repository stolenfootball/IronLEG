use serde::Serialize;

#[derive(Debug, Copy, Clone, Serialize)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    SP,
    BF,
    LR,
    PC,
}

impl Register {
    pub fn from_i32(reg: i32) -> Register {
        match reg {
            00 => Register::R0,
            01 => Register::R1,
            02 => Register::R2,
            03 => Register::R3,
            04 => Register::R4,
            05 => Register::R5,
            06 => Register::R6,
            07 => Register::R7,
            08 => Register::R8,
            09 => Register::R9,
            10 => Register::R10,
            11 => Register::R11,
            12 => Register::SP,
            13 => Register::BF,
            14 => Register::LR,
            15 => Register::PC,
            _ => panic!("Register convert failure: {}", reg)
        }
    }
}

#[derive(Debug)]
pub struct Registers {
    pub registers: [i32; 16],
    pub in_use: [bool; 16],
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            registers: [0; 16],
            in_use: [false; 16],
        }
    }

    pub fn get_reg(&self, reg: Register) -> i32 {
        self.registers[reg as usize]
    }

    pub fn set_reg(&mut self, reg: Register, value: i32) {
        self.registers[reg as usize] = value;
    }

    pub fn set_in_use(&mut self, reg: Register, value: bool) {
        self.in_use[reg as usize] = value;
    }

    pub fn is_in_use(&self, reg: Register) -> bool {
        self.in_use[reg as usize]
    }

    pub fn clear_in_use(&mut self) {
        self.in_use.iter_mut().for_each(|x| *x = false);
    }

    pub fn reset(&mut self) {
        self.clear_in_use();
        self.registers = [0; 16];
    }
}

