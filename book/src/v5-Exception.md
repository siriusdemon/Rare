# Exception

In fact, we have already learned how to use `Exception`. In the preceding chapter, when `execute` encounters an illegal instruction, it returns an exception to the `main` function. Then the `main` function will print the exception, break the loop and finally exits. In this chapter, we will handle exception properly rather than just terminate the program.

The following text comes from the RISC-V unprivileged ISA:

In RISC-V hart, we use the term exception to refer to an unusual condition occurring at run time associated with an instruction in the current RISC-V hart. We use the term interrupt to refer to an external asynchronous event that may cause a RISC-V hart to experience an unexpected transfer of control.  We use the term trap to refer to the transfer of control to a trap handler caused by either an exception or an interrupt.

RISC-V also defines four types of trap. What we need here is the fatal trap. When we encounter a fatal trap, we will terminate the emulator.


事实上，我们已经用过异常了。在前面的章节中，当 execute 函数遇到一个非法指令时，它会返回一个异常给 main 函数。main 函数则会打印该异常。随后中止循环并退出。以下引用了 RISC-V 非特权架构文档的内容：

在一个 RISC-V hart 中，我们采用术语“异常”来表示当前这个 hart 在运行某一条指令时所遇到的异常状况。我们使用术语“中断”来表示一个能够引起 RISC-V hart 进行控制权转移的外部事件。使用“陷阱”（trap）表示由异常或者中断引起的控制权转移。在《手把手教你设计 RISC-V》一书中，作者用异常表示 trap，并用狭义异常表示 exception，用狭义中断表示 interrupt。

RISC-V 还定义了四种属性的 trap，我们只关注 fatal trap。当发生一个 fatal（严重的）的异常时，我们就退出模拟器程序。

### 1. Exception type

RISC-V has defined 14 exception types. When a trap is taken into M-mode, `mcause` is written with a code indicating the event that causes the trap, while `mtval` may be written with exception-specific information to assist software in handling the trap. Trap taken in S-mode is similar.

The cause registers contain an interrupt bit and a 15-bit exception code. The interrupt bit is set when the trap is caused by an interrupt. We will talk more about interrupt in next three chapters.

RISC-V 定义了 14 种异常类型。当陷入 M 模式的时候，造成该 trap 的异常编码会被写进 mcause，同时根据异常的类型，mtval 可能会写入一些辅助的信息以帮助软件 处理该异常。陷入 S 模型时也是相似的。 

![cause register](./images/mcause-scause.png)
<p class="comment">mcause or scause register. From RISC-V Privileged<p>

![exception](./images/exception.png)
<p class="comment">Exception table. From RISC-V Privileged<p>

For trap value register, RISC-V defines following rules:
+ If stval or mtval is written with a nonzero value when a breakpoint, address-misaligned, access-fault, or page-fault exception occurs on an instruction fetch, load, or store, then stval will contain the faulting virtual address.  
+ If stval or mtval is written with a nonzero value when a misaligned load or store causes an access-fault or page-fault exception, then stval will contain the virtual address of the portion of the access that caused the fault
+ The stval and mtval register can optionally also be used to return the faulting instruction bits on an illegal instruction exception.

对于 mtval 寄存器，RISC-V 规定：

+ 当进行取指和访存时，若发生断点、地址不对齐、访问错误或者页错误等异常，则将发生异常的那个虚拟地址写入 mtval。
+ 当一个没对齐的访存造成访问错误或者页错误的时候，则将发生异常的那个虚拟地址写入 mtval。
+ 在遇到非法指令时，mtval 还可以用于保存该非法指令的值。

对于 stval 也是一样的。

### 2. Exception Delegation

By default, all traps at any privilege level are handled in machine mode, though a machine-mode handler can redirect traps back to the appropriate level with the MRET instruction. To increase performance, implementations can provide individual read/write bits within `medeleg` and `mideleg` to indicate that certain exceptions and interrupts should be processed directly by a lower privilege level. 

In systems with S-mode, the medeleg and mideleg registers must exist, and setting a bit in `medeleg` or `mideleg` will delegate the corresponding trap, when occurring in S-mode or U-mode, to the S-mode trap handler.

`medeleg` has a bit position allocated for every synchronous exception shown in the Exception Table above, with the index of the bit position equal to the value returned in the `mcause` register.

Refer to Section 3.1.8 of RISC-V Privileged for more details.

默认情况下，所有的 trap 都是在机器模式下处理的。虽然机器模式的处理程序可以通过 mret 指令将 trap 重定向至其他合适的特权等级。但是，这样的性能不如直接通过在`medeleg`和`mideleg`寄存器中设置特定的位，将异常和中断直接在低等级的模式（通常是 S 模式）中处理。

对于一个实现了 S 模式的系统，`medeleg`和`mideleg`寄存器必须存在。在其中设置特定的位，将会使得在 S 模式或者 U 模型中发生的相应的 trap 交由 S 模式进行处理。

`medeleg` 是一个 64 位的寄存器，每一个位对应于上面异常类型表格中的一种异常。欲知详情，可以查看 RISC-V 特权架构文档。

### 3. Exception Implementation

Let's take a close look at the `exception.rs`, which have stayed in our src directory for a long time.

现在我们可以看下 `exception.rs`，这份代码已经存在许久了。

<p class="filename">exception.rs</p>

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

