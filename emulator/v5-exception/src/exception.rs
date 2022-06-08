use std::fmt;

#[derive(Copy, Clone)]
pub enum RvException {
    // Riscv Standard Exception
    InstructionAddrMisaligned(u64),
    InstructionAccessFault(u64),
    IllegalInstruction(u64),
    Breakpoint(u64),
    LoadAccessMisaligned(u64),
    LoadAccessFault(u64),
    StoreOrAMOAddrMisaligned(u64),
    StoreOrAMOAccessFault(u64),
    EnvironmentCallFromUmode(u64),
    EnvironmentCallFromSmode(u64),
    EnvironmentCallFromMmode(u64),
    InstructionPageFault(u64),
    LoadPageFault(u64),
    StoreOrAMOPageFault(u64),
}

use RvException::*;
impl fmt::Display for RvException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstructionAddrMisaligned(addr) => write!(f, "Instruction address misaligned {:#x}", addr),
            InstructionAccessFault(addr) => write!(f, "Instruction access fault {:#x}", addr),
            IllegalInstruction(inst) => write!(f, "Illegal instruction {:#x}", inst),
            Breakpoint(inst) => write!(f, "Breakpoint {:#x}", inst),
            LoadAccessMisaligned(addr) => write!(f, "Load access {:#x}", addr),
            LoadAccessFault(addr) => write!(f, "Load access fault {:#x}", addr),
            StoreOrAMOAddrMisaligned(addr) => write!(f, "Store or AMO address misaliged {:#x}", addr),
            StoreOrAMOAccessFault(addr) => write!(f, "Store or AMO access fault {:#x}", addr),
            EnvironmentCallFromUmode(inst) => write!(f, "Environment call from U-mode {:#x}", inst),
            EnvironmentCallFromSmode(inst) => write!(f, "Environment call from S-mode {:#x}", inst),
            EnvironmentCallFromMmode(inst) => write!(f, "Environment call from M-mode {:#x}", inst),
            InstructionPageFault(addr) => write!(f, "Instruction page fault {:#x}", addr),
            LoadPageFault(addr) => write!(f, "Load page fault {:#x}", addr),
            StoreOrAMOPageFault(addr) => write!(f, "Store or AMO page fault {:#x}", addr),
        }
    }
}


impl RvException {
    pub fn value(self) -> u64 {
        match self {
            InstructionAddrMisaligned(addr) => addr,
            InstructionAccessFault(addr) => addr,
            IllegalInstruction(inst) => inst,
            Breakpoint(inst) => inst,
            LoadAccessMisaligned(addr) => addr,
            LoadAccessFault(addr) => addr,
            StoreOrAMOAddrMisaligned(addr) => addr,
            StoreOrAMOAccessFault(addr) => addr,
            EnvironmentCallFromUmode(inst) => inst,
            EnvironmentCallFromSmode(inst) => inst,
            EnvironmentCallFromMmode(inst) => inst,
            InstructionPageFault(inst) => inst,
            LoadPageFault(addr) => addr,
            StoreOrAMOPageFault(addr) => addr,
        }
    }

    pub fn code(self) -> u64 {
        match self {
            InstructionAddrMisaligned(_) => 0,
            InstructionAccessFault(_) => 1,
            IllegalInstruction(_) => 2,
            Breakpoint(_) => 3,
            LoadAccessMisaligned(_) => 4,
            LoadAccessFault(_) => 5,
            StoreOrAMOAddrMisaligned(_) => 6,
            StoreOrAMOAccessFault(_) => 7,
            EnvironmentCallFromUmode(_) => 8,
            EnvironmentCallFromSmode(_) => 9,
            EnvironmentCallFromMmode(_) => 11,
            InstructionPageFault(_) => 12,
            LoadPageFault(_) => 13,
            StoreOrAMOPageFault(_) => 15,
        }
    }

    pub fn is_fatal(self) -> bool {
        match self {
            InstructionAddrMisaligned(_)
            | InstructionAccessFault(_)
            | LoadAccessFault(_)
            | StoreOrAMOAddrMisaligned(_)
            | StoreOrAMOAccessFault(_) 
            | IllegalInstruction(_) => true,
            _else => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_code() {
        let e = RvException::IllegalInstruction(0x0);
        assert_eq!(e.value(), 0);
        assert_eq!(e.value(), 0);
        assert_eq!(e.code(), 2);
    }
}