const GLOBAL_HISTORY_SIZE: usize = 16;
const COUNTER_BITS: u32 = 2;
const PATTERN_HISTORY_TABLE_LENGTH: usize = 2_usize.pow(GLOBAL_HISTORY_SIZE as u32);

pub struct Predictor {
    global_history_register: usize,
    pattern_history_table: Vec<u8>
}

impl Predictor {
    pub fn new() -> Predictor {
        Predictor {
            global_history_register: 0,
            pattern_history_table: vec![2_u8.pow(COUNTER_BITS); 
                                        PATTERN_HISTORY_TABLE_LENGTH],
        }
    }

    fn get_table_index(&self, program_counter: u32) -> usize {
        (program_counter as usize ^ self.global_history_register) & (PATTERN_HISTORY_TABLE_LENGTH - 1)
    }

    pub fn predict(&self, program_counter: u32) -> bool {
        let index = self.get_table_index(program_counter);
        self.pattern_history_table[index] > (2_u8.pow(COUNTER_BITS) / 2)
    }

    pub fn update(&mut self, program_counter: u32, branch_taken: bool) {
        let index = self.get_table_index(program_counter);
        if branch_taken {
            self.pattern_history_table[index] += if self.pattern_history_table[index] < 4 { 1 } else { 0 };
            self.global_history_register = (self.global_history_register << 1) | 1 & (PATTERN_HISTORY_TABLE_LENGTH - 1);
        } else {
            self.pattern_history_table[index] -= if self.pattern_history_table[index] > 0 { 1 } else { 0 };
            self.global_history_register = (self.global_history_register << 1)     & (PATTERN_HISTORY_TABLE_LENGTH - 1);
        }
    }

}