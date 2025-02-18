use crate::memory::Memory;
use self::registers::Registers;

pub mod instruction;
pub mod registers;
pub mod pipeline;

pub struct Context<'a> {
    pub registers: Box<&'a mut Registers>,
    pub memory: &'a mut dyn Memory,
}