# Riscv 模拟器系列教程

本教程基于[Asami](https://github.com/d0iasm) 所写的模拟器[教程](https://book.rvemu.app/)。虽然作者只写到第三章，但她已经事先将所有的代码划分成了十个章节。所以看着代码也能够一步一步地构建出这个模拟器。

最终的模拟器可以运行 [xv6](https://pdos.csail.mit.edu/6.828/2012/xv6.html) 操作系统。


我的计划是：跟着她的代码和教程一步一步地做出这个模拟器，然后写一个系列完整的中文教程加以说明。

### 前置

本教程假设读者已经对 Riscv 有一定的了解，因此教程中不会对 Riscv 作过多的解释。推荐通过阅读以下资料来了解 Riscv。

+ [Riscv 标准](https://riscv.org/technical/specifications/)
+ [Riscv Reader](https://zh.webbooksnow.art/dl/16429281/d4417e)

教程中使用的很多插图来自 Riscv Reader.

### 环境

+ Linux / WSL

我们需要用到 gcc 的工具来生成测试的二进制文件。`ubuntu` 下安装

```sh
sudo apt install binutils-riscv64-unknown-elf gcc-riscv64-unknown-elf
```

### 目录

1. [加法器 CPU](./v1-CPU-Adder.md)
2. [内存和总线](./v2-Memory-and-Bus.md)
3. [csrs寄存器](./v3-csrs.md)
4. [特权模式](./v4-privileged-mode.md)
5. [异常](./v5-exceptions.md)
6. [PLIC & CLINT](./v6-plic-clint.md)
7. [UART](./v7-uart.md)
8. [中断](./v8-interrupts.md)
9. [Virtio](./v9-virtio.md)
10. [虚拟内存系统](./v10-virtual-memory-system.md)

说明: 原作者划分了硬件和 ISA 指令集两部分内容，我觉得合并成一个更适合，所以进行了合并。

