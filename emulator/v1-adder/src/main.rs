use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;


pub const DRAM_SIZE: u64 = 1024 * 1024 * 128;


struct Cpu {
    regs: [u64; 32],
    pc: u64,
    dram: Vec<u8>,
}


impl Cpu {
    fn new(code: Vec<u8>) -> Self {
        let mut regs = [0; 32];
        regs[2] = DRAM_SIZE - 1;
        Self {regs, pc: 0, dram: code}
    }

    pub fn dump_registers(&self) {
        let mut output = String::new();
        let abi = [
            "zero", " ra ", " sp ", " gp ", " tp ", " t0 ", " t1 ", " t2 ", 
            " s0 ", " s1 ", " a0 ", " a1 ", " a2 ", " a3 ", " a4 ", " a5 ", 
            " a6 ", " a7 ", " s2 ", " s3 ", " s4 ", " s5 ", " s6 ", " s7 ", 
            " s8 ", " s9 ", " s10", " s11", " t3 ", " t4 ", " t5 ", " t6 ",
        ];
        

        for i in (0..32).step_by(4) {
            let i0 = format!("x{}", i);
            let i1 = format!("x{}", i + 1); 
            let i2 = format!("x{}", i + 2);
            let i3 = format!("x{}", i + 3); 
            let line = format!(
                "{:3}({}) = {:<#18x} {:3}({}) = {:<#18x} {:3}({}) = {:<#18x} {:3}({}) = {:<#18x}\n",
                i0, abi[i], self.regs[i], 
                i1, abi[i + 1], self.regs[i + 1], 
                i2, abi[i + 2], self.regs[i + 2], 
                i3, abi[i + 3], self.regs[i + 3],
            );
            output = output + &line;
        }

        println!("{}", output);
    }

    fn fetch(&self) -> u32 {
        let index = self.pc as usize;
        let inst = self.dram[index] as u32 
                | ((self.dram[index + 1] as u32) << 8)
                | ((self.dram[index + 2] as u32) << 16)
                | ((self.dram[index + 3] as u32) << 24);
        return inst;
    }

    fn execute(&mut self, inst: u32) {
        let opcode = inst & 0x7f;
        let rd = ((inst >> 7) & 0x1f) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;

        // x0 is hardwired zero
        self.regs[0] = 0;

        match opcode {
            0x13 => {
                // addi
                let imm = ((inst & 0xfff0_0000) as i64 >> 20) as u64;
                self.regs[rd] = self.regs[rs1].wrapping_add(imm);
            }
            0x33 => {
                self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]);
            }

            _ => {
                dbg!(format!("Invalid opcode: {:#x}", opcode)); 
            }
        }
    }
}




fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!(
            "Usage:\n\
            - cargo run <filename>"
        );
        return Ok(());
    }

    let mut file = File::open(&args[1])?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;

    let mut cpu = Cpu::new(code);


    while cpu.pc < cpu.dram.len() as u64 {
        let inst = cpu.fetch();
        cpu.execute(inst);
        cpu.pc += 4;
    }
    cpu.dump_registers();

    Ok(())
}
