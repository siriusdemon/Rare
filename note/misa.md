# Riscv MISA

### Machine Trap Delegation Registers

In systems with S-mode, the medeleg and mideleg registers must exist, and setting a bit in medeleg or mideleg will delegate the corresponding trap, when occurring in S-mode or U-mode, to the S-mode trap handler. In systems without S-mode, the medeleg and mideleg registers should not
exist.

