# Exception

In last chapter, we have learned that how RISC-V hart return from traps in certain privilege mode. But we still don't know the condition which causes a hart take traps. In RISC-V, there are two conditions: exception and interrupt can triggle that.

The following text comes from the RISC-V unprivileged ISA:

In RISC-V hart, we use the term exception to refer to an unusual condition occurring at run time associated with an instruction in the current RISC-V hart. We use the term interrupt to refer to an external asynchronous event that may cause a RISC-V hart to experience an unexpected transfer of control.  We use the term trap to refer to the transfer of control to a trap handler caused by either an exception or an interrupt.

RISC-V also defines four types of trap. What we need here is the fatal trap. When we encounter a fatal trap, we will terminate the emulator.

### 1. Exception type

We have already learned how to use Exception. In the preceding chapter, when `execute` encounters an illegal instruction, it returns an exception to the `main` function. Then the `main` function will print the exception, break the loop and finally exits. 

![exception](./images/exception.png)

<p class="comment">Exception type and code. From RISC-V Privileged<p>

Let's take a close look at the `exception.rs`.

<p class="filename">exception.rs<p>

```rs
#[derive(Debug, Copy, Clone)]
pub enum Exception {
    InstructionAddrMisaligned(u64),
    InstructionAccessFault(u64),
    IllegalInstruction(u64),
    Breakpoint(u64),
    LoadAccessMisaligned(u64),
    LoadAccessFault(u64),
    StoreAMOAddrMisaligned(u64),
    StoreAMOAccessFault(u64),
    EnvironmentCallFromUMode(u64),
    EnvironmentCallFromSMode(u64),
    EnvironmentCallFromMMode(u64),
    InstructionPageFault(u64),
    LoadPageFault(u64),
    StoreAMOPageFault(u64),
}
```

The `trap value` captured by each exception will be stored in the `stval` or `mtval`, which may have different meaning depends on the type of exception. The RISC-V Specification defines following rules:

+ If stval or mtval is written with a nonzero value when a breakpoint, address-misaligned, access-fault, or page-fault exception occurs on an instruction fetch, load, or store, then stval will contain the faulting virtual address.  
+ If stval or mtval is written with a nonzero value when a misaligned load or store causes an access-fault or page-fault exception, then stval will contain the virtual address of the portion of the access that caused the fault
+ The stval and mtval register can optionally also be used to return the faulting instruction bits on an illegal instruction exception.

We implement the `value` function to return the trap value.

<p class="filename">exception.rs</p>

```rs
impl Exception {
    pub fn value(self) -> u64 {
        match self {
            InstructionAddrMisaligned(addr) => addr,
            InstructionAccessFault(addr) => addr,
            IllegalInstruction(inst) => inst,
            Breakpoint(pc) => pc,
            LoadAccessMisaligned(addr) => addr,
            LoadAccessFault(addr) => addr,
            StoreAMOAddrMisaligned(addr) => addr,
            StoreAMOAccessFault(addr) => addr,
            EnvironmentCallFromUMode(pc) => pc,
            EnvironmentCallFromSMode(pc) => pc,
            EnvironmentCallFromMMode(pc) => pc,
            InstructionPageFault(addr) => addr,
            LoadPageFault(addr) => addr,
            StoreAMOPageFault(addr) => addr,
        }
    }
}
```







We have also implemented the `Display` trap for debug.

<p class="filename">exception.rs</p>

```rs
use Exception::*;
impl fmt::Display for Exception {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstructionAddrMisaligned(addr) => write!(f, "Instruction address misaligned {:#x}", addr),
            InstructionAccessFault(addr) => write!(f, "Instruction access fault {:#x}", addr),
            IllegalInstruction(inst) => write!(f, "Illegal instruction {:#x}", inst),
            Breakpoint(pc) => write!(f, "Breakpoint {:#x}", pc),
            LoadAccessMisaligned(addr) => write!(f, "Load access {:#x}", addr),
            LoadAccessFault(addr) => write!(f, "Load access fault {:#x}", addr),
            StoreAMOAddrMisaligned(addr) => write!(f, "Store or AMO address misaliged {:#x}", addr),
            StoreAMOAccessFault(addr) => write!(f, "Store or AMO access fault {:#x}", addr),
            EnvironmentCallFromUMode(pc) => write!(f, "Environment call from U-mode {:#x}", pc),
            EnvironmentCallFromSMode(pc) => write!(f, "Environment call from S-mode {:#x}", pc),
            EnvironmentCallFromMMode(pc) => write!(f, "Environment call from M-mode {:#x}", pc),
            InstructionPageFault(addr) => write!(f, "Instruction page fault {:#x}", addr),
            LoadPageFault(addr) => write!(f, "Load page fault {:#x}", addr),
            StoreAMOPageFault(addr) => write!(f, "Store or AMO page fault {:#x}", addr),
        }
    }
}
```



