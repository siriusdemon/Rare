//! The cpu module contains `Cpu` and implementarion for it.

#![allow(dead_code)]

use crate::bus::*;
use crate::exception::*;
use crate::param::*;
use crate::csr::*;


/// The `Cpu` struct that contains registers, a program coutner, system bus that connects
/// peripheral devices, and control and status registers.
pub struct Cpu {
    /// 32 64-bit integer registers.
    pub regs: [u64; 32],
    /// Program counter to hold the the dram address of the next instruction that would be executed.
    pub pc: u64,
    /// System bus that transfers data between CPU and peripheral devices.
    pub bus: Bus,
    /// Control and status registers. RISC-V ISA sets aside a 12-bit encoding space (csr[11:0]) for
    /// up to 4096 CSRs.
    pub csr: Csr,
}

const RVABI: [&str; 32] = [
    "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", 
    "s0", "s1", "a0", "a1", "a2", "a3", "a4", "a5", 
    "a6", "a7", "s2", "s3", "s4", "s5", "s6", "s7", 
    "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
];
 
impl Cpu {
    /// Create a new `Cpu` object.
    pub fn new(code: Vec<u8>) -> Self {
        let mut regs = [0; 32];
        regs[2] = DRAM_END;
        let pc = DRAM_BASE;
        let bus = Bus::new(code);
        let csr = Csr::new();

        Self {regs, pc, bus, csr}
    }

    pub fn reg(&self, r: &str) -> u64 {
        match RVABI.iter().position(|&x| x == r) {
            Some(i) => self.regs[i],
            None => match r {
                "pc" => self.pc,
                "fp" => self.reg("s0"),
                r if r.starts_with("x") => {
                    if let Ok(i) = r[1..].parse::<usize>() {
                        if i <= 31 { return self.regs[i]; }
                        panic!("Invalid register {}", r);
                    }
                    panic!("Invalid register {}", r);
                }
                "mhartid" => self.csr.load(MHARTID),
                "mstatus" => self.csr.load(MSTATUS),
                "mtvec" => self.csr.load(MTVEC),
                "mepc" => self.csr.load(MEPC),
                "mcause" => self.csr.load(MCAUSE),
                "mtval" => self.csr.load(MTVAL),
                "medeleg" => self.csr.load(MEDELEG),
                "mscratch" => self.csr.load(MSCRATCH),
                "MIP" => self.csr.load(MIP),
                "mcounteren" => self.csr.load(MCOUNTEREN),
                "sstatus" => self.csr.load(SSTATUS),
                "stvec" => self.csr.load(STVEC),
                "sepc" => self.csr.load(SEPC),
                "scause" => self.csr.load(SCAUSE),
                "stval" => self.csr.load(STVAL),
                "sscratch" => self.csr.load(SSCRATCH),
                "SIP" => self.csr.load(SIP),
                "SATP" => self.csr.load(SATP),
                _ => panic!("Invalid register {}", r),
            }
        }
    }

    pub fn dump_pc(&self) {
        println!("{:-^80}", "PC register");
        println!("PC = {:#x}\n", self.pc);
    }

    pub fn dump_registers(&mut self) {
        println!("{:-^80}", "registers");
        let mut output = String::new();
        self.regs[0] = 0;

        for i in (0..32).step_by(4) {
            let i0 = format!("x{}", i);
            let i1 = format!("x{}", i + 1); 
            let i2 = format!("x{}", i + 2);
            let i3 = format!("x{}", i + 3); 
            let line = format!(
                "{:3}({:^4}) = {:<#18x} {:3}({:^4}) = {:<#18x} {:3}({:^4}) = {:<#18x} {:3}({:^4}) = {:<#18x}\n",
                i0, RVABI[i], self.regs[i], 
                i1, RVABI[i + 1], self.regs[i + 1], 
                i2, RVABI[i + 2], self.regs[i + 2], 
                i3, RVABI[i + 3], self.regs[i + 3],
            );
            output = output + &line;
        }

        println!("{}", output);
    }

    /// Print values in some csrs.
    pub fn dump_csrs(&self) {
        self.csr.dump_csrs();
    }

    /// Load a value from a dram.
    pub fn load(&mut self, addr: u64, size: u64) -> Result<u64, Exception> {
        self.bus.load(addr, size)
    }

