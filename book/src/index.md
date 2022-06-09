# Riscv 模拟器系列教程

本教程基于[Asami](https://github.com/d0iasm) 所写的模拟器[教程](https://book.rvemu.app/)。虽然作者只写到第三章，但她已经事先将所有的代码划分成了十个章节。所以看着代码也能够一步一步地构建出这个模拟器。

最终的模拟器可以运行 [xv6](https://pdos.csail.mit.edu/6.828/2012/xv6.html) 操作系统。


我的计划是：跟着她的代码和教程一步一步地做出这个模拟器，然后写一个系列完整的中文教程加以说明。该教程与原作一样，使用[mdbook](https://github.com/rust-lang/mdBook)构建。

+ 项目地址：[Github Rare](https://github.com/siriusdemon/Rare)
+ 在线教程：[Github.io Rare](https://siriusdemon.github.io/Rare/)

推荐的使用方法：

+ clone 该项目到本地
+ cd Rare/book && mdbook serve 打开本地教程
+ 需要的时候，使用 git pull 更新

### 前置

本教程假设读者已经对 Rust 和 Riscv 有一定的了解，因此教程中不会对 Rust & Riscv 作过多的解释，而是专注于模拟器本身。推荐通过阅读以下资料来了解 Riscv。

+ [Riscv 标准](https://riscv.org/technical/specifications/)
+ [Riscv Reader](https://zh.webbooksnow.art/dl/16429281/d4417e)
+ [手把手教你设计RISC-V处理器](https://zh.webbooksnow.art/book/18067855/bd7a8a)

至于 Rust，安装好环境之后，可以通过运行 `rustup docs` 来访问`the book`以及 Rust 文档。


### 环境

+ Linux / WSL

我们需要用到 clang 的工具来生成测试的二进制文件，可以从[LLVM](https://releases.llvm.org/)官网下载预编译版本。我使用的版本是 [clang-12](https://github.com/llvm/llvm-project/releases/tag/llvmorg-12.0.0)，更新的版本应该也可以。



### 目录

1. [x] [加法器 CPU](./v1-CPU-Adder.md)
2. [x] [内存和总线](./v2-Memory-and-Bus.md)
3. [x] [控制状态寄存器](./v3-CSR.md)
4. [ ] [特权模式](./v4-privilege-mode.md)
5. [ ] [异常](./v5-exceptions.md)
6. [ ] [PLIC & CLINT](./v6-plic-clint.md)
7. [ ] [UART](./v7-uart.md)
8. [ ] [中断](./v8-interrupts.md)
9. [ ] [Virtio](./v9-virtio.md)
10. [ ] [虚拟内存系统](./v10-virtual-memory-system.md)

说明: 原作者划分了硬件和 ISA 指令集两部分内容，我觉得合并成一个更适合，所以进行了合并。


### 特别说明

实践的过程中，读者可能会发现教程上的代码与实际的代码有些不同。这是因为我在编写的过程中会适当地进行重构。如果教程本身写得早，重构之后，就需要对教程进行修改。有时候我会遗漏要修改的地方，有时候则是觉得不必提及，读者看源码即能理解。

如有任何疑惑，欢迎在项目上提 issue。