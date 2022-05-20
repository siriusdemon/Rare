use crate::bus::Bus;
use crate::{DRAM_SIZE, DRAM_BASE};
use crate::exception::RvException::{self, InvalidInstruction};


// Machine-level CSRs.
/// Hardware thread ID.
pub const MHARTID: usize = 0xf14;
/// Machine status register.
pub const MSTATUS: usize = 0x300;
/// Machine exception delefation register.
pub const MEDELEG: usize = 0x302;
/// Machine interrupt delefation register.
pub const MIDELEG: usize = 0x303;
/// Machine interrupt-enable register.
pub const MIE: usize = 0x304;
/// Machine trap-handler base address.
pub const MTVEC: usize = 0x305;
/// Machine counter enable.
pub const MCOUNTEREN: usize = 0x306;
/// Scratch register for machine trap handlers.
pub const MSCRATCH: usize = 0x340;
/// Machine exception program counter.
pub const MEPC: usize = 0x341;
/// Machine trap cause.
pub const MCAUSE: usize = 0x342;
/// Machine bad address or instruction.
pub const MTVAL: usize = 0x343;
/// Machine interrupt pending.
pub const MIP: usize = 0x344;

// Supervisor-level CSRs.
/// Supervisor status register.
pub const SSTATUS: usize = 0x100;
/// Supervisor interrupt-enable register.
pub const SIE: usize = 0x104;
/// Supervisor trap handler base address.
pub const STVEC: usize = 0x105;
/// Scratch register for supervisor trap handlers.
pub const SSCRATCH: usize = 0x140;
/// Supervisor exception program counter.
pub const SEPC: usize = 0x141;
/// Supervisor trap cause.
pub const SCAUSE: usize = 0x142;
/// Supervisor bad address or instruction.
pub const STVAL: usize = 0x143;
/// Supervisor interrupt pending.
pub const SIP: usize = 0x144;
/// Supervisor address translation and protection.
pub const SATP: usize = 0x180;


pub struct Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    pub bus: Bus,
    pub csrs: [u64; 4096],
}


impl Cpu {
    pub fn new(code: Vec<u8>) -> Self {
        let mut regs = [0; 32];
        regs[2] = DRAM_SIZE + DRAM_BASE;

        let bus = Bus::new(code);
        let csrs = [0; 4096];

        Self {regs, pc: DRAM_BASE, bus, csrs}
    }

