pub const DRAM_SIZE: u64 = 1024 * 1024 * 128;
pub const DRAM_BASE: u64 = 0x8000_0000;
pub const DRAM_END: u64 = DRAM_SIZE + DRAM_BASE - 1;


// extern compiler only for testing
pub const RV_GCC: &str = "riscv64-unknown-elf-gcc";
pub const RV_OBJCOPY: &str = "riscv64-unknown-elf-objcopy";
