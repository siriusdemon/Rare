// Riscv assembly in S Expression format
//
// e.g.
// (nop)
// (addi x29 x0 5)
// (addi x30 x0 37)
// (add x31 x29 x30)


use std::fs::File;
use std::io::Write;
use std::mem::transmute;
use std::fmt;

use crate::sexpr::*;

pub enum Riscv {
    Nop,
    Reg {reg: String, line: usize, col: usize},
    Imm {val: u32, line: usize, col: usize},
    Op0 {op: String,  line: usize, col: usize},
    Op3 {op: String, e1: Box<Riscv>, e2: Box<Riscv>, e3: Box<Riscv>, line: usize, col: usize },
}

impl fmt::Display for Riscv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Riscv::*;
        match self {
            Nop => write!(f, "nop\n"),
            Reg {reg, line, col} => write!(f, "{}", reg),
            Imm {val, line, col} => write!(f, "{}", val),
            Op0 {op, line, col}  => write!(f, "{}", op),
            Op3 {op, e1, e2, e3, line, col} => write!(f, "{} {}, {}, {}", op, e1, e2, e3),
        }
    }
}


fn op3(op: String, e1: Riscv, e2: Riscv, e3: Riscv, line: usize, col: usize) -> Riscv {
    Riscv::Op3 {op, e1: Box::new(e1), e2: Box::new(e2), e3: Box::new(e3), line, col}
}

fn is_valid_op(op: &str) -> bool {
    let valid_ops = ["add", "addi"];
    return valid_ops.contains(&op);
}

fn is_reg(op: &str) -> bool {
    // s0, fp, x8 name the same register.
    const regs: [&str; 65] = [
        "x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7", "x8", "x9", "x10", "x11",
        "x12", "x13", "x14", "x15", "x16", "x17", "x18", "x19", "x20", "x21",
        "x22", "x23", "x24", "x25", "x26", "x27", "x28", "x29", "x30", "x31",
        "zero", "ra", "sp", "gp", "tp", "t0", "t1", "t2", "s0", "fp", "s1", "a0", 
        "a1", "a2", "a3", "a4", "a5", "a6", "a7", "s2", "s3", "s4", "s5", "s6", 
        "s7", "s8", "s9", "s10", "s11", "t3", "t4", "t5", "t6",
    ];
    return regs.contains(&op);
}

pub struct RiscvParser {
    exprs: Vec<Expr>
}

impl RiscvParser {
    pub fn new(text: &str) -> RiscvParser {
        let exprs = Scanner::new(text).scan();
        Self { exprs }
    }
    
    pub fn parse(self) -> Vec<Riscv> {
        self.exprs.into_iter().map(|e| {
            if let Expr::List {value, line, col} = e {
                if value.len() == 4 {
                    return Self::expr_to_op3(value);
                }
                panic!("Invalid op length: {}", value.len());
            }
            panic!("Invalid riscv assembly: {}", e);
        }).collect()
    }

    fn expr_to_op3(mut value: Vec<Expr>) -> Riscv {
        let oprands = value.split_off(1);
        let op = value.pop().unwrap();
        if let Expr::Symbol {value: op, line, col} = op {
            if is_valid_op(&op) {
                let mut oprands: Vec<Riscv> = oprands.into_iter().map(|e| Self::atom_to_riscv(e)).collect();
                let e3 = oprands.pop().unwrap();
                let e2 = oprands.pop().unwrap();
                let e1 = oprands.pop().unwrap();
                return op3(op, e1, e2, e3, line, col);
            }
            panic!("Invalid riscv operation: {}", op);
        } 
        panic!("Invalid riscv assembly: {}", op);
    }

    fn atom_to_riscv(e: Expr) -> Riscv {
        match e {
            Expr::Symbol { value, line, col } if is_reg(value.as_str()) => {
                Riscv::Reg {reg: value, line, col}
            }
            Expr::UInt { value, line, col } => {
                Riscv::Imm {val: value.parse().unwrap(), line, col}
            }
            Expr::SInt { value, line, col } => {
                Riscv::Imm {val: value.parse().unwrap(), line, col}
            }
            _ => panic!("Invalil atom expression {}", e),
        }
    }
}


pub struct RiscvAssembly {
    code: Vec<Riscv>
}


fn reg_to_code(asm: Riscv) -> u32 {
    match asm {
        Riscv::Reg {reg, line, col} => {
            match reg.as_str() {
                "x0" | "zero" => 0, "x1" | "ra" => 1, "x2" | "sp" => 2, "x3" | "gp" => 3,
                "x4" | "tp" => 4, "x5" | "t0" => 5, "x6" | "t1" => 6, "x7" | "t2" => 7,
                "x8" | "s0" | "fp" => 8, "x9" | "s1" => 9, "x10" | "a0" => 10, "x11" | "a1" => 11,
                "x12" | "a2" => 12, "x13" | "a3" => 13, "x14" | "a4" => 14, "x15" | "a5" => 15,
                "x16" | "a6" => 16, "x17" | "a7" => 17, "x18" | "s2" => 18, "x19" | "s3" => 19,
                "x20" | "s4" => 20, "x21" | "s5" => 21, "x22" | "s6" => 22, "x23" | "s7" => 23,
                "x24" | "s8" => 24, "x25" | "s9" => 25, "x26" | "s10" => 26, "x27" | "s11" => 27,
                "x28" | "t3" => 28, "x29" | "t4" => 29, "x30" | "t5" => 30, "x31" | "t6" => 31,
                _ => panic!("Invalid register {} at line {}, col {}", reg, line, col),
            }
        }
        _ => panic!("Expect a register, found {}", asm),
    }
}

fn imm_to_code(asm: Riscv) -> u32 {
    match asm {
        Riscv::Imm { val, line, col } => val,
        _ => panic!("Invalid Immediate {}", asm),
    }
}

impl RiscvAssembly {
    pub fn new(code: Vec<Riscv>) -> RiscvAssembly {
        Self { code }
    }

    pub fn compile(self, filename: &str) -> Result<(), std::io::Error> {
        let mut file = File::create(filename)?;
        for code in self.code {
            match code {
                Riscv::Op3 { op, e1, e2, e3, line, col} => {
                    match op.as_str() {
                        "addi" => {
                            let rd = reg_to_code(*e1);
                            let rs1 = reg_to_code(*e2);
                            let imm = imm_to_code(*e3);
                            let inst: u32 = (imm << 20) | (rs1 << 15) | (rd << 7) | 0b0010011;
                            let bytes: [u8; 4] = unsafe { transmute(inst.to_le()) };
                            file.write(&bytes)?;
                        }
                        "add" => {
                            let rd = reg_to_code(*e1);
                            let rs1 = reg_to_code(*e2);
                            let rs2 = reg_to_code(*e3);
                            let inst: u32 = (rs2 << 20) | (rs1 << 15) | (rd << 7) | 0b0110011;
                            let bytes: [u8; 4] = unsafe { transmute(inst.to_le()) };
                            file.write(&bytes);
                        }
                        other => panic!("Not implemented yet! {}", other),
                    }
                }
                other => panic!("Not implemented yet! {}", other),
            };
        }
        return Ok(());
    }
}


#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_addi() {
        let s = "(addi x10 x0 17)";
        let asm = RiscvParser::new(s).parse();
        let asm_str = format!("{}", asm[0]);
        assert_eq!(&asm_str, "addi x10, x0, 17");
    }
}