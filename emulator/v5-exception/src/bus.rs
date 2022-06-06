use crate::{DRAM_BASE, DRAM_END};
use crate::dram::Dram;
use crate::exception::RvException;

pub struct Bus {
    dram: Dram,
}

// Bus is used to transfer data, so check data access size here is appropriate
impl Bus {
    pub fn new(code: Vec<u8>) -> Bus {
        Self { dram: Dram::new(code) }
    }
    pub fn load(&self, addr: u64, size: u64) -> Result<u64, RvException> {
        match addr {
            DRAM_BASE..=DRAM_END => self.dram.load(addr, size),
            _ => Err(RvException::LoadAccessFault(addr)),
        }
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), RvException> {
        match addr {
            DRAM_BASE..=DRAM_END => self.dram.store(addr, size, value),
            _ => Err(RvException::StoreOrAMOAccessFault(addr)),
        }
    }
}