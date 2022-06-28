# Rare: Rust A RISC-V Emulator
RISC-V 模拟器教程

This tutorial is based on [Asami's excellent tutorial](https://book.rvemu.app). Although the author haven't finished it, she have already separated the code into 10 stages, which makes this tutorial become possible.

When you complete this tutorial, the emulator is able to run xv6, a UNIX-like operation system running on RISC-V.

I am planning to follow her code build the emulator step by step. When finished, I will write a complete tutorial to help reader to get the details. My tutorial will use the same tool `mdbook` as the original author, to build.

+ Project: [Github Rare](https://github.com/siriusdemon/Rare)
+ Tutorial: [Github.io Rare](https://siriusdemon.github.io/Rare/)

本教程基于[Asami](https://github.com/d0iasm) 所写的模拟器[教程](https://book.rvemu.app/)。虽然作者只写到第三章，但她已经事先将所有的代码划分成了十个章节。所以看着代码也能够一步一步地构建出这个模拟器。

最终的模拟器可以运行 [xv6](https://pdos.csail.mit.edu/6.828/2012/xv6.html) 操作系统。

我的计划是：跟着她的代码和教程一步一步地做出这个模拟器，然后写一个系列完整的中文教程加以说明。该教程与原作一样，使用[mdbook](https://github.com/rust-lang/mdBook)构建。

+ 项目地址：[Github Rare](https://github.com/siriusdemon/Rare)
+ 在线教程：[Github.io Rare](https://siriusdemon.github.io/Rare/)

### Prerequisite
前置

This tutorial assumes readers already have been familiar with `Rust` and `RISC-V`. If not, you might want to read the following materials to learn about `RISC-V`.

本教程假设读者已经对 Rust 和 Riscv 有一定的了解，因此教程中不会对 Rust & Riscv 作过多的解释，而是专注于模拟器本身。推荐通过阅读以下资料来了解 Riscv。

+ [RISC-V Specifications](https://riscv.org/technical/specifications/)
+ [RISC-V Reader](https://zh.webbooksnow.art/dl/16429281/d4417e)
+ [手把手教你设计RISC-V处理器](https://zh.webbooksnow.art/book/18067855/bd7a8a)

For `Rust`, you can read `the book` after you have installed the toolchain. Open your terminal and type `rustup docs`, your browser will open a new page for you to navigate to `the book` and other docs.

至于 Rust，安装好环境之后，可以通过运行 `rustup docs` 来访问`the book`以及 Rust 文档。


### Develop envrionment
开发环境

+ Linux / WSL

We nned the `clang` toolchain to generate some files used in testing. You can download the precompiled version from [here]((https://releases.llvm.org/)). The version I used is [clang-12](https://github.com/llvm/llvm-project/releases/tag/llvmorg-12.0.0).

我们需要用到 clang 的工具来生成测试的二进制文件，可以从[LLVM](https://releases.llvm.org/)官网下载预编译版本。我使用的版本是 [clang-12](https://github.com/llvm/llvm-project/releases/tag/llvmorg-12.0.0)，更新的版本应该也可以。


### How to use

+ clone this project
+ use `cd Rare/book && mdbook serve` to open this tutorial locally
+ use `git pull` to update when needed


推荐的使用方法
+ clone 该项目到本地
+ cd Rare/book && mdbook serve 打开本地教程
+ 需要的时候，使用 git pull 更新


### Catelogue
目录

1. [x] [Adder 加法器](./v1-CPU-Adder.md)
2. [x] [Memory and Bus 内存和总线](./v2-Memory-and-Bus.md)
3. [x] [Control Status Register 控制状态寄存器](./v3-CSR.md)
4. [ ] [Privilege Mode 特权模式](./v4-privilege-mode.md)
5. [ ] [Exception 异常](./v5-exceptions.md)
6. [ ] [PLIC & CLINT](./v6-plic-clint.md)
7. [ ] [UART](./v7-uart.md)
8. [ ] [Interrupt 中断](./v8-interrupts.md)
9. [ ] [Virtio](./v9-virtio.md)
10. [ ] [Page Table 页表](./v10-page-table.md)

The original author separate the tutorial into two parts: Hardware and ISA. I have merged them here. 

原作者划分了硬件和 ISA 指令集两部分内容，我觉得合并成一个更适合，所以进行了合并。

### Note

When you travel through this tutorial, sometimes, you may notice some code in current chapter is different from last one's. This is because I will do some refactor when needed. Welcome to open an issue on github if you have any questions.

实践的过程中，读者可能会发现本章的部分代码与上一章的不一样。这是因为我在编写的过程中会适当地进行重构。如有任何疑惑，欢迎在项目上提 issue。