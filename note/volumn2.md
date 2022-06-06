# Volumn 2

### Machine Trap Delegation Registers (medeleg and mideleg)

In systems with S-mode, the medeleg and mideleg registers must exist, and setting a bit in medeleg or mideleg will delegate the corresponding trap, when occurring in S-mode or U-mode, to the S-mode trap handler. In systems without S-mode, the medeleg and mideleg registers should not
exist.

When a trap is delegated to S-mode, the scause register is written with the trap cause; the sepc register is written with the virtual address of the instruction that took the trap; the stval register is written with an exception-specific datum; the SPP field of mstatus is written with the active privilege mode at the time of the trap; the SPIE field of mstatus is written with the value of the SIE field at the time of the trap; and the SIE field of mstatus is cleared. The mcause, mepc, and mtval registers and the MPP and MPIE fields of mstatus are not written.

Traps never transition from a more-privileged mode to a less-privileged mode. By contrast, traps may be taken horizontally. 

### Machine Interrupt Registers (mip and mie)

Restricted views of the mip and mie registers appear as the sip and sie registers for supervisor level. If an interrupt is delegated to S-mode by setting a bit in the mideleg register, it becomes visible in the sip register and is maskable using the sie register. Otherwise, the corresponding bits in sip and sie are read-only zero.

### Supervisor Interrupt Registers (sip and sie)

The sip and sie registers are subsets of the mip and mie registers. Reading any implemented field, or writing any writable field, of sip/sie effects a read or write of the homonymous field of mip/mie.

### 3.3.1 Environment Call and Breakpoint

ECALL and EBREAK cause the receiving privilege mode’s epc register to be set to the address of the ECALL or EBREAK instruction itself, not the address of the following instruction. As ECALL and EBREAK cause synchronous exceptions, they are not considered to retire, and should not increment the minstret CSR

### 3.3.2 Trap-Return Instructions

An xRET instruction can be executed in privilege mode x or higher, where executing a lower-privilege xRET instruction will pop the relevant lower-privilege interrupt enable and privilege mode stack. In addition to manipulating the privilege stack as described in Section 3.1.6.1, xRET sets the pc to the value stored in the xepc register.

### 3.1.6 Machine Status Registers (mstatus and mstatush)

The mstatus register keeps track of and controls the hart’s current operating state. A restricted view of mstatus appears as the sstatus register in the S-level ISA

+ MIE / SIE: global interrupt enable for M-mode and S-mode respectively. When in x mode, higher mode y interrupt are enable regardless of the yIE bit, while lower mode z interrupt are disable regardless of zIE bit. Higher mode y can using yIE register to disable certain interrupts before enter a lower mode.

+ xPIE / xPP: xPIE holds previous interrupt-enable bit before enter the trap. xPP holds previous privilege mode. When a trap is taken from privilege mode y into privilege mode x, xPIE is set to the value of xIE; xIE is set to 0; and xPP is set to y.

```rs
assume current privilege mode is y
when trap from y into x cause:
xPIE = xIE
xIE = 0 (efficiently disable interrupt upon entry)
xPP = y
```

An MRET or SRET instruction is used to return from a trap in M-mode or S-mode respectively.

```rs
assume current privilege mode is x and xPP = y
when execute an xRET cause:
xIE = xPIE
privilege mode = xPP (aka. y)
xPIE = 1
xPP = least-privileged supported mode (U or M)
MPRV = 0 if xPP != M
```

+ SXL / UXL: only in RV64, control UXLEN, SXLEN. In RV32, UXLEN = SXLEN = 32.
+ MPRV:  read-only if U-mode is not supported. When MPRV = 1, load and store memory addresses are translated and protected, and endianness is applied, as though the current privilege mode were set to MPP. Instruction address-translation and protection are unaffected by the setting of MPRV. 

+ MXR: When MXR = 1, load readable and executable virtual memory will succeed. When MXR = 0, only load readable will succeed. Read-only if S-mode is not supported.

+ SUM: allow S-mode memory accesses to pages that are accessible by U-mode.
+ MBE / SBE / UBE: control the endianness of memory accesses other than instruction fetches. Instruction fetches are always little-endian.

### 4.1.7 Supervisor Exception Program Counter (sepc)

When a trap is taken into S-mode, sepc is written with the virtual address of the instruction that was interrupted or that encountered the exception. Otherwise, sepc is never written by the implementation, though it may be explicitly written by software.


