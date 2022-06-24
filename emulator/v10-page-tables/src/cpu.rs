//! The cpu module contains `Cpu` and implementarion for it.

#![allow(dead_code)]

use crate::bus::*;
use crate::dram::*;
use crate::plic::*;
use crate::exception::*;
use crate::interrupt::*;
use crate::uart::*;
use crate::virtio::*;
use crate::param::*;
use crate::csr::*;

/// The privileged mode.
#[derive(Debug, PartialEq, PartialOrd, Eq, Copy, Clone)]
pub enum Mode {
    User = 0b00,
    Supervisor = 0b01,
    Machine = 0b11,
}

/// Access type that is used in the virtual address translation process. It decides which exception
/// should raises (InstructionPageFault, LoadPageFault or StoreAMOPageFault).
#[derive(Debug, PartialEq, PartialOrd)]
pub enum AccessType {
    /// Raises the exception InstructionPageFault. It is used for an instruction fetch.
    Instruction,
    /// Raises the exception LoadPageFault.
    Load,
    /// Raises the exception StoreAMOPageFault.
    Store,
}

/// The `Cpu` struct that contains registers, a program coutner, system bus that connects
/// peripheral devices, and control and status registers.
pub struct Cpu {
    /// 32 64-bit integer registers.
    pub regs: [u64; 32],
    /// Program counter to hold the the dram address of the next instruction that would be executed.
    pub pc: u64,
    /// The current privilege mode.
    pub mode: Mode,
    /// System bus that transfers data between CPU and peripheral devices.
    pub bus: Bus,
    /// Control and status registers. RISC-V ISA sets aside a 12-bit encoding space (csr[11:0]) for
    /// up to 4096 CSRs.
    pub csr: Csr,
    /// SV39 paging flag.
    pub enable_paging: bool,
    /// physical page number (PPN) × PAGE_SIZE (4096).
    pub page_table: u64,
}

impl Cpu {
    /// Create a new `Cpu` object.
    pub fn new(binary: Vec<u8>, disk_image: Vec<u8>) -> Self {
        // The stack pointer (SP) must be set up at first.
        let mut regs = [0; 32];
        regs[2] = DRAM_BASE + DRAM_SIZE;

        Self {
            regs,
            // The program counter starts from the start address of a dram.
            pc: DRAM_BASE,
            mode: Mode::Machine,
            bus: Bus::new(binary, disk_image),
            csr: Csr::new(),
            enable_paging: false,
            page_table: 0,
        }
    }

    /// Print values in all registers (x0-x31).
    pub fn dump_registers(&self) {
        let mut output = String::from("");
        let abi = [
            "zero", " ra ", " sp ", " gp ", " tp ", " t0 ", " t1 ", " t2 ", " s0 ", " s1 ", " a0 ",
            " a1 ", " a2 ", " a3 ", " a4 ", " a5 ", " a6 ", " a7 ", " s2 ", " s3 ", " s4 ", " s5 ",
            " s6 ", " s7 ", " s8 ", " s9 ", " s10", " s11", " t3 ", " t4 ", " t5 ", " t6 ",
        ];
        for i in (0..32).step_by(4) {
            output = format!(
                "{}\n{}",
                output,
                format!(
                    "x{:02}({})={:>#18x} x{:02}({})={:>#18x} x{:02}({})={:>#18x} x{:02}({})={:>#18x}",
                    i,
                    abi[i],
                    self.regs[i],
                    i + 1,
                    abi[i + 1],
                    self.regs[i + 1],
                    i + 2,
                    abi[i + 2],
                    self.regs[i + 2],
                    i + 3,
                    abi[i + 3],
                    self.regs[i + 3],
                )
            );
        }
        println!("{}", output);
    }

    /// Print values in some csrs.
    pub fn dump_csrs(&self) {
        self.csr.dump_csrs();
    }

