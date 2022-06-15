use crate::param::{CLINT_BASE};
use crate::exception::RvException;

use RvException::*;

pub const CLINT_MTIMECMP: u64 = CLINT_BASE + 0x4000;
pub const CLINT_MTIME: u64 = CLINT_BASE + 0xbff8;

pub struct Clint {
    mtime: u64,
    mtimecmp: u64,
}

impl Clint {
    pub fn new() -> Self {
        Self { mtime: 0, mtimecmp: 0 }
    }
    
    pub fn load(&self, addr: u64, size: u64) -> Result<u64, RvException> {
        if size != 64 {
            return Err(LoadAccessFault(addr));
        }
        match addr {
            CLINT_MTIMECMP => Ok(self.mtimecmp),
            CLINT_MTIME => Ok(self.mtime),
            _ => Err(LoadAccessFault(addr)),
        }
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), RvException> {
        if size != 64 {
            return Err(LoadAccessFault(addr));
        }
        match addr {
            CLINT_MTIMECMP => Ok(self.mtimecmp = value),
            CLINT_MTIME => Ok(self.mtime = value),
            _ => Err(StoreOrAMOAccessFault(addr)),
        }
    }

}