    /// Store a value to a dram.
    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), Exception> {
        self.bus.store(addr, size, value)
    }

    /// Get an instruction from the dram.
    pub fn fetch(&mut self) -> Result<u64, Exception> {
        self.bus.load(self.pc, 32)
    }


    #[inline]
    pub fn update_pc(&mut self) -> Result<u64, Exception> {
        return Ok(self.pc + 4);
    }

    /// Execute an instruction after decoding. Return true if an error happens, otherwise false.
    pub fn execute(&mut self, inst: u64) -> Result<u64, Exception> {
        let opcode = inst & 0x0000007f;
        let rd = ((inst & 0x00000f80) >> 7) as usize;
        let rs1 = ((inst & 0x000f8000) >> 15) as usize;
        let rs2 = ((inst & 0x01f00000) >> 20) as usize;
        let funct3 = (inst & 0x00007000) >> 12;
        let funct7 = (inst & 0xfe000000) >> 25;

        // Emulate that register x0 is hardwired with all bits equal to 0.
        self.regs[0] = 0;

        match opcode {
            0x03 => {
                // imm[11:0] = inst[31:20]
                let imm = ((inst as i32 as i64) >> 20) as u64;
                let addr = self.regs[rs1].wrapping_add(imm);
                match funct3 {
                    0x0 => {
                        // lb
                        let val = self.load(addr, 8)?;
                        self.regs[rd] = val as i8 as i64 as u64;
                        return self.update_pc();
                    }
                    0x1 => {
                        // lh
                        let val = self.load(addr, 16)?;
                        self.regs[rd] = val as i16 as i64 as u64;
                        return self.update_pc();
                    }
                    0x2 => {
                        // lw
                        let val = self.load(addr, 32)?;
                        self.regs[rd] = val as i32 as i64 as u64;
                        return self.update_pc();
                    }
                    0x3 => {
                        // ld
                        let val = self.load(addr, 64)?;
                        self.regs[rd] = val;
                        return self.update_pc();
                    }
                    0x4 => {
                        // lbu
                        let val = self.load(addr, 8)?;
                        self.regs[rd] = val;
                        return self.update_pc();
                    }
                    0x5 => {
                        // lhu
                        let val = self.load(addr, 16)?;
                        self.regs[rd] = val;
                        return self.update_pc();
                    }
                    0x6 => {
                        // lwu
                        let val = self.load(addr, 32)?;
                        self.regs[rd] = val;
                        return self.update_pc();
                    }
                    _ => Err(Exception::IllegalInstruction(inst)),
                    
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
                        return self.update_pc();
                    }
                    0x1 => {
                        // slli
                        self.regs[rd] = self.regs[rs1] << shamt;
                        return self.update_pc();
                    }
                    0x2 => {
                        // slti
                        self.regs[rd] = if (self.regs[rs1] as i64) < (imm as i64) { 1 } else { 0 };
                        return self.update_pc();
                    }
                    0x3 => {
                        // sltiu
                        self.regs[rd] = if self.regs[rs1] < imm { 1 } else { 0 };
                        return self.update_pc();
                    }
                    0x4 => {
                        // xori
                        self.regs[rd] = self.regs[rs1] ^ imm;
                        return self.update_pc();
                    }
                    0x5 => {
                        match funct7 >> 1 {
                            // srli
                            0x00 => {
                                self.regs[rd] = self.regs[rs1].wrapping_shr(shamt);
                                return self.update_pc();
                            },
                            // srai
                            0x10 => {
                                self.regs[rd] = (self.regs[rs1] as i64).wrapping_shr(shamt) as u64;
                                return self.update_pc();
                            }
                            _ => Err(Exception::IllegalInstruction(inst)),
                        }
                    }
                    0x6 => {
                        self.regs[rd] = self.regs[rs1] | imm;
                        return self.update_pc();
                    }, // ori
                    0x7 => {
                        self.regs[rd] = self.regs[rs1] & imm; // andi
                        return self.update_pc();
                    }
                    _ => Err(Exception::IllegalInstruction(inst)),
                }
            }
            0x17 => {
                // auipc
                let imm = (inst & 0xfffff000) as i32 as i64 as u64;
                self.regs[rd] = self.pc.wrapping_add(imm);
                return self.update_pc();
            }
            0x1b => {
                let imm = ((inst as i32 as i64) >> 20) as u64;
                // "SLLIW, SRLIW, and SRAIW encodings with imm[5] ̸= 0 are reserved."
                let shamt = (imm & 0x1f) as u32;
                match funct3 {
                    0x0 => {
                        // addiw
                        self.regs[rd] = self.regs[rs1].wrapping_add(imm) as i32 as i64 as u64;
                        return self.update_pc();
                    }
                    0x1 => {
                        // slliw
                        self.regs[rd] = self.regs[rs1].wrapping_shl(shamt) as i32 as i64 as u64;
                        return self.update_pc();
                    }
                    0x5 => {
                        match funct7 {
                            0x00 => {
                                // srliw
                                self.regs[rd] = (self.regs[rs1] as u32).wrapping_shr(shamt) as i32
                                    as i64 as u64;
                                return self.update_pc();
                            }
                            0x20 => {
                                // sraiw
                                self.regs[rd] =
                                    (self.regs[rs1] as i32).wrapping_shr(shamt) as i64 as u64;
                                return self.update_pc();
                            }
                            _ => Err(Exception::IllegalInstruction(inst)),
                        }
                    }
                    _ => Err(Exception::IllegalInstruction(inst)),
                    
                }
            }
            0x23 => {
                // imm[11:5|4:0] = inst[31:25|11:7]
                let imm = (((inst & 0xfe000000) as i32 as i64 >> 20) as u64) | ((inst >> 7) & 0x1f);
                let addr = self.regs[rs1].wrapping_add(imm);
                match funct3 {
                    0x0 => {self.store(addr, 8, self.regs[rs2])?;  self.update_pc()}, // sb
                    0x1 => {self.store(addr, 16, self.regs[rs2])?; self.update_pc()}, // sh
                    0x2 => {self.store(addr, 32, self.regs[rs2])?; self.update_pc()}, // sw
                    0x3 => {self.store(addr, 64, self.regs[rs2])?; self.update_pc()}, // sd
                    _ => unreachable!(),
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
                        return self.update_pc();
                    }
                    (0x0, 0x01) => {
                        // mul
                        self.regs[rd] = self.regs[rs1].wrapping_mul(self.regs[rs2]);
                        return self.update_pc();
                    }
                    (0x0, 0x20) => {
                        // sub
                        self.regs[rd] = self.regs[rs1].wrapping_sub(self.regs[rs2]);
                        return self.update_pc();
                    }
                    (0x1, 0x00) => {
                        // sll
                        self.regs[rd] = self.regs[rs1].wrapping_shl(shamt);
                        return self.update_pc();
                    }
                    (0x2, 0x00) => {
                        // slt
                        self.regs[rd] = if (self.regs[rs1] as i64) < (self.regs[rs2] as i64) { 1 } else { 0 };
                        return self.update_pc();
                    }
                    (0x3, 0x00) => {
                        // sltu
                        self.regs[rd] = if self.regs[rs1] < self.regs[rs2] { 1 } else { 0 };
                        return self.update_pc();
                    }
                    (0x4, 0x00) => {
                        // xor
                        self.regs[rd] = self.regs[rs1] ^ self.regs[rs2];
                        return self.update_pc();
                    }
                    (0x5, 0x00) => {
                        // srl
                        self.regs[rd] = self.regs[rs1].wrapping_shr(shamt);
                        return self.update_pc();
                    }
                    (0x5, 0x20) => {
                        // sra
                        self.regs[rd] = (self.regs[rs1] as i64).wrapping_shr(shamt) as u64;
                        return self.update_pc();
                    }
                    (0x6, 0x00) => {
                        // or
                        self.regs[rd] = self.regs[rs1] | self.regs[rs2];
                        return self.update_pc();
                    }
                    (0x7, 0x00) => {
                        // and
                        self.regs[rd] = self.regs[rs1] & self.regs[rs2];
                        return self.update_pc();
                    }
                    _ => Err(Exception::IllegalInstruction(inst)),
                }
            }
            0x37 => {
                // lui
                self.regs[rd] = (inst & 0xfffff000) as i32 as i64 as u64;
                return self.update_pc();
            }
            0x3b => {
                // "The shift amount is given by rs2[4:0]."
                let shamt = (self.regs[rs2] & 0x1f) as u32;
                match (funct3, funct7) {
                    (0x0, 0x00) => {
                        // addw
                        self.regs[rd] =
                            self.regs[rs1].wrapping_add(self.regs[rs2]) as i32 as i64 as u64;
                        return self.update_pc();
                    }
                    (0x0, 0x20) => {
                        // subw
                        self.regs[rd] =
                            ((self.regs[rs1].wrapping_sub(self.regs[rs2])) as i32) as u64;
                        return self.update_pc();
                    }
                    (0x1, 0x00) => {
                        // sllw
                        self.regs[rd] = (self.regs[rs1] as u32).wrapping_shl(shamt) as i32 as u64;
                        return self.update_pc();
                    }
                    (0x5, 0x00) => {
                        // srlw
                        self.regs[rd] = (self.regs[rs1] as u32).wrapping_shr(shamt) as i32 as u64;
                        return self.update_pc();
                    }
                    (0x5, 0x20) => {
                        // sraw
                        self.regs[rd] = ((self.regs[rs1] as i32) >> (shamt as i32)) as u64;
                        return self.update_pc();
                    }
                    _ => Err(Exception::IllegalInstruction(inst)),
                }
            }
            0x63 => {
                // imm[12|10:5|4:1|11] = inst[31|30:25|11:8|7]
                let imm = (((inst & 0x80000000) as i32 as i64 >> 19) as u64)
                    | ((inst & 0x80) << 4) // imm[11]
                    | ((inst >> 20) & 0x7e0) // imm[10:5]
                    | ((inst >> 7) & 0x1e); // imm[4:1]

                match funct3 {
                    0x0 => {
                        // beq
                        if self.regs[rs1] == self.regs[rs2] {
                            return Ok(self.pc.wrapping_add(imm));
                        }
                        return self.update_pc();
                    }
                    0x1 => {
                        // bne
                        if self.regs[rs1] != self.regs[rs2] {
                            return Ok(self.pc.wrapping_add(imm));
                        }
                        return self.update_pc();
                    }
                    0x4 => {
                        // blt
                        if (self.regs[rs1] as i64) < (self.regs[rs2] as i64) {
                            return Ok(self.pc.wrapping_add(imm));
                        }
                        return self.update_pc();
                    }
                    0x5 => {
                        // bge
                        if (self.regs[rs1] as i64) >= (self.regs[rs2] as i64) {
                            return Ok(self.pc.wrapping_add(imm));
                        }
                        return self.update_pc();
                    }
                    0x6 => {
                        // bltu
                        if self.regs[rs1] < self.regs[rs2] {
                            return Ok(self.pc.wrapping_add(imm));
                        }
                        return self.update_pc();
                    }
                    0x7 => {
                        // bgeu
                        if self.regs[rs1] >= self.regs[rs2] {
                            return Ok(self.pc.wrapping_add(imm));
                        }
                        return self.update_pc();
                    }
                    _ => Err(Exception::IllegalInstruction(inst)),
                    
                }
            }
            0x67 => {
                // jalr
                let t = self.pc + 4;

                let imm = ((((inst & 0xfff00000) as i32) as i64) >> 20) as u64;
                let new_pc = (self.regs[rs1].wrapping_add(imm)) & !1;

                self.regs[rd] = t;
                return Ok(new_pc);
            }
            0x6f => {
                // jal
                self.regs[rd] = self.pc + 4;

                // imm[20|10:1|11|19:12] = inst[31|30:21|20|19:12]
                let imm = (((inst & 0x80000000) as i32 as i64 >> 11) as u64) // imm[20]
                    | (inst & 0xff000) // imm[19:12]
                    | ((inst >> 9) & 0x800) // imm[11]
                    | ((inst >> 20) & 0x7fe); // imm[10:1]

                return Ok(self.pc.wrapping_add(imm));
            }
            0x73 => {
                let csr_addr = ((inst & 0xfff00000) >> 20) as usize;
                match funct3 {
                    0x1 => {
                        // csrrw
                        let t = self.csr.load(csr_addr);
                        self.csr.store(csr_addr, self.regs[rs1]);
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    0x2 => {
                        // csrrs
                        let t = self.csr.load(csr_addr);
                        self.csr.store(csr_addr, t | self.regs[rs1]);
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    0x3 => {
                        // csrrc
                        let t = self.csr.load(csr_addr);
                        self.csr.store(csr_addr, t & (!self.regs[rs1]));
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    0x5 => {
                        // csrrwi
                        let zimm = rs1 as u64;
                        self.regs[rd] = self.csr.load(csr_addr);
                        self.csr.store(csr_addr, zimm);
                        return self.update_pc();
                    }
                    0x6 => {
                        // csrrsi
                        let zimm = rs1 as u64;
                        let t = self.csr.load(csr_addr);
                        self.csr.store(csr_addr, t | zimm);
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    0x7 => {
                        // csrrci
                        let zimm = rs1 as u64;
                        let t = self.csr.load(csr_addr);
                        self.csr.store(csr_addr, t & (!zimm));
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    _ => Err(Exception::IllegalInstruction(inst)),
                }
            }
            _ => Err(Exception::IllegalInstruction(inst)),
        }
    }
}



#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{Write, Read};
    use std::process::Command;
    use super::*;

    fn generate_rv_assembly(c_src: &str) {
        let cc = "clang";
        let output = Command::new(cc).arg("-S")
                            .arg(c_src)
                            .arg("-nostdlib")
                            .arg("-march=rv64g")
                            .arg("-mabi=lp64")
                            .arg("--target=riscv64")
                            .arg("-mno-relax")
                            .output()
                            .expect("Failed to generate rv assembly");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    fn generate_rv_obj(assembly: &str) {
        let cc = "clang";
        let pieces: Vec<&str> = assembly.split(".").collect();
        let output = Command::new(cc).arg("-Wl,-Ttext=0x0")
                            .arg("-nostdlib")
                            .arg("-march=rv64g")
                            .arg("-mabi=lp64")
                            .arg("--target=riscv64")
                            .arg("-mno-relax")
                            .arg("-o")
                            .arg(&pieces[0])
                            .arg(assembly)
                            .output()
                            .expect("Failed to generate rv object");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    fn generate_rv_binary(obj: &str) {
        let objcopy = "llvm-objcopy";
        let output = Command::new(objcopy).arg("-O")
                                .arg("binary")
                                .arg(obj)
                                .arg(obj.to_owned() + ".bin")
                                .output()
                                .expect("Failed to generate rv binary");
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }

    fn rv_helper(code: &str, testname: &str, n_clock: usize) -> Result<Cpu, std::io::Error> {
        let filename = testname.to_owned() + ".s";
        let mut file = File::create(&filename)?;
        file.write(&code.as_bytes())?;
        generate_rv_obj(&filename);
        generate_rv_binary(testname);
        let mut file_bin = File::open(testname.to_owned() + ".bin")?;
        let mut code = Vec::new();
        file_bin.read_to_end(&mut code)?;
        let mut cpu = Cpu::new(code);

        for _i in 0..n_clock {
            let inst = match cpu.fetch() {
                Ok(inst) => inst,
                Err(_err) => break,
            };
            match cpu.execute(inst) {
                Ok(new_pc) => cpu.pc = new_pc,
                Err(err) => println!("{}", err),
            };
        }

        return Ok(cpu);
    }

    macro_rules! riscv_test {
        ( $code:expr, $name:expr, $clock:expr, $($real:expr => $expect:expr),* ) => {
            match rv_helper($code, $name, $clock) {
                Ok(cpu) => { 
                    $(assert_eq!(cpu.reg($real), $expect);)*
                }
                Err(e) => { println!("error: {}", e); assert!(false); }
            } 
        };
    }

    #[test]
    fn test_addi() {
        let code = "addi x31, x0, 42";
        riscv_test!(code, "test_addi", 1, "x31" => 42);
    }

    #[test]
    fn test_simple() {
        // this is the assembly code of simple.c
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
        riscv_test!(code, "test_simple", 20, "a0" => 42);
    }

    #[test]
    fn test_lui() {
        let code = "lui a0, 42";
        riscv_test!(code, "test_lui", 1, "a0" => 42 << 12);
    }

    #[test]
    fn test_auipc() {
        let code = "auipc a0, 42";
        riscv_test!(code, "test_auipc", 1, "a0" => DRAM_BASE + (42 << 12));
    }

    #[test]
    fn test_jal() {
        let code = "jal a0, 42";
        riscv_test!(code, "test_jal", 1, "a0" => DRAM_BASE + 4, "pc" => DRAM_BASE + 42);
    }

    #[test]
    fn test_jalr() {
        let code = "
            addi a1, zero, 42
            jalr a0, -8(a1)
        ";
        riscv_test!(code, "test_jalr", 2, "a0" => DRAM_BASE + 8, "pc" => 34);
    }

    #[test]
    fn test_beq() {
        let code = "
            beq  x0, x0, 42
        ";
        riscv_test!(code, "test_beq", 3, "pc" => DRAM_BASE + 42);
    }

    #[test]
    fn test_bne() {
        let code = "
            addi x1, x0, 10
            bne  x0, x1, 42
        ";
        riscv_test!(code, "test_bne", 5, "pc" => DRAM_BASE + 42 + 4);
    }

    #[test]
    fn test_blt() {
        let code = "
            addi x1, x0, 10
            addi x2, x0, 20
            blt  x1, x2, 42
        ";
        riscv_test!(code, "test_blt", 10, "pc" => DRAM_BASE + 42 + 8);
    }

    #[test]
    fn test_bge() {
        let code = "
            addi x1, x0, 10
            addi x2, x0, 20
            bge  x2, x1, 42
        ";
        riscv_test!(code, "test_bge", 10, "pc" => DRAM_BASE + 42 + 8);
    }

    #[test]
    fn test_bltu() {
        let code = "
            addi x1, x0, 10
            addi x2, x0, 20
            bltu x1, x2, 42
        ";
        riscv_test!(code, "test_bltu", 10, "pc" => DRAM_BASE + 42 + 8);
    }

    #[test]
    fn test_bgeu() {
        let code = "
            addi x1, x0, 10
            addi x2, x0, 20
            bgeu x2, x1, 42
        ";
        riscv_test!(code, "test_bgeu", 10, "pc" => DRAM_BASE + 42 + 8);
    }

    #[test]
    fn test_store_load1() {
        let code = "
            addi s0, zero, 256
            addi sp, sp, -16
            sd   s0, 8(sp)
            lb   t1, 8(sp)
            lh   t2, 8(sp)
        ";
        riscv_test!(code, "test_store_load1", 10, "t1" => 0, "t2" => 256);
    }

    #[test]
    fn test_slt() {
        let code = "
            addi t0, zero, 14
            addi t1, zero, 24
            slt  t2, t0, t1
            slti t3, t0, 42
            sltiu t4, t0, 84
        ";
        riscv_test!(code, "test_slt", 7, "t2" => 1, "t3" => 1, "t4" => 1);
    }

    #[test]
    fn test_xor() {
        let code = "
            addi a0, zero, 0b10
            xori a1, a0, 0b01
            xor a2, a1, a1 
        ";
        riscv_test!(code, "test_xor", 5, "a1" => 3, "a2" => 0);
    }

    #[test]
    fn test_or() {
        let code = "
            addi a0, zero, 0b10
            ori  a1, a0, 0b01
            or   a2, a0, a0
        ";
        riscv_test!(code, "test_or", 3, "a1" => 0b11, "a2" => 0b10);
    }

    #[test]
    fn test_and() {
        let code = "
            addi a0, zero, 0b10 
            andi a1, a0, 0b11
            and  a2, a0, a1
        ";
        riscv_test!(code, "test_and", 3, "a1" => 0b10, "a2" => 0b10);
    }

    #[test]
    fn test_sll() {
        let code = "
            addi a0, zero, 1
            addi a1, zero, 5
            sll  a2, a0, a1
            slli a3, a0, 5
            addi s0, zero, 64
            sll  a4, a0, s0
        ";
        riscv_test!(code, "test_sll", 10, "a2" => 1 << 5, "a3" => 1 << 5, "a4" => 1);
    }

    #[test]
    fn test_sra_srl() {
        let code = "
            addi a0, zero, -8
            addi a1, zero, 1
            sra  a2, a0, a1
            srai a3, a0, 2
            srli a4, a0, 2
            srl  a5, a0, a1
        ";
        riscv_test!(code, "test_sra_srl", 10, "a2" => -4 as i64 as u64, "a3" => -2 as i64 as u64, 
                                              "a4" => -8 as i64 as u64 >> 2, "a5" => -8 as i64 as u64 >> 1);
    }

    #[test]
    fn test_word_op() {
        let code = "
            addi a0, zero, 42 
            lui  a1, 0x7f000
            addw a2, a0, a1
        ";
        riscv_test!(code, "test_word_op", 29, "a2" => 0x7f00002a);
    }

    #[test]
    fn test_csrs1() {
        let code = "
            addi t0, zero, 1
            addi t1, zero, 2
            addi t2, zero, 3
            csrrw zero, mstatus, t0
            csrrs zero, mtvec, t1
            csrrw zero, mepc, t2
            csrrc t2, mepc, zero
            csrrwi zero, sstatus, 4
            csrrsi zero, stvec, 5
            csrrwi zero, sepc, 6
            csrrci zero, sepc, 0 
        ";
        riscv_test!(code, "test_csrs1", 20, "mstatus" => 1, "mtvec" => 2, "mepc" => 3,
                                            "sstatus" => 0, "stvec" => 5, "sepc" => 6);
    }
}