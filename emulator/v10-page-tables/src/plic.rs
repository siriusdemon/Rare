use crate::param::*;
use crate::exception::RvException;

use RvException::*;



pub struct Plic {
    pending: u64,
    senable: u64,
    spriority: u64,
    sclaim: u64,
}

impl Plic {
    pub fn new() -> Self {
        Self {pending: 0, senable: 0, spriority: 0, sclaim: 0}
    }

    pub fn load(&self, addr: u64, size: u64) -> Result<u64, RvException> {
        if size != 32 {
            return Err(LoadAccessFault(addr));
        }
        match addr {
            PLIC_PENDING => Ok(self.pending),
            PLIC_SENABLE => Ok(self.senable),
            PLIC_SPRIORITY => Ok(self.spriority),
            PLIC_SCLAIM => Ok(self.sclaim),
            _ => Err(LoadAccessFault(addr)),
        }
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), RvException> {
        if size != 32 {
            return Err(StoreOrAMOAccessFault(addr));
        }
        match addr {
            PLIC_PENDING => Ok(self.pending = value),
            PLIC_SENABLE => Ok(self.senable = value),
            PLIC_SPRIORITY => Ok(self.spriority = value),
            PLIC_SCLAIM => Ok(self.sclaim = value),
            _ => Err(StoreOrAMOAccessFault(addr)),
        }
    }
}