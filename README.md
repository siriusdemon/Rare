# Rare: Rust A Riscv Emulator
RISC-V 模拟器教程

This tutorial is based on [Asami's excellent tutorial](https://book.rvemu.app). Although the author haven't finished it, she have already separated the code into 10 stages, which makes this tutorial become possible.

When you complete this tutorial, the emulator is able to run xv6, a UNIX-like operation system running on RISC-V.

I am planning to follow her code to build the emulator step by step. When finished, I will write a complete tutorial to help reader to get the details. My tutorial will use the same tool `mdbook` as the original author, to build.

+ Project: [Github Rare](https://github.com/siriusdemon/Rare)
+ Tutorial: [Github.io Rare](https://siriusdemon.github.io/Rare/)

### Prerequisite

This tutorial assumes readers already have been familiar with `Rust` and `RISC-V`. If not, you might want to read the following materials to learn about `RISC-V`.

+ [RISC-V Specifications](https://riscv.org/technical/specifications/)
+ [RISC-V Reader](https://zh.webbooksnow.art/dl/16429281/d4417e)
+ [手把手教你设计RISC-V处理器](https://zh.webbooksnow.art/book/18067855/bd7a8a)

For `Rust`, you can read `the book` after you have installed the toolchain. Open your terminal and type `rustup docs`, your browser will open a new page for you to navigate to `the book` and other docs.


### Develop envrionment

+ Linux / WSL

We need the `clang` toolchain to generate some files used in testing. You can download the precompiled version from [here]((https://releases.llvm.org/)). The version I used is [clang-12](https://github.com/llvm/llvm-project/releases/tag/llvmorg-12.0.0).


### How to use

+ clone this project
+ use `cd Rare/book && mdbook serve` to open this tutorial locally
+ use `git pull` to update when needed

### Catelogue

1. [x] [Adder 加法器](./book/src/v1-CPU-Adder.md)
2. [x] [Memory and Bus 内存和总线](./book/src/v2-Memory-and-Bus.md)
3. [x] [Control Status Register 控制状态寄存器](./book/src/v3-CSR.md)
4. [x] [Privilege Mode 特权模式](./book/src/v4-Privilege-Mode.md)
5. [x] [Exception 异常](./book/src/v5-Exceptions.md)
6. [x] [PLIC & CLINT](./book/src/v6-Plic-Clint.md)
7. [x] [UART](./book/src/v7-Uart.md)
8. [x] [Interrupt 中断](./book/src/v8-Interrupts.md)
9. [x] [Virtio](./book/src/v9-Virtio.md)
10. [x] [Page Table 页表](./book/src/v10-Page-Table.md)

The original author separate the tutorial into two parts: Hardware and ISA. I have merged them here. 


### Note

When you travel through this tutorial, sometimes, you may notice some code in current chapter is different from last one's. This is because I will do some refactor when needed. Welcome to open an issue on github if you have any questions.