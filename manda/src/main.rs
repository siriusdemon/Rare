mod sexpr;
mod parser;
mod asm;
mod test;


use asm::RiscvParser;
use asm::RiscvAssembly;

fn main() {
    let asm = "
        (addi x5 x0 40)
        (addi x6 x0 2)
        (add  x7 x5 x6)
    ";
    let parser = RiscvParser::new(asm);
    let code = parser.parse();
    let assembler = RiscvAssembly::new(code);
    assembler.compile("addi-add.bin");
}