The `trap value` captured by each exception will be stored in the `stval` or `mtval`, which may have different meaning depends on the type of exception as we have mentioned above.

每个异常都会带一个异常值，这个值将会被写进 stval 或者 mtval。

We implement the `value` function to return the trap value and the `code` function to return the exception code. We have also provided a function `is_fatal`, which determines whether the exception is fatal.

我们实现了三个函数：value，code，is_fatal，分别用于返回异常值，异常代码以及该异常是否为严重异常。

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

    pub fn code(self) -> u64 {
        match self {
            InstructionAddrMisaligned(_) => 0,
            InstructionAccessFault(_) => 1,
            IllegalInstruction(_) => 2,
            Breakpoint(_) => 3,
            LoadAccessMisaligned(_) => 4,
            LoadAccessFault(_) => 5,
            StoreAMOAddrMisaligned(_) => 6,
            StoreAMOAccessFault(_) => 7,
            EnvironmentCallFromUMode(_) => 8,
            EnvironmentCallFromSMode(_) => 9,
            EnvironmentCallFromMMode(_) => 11,
            InstructionPageFault(_) => 12,
            LoadPageFault(_) => 13,
            StoreAMOPageFault(_) => 15,
        }
    }

    pub fn is_fatal(self) -> bool {
        match self {
            InstructionAddrMisaligned(_)
            | InstructionAccessFault(_)
            | LoadAccessFault(_)
            | StoreAMOAddrMisaligned(_)
            | StoreAMOAccessFault(_) 
            | IllegalInstruction(_) => true,
            _else => false,
        }
    }
}
```

### 4. Handle exception in CPU

We summarize the whole procedure of handling exception as following:

1. update hart's privilege mode (M or S according to current mode and exception setting).
2. save current pc in epc (sepc in S-mode, mepc in M-mode)
3. set pc to trap vector (stvec in S-mode, mtvec in M-mode)
4. set cause register with exception code (scause in S-mode, mcause in M-mode)
5. set trap value properly (stval in S-mode, mtval in M-mode)
6. set xPIE to xIE (SPIE in S-mode, MPIE in M-mode)
7. clear up xIE (SIE in S-mode, MIE in M-mode)
8. set xPP to previous mode.

The translation is straightforward.

我们将处理异常的流程总结如下

1. 更新 hart 的特权模式
2. 将 pc 保存到 sepc 或者 mepc
3. 设置 pc 的值为 stvec 或者 mtvec
4. scause 或者 mcause 保存异常代码
5. 设置 stval 或者 mtval
6. 令 xIE = xPIE
7. 清零 xIE
8. xPP 保存进入异常前的特权模式

翻译成代码如下：

<p class="filename">cpu.rs</p>

```rs
impl Cpu {
    pub fn handle_exception(&mut self, e: Exception) {
        let pc = self.pc; 
        let mode = self.mode;
        let cause = e.code();
        // if an exception happen in U-mode or S-mode, and the exception is delegated to S-mode.
        // then this exception should be handled in S-mode.
        let trap_in_s_mode = mode <= Supervisor && self.csr.is_medelegated(cause);
        let (STATUS, TVEC, CAUSE, TVAL, EPC, MASK_PIE, pie_i, MASK_IE, ie_i, MASK_PP, pp_i) 
            = if trap_in_s_mode {
                self.mode = Supervisor;
                (SSTATUS, STVEC, SCAUSE, STVAL, SEPC, MASK_SPIE, 5, MASK_SIE, 1, MASK_SPP, 8)
            } else {
                self.mode = Machine;
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
        self.csr.store(TVAL, e.value());
        // 3.1.6 covers both sstatus and mstatus.
        let mut status = self.csr.load(STATUS);
        // get SIE or MIE
        let ie = (status & MASK_IE) >> ie_i;
        // set SPIE = SIE / MPIE = MIE
        status = (status & !MASK_PIE) | (ie << pie_i);
        // set SIE = 0 / MIE = 0
        status &= !MASK_IE; 
        // set SPP / MPP = previous mode
        status = (status & !MASK_PP) | (mode << pp_i);
        self.csr.store(STATUS, status);
    }
}
```

Finally, we update the loop in `main` function as following:

最后，我们需要更新主函数中的循环。


<p class="filename">main.rs</p>

```rs
fn main() -> io::Result<()> {
    // ...
    loop {
        let inst = match cpu.fetch() {
            // Break the loop if an error occurs.
            Ok(inst) => inst,
            Err(e) => {
                cpu.handle_exception(e);
                if e.is_fatal() {
                    println!("{}", e);
                    break;
                }
                continue;
            }
        };
        match cpu.execute(inst) {
            // Break the loop if an error occurs.
            Ok(new_pc) => cpu.pc = new_pc,
            Err(e) => {
                cpu.handle_exception(e);
                if e.is_fatal() {
                    println!("{}", e);
                    break;
                }
            }
        };
    }
}
```


### 5. Conclusion

In this chapter, we learn the exception type in RISC-V and the full story of exception handling. Now our emulator is able to handle exceptions, though the valuable exception we can handle only happen in the last chapter, which introduce virtual memory system and page table. From the next chapter, we will gradually introduce several devices and complete the interrupt handling.

在一章中，我们学习了RISC-V 的异常类型以及处理异常的全过程。现在我们的模拟器已经可以处理异常了。从下一章开始，我们会逐步介绍几个外设，并完善中断的处理。