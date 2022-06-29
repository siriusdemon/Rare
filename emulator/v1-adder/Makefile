CC=clang
LFLASG=-Wl,-Ttext=0x0
CFLAGS=-nostdlib --target=riscv64 -march=rv64g -mno-relax

OBJCOPY=llvm-objcopy
OBJFORMAT=-O binary


add-addi.bin: add-addi.S
	${CC} ${LFLASG} ${CFLAGS} -o add-addi add-addi.S
	${OBJCOPY} ${OBJFORMAT} add-addi add-addi.bin

