GCC=riscv64-unknown-elf-gcc
LFLASG=-Wl,-Ttext=0x0
CFLAGS=-nostdlib 

OBJCOPY=riscv64-unknown-elf-objcopy
OBJFORMAT=-O binary


add-addi.bin: add-addi.s
	${GCC} ${LFLASG} ${CFLAGS} -o add-addi add-addi.s
	${OBJCOPY} ${OBJFORMAT} add-addi add-addi.bin

