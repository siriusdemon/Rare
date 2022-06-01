# Riscv MISA

### Endianness

The MBE, SBE, and UBE bits in mstatus and mstatush are WARL fields that control the endianness of memory accesses other than instruction fetches. Instruction fetches are always little-endian

### Machine Trap Delegation Registers (medeleg and mideleg)

In systems with S-mode, the medeleg and mideleg registers must exist, and setting a bit in medeleg or mideleg will delegate the corresponding trap, when occurring in S-mode or U-mode, to the S-mode trap handler. In systems without S-mode, the medeleg and mideleg registers should not
exist.

When a trap is delegated to S-mode, the scause register is written with the trap cause; the sepc register is written with the virtual address of the instruction that took the trap; the stval register is written with an exception-specific datum; the SPP field of mstatus is written with the active privilege mode at the time of the trap; the SPIE field of mstatus is written with the value of the SIE field at the time of the trap; and the SIE field of mstatus is cleared. The mcause, mepc, and mtval registers and the MPP and MPIE fields of mstatus are not written.

Traps never transition from a more-privileged mode to a less-privileged mode. By contrast, traps may be taken horizontally. 

### Machine Interrupt Registers (mip and mie)

Restricted views of the mip and mie registers appear as the sip and sie registers for supervisor level. If an interrupt is delegated to S-mode by setting a bit in the mideleg register, it becomes visible in the sip register and is maskable using the sie register. Otherwise, the corresponding bits in sip and sie are read-only zero.

### Supervisor Interrupt Registers (sip and sie)

The sip and sie registers are subsets of the mip and mie registers. Reading any implemented field, or writing any writable field, of sip/sie effects a read or write of the homonymous field of mip/mie.