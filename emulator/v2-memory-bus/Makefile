fib.bin: fib.c
	clang -nostdlib -march=rv64g -mabi=lp64 -mno-relax -S fib.c
	clang -Wl,-Ttext=0x0 -nostdlib -march=rv64g -mabi=lp64 -mno-relax -o fib fib.s
	llvm-objcopy -O binary fib fib.bin

simple.bin: simple.c
	clang -nostdlib -march=rv64g -mabi=lp64 -mno-relax -S simple.c
	clang -Wl,-Ttext=0x0 -nostdlib -march=rv64g -mabi=lp64 -mno-relax -o simple simple.s
	llvm-objcopy -O binary simple simple.bin

clean:
	rm -f fib simple *.s
	rm -f fib.bin simple.bin test_*
