#[allow(dead_code)]

pub const NUM_CSRS: usize = 4096;
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


// mstatus and sstatus field mask
pub const BIT_SIE: u64 = 1 << 1; 
pub const BIT_MIE: u64 = 1 << 3;
pub const BIT_SPIE: u64 = 1 << 5; 
pub const BIT_UBE: u64 = 1 << 6; 
pub const BIT_MPIE: u64 = 1 << 7;
pub const BIT_SPP: u64 = 1 << 8; 
pub const BIT_VS: u64 = 0b11 << 9;
pub const BIT_MPP: u64 = 0b11 << 11;
pub const BIT_FS: u64 = 0b11 << 13; 
pub const BIT_XS: u64 = 0b11 << 15; 
pub const BIT_MPRV: u64 = 1 << 17;
pub const BIT_SUM: u64 = 1 << 18; 
pub const BIT_MXR: u64 = 1 << 19; 
pub const BIT_TVM: u64 = 1 << 20;
pub const BIT_TW: u64 = 1 << 21;
pub const BIT_TSR: u64 = 1 << 22;
pub const BIT_UXL: u64 = 0b11 << 32; 
pub const BIT_SXL: u64 = 0b11 << 34;
pub const BIT_SBE: u64 = 1 << 36;
pub const BIT_MBE: u64 = 1 << 37;
pub const BIT_SD: u64 = 1 << 63; 
pub const SSTATUS_MASK: u64 = BIT_SIE | BIT_SPIE | BIT_UBE | BIT_SPP | BIT_FS 
                            | BIT_XS  | BIT_SUM  | BIT_MXR | BIT_UXL | BIT_SD;


pub struct Csr {
    csrs: [u64; NUM_CSRS],
}

impl Csr {
    pub fn new(csrs: [u64; NUM_CSRS]) -> Csr {
        Self { csrs }
    }

    pub fn dump_csrs(&self) {
        println!("{:-^80}", "control status registers");
        let output = format!(
            "{}\n{}\n",
            format!(
                "mstatus = {:<#18x}  mtvec = {:<#18x}  mepc = {:<#18x}  mcause = {:<#18x}",
                self.load(MSTATUS),
                self.load(MTVEC),
                self.load(MEPC),
                self.load(MCAUSE),
            ),
            format!(
                "sstatus = {:<#18x}  stvec = {:<#18x}  sepc = {:<#18x}  scause = {:<#18x}",
                self.load(SSTATUS),
                self.load(STVEC),
                self.load(SEPC),
                self.load(SCAUSE),
            ),
        );
        println!("{}", output);
    }

    pub fn load(&self, addr: usize) -> u64 {
        match addr {
            SIE => self.csrs[MIE] & self.csrs[MIDELEG],
            SIP => self.csrs[MIP] & self.csrs[MIDELEG],
            SSTATUS => self.csrs[MSTATUS] & SSTATUS_MASK,
            _ => self.csrs[addr],
        }
    }

    pub fn store(&mut self, addr: usize, value: u64) {
        match addr {
            SIE => self.csrs[MIE] = (self.csrs[MIE] & !self.csrs[MIDELEG]) | (value & self.csrs[MIDELEG]),
            SIP => self.csrs[MIP] = (self.csrs[MIE] & !self.csrs[MIDELEG]) | (value & self.csrs[MIDELEG]),
            SSTATUS => self.csrs[MSTATUS] = (self.csrs[MSTATUS] & !SSTATUS_MASK) | (value & SSTATUS_MASK),
            _ => self.csrs[addr] = value,
        }
    }
}