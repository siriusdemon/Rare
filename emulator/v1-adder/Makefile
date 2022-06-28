CC=clang
LFLASG=-Wl,-Ttext=0x0
CFLAGS=-nostdlib --target=riscv64 -march=rv64g -mno-relax

OBJCOPY=llvm-objcopy
OBJFORMAT=-O binary


add-addi.bin: add-addi.s
	${CC} ${LFLASG} ${CFLAGS} -o add-addi add-addi.s
	${OBJCOPY} ${OBJFORMAT} add-addi add-addi.bin

