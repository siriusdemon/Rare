use crate::param::*;
use crate::dram::Dram;
use crate::plic::Plic;
use crate::clint::Clint;
use crate::uart::Uart;
use crate::exception::RvException;

pub struct Bus {
    pub dram: Dram,
    pub plic: Plic,
    pub clint: Clint,
    pub uart: Uart,
}


// Bus is used to transfer data, so check data access size here is appropriate
impl Bus {
    pub fn new(code: Vec<u8>) -> Bus {
        Self { 
            dram: Dram::new(code),
            clint: Clint::new(),
            plic: Plic::new(),
            uart: Uart::new(),
        }
    }
    pub fn load(&mut self, addr: u64, size: u64) -> Result<u64, RvException> {
        match addr {
            CLINT_BASE..=CLINT_END => self.clint.load(addr, size),
            PLIC_BASE..=PLIC_END => self.plic.load(addr, size),
            DRAM_BASE..=DRAM_END => self.dram.load(addr, size),
            UART_BASE..=UART_END => self.uart.load(addr, size),
            _ => Err(RvException::LoadAccessFault(addr)),
        }
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), RvException> {
        match addr {
            CLINT_BASE..=CLINT_END => self.clint.store(addr, size, value),
            PLIC_BASE..=PLIC_END => self.plic.store(addr, size, value),
            DRAM_BASE..=DRAM_END => self.dram.store(addr, size, value),
            UART_BASE..=UART_END => self.uart.store(addr, size, value),
            _ => Err(RvException::StoreOrAMOAccessFault(addr)),
        }
    }
}