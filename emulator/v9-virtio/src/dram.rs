use crate::{DRAM_SIZE, DRAM_BASE, DRAM_END};
use crate::exception::RvException;


pub struct Dram {
    pub dram: Vec<u8>,
}

impl Dram {
    pub fn new(code: Vec<u8>) -> Dram {
        let mut dram = vec![0; DRAM_SIZE as usize];
        dram.splice(..code.len(), code.into_iter());
        Self { dram }
    }

    // addr/size must be valid. Check in bus
    pub fn load(&self, addr: u64, size: u64) -> Result<u64, RvException> {
        let nbytes = size / 8;
        if (nbytes + addr - 1) > DRAM_END {
            return Err(RvException::LoadAccessFault(addr));
        }

        let index = (addr - DRAM_BASE) as usize;
        let mut code = self.dram[index] as u64;
        for i in 1..nbytes {
            code |= (self.dram[index + i as usize] as u64) << (i * 8);
        }

        return Ok(code);
    }

    // addr/size must be valid. Check in bus
    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), RvException> {
        let nbytes = size / 8;
        if (nbytes + addr - 1) > DRAM_END {
            return Err(RvException::StoreOrAMOAccessFault(addr));
        }

        let index = (addr - DRAM_BASE) as usize;
        for i in 0..nbytes {
            let offset = 8 * i as usize;
            self.dram[index + i as usize] = ((value >> offset) & 0xff) as u8;
        }
        return Ok(())
    }
}