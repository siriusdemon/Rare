fib.bin: fib.c
	riscv64-unknown-elf-gcc -S fib.c
	riscv64-unknown-elf-gcc -Wl,-Ttext=0x0 -nostdlib -march=rv64i -mabi=lp64 -o fib fib.s
	riscv64-unknown-elf-objcopy -O binary fib fib.bin

simple.bin: simple.c
	riscv64-unknown-elf-gcc -S simple.c
	riscv64-unknown-elf-gcc -Wl,-Ttext=0x0 -nostdlib -march=rv64i -mabi=lp64 -o simple simple.s
	riscv64-unknown-elf-objcopy -O binary simple simple.bin

clean:
	rm -f fib simple *.s
	rm -f fib.bin simple.bin
