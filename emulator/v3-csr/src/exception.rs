use std::fmt;

#[derive(Debug)]
pub enum RvException {
    InvalidAddress(u64),
    InvalidSize(u64),
    InvalidInstruction(u64),
}

impl fmt::Display for RvException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RvException::*;
        match self {
            InvalidAddress(addr) => write!(f, "Invalid Address {:#x}", addr),
            InvalidSize(size) => write!(f, "Invalid size {}", size),
            InvalidInstruction(inst) => write!(f, "Invalid instruction {:#x}", inst),
        }
    }
}