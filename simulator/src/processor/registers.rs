#[derive(Debug, Copy, Clone)]
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

#[derive(Debug)]
pub struct Registers {
    pub registers: [u32; 16],
    in_use: [bool; 16],
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            registers: [0; 16],
            in_use: [false; 16],
        }
    }

    pub fn get_reg(&self, reg: Register) -> u32 {
        self.registers[reg as usize]
    }

    pub fn set_reg(&mut self, reg: Register, value: u32) {
        self.registers[reg as usize] = value;
    }

    pub fn set_in_use(&mut self, reg: Register, value: bool) {
        self.in_use[reg as usize] = value;
    }

    pub fn is_in_use(&self, reg: Register) -> bool {
        self.in_use[reg as usize]
    }
}

