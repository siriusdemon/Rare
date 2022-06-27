// memory layout following QEMU
// https://github.com/qemu/qemu/blob/master/hw/riscv/virt.c#L46-L63 
pub const DRAM_BASE: u64 = 0x8000_0000;
pub const DRAM_SIZE: u64 = 1024 * 1024 * 128;
pub const DRAM_END: u64 = DRAM_SIZE + DRAM_BASE - 1;