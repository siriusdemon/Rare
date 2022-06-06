use std::fmt;

#[derive(Debug)]
pub enum RvException {
    // Riscv Standard Exception
    InstructionAddrMisaligned(u64),
    InstructionAccessFault(u64),
    IllegalInstruction(u64),
    Breakpoint(u64),
    LoadAccessMisaligned(u64),
    LoadAccessFault(u64),
    StoreOrAMOAddrMiisaligned(u64),
    StoreOrAMOAccessFault(u64),
    EnvironmentCallFromUmode(u64),
    EnvironmentCallFromSmode(u64),
    EnvironmentCallFromMmode(u64),
    InstructionPageFault(u64),
    LoadPageFault(u64),
    StoreOrAMOPageFault(u64),
    // Custom Exception, number range [24-31], [48-63]
    CustomInvalidSize(u64),
}

impl fmt::Display for RvException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use RvException::*;
        match self {
            InstructionAddrMisaligned(addr) => write!(f, "Instruction address misaligned {:#x}", addr),
            InstructionAccessFault(addr) => write!(f, "Instruction access fault {:#x}", addr),
            IllegalInstruction(inst) => write!(f, "Illegal instruction {:#x}", inst),
            InvalidInstruction(inst) => write!(f, "Invalid instruction {:#x}", inst),
            Breakpoint(inst) => write!(f, "Breakpoint {:#x}", inst),
            LoadAccessMisaligned(addr) => write!(f, "Load access {:#x}", addr),
            LoadAccessFault(addr) => write!(f, "Load access fault {:#x}", addr),
            StoreOrAmoAddrMiisaligned(addr) => write!(f, "Store or AMO address misaliged {:#x}", addr),
            StoreOrAMOAccessFault(addr) => write!(f, "Store or AMO access fault {:#x}", addr),
            EnvironmentCallFromUmode(inst) => write!(f, "Environment call from U-mode", inst),
            EnvironmentCallFromSmode(inst) => write!(f, "Environment call from S-mode", inst),
            EnvironmentCallFromMmode(inst) => write!(f, "Environment call from M-mode", inst),
            InstructionPageFault(inst) => write!(f, "Instruction page fault {:#x}", inst),
            LoadPageFault(addr) => write!(f, "Load page fault {:#x}", addr),
            StoreOrAMOPageFault(addr) => write!(f, "Store or AMO page fault {:#x}", addr),
            CustomInvalidSize(size) => write!(f, "Custion Exception: Invalid size {}", size),
        }
    }
}

