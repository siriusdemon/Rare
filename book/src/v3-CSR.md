# 控制状态寄存器

RISC-V 为每一个 hart 定义了一个独立的控制状态寄存器的地址空间，有 4096 个之多。其中已经分配的地址相对较少，具体可参见 RISC-V 标准卷二的说明。

下表基本包含了我们这个小项目需要用到的（也就是 xv6 所需要的）寄存器。

![misa-csr](./images/misa-csr.png)
<p class="comment">图片来自 RISC-V 卷2</p>

![sisa-csr](./images/sisa-csr.png)
<p class="comment">图片来自 RISC-V 卷2</p>


这些寄存器的具体意义可参考 RISC-V 标准中的说明，故不赘述。唯一值得一提的是，`sie`和`sip`是`mie`和`mip`的子集。实际上并不存在`sie`和`sip`这两个寄存器。

标准中说：通过在`mideleg`中设置特定的位，将一个中断从 M-mode 代理给 S-mode，则这个中断在`sip`中可见且可被 `sie`屏蔽。反之，该位在`sip`和`sie`中皆为 0。此外，读写`sip`和`sie`相当于读写`mip`和`mie`。它们的关系可表述为

```rs
sip == mip & mideleg 
sie == mie & mideleg
```

### 添加 CSRs

根据上表，我们先录入所需要的寄存器地址。

<p class="filename">cpu.rs</p>

```rs
// Machine-level CSRs.
pub const MHARTID: usize = 0xf14;
pub const MSTATUS: usize = 0x300;
pub const MEDELEG: usize = 0x302;
pub const MIDELEG: usize = 0x303;
pub const MIE: usize = 0x304;
pub const MTVEC: usize = 0x305;
pub const MCOUNTEREN: usize = 0x306;
pub const MSCRATCH: usize = 0x340;
pub const MEPC: usize = 0x341;
pub const MCAUSE: usize = 0x342;
pub const MTVAL: usize = 0x343;
pub const MIP: usize = 0x344;

// Supervisor-level CSRs.
pub const SSTATUS: usize = 0x100;
pub const SIE: usize = 0x104;
pub const STVEC: usize = 0x105;
pub const SSCRATCH: usize = 0x140;
pub const SEPC: usize = 0x141;
pub const SCAUSE: usize = 0x142;
pub const STVAL: usize = 0x143;
pub const SIP: usize = 0x144;
pub const SATP: usize = 0x180;
```


CPU 需要开辟一个 4096 的地址空间，同时，为了模拟`sie`和`sip`，我们还需要定义两个辅助函数（这是原作者的设计）。

<p class="filename">cpu.rs</p>

```rs
pub struct Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    pub bus: Bus,
    pub csrs: [u64; 4096],
}

impl Cpu {
    // new 函数也需要相应修改
    // ...
    pub fn load_csr(&self, addr: usize) -> u64 {
        match addr {
            SIE => self.csrs[MIE] & self.csrs[MIDELEG],
            SIP => self.csrs[MIP] & self.csrs[MIDELEG],
            _ => self.csrs[addr],
        }
    }

    pub fn store_csr(&mut self, addr: usize, value: u64) {
        match addr {
            SIE => self.csrs[MIE] = (self.csrs[MIE] & !self.csrs[MIDELEG]) | (value & self.csrs[MIDELEG]),
            SIP => self.csrs[MIP] = (self.csrs[MIP] & !self.csrs[MIDELEG]) | (value & self.csrs[MIDELEG]),
            _ => self.csrs[addr] = value,
        }
    }
}
```

当我们读取`sie`时，我们读的是`mie`与`mideleg`相与的结果，当我们写`sie`时，我们同样只写`mideleg`中为`1`的位，其他的位保持不变。读写`sip`与此类似。


### CSR 指令

我们已经为 CPU 添加了 CSR 地址空间，现在需要支持执行 CSR 指令。CSR 的指令共有 6 个。

![csr-inst](./images/csr-inst.png)

<p class="comment">图片来自 RISC-V Volumn 1: Zicsr</p>

指令的`csr`字段有 12 位，编码的是寄存器的地址。(2^12 = 4096)。指令的含义如下：

![csr-inst1](./images/csr-inst1.png)
![csr-inst2](./images/csr-inst2.png)
![csr-inst3](./images/csr-inst3.png)


<p class="comment">图片来自 RISC-V Reader</p>


可以自行实现，也可以复制项目中的源码。实现以上六个指令之后，可以进行下面的测试。

<p class="filename">cpu.rs</p>

```rs
mod test {
    // ...
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
                                            "sstatus" => 4, "stvec" => 5, "sepc" => 6);
    }
}
```