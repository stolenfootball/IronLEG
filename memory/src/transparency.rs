pub mod transparency {
    use crate::memory;
    pub trait Transparency {
        fn peek_line(&self, addr: usize) -> &Vec<usize>;
        fn peek_access(&self) -> &memory::MemoryAccess;
    }
}