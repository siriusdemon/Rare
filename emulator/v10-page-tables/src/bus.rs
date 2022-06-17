use crate::param::*;
use crate::dram::Dram;
use crate::plic::Plic;
use crate::clint::Clint;
use crate::uart::Uart;
use crate::virtio::Virtio;
use crate::exception::RvException;

pub struct Bus {
    dram: Dram,
    plic: Plic,
    clint: Clint,
    pub uart: Uart,
    pub virtio: Virtio,
}


// Bus is used to transfer data, so check data access size here is appropriate
impl Bus {
    pub fn new(code: Vec<u8>, disk_image: Vec<u8>) -> Bus {
        Self { 
            dram: Dram::new(code),
            clint: Clint::new(),
            plic: Plic::new(),
            uart: Uart::new(),
            virtio: Virtio::new(disk_image),
        }
    }
    pub fn load(&mut self, addr: u64, size: u64) -> Result<u64, RvException> {
        match addr {
            CLINT_BASE..=CLINT_END => self.clint.load(addr, size),
            PLIC_BASE..=PLIC_END => self.plic.load(addr, size),
            DRAM_BASE..=DRAM_END => self.dram.load(addr, size),
            UART_BASE..=UART_END => self.uart.load(addr, size),
            VIRTIO_BASE..=VIRTIO_END => self.virtio.load(addr, size),
            _ => Err(RvException::LoadAccessFault(addr)),
        }
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), RvException> {
        match addr {
            CLINT_BASE..=CLINT_END => self.clint.store(addr, size, value),
            PLIC_BASE..=PLIC_END => self.plic.store(addr, size, value),
            DRAM_BASE..=DRAM_END => self.dram.store(addr, size, value),
            UART_BASE..=UART_END => self.uart.store(addr, size, value),
            VIRTIO_BASE..=VIRTIO_END => self.virtio.store(addr, size, value),
            _ => Err(RvException::StoreOrAMOAccessFault(addr)),
        }
    }
}