    pub fn load(&self, addr: u64, size: u64) -> Result<u64, RvException>{
        self.bus.load(addr, size)
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), RvException> {
        self.bus.store(addr, size, value)
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

    pub fn dump_csrs(&self) {
        let output = format!(
            "{}\n{}",
            format!(
                "mstatus={:<#18x} mtvec={:<#18x} mepc={:<#18x} mcause={:<#18x}",
                self.load_csr(MSTATUS),
                self.load_csr(MTVEC),
                self.load_csr(MEPC),
                self.load_csr(MCAUSE),
            ),
            format!(
                "sstatus={:<#18x} stvec={:<#18x} sepc={:<#18x} scause={:<#18x}",
                self.load_csr(SSTATUS),
                self.load_csr(STVEC),
                self.load_csr(SEPC),
                self.load_csr(SCAUSE),
            ),
        );
        println!("{}", output);
    }

    pub fn load_csr(&self, addr: usize) -> u64 {
        match addr {
            SIE => self.csrs[MIE] & self.csrs[MIDELEG],
            _ => self.csrs[addr],
        }
    }

    /// Store a value to a CSR.
    pub fn store_csr(&mut self, addr: usize, value: u64) {
        match addr {
            SIE => {
                self.csrs[MIE] =
                    (self.csrs[MIE] & !self.csrs[MIDELEG]) | (value & self.csrs[MIDELEG]);
            }
            _ => self.csrs[addr] = value,
        }
    }


    pub fn fetch(&self) -> Result<u64, RvException> {
        self.bus.load(self.pc, 32)
    }

    pub fn execute(&mut self, inst: u64) -> Result<(), RvException> {
        let opcode = inst & 0x7f;
        let rd = ((inst >> 7) & 0x1f) as usize;
        let rs1 = ((inst >> 15) & 0x1f) as usize;
        let rs2 = ((inst >> 20) & 0x1f) as usize;
        let funct3 = (inst >> 12) & 0x7;
        let funct7 = (inst >> 25) & 0x7f;

        // x0 is hardwired zero
        self.regs[0] = 0;

        match opcode {
            0x03 => {
                let imm = ((inst as i32 as i64) >> 20) as u64;
                let addr = self.regs[rs1].wrapping_add(imm);
                match funct3 {
                    0x0 => {
                        let val = self.load(addr, 8)?;
                        self.regs[rd] = val as i8 as i64 as u64;
                        return Ok(());
                    }
                    0x1 => {
                        let val = self.load(addr, 16)?;
                        self.regs[rd] = val as i16 as i64 as u64;
                        return Ok(());
                    }
                    0x2 => {
                        let val = self.load(addr, 32)?;
                        self.regs[rd] = val as i32 as i64 as u64;
                        return Ok(());
                    }
                    0x3 => {
                        let val = self.load(addr, 64)?;
                        self.regs[rd] = val;
                        return Ok(());
                    }
                    0x4 => {
                        let val = self.load(addr, 8)?;
                        self.regs[rd] = val;
                        return Ok(());
                    }
                    0x5 => {
                        let val = self.load(addr, 16)?;
                        self.regs[rd] = val;
                        return Ok(());
                    }
                    0x6 => {
                        let val = self.load(addr, 32)?;
                        self.regs[rd] = val;
                        return Ok(());
                    }
                    _ => Err(InvalidInstruction(inst)),
                    
                }
            }
        
            0x13 => {
                // imm[11:0] = inst[31:20]
                let imm = ((inst & 0xfff00000) as i32 as i64 >> 20) as u64;
                // "The shift amount is encoded in the lower 6 bits of the I-immediate field for RV64I."
                let shamt = (imm & 0x3f) as u32;
                match funct3 {
                    0x0 => {
                        // addi
                        self.regs[rd] = self.regs[rs1].wrapping_add(imm);
                        return Ok(());
                    }
                    0x1 => {
                        // slli
                        self.regs[rd] = self.regs[rs1] << shamt;
                        return Ok(());
                    }
                    0x2 => {
                        // slti
                        self.regs[rd] = if (self.regs[rs1] as i64) < (imm as i64) { 1 } else { 0 };
                        return Ok(());
                    }
                    0x3 => {
                        // sltiu
                        self.regs[rd] = if self.regs[rs1] < imm { 1 } else { 0 };
                        return Ok(());
                    }
                    0x4 => {
                        // xori
                        self.regs[rd] = self.regs[rs1] ^ imm;
                        return Ok(());
                    }
                    0x5 => match funct7 >> 1 {
                        // srli
                        0x00 => {
                            self.regs[rd] = self.regs[rs1].wrapping_shr(shamt);
                            return Ok(());
                        }
                        // srai
                        0x10 => {
                            self.regs[rd] = (self.regs[rs1] as i64).wrapping_shr(shamt) as u64;
                            return Ok(());
                        }
                        _ => Err(InvalidInstruction(inst)),
                    }
                    0x6 => {
                        self.regs[rd] = self.regs[rs1] | imm; // ori
                        return Ok(());
                    }
                    0x7 => {
                        self.regs[rd] = self.regs[rs1] & imm; // andi
                        return Ok(());
                    }
                    _ => Err(InvalidInstruction(inst)),
                }
            }
            0x17 => {
                // auipc
                let imm = (inst & 0xfffff000) as i32 as i64 as u64;
                self.regs[rd] = self.pc.wrapping_add(imm).wrapping_sub(4);
                return Ok(());
            }
            0x1b => {
                let imm = ((inst as i32 as i64) >> 20) as u64;
                // "SLLIW, SRLIW, and SRAIW encodings with imm[5] ̸= 0 are reserved."
                let shamt = (imm & 0x1f) as u32;
                match funct3 {
                    0x0 => {
                        // addiw
                        self.regs[rd] = self.regs[rs1].wrapping_add(imm) as i32 as i64 as u64;
                        return Ok(());
                    }
                    0x1 => {
                        // slliw
                        self.regs[rd] = self.regs[rs1].wrapping_shl(shamt) as i32 as i64 as u64;
                        return Ok(());
                    }
                    0x5 => {
                        match funct7 {
                            0x00 => {
                                // srliw
                                self.regs[rd] = (self.regs[rs1] as u32).wrapping_shr(shamt) as i32 as i64 as u64;
                                return Ok(());
                            }
                            0x20 => {
                                // sraiw
                                self.regs[rd] = (self.regs[rs1] as i32).wrapping_shr(shamt) as i64 as u64;
                                return Ok(());
                            }
                            _ => Err(InvalidInstruction(inst)),
                        }
                    }
                    _ => Err(InvalidInstruction(inst)),
                }
            }
            0x23 => {
                let imm = ((inst & 0xfe00_0000) as i32 as i64 >> 20) as u64 | ((inst >> 7) & 0x1f) as u64;
                let addr = self.regs[rs1].wrapping_add(imm);
                match funct3 {
                    0x0 => self.store(addr, 8, self.regs[rs2]),
                    0x1 => self.store(addr, 16, self.regs[rs2]),
                    0x2 => self.store(addr, 32, self.regs[rs2]),
                    0x3 => self.store(addr, 64, self.regs[rs2]),
                    _ => Err(InvalidInstruction(inst)),
                }
            }
            0x33 => {
                // "SLL, SRL, and SRA perform logical left, logical right, and arithmetic right
                // shifts on the value in register rs1 by the shift amount held in register rs2.
                // In RV64I, only the low 6 bits of rs2 are considered for the shift amount."
                let shamt = ((self.regs[rs2] & 0x3f) as u64) as u32;
                match (funct3, funct7) {
                    (0x0, 0x00) => {
                        // add
                        self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]);
                        return Ok(());
                    }
                    (0x0, 0x01) => {
                        // mul
                        self.regs[rd] = self.regs[rs1].wrapping_mul(self.regs[rs2]);
                        return Ok(());
                    }
                    (0x0, 0x20) => {
                        // sub
                        self.regs[rd] = self.regs[rs1].wrapping_sub(self.regs[rs2]);
                        return Ok(());
                    }
                    (0x1, 0x00) => {
                        // sll
                        self.regs[rd] = self.regs[rs1].wrapping_shl(shamt);
                        return Ok(());
                    }
                    (0x2, 0x00) => {
                        // slt
                        self.regs[rd] = if (self.regs[rs1] as i64) < (self.regs[rs2] as i64) { 1 } else { 0 };
                        return Ok(());
                    }
                    (0x3, 0x00) => {
                        // sltu
                        self.regs[rd] = if self.regs[rs1] < self.regs[rs2] { 1 } else { 0 };
                        return Ok(());
                    }
                    (0x4, 0x00) => {
                        // xor
                        self.regs[rd] = self.regs[rs1] ^ self.regs[rs2];
                        return Ok(());
                    }
                    (0x5, 0x00) => {
                        // srl
                        self.regs[rd] = self.regs[rs1].wrapping_shr(shamt);
                        return Ok(());
                    }
                    (0x5, 0x20) => {
                        // sra
                        self.regs[rd] = (self.regs[rs1] as i64).wrapping_shr(shamt) as u64;
                        return Ok(());
                    }
                    (0x6, 0x00) => {
                        // or
                        self.regs[rd] = self.regs[rs1] | self.regs[rs2];
                        return Ok(());
                    }
                    (0x7, 0x00) => {
                        // and
                        self.regs[rd] = self.regs[rs1] & self.regs[rs2];
                        return Ok(());
                    }
                    _ => Err(InvalidInstruction(inst)),
                }
            }
            0x37 => {
                // lui
                self.regs[rd] = (inst & 0xfffff000) as i32 as i64 as u64;
                return Ok(());
            }
            0x3b => {
                // "The shift amount is given by rs2[4:0]."
                let shamt = (self.regs[rs2] & 0x1f) as u32;
                match (funct3, funct7) {
                    (0x0, 0x00) => {
                        // addw
                        self.regs[rd] = self.regs[rs1].wrapping_add(self.regs[rs2]) as i32 as i64 as u64;
                        return Ok(());
                    }
                    (0x0, 0x20) => {
                        // subw
                        self.regs[rd] = ((self.regs[rs1].wrapping_sub(self.regs[rs2])) as i32) as u64;
                        return Ok(());
                    }
                    (0x1, 0x00) => {
                        // sllw
                        self.regs[rd] = (self.regs[rs1] as u32).wrapping_shl(shamt) as i32 as u64;
                        return Ok(());
                    }
                    (0x5, 0x00) => {
                        // srlw
                        self.regs[rd] = (self.regs[rs1] as u32).wrapping_shr(shamt) as i32 as u64;
                        return Ok(());
                    }
                    (0x5, 0x20) => {
                        // sraw
                        self.regs[rd] = ((self.regs[rs1] as i32) >> (shamt as i32)) as u64;
                        return Ok(());
                    }
                    _ => Err(InvalidInstruction(inst)), 
                }
            }
            0x63 => {
                // imm[12|10:5|4:1|11] = inst[31|30:25|11:8|7]
                let imm = (((inst & 0x80000000) as i32 as i64 >> 19) as u64)
                    | ((inst & 0x80) << 4) as u64// imm[11]
                    | ((inst >> 20) & 0x7e0) as u64// imm[10:5]
                    | ((inst >> 7) & 0x1e) as u64; // imm[4:1]

                match funct3 {
                    0x0 => {
                        // beq
                        if self.regs[rs1] == self.regs[rs2] {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                        return Ok(());
                    }
                    0x1 => {
                        // bne
                        if self.regs[rs1] != self.regs[rs2] {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                        return Ok(());
                    }
                    0x4 => {
                        // blt
                        if (self.regs[rs1] as i64) < (self.regs[rs2] as i64) {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                        return Ok(());
                    }
                    0x5 => {
                        // bge
                        if (self.regs[rs1] as i64) >= (self.regs[rs2] as i64) {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                        return Ok(());
                    }
                    0x6 => {
                        // bltu
                        if self.regs[rs1] < self.regs[rs2] {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                        return Ok(());
                    }
                    0x7 => {
                        // bgeu
                        if self.regs[rs1] >= self.regs[rs2] {
                            self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                        }
                        return Ok(());
                    }
                    _ => Err(InvalidInstruction(inst)),
                }
            }
            0x67 => {
                // jalr
                // Note: Don't add 4 because the pc already moved on.
                let t = self.pc;
                let imm = ((((inst & 0xfff00000) as i32) as i64) >> 20) as u64;
                self.pc = (self.regs[rs1].wrapping_add(imm)) & !1;
                self.regs[rd] = t;
                return Ok(());
            }
            0x6f => {
                // jal
                self.regs[rd] = self.pc;
                // imm[20|10:1|11|19:12] = inst[31|30:21|20|19:12]
                let imm = (((inst & 0x80000000) as i32 as i64 >> 11) as u64) // imm[20]
                    | (inst & 0xff000)  as u64// imm[19:12]
                    | ((inst >> 9) & 0x800) as u64// imm[11]
                    | ((inst >> 20) & 0x7fe) as u64; // imm[10:1]
                self.pc = self.pc.wrapping_add(imm).wrapping_sub(4);
                return Ok(());
            }
            0x73 => {
                let csr_addr = ((inst & 0xfff00000) >> 20) as usize;
                match funct3 {
                    0x1 => {
                        // csrrw
                        let t = self.load_csr(csr_addr);
                        self.store_csr(csr_addr, self.regs[rs1]);
                        self.regs[rd] = t;
                        return Ok(());
                    }
                    0x2 => {
                        // csrrs
                        let t = self.load_csr(csr_addr);
                        self.store_csr(csr_addr, t | self.regs[rs1]);
                        self.regs[rd] = t;
                        return Ok(());
                    }
                    0x3 => {
                        // csrrc
                        let t = self.load_csr(csr_addr);
                        self.store_csr(csr_addr, t & (!self.regs[rs1]));
                        self.regs[rd] = t;
                        return Ok(());
                    }
                    0x5 => {
                        // csrrwi
                        let zimm = rs1 as u64;
                        self.regs[rd] = self.load_csr(csr_addr);
                        self.store_csr(csr_addr, zimm);
                        return Ok(());
                    }
                    0x6 => {
                        // csrrsi
                        let zimm = rs1 as u64;
                        let t = self.load_csr(csr_addr);
                        self.store_csr(csr_addr, t | zimm);
                        self.regs[rd] = t;
                        return Ok(());
                    }
                    0x7 => {
                        // csrrci
                        let zimm = rs1 as u64;
                        let t = self.load_csr(csr_addr);
                        self.store_csr(csr_addr, t & (!zimm));
                        self.regs[rd] = t;
                        return Ok(());
                    }
                    _ => Err(InvalidInstruction(inst)),
                }
            }
            _ => Err(InvalidInstruction(inst)),
        }
    }
}


#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{Write, Read};
    use std::process::Command;
    use crate::param::{RV_GCC, RV_OBJCOPY};
    use super::*;

    fn generate_rv_assembly(c_src: &str) {
        Command::new(RV_GCC).arg("-S")
                            .arg(c_src)
                            .output()
                            .expect("Failed to generate rv assembly");
    }

    fn generate_rv_obj(assembly: &str) {
        let pieces: Vec<&str> = assembly.split(".").collect();
        Command::new(RV_GCC).arg("-Wl,-Ttext=0x0")
                            .arg("-nostdlib")
                            .arg("-march=rv64i")
                            .arg("-mabi=lp64")
                            .arg("-o")
                            .arg(&pieces[0])
                            .arg(assembly)
                            .output()
                            .expect("Failed to generate rv object");
    }

    fn generate_rv_binary(obj: &str) {
        Command::new(RV_OBJCOPY).arg("-O")
                                .arg("binary")
                                .arg(obj)
                                .arg(obj.to_owned() + ".bin")
                                .output()
                                .expect("Failed to generate rv binary");
    }

    fn rv_helper(code: &str, testname: &str, n_clock: usize) -> Result<Cpu, std::io::Error> {
        let filename = testname.to_owned() + ".s";
        let mut file = File::create(&filename)?;
        file.write(&code.as_bytes());
        generate_rv_obj(&filename);
        generate_rv_binary(testname);
        let mut file_bin = File::open(testname.to_owned() + ".bin")?;
        let mut code = Vec::new();
        file_bin.read_to_end(&mut code)?;
        let mut cpu = Cpu::new(code);

        for _i in 0..n_clock {
            let inst = match cpu.fetch() {
                Ok(inst) => inst,
                Err(err) => break,
            };
            cpu.pc += 4;
            match cpu.execute(inst) {
                Ok(_) => (),
                Err(err) => println!("{}", err),
            };
        }

        return Ok(cpu);
    }

    #[test]
    fn test_sp()  {
        let code = "
            addi sp, sp, -16 
            sd ra, 8(sp)
        ";
        match rv_helper(code, "test_sp", 2) {
            Ok(cpu) => assert_eq!(cpu.regs[2], DRAM_BASE + DRAM_SIZE - 16),
            Err(e) => { println!("error: {}", e); assert!(false); }
        }
    }

    #[test]
    fn test_addi() {
        let code = "addi x31, x0, 42";
        match rv_helper(code, "test_addi", 2) {
            Ok(cpu) => assert_eq!(cpu.regs[31], 42),
            Err(e) => { println!("error: {}", e); assert!(false); }
        }
    }

    #[test]
    fn test_simple() {
        let code = "
        	addi	sp,sp,-16
            sd	s0,8(sp)
            addi	s0,sp,16
            li	a5,42
            mv	a0,a5
            ld	s0,8(sp)
            addi	sp,sp,16
            jr	ra
        ";
        match rv_helper(code, "test_simple", 20) {
            Ok(cpu) => assert_eq!(cpu.regs[10], 42),
            Err(e) => { println!("error: {}", e); assert!(false); }
        }
    }
}