    pub fn handle_exception(&mut self, e: Exception) {
        // the process to handle exception in S-mode and M-mode is similar,
        // includes following steps:
        // 0. set xPP to current mode.
        // 1. update hart's privilege mode (M or S according to current mode and exception setting).
        // 2. save current pc in epc (sepc in S-mode, mepc in M-mode)
        // 3. set pc to trap vector (stvec in S-mode, mtvec in M-mode)
        // 4. set cause to exception code (scause in S-mode, mcause in M-mode)
        // 5. set trap value properly (stval in S-mode, mtval in M-mode)
        // 6. set xPIE to xIE (SPIE in S-mode, MPIE in M-mode)
        // 7. clear up xIE (SIE in S-mode, MIE in M-mode)
        use Exception::*;
        let pc = self.pc - 4; 
        let mode = self.mode;
        let cause = e.code();
        // if an exception happen in U-mode or S-mode, and the exception is delegated to S-mode.
        // then this exception should be handled in S-mode.
        let trap_in_s_mode = mode <= Mode::Supervisor && self.csr.is_medelegated(cause);
        let (STATUS, TVEC, CAUSE, TVAL, EPC, MASK_PIE, pie_i, MASK_IE, ie_i, MASK_PP, pp_i) 
            = if trap_in_s_mode {
                self.mode = Mode::Supervisor;
                (SSTATUS, STVEC, SCAUSE, STVAL, SEPC, MASK_SPIE, 5, MASK_SIE, 1, MASK_SPP, 8)
            } else {
                self.mode = Mode::Machine;
                (MSTATUS, MTVEC, MCAUSE, MTVAL, MEPC, MASK_MPIE, 7, MASK_MIE, 3, MASK_MPP, 11)
            };
        // 3.1.7 & 4.1.2
        // The BASE field in tvec is a WARL field that can hold any valid virtual or physical address,
        // subject to the following alignment constraints: the address must be 4-byte aligned
        self.pc = self.csr.load(TVEC) & !0b11;
        // 3.1.14 & 4.1.7
        // When a trap is taken into S-mode (or M-mode), sepc (or mepc) is written with the virtual address 
        // of the instruction that was interrupted or that encountered the exception.
        self.csr.store(EPC, pc);
        // 3.1.15 & 4.1.8
        // When a trap is taken into S-mode (or M-mode), scause (or mcause) is written with a code indicating 
        // the event that caused the trap.
        self.csr.store(CAUSE, cause);
        // 3.1.16 & 4.1.9
        // If stval is written with a nonzero value when a breakpoint, address-misaligned, access-fault, or
        // page-fault exception occurs on an instruction fetch, load, or store, then stval will contain the
        // faulting virtual address.
        // If stval is written with a nonzero value when a misaligned load or store causes an access-fault or
        // page-fault exception, then stval will contain the virtual address of the portion of the access that
        // caused the fault
        let addr = match e {
            InstructionAddrMisaligned(addr) 
            | InstructionAccessFault(addr) 
            | InstructionPageFault(addr) => addr,
            LoadAccessMisaligned(addr)
            | LoadAccessFault(addr)
            | LoadPageFault(addr) => addr,
            StoreAMOAddrMisaligned(addr)
            | StoreAMOAccessFault(addr)
            | StoreAMOPageFault(addr) => addr,
            _ => 0,
        };
        self.csr.store(TVAL, 0);
        // 3.1.6 covers both sstatus and mstatus.
        let mut status = self.csr.load(STATUS);
        // get SIE or MIE
        let ie = (status & MASK_IE) >> ie_i;
        // set SPIE = SIE / MPIE = MIE
        status = (status & !MASK_PIE) | (ie << pie_i);
        // set SIE = 0 / MIE = 0
        status &= !MASK_IE; 
        // set SPP / MPP = previous mode
        let mode = match mode {
            Mode::Supervisor => 0b01,
            Mode::Machine => 0b11,
            Mode::User => 0b00,
        };
        status = (status & !MASK_PP) | (mode << pp_i);
        self.csr.store(STATUS, status);
    }


    pub fn handle_interrupt(&mut self, interrupt: Interrupt) {
        // similar to handle exception
        let pc = self.pc; 
        let mode = self.mode;
        let cause = interrupt.code();
        // although cause contains a interrupt bit. Shift the cause make it out.
        let trap_in_s_mode = mode <= Mode::Supervisor && self.csr.is_midelegated(cause);
        let (STATUS, TVEC, CAUSE, TVAL, EPC, MASK_PIE, pie_i, MASK_IE, ie_i, MASK_PP, pp_i) 
            = if trap_in_s_mode {
                self.mode = Mode::Supervisor;
                (SSTATUS, STVEC, SCAUSE, STVAL, SEPC, MASK_SPIE, 5, MASK_SIE, 1, MASK_SPP, 8)
            } else {
                self.mode = Mode::Machine;
                (MSTATUS, MTVEC, MCAUSE, MTVAL, MEPC, MASK_MPIE, 7, MASK_MIE, 3, MASK_MPP, 11)
            };
        // 3.1.7 & 4.1.2
        // When MODE=Direct, all traps into machine mode cause the pc to be set to the address in the BASE field. 
        // When MODE=Vectored, all synchronous exceptions into machine mode cause the pc to be set to the address 
        // in the BASE field, whereas interrupts cause the pc to be set to the address in the BASE field plus four 
        // times the interrupt cause number. 
        let tvec = self.csr.load(TVEC);
        let tvec_mode = tvec & 0b11;
        let tvec_base = tvec & !0b11;
        match tvec_mode { // DIrect
            0 => self.pc = tvec_base,
            1 => self.pc = tvec_base + cause << 2,
            _ => unreachable!(),
        };
        // 3.1.14 & 4.1.7
        // When a trap is taken into S-mode (or M-mode), sepc (or mepc) is written with the virtual address 
        // of the instruction that was interrupted or that encountered the exception.
        self.csr.store(EPC, pc);
        // 3.1.15 & 4.1.8
        // When a trap is taken into S-mode (or M-mode), scause (or mcause) is written with a code indicating 
        // the event that caused the trap.
        self.csr.store(CAUSE, cause);
        // 3.1.16 & 4.1.9
        // When a trap is taken into M-mode, mtval is either set to zero or written with exception-specific 
        // information to assist software in handling the trap. 
        self.csr.store(TVAL, 0);
        // 3.1.6 covers both sstatus and mstatus.
        let mut status = self.csr.load(STATUS);
        // get SIE or MIE
        let ie = (status & MASK_IE) >> ie_i;
        // set SPIE = SIE / MPIE = MIE
        status = (status & !MASK_PIE) | (ie << pie_i);
        // set SIE = 0 / MIE = 0
        status &= !MASK_IE; 
        // set SPP / MPP = previous mode
        let mode = match mode {
            Mode::Supervisor => 0b01,
            Mode::Machine => 0b11,
            Mode::User => 0b00,
        };
        status = (status & !MASK_PP) | (mode << pp_i);
        self.csr.store(STATUS, status);
    }


