# Chapter 1

## Generate the binary file

```sh
riscv-[anything]-as -o addi-add addi-add.s
riscv-[anything]-objcopy -O binary addi-add addi-add.bin
```

in My machine [WSL ubuntu20.04], it happens to

```sh
riscv64-linux-gnu-as -o addi-add addi-add.s
riscv64-linux-gnu-objcopy -O binary addi-add addi-add.bin
```
