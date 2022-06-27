//! The bus module contains the system bus which can access the memroy or memory-mapped peripheral
//! devices.
use crate::param::*;
use crate::dram::Dram;
use crate::exception::*;

pub struct Bus {
    dram: Dram,
}


// Bus is used to transfer data, so check data access size here is appropriate
impl Bus {
    pub fn new(code: Vec<u8>, disk_image: Vec<u8>) -> Bus {
        Self { 
            dram: Dram::new(code),
        }
    }
    pub fn load(&mut self, addr: u64, size: u64) -> Result<u64, Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => self.dram.load(addr, size),
            _ => Err(Exception::LoadAccessFault(addr)),
        }
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), Exception> {
        match addr {
            DRAM_BASE..=DRAM_END => self.dram.store(addr, size, value),
            _ => Err(Exception::StoreAMOAccessFault(addr)),
        }
    }
}