    pub fn check_pending_interrupt(&mut self) -> Option<Interrupt> {
        use Interrupt::*;
        // 3.1.6.1
        // When a hart is executing in privilege mode x, interrupts are globally enabled when x IE=1 and globally 
        // disabled when xIE=0. Interrupts for lower-privilege modes, w<x, are always globally disabled regardless 
        // of the setting of any global wIE bit for the lower-privilege mode. Interrupts for higher-privilege modes, 
        // y>x, are always globally enabled regardless of the setting of the global yIE bit for the higher-privilege 
        // mode. Higher-privilege-level code can use separate per-interrupt enable bits to disable selected higher-
        // privilege-mode interrupts before ceding control to a lower-privilege mode
 
        // 3.1.9 & 4.1.3
        // An interrupt i will trap to M-mode (causing the privilege mode to change to M-mode) if all of
        // the following are true: (a) either the current privilege mode is M and the MIE bit in the mstatus
        // register is set, or the current privilege mode has less privilege than M-mode; (b) bit i is set in both
        // mip and mie; and (c) if register mideleg exists, bit i is not set in mideleg.
        match self.mode {
            Mode::Machine => {
                // Check if the MIE bit is enabled.
                if (self.load_csr(MSTATUS) >> 3) & 1 == 0 {
                    return None;
                }
            }
            Mode::Supervisor => {
                // Check if the SIE bit is enabled.
                if (self.load_csr(SSTATUS) >> 1) & 1 == 0 {
                    return None;
                }
            }
            _ => {}
        }
       
        // 3.1.9 & 4.1.3
        // Multiple simultaneous interrupts destined for M-mode are handled in the following decreasing
        // priority order: MEI, MSI, MTI, SEI, SSI, STI.
        if self.bus.uart.is_interrupting() {
            self.bus.store(PLIC_SCLAIM, 32, UART_IRQ).unwrap();
            self.csr.store(MIP, self.csr.load(MIP) | MASK_SEIP); 
        } else if self.bus.virtio.is_interrupting() {
            self.disk_access();
            self.bus.store(PLIC_SCLAIM, 32, VIRTIO_IRQ).unwrap();  
            self.csr.store(MIP, self.csr.load(MIP) | MASK_SEIP);
        }

        let pending = self.csr.load(MIE) & self.csr.load(MIP);

        if (pending & MASK_MEIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP) & !MASK_MEIP);
            return Some(MachineExternalInterrupt);
        }
        if (pending & MASK_MSIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP) & !MASK_MSIP);
            return Some(MachineSoftwareInterrupt);
        }
        if (pending & MASK_MTIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP) & !MASK_MTIP);
            return Some(MachineTimerInterrupt);
        }
        if (pending & MASK_SEIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP) & !MASK_SEIP);
            return Some(SupervisorExternalInterrupt);
        }
        if (pending & MASK_SSIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP) & !MASK_SSIP);
            return Some(SupervisorSoftwareInterrupt);
        }
        if (pending & MASK_STIP) != 0 {
            self.csr.store(MIP, self.csr.load(MIP) & !MASK_STIP);
            return Some(SupervisorTimerInterrupt);
        }
        return None;
    }


    pub fn disk_access(&mut self) {
        let desc_addr = self.bus.virtio.desc_addr();
        let avail_addr = desc_addr + 0x40;
        let used_addr = desc_addr + 4096;
        
        let offset = self.bus.load(avail_addr.wrapping_add(1), 16).unwrap();
        let index = self.bus.load(avail_addr.wrapping_add(offset % DESC_NUM)
                                            .wrapping_add(2), 16).unwrap();
        let desc_addr0 = desc_addr + VRING_DESC_SIZE * index;
        let addr0 = self.bus.load(desc_addr0, 64).unwrap();
        let next0 = self.bus.load(desc_addr0.wrapping_add(14), 16).unwrap();
        let desc_addr1 = desc_addr + VRING_DESC_SIZE * next0;
        let addr1 = self.bus.load(desc_addr1, 64).unwrap();
        let len1 = self.bus.load(desc_addr1.wrapping_add(8), 32).unwrap();
        let flags1 = self.bus.load(desc_addr1.wrapping_add(12), 16).unwrap();
        let blk_sector = self.bus.load(addr0.wrapping_add(8), 64).unwrap();
        match (flags1 & 2) == 0 {
            true => {
                for i in 0..len1 {
                    let data = self.bus.load(addr1 + 1, 8).unwrap();
                    self.bus.virtio.write_disk(blk_sector * 512 + i, data);
                }
            }
            false => {
                for i in 0..len1 {
                    let data = self.bus.virtio.read_disk(blk_sector * 512 + i);
                    self.bus.store(addr1 + i, 8, data as u64);
                }
            }
        }

        let new_id = self.bus.virtio.get_new_id();
        self.bus.store(used_addr.wrapping_add(2), 16, new_id % 8).unwrap();
    }

    fn update_paging(&mut self, csr_addr: usize) {
        if csr_addr != SATP { return; }

        // Read the physical page number (PPN) of the root page table, i.e., its
        // supervisor physical address divided by 4 KiB.
        self.page_table = (self.csr.load(SATP) & ((1 << 44) - 1)) * PAGE_SIZE;

        // Read the MODE field, which selects the current address-translation scheme.
        let mode = self.csr.load(SATP) >> 60;

        // Enable the SV39 paging if the value of the mode field is 8.
        self.enable_paging = mode == 8;
    }

    /// Translate a virtual address to a physical address for the paged virtual-dram system.
    pub fn translate(&mut self, addr: u64, access_type: AccessType) -> Result<u64, Exception> {
        if !self.enable_paging {
            return Ok(addr);
        }

        // The following comments are cited from 4.3.2 Virtual Address Translation Process
        // in "The RISC-V Instruction Set Manual Volume II-Privileged Architecture_20190608".

        // "A virtual address va is translated into a physical address pa as follows:"
        let levels = 3;
        let vpn = [
            (addr >> 12) & 0x1ff,
            (addr >> 21) & 0x1ff,
            (addr >> 30) & 0x1ff,
        ];

        // "1. Let a be satp.ppn × PAGESIZE, and let i = LEVELS − 1. (For Sv32, PAGESIZE=212
        //     and LEVELS=2.)"
        let mut a = self.page_table;
        let mut i: i64 = levels - 1;
        let mut pte;
        loop {
            // "2. Let pte be the value of the PTE at address a+va.vpn[i]×PTESIZE. (For Sv32,
            //     PTESIZE=4.) If accessing pte violates a PMA or PMP check, raise an access
            //     exception corresponding to the original access type."
            pte = self.bus.load(a + vpn[i as usize] * 8, 64)?;

            // "3. If pte.v = 0, or if pte.r = 0 and pte.w = 1, stop and raise a page-fault
            //     exception corresponding to the original access type."
            let v = pte & 1;
            let r = (pte >> 1) & 1;
            let w = (pte >> 2) & 1;
            let x = (pte >> 3) & 1;
            if v == 0 || (r == 0 && w == 1) {
                match access_type {
                    AccessType::Instruction => return Err(Exception::InstructionPageFault(addr)),
                    AccessType::Load => return Err(Exception::LoadPageFault(addr)),
                    AccessType::Store => return Err(Exception::StoreAMOPageFault(addr)),
                }
            }

            // "4. Otherwise, the PTE is valid. If pte.r = 1 or pte.x = 1, go to step 5.
            //     Otherwise, this PTE is a pointer to the next level of the page table.
            //     Let i = i − 1. If i < 0, stop and raise a page-fault exception
            //     corresponding to the original access type. Otherwise,
            //     let a = pte.ppn × PAGESIZE and go to step 2."
            if r == 1 || x == 1 {
                break;
            }
            i -= 1;
            let ppn = (pte >> 10) & 0x0fff_ffff_ffff;
            a = ppn * PAGE_SIZE;
            if i < 0 {
                match access_type {
                    AccessType::Instruction => return Err(Exception::InstructionPageFault(addr)),
                    AccessType::Load => return Err(Exception::LoadPageFault(addr)),
                    AccessType::Store => return Err(Exception::StoreAMOPageFault(addr)),
                }
            }
        }

        // A leaf PTE has been found.
        let ppn = [
            (pte >> 10) & 0x1ff,
            (pte >> 19) & 0x1ff,
            (pte >> 28) & 0x03ff_ffff,
        ];

        // We skip implementing from step 5 to 7.

        // "5. A leaf PTE has been found. Determine if the requested dram access is allowed by
        //     the pte.r, pte.w, pte.x, and pte.u bits, given the current privilege mode and the
        //     value of the SUM and MXR fields of the mstatus register. If not, stop and raise a
        //     page-fault exception corresponding to the original access type."

        // "6. If i > 0 and pte.ppn[i − 1 : 0] ̸= 0, this is a misaligned superpage; stop and
        //     raise a page-fault exception corresponding to the original access type."

        // "7. If pte.a = 0, or if the dram access is a store and pte.d = 0, either raise a
        //     page-fault exception corresponding to the original access type, or:
        //     • Set pte.a to 1 and, if the dram access is a store, also set pte.d to 1.
        //     • If this access violates a PMA or PMP check, raise an access exception
        //     corresponding to the original access type.
        //     • This update and the loading of pte in step 2 must be atomic; in particular, no
        //     intervening store to the PTE may be perceived to have occurred in-between."

        // "8. The translation is successful. The translated physical address is given as
        //     follows:
        //     • pa.pgoff = va.pgoff.
        //     • If i > 0, then this is a superpage translation and pa.ppn[i−1:0] =
        //     va.vpn[i−1:0].
        //     • pa.ppn[LEVELS−1:i] = pte.ppn[LEVELS−1:i]."
        let offset = addr & 0xfff;
        match i {
            0 => {
                let ppn = (pte >> 10) & 0x0fff_ffff_ffff;
                Ok((ppn << 12) | offset)
            }
            1 => {
                // Superpage translation. A superpage is a dram page of larger size than an
                // ordinary page (4 KiB). It reduces TLB misses and improves performance.
                Ok((ppn[2] << 30) | (ppn[1] << 21) | (vpn[0] << 12) | offset)
            }
            2 => {
                // Superpage translation. A superpage is a dram page of larger size than an
                // ordinary page (4 KiB). It reduces TLB misses and improves performance.
                Ok((ppn[2] << 30) | (vpn[1] << 21) | (vpn[0] << 12) | offset)
            }
            _ => match access_type {
                AccessType::Instruction => return Err(Exception::InstructionPageFault(addr)),
                AccessType::Load => return Err(Exception::LoadPageFault(addr)),
                AccessType::Store => return Err(Exception::StoreAMOPageFault(addr)),
            },
        }
    }

    /// Load a value from a CSR.
    pub fn load_csr(&self, addr: usize) -> u64 {
        self.csr.load(addr)
    }

    /// Store a value to a CSR.
    pub fn store_csr(&mut self, addr: usize, value: u64) {
        self.csr.store(addr, value)
    }

    /// Load a value from a dram.
    pub fn load(&mut self, addr: u64, size: u64) -> Result<u64, Exception> {
        let p_addr = self.translate(addr, AccessType::Load)?;
        self.bus.load(p_addr, size)
    }

    /// Store a value to a dram.
    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), Exception> {
        let p_addr = self.translate(addr, AccessType::Store)?;
        self.bus.store(p_addr, size, value)
    }

    /// Get an instruction from the dram.
    pub fn fetch(&mut self) -> Result<u64, Exception> {
        let p_pc = self.translate(self.pc, AccessType::Instruction)?;
        match self.bus.load(p_pc, 32) {
            Ok(inst) => Ok(inst),
            Err(_e) => Err(Exception::InstructionAccessFault(self.pc)),
        }
    }


    #[inline]
    pub fn update_pc(&mut self) -> Result<(), Exception> {
        self.pc += 4;
        return Ok(());
    }

    /// Execute an instruction after decoding. Return true if an error happens, otherwise false.
    pub fn execute(&mut self, inst: u64) -> Result<(), Exception> {
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
                    _ => {
                        println!(
                            "not implemented yet: opcode {:#x} funct3 {:#x}",
                            opcode, funct3
                        );
                        return Err(Exception::IllegalInstruction(inst));
                    }
                }
            }
            0x0f => {
                // A fence instruction does nothing because this emulator executes an
                // instruction sequentially on a single thread.
                match funct3 {
                    0x0 => { // fence
                        return self.update_pc();
                    }
                    _ => {
                        println!(
                            "not implemented yet: opcode {:#x} funct3 {:#x}",
                            opcode, funct3
                        );
                        return Err(Exception::IllegalInstruction(inst));
                    }
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
                        self.regs[rd] = if (self.regs[rs1] as i64) < (imm as i64) {
                            1
                        } else {
                            0
                        };
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
                            _ => {
                                println!("go here");
                                return self.update_pc();
                            }
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
                    _ => {
                        println!("go here");
                        return self.update_pc();
                    }
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
                            _ => {
                                println!(
                                    "not implemented yet: opcode {:#x} funct7 {:#x}",
                                    opcode, funct7
                                );
                                return Err(Exception::IllegalInstruction(inst));
                            }
                        }
                    }
                    _ => {
                        println!(
                            "not implemented yet: opcode {:#x} funct3 {:#x}",
                            opcode, funct3
                        );
                        return Err(Exception::IllegalInstruction(inst));
                    }
                }
            }
            0x23 => {
                // imm[11:5|4:0] = inst[31:25|11:7]
                let imm = (((inst & 0xfe000000) as i32 as i64 >> 20) as u64) | ((inst >> 7) & 0x1f);
                let addr = self.regs[rs1].wrapping_add(imm);
                match funct3 {
                    0x0 => {self.store(addr, 8, self.regs[rs2])?; self.update_pc() },  // sb
                    0x1 => {self.store(addr, 16, self.regs[rs2])?; self.update_pc()}, // sh
                    0x2 => {self.store(addr, 32, self.regs[rs2])?; self.update_pc()}, // sw
                    0x3 => {self.store(addr, 64, self.regs[rs2])?; self.update_pc()}, // sd
                    _ => unreachable!(),
                }
            }
            0x2f => {
                // RV64A: "A" standard extension for atomic instructions
                let funct5 = (funct7 & 0b1111100) >> 2;
                let _aq = (funct7 & 0b0000010) >> 1; // acquire access
                let _rl = funct7 & 0b0000001; // release access
                match (funct3, funct5) {
                    (0x2, 0x00) => {
                        // amoadd.w
                        let t = self.load(self.regs[rs1], 32)?;
                        self.store(self.regs[rs1], 32, t.wrapping_add(self.regs[rs2]))?;
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    (0x3, 0x00) => {
                        // amoadd.d
                        let t = self.load(self.regs[rs1], 64)?;
                        self.store(self.regs[rs1], 64, t.wrapping_add(self.regs[rs2]))?;
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    (0x2, 0x01) => {
                        // amoswap.w
                        let t = self.load(self.regs[rs1], 32)?;
                        self.store(self.regs[rs1], 32, self.regs[rs2])?;
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    (0x3, 0x01) => {
                        // amoswap.d
                        let t = self.load(self.regs[rs1], 64)?;
                        self.store(self.regs[rs1], 64, self.regs[rs2])?;
                        self.regs[rd] = t;
                        return self.update_pc();
                    }
                    _ => {
                        println!(
                            "not implemented yet: opcode {:#x} funct3 {:#x} funct7 {:#x}",
                            opcode, funct3, funct7
                        );
                        return Err(Exception::IllegalInstruction(inst));
                    }
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
                        self.regs[rd] = if (self.regs[rs1] as i64) < (self.regs[rs2] as i64) {
                            1
                        } else {
                            0
                        };
                        return self.update_pc();
                    }
                    (0x3, 0x00) => {
                        // sltu
                        self.regs[rd] = if self.regs[rs1] < self.regs[rs2] {
                            1
                        } else {
                            0
                        };
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
                    _ => {
                        println!(
                            "not implemented yet: opcode {:#x} funct3 {:#x} funct7 {:#x}",
                            opcode, funct3, funct7
                        );
                        return Err(Exception::IllegalInstruction(inst));
                    }
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
                    (0x5, 0x01) => {
                        // divu
                        self.regs[rd] = match self.regs[rs2] {
                            0 => {
                                // TODO: Set DZ (Divide by Zero) in the FCSR csr flag to 1.
                                0xffffffff_ffffffff
                            }
                            _ => {
                                let dividend = self.regs[rs1];
                                let divisor = self.regs[rs2];
                                dividend.wrapping_div(divisor)
                            }
                        };
                        return self.update_pc();
                    }
                    (0x5, 0x20) => {
                        // sraw
                        self.regs[rd] = ((self.regs[rs1] as i32) >> (shamt as i32)) as u64;
                        return self.update_pc();
                    }
                    (0x7, 0x01) => {
                        // remuw
                        self.regs[rd] = match self.regs[rs2] {
                            0 => self.regs[rs1],
                            _ => {
                                let dividend = self.regs[rs1] as u32;
                                let divisor = self.regs[rs2] as u32;
                                dividend.wrapping_rem(divisor) as i32 as u64
                            }
                        };
                        return self.update_pc();
                    }
                    _ => {
                        println!(
                            "not implemented yet: opcode {:#x} funct3 {:#x} funct7 {:#x}",
                            opcode, funct3, funct7
                        );
                        return Err(Exception::IllegalInstruction(inst));
                    }
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
                            self.pc = self.pc.wrapping_add(imm);
                            return Ok(());
                        }
                        return self.update_pc();
                    }
                    0x1 => {
                        // bne
                        if self.regs[rs1] != self.regs[rs2] {
                            self.pc = self.pc.wrapping_add(imm);
                            return Ok(());
                        }
                        return self.update_pc();
                    }
                    0x4 => {
                        // blt
                        if (self.regs[rs1] as i64) < (self.regs[rs2] as i64) {
                            self.pc = self.pc.wrapping_add(imm);
                            return Ok(());
                        }
                        return self.update_pc();
                    }
                    0x5 => {
                        // bge
                        if (self.regs[rs1] as i64) >= (self.regs[rs2] as i64) {
                            self.pc = self.pc.wrapping_add(imm);
                            return Ok(());
                        }
                        return self.update_pc();
                    }
                    0x6 => {
                        // bltu
                        if self.regs[rs1] < self.regs[rs2] {
                            self.pc = self.pc.wrapping_add(imm);
                            return Ok(());
                        }
                        return self.update_pc();
                    }
                    0x7 => {
                        // bgeu
                        if self.regs[rs1] >= self.regs[rs2] {
                            self.pc = self.pc.wrapping_add(imm);
                            return Ok(());
                        }
                        return self.update_pc();
                    }
                    _ => {
                        println!(
                            "not implemented yet: opcode {:#x} funct3 {:#x}",
                            opcode, funct3
                        );
                        return Err(Exception::IllegalInstruction(inst));
                    }
                }
            }
            0x67 => {
                // jalr
                let t = self.pc + 4;

                let imm = ((((inst & 0xfff00000) as i32) as i64) >> 20) as u64;
                self.pc = (self.regs[rs1].wrapping_add(imm)) & !1;

                self.regs[rd] = t;
                return Ok(());
            }
            0x6f => {
                // jal
                self.regs[rd] = self.pc + 4;

                // imm[20|10:1|11|19:12] = inst[31|30:21|20|19:12]
                let imm = (((inst & 0x80000000) as i32 as i64 >> 11) as u64) // imm[20]
                    | (inst & 0xff000) // imm[19:12]
                    | ((inst >> 9) & 0x800) // imm[11]
                    | ((inst >> 20) & 0x7fe); // imm[10:1]

                self.pc = self.pc.wrapping_add(imm);
                return Ok(());
            }
            0x73 => {
                let csr_addr = ((inst & 0xfff00000) >> 20) as usize;
                match funct3 {
                    0x0 => {
                        match (rs2, funct7) {
                            (0x0, 0x0) => {
                                // ecall
                                // Makes a request of the execution environment by raising an
                                // environment call exception.
                                match self.mode {
                                    Mode::User => {
                                        self.update_pc()?;
                                        return Err(Exception::EnvironmentCallFromUMode(self.pc));
                                    }
                                    Mode::Supervisor => {
                                        self.update_pc()?;
                                        return Err(Exception::EnvironmentCallFromSMode(self.pc));
                                    }
                                    Mode::Machine => {
                                        self.update_pc()?;
                                        return Err(Exception::EnvironmentCallFromMMode(self.pc));
                                    }
                                }
                            }
                            (0x1, 0x0) => {
                                // ebreak
                                // Makes a request of the debugger bu raising a Breakpoint
                                // exception.
                                self.update_pc()?;
                                return Err(Exception::Breakpoint(self.pc));
                            }
                            (0x2, 0x8) => {
                                // sret
                                // The SRET instruction returns from a supervisor-mode exception
                                // handler. It does the following operations:
                                // - Sets the pc to CSRs[sepc].
                                // - Sets the privilege mode to CSRs[sstatus].SPP.
                                // - Sets CSRs[sstatus].SIE to CSRs[sstatus].SPIE.
                                // - Sets CSRs[sstatus].SPIE to 1.
                                // - Sets CSRs[sstatus].SPP to 0.
                                self.pc = self.load_csr(SEPC);
                                // When the SRET instruction is executed to return from the trap
                                // handler, the privilege level is set to user mode if the SPP
                                // bit is 0, or supervisor mode if the SPP bit is 1. The SPP bit
                                // is the 8th of the SSTATUS csr.
                                self.mode = match (self.load_csr(SSTATUS) >> 8) & 1 {
                                    1 => Mode::Supervisor,
                                    _ => Mode::User,
                                };
                                // The SPIE bit is the 5th and the SIE bit is the 1st of the
                                // SSTATUS csr.
                                self.store_csr(
                                    SSTATUS,
                                    if ((self.load_csr(SSTATUS) >> 5) & 1) == 1 {
                                        self.load_csr(SSTATUS) | (1 << 1)
                                    } else {
                                        self.load_csr(SSTATUS) & !(1 << 1)
                                    },
                                );
                                self.store_csr(SSTATUS, self.load_csr(SSTATUS) | (1 << 5));
                                self.store_csr(SSTATUS, self.load_csr(SSTATUS) & !(1 << 8));
                                return Ok(());
                            }
                            (0x2, 0x18) => {
                                // mret
                                // The MRET instruction returns from a machine-mode exception
                                // handler. It does the following operations:
                                // - Sets the pc to CSRs[mepc].
                                // - Sets the privilege mode to CSRs[mstatus].MPP.
                                // - Sets CSRs[mstatus].MIE to CSRs[mstatus].MPIE.
                                // - Sets CSRs[mstatus].MPIE to 1.
                                // - Sets CSRs[mstatus].MPP to 0.
                                self.pc = self.load_csr(MEPC);
                                // MPP is two bits wide at [11..12] of the MSTATUS csr.
                                self.mode = match (self.load_csr(MSTATUS) >> 11) & 0b11 {
                                    2 => Mode::Machine,
                                    1 => Mode::Supervisor,
                                    _ => Mode::User,
                                };
                                // The MPIE bit is the 7th and the MIE bit is the 3rd of the
                                // MSTATUS csr.
                                self.store_csr(
                                    MSTATUS,
                                    if ((self.load_csr(MSTATUS) >> 7) & 1) == 1 {
                                        self.load_csr(MSTATUS) | (1 << 3)
                                    } else {
                                        self.load_csr(MSTATUS) & !(1 << 3)
                                    },
                                );
                                self.store_csr(MSTATUS, self.load_csr(MSTATUS) | (1 << 7));
                                self.store_csr(MSTATUS, self.load_csr(MSTATUS) & !(0b11 << 11));
                                return Ok(());
                            }
                            (_, 0x9) => {
                                // sfence.vma
                                // Do nothing.
                                return self.update_pc();
                            }
                            _ => {
                                println!(
                                    "not implemented yet: opcode {:#x} funct3 {:#x} funct7 {:#x}",
                                    opcode, funct3, funct7
                                );
                                return Err(Exception::IllegalInstruction(inst));
                            }
                        }
                    }
                    0x1 => {
                        // csrrw
                        let t = self.load_csr(csr_addr);
                        self.store_csr(csr_addr, self.regs[rs1]);
                        self.regs[rd] = t;

                        self.update_paging(csr_addr);
                        return self.update_pc();
                    }
                    0x2 => {
                        // csrrs
                        let t = self.load_csr(csr_addr);
                        self.store_csr(csr_addr, t | self.regs[rs1]);
                        self.regs[rd] = t;

                        self.update_paging(csr_addr);
                        return self.update_pc();
                    }
                    0x3 => {
                        // csrrc
                        let t = self.load_csr(csr_addr);
                        self.store_csr(csr_addr, t & (!self.regs[rs1]));
                        self.regs[rd] = t;

                        self.update_paging(csr_addr);
                        return self.update_pc();
                    }
                    0x5 => {
                        // csrrwi
                        let zimm = rs1 as u64;
                        self.regs[rd] = self.load_csr(csr_addr);
                        self.store_csr(csr_addr, zimm);

                        self.update_paging(csr_addr);
                        return self.update_pc();
                    }
                    0x6 => {
                        // csrrsi
                        let zimm = rs1 as u64;
                        let t = self.load_csr(csr_addr);
                        self.store_csr(csr_addr, t | zimm);
                        self.regs[rd] = t;

                        self.update_paging(csr_addr);
                        return self.update_pc();
                    }
                    0x7 => {
                        // csrrci
                        let zimm = rs1 as u64;
                        let t = self.load_csr(csr_addr);
                        self.store_csr(csr_addr, t & (!zimm));
                        self.regs[rd] = t;

                        self.update_paging(csr_addr);
                        return self.update_pc();
                    }
                    _ => {
                        println!(
                            "not implemented yet: opcode {:#x} funct3 {:#x}",
                            opcode, funct3
                        );
                        return Err(Exception::IllegalInstruction(inst));
                    }
                }
            }
            _ => {
                dbg!(format!("not implemented yet: opcode {:#x}", opcode));
                return Err(Exception::IllegalInstruction(inst));
            }
        }
        // return Ok(());
    }
}
