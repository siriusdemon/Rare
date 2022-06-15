use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;


mod param;
mod dram;
mod cpu;
mod bus;
mod exception;
mod csr;
mod plic;
mod clint;
mod uart;

pub use param::*;
use cpu::Cpu;




fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        println!(
            "Usage:\n\
            - rvemu <filename>\n\
            - cargo run <filename>"
        );
        return Ok(());
    }

    let mut file = File::open(&args[1])?;
    let mut code = Vec::new();
    file.read_to_end(&mut code)?;

    let mut cpu = Cpu::new(code);

    loop {
        let inst = match cpu.fetch() {
            Ok(inst) => inst,
            Err(e) => { 
                cpu.handle_exception(e); 
                if e.is_fatal() {
                    println!("Riscv exception: {}", e);
                    break;
                }
                continue;
            }
        };
        match cpu.execute(inst) {
            Ok(_) => (),
            Err(e) => {
                cpu.handle_exception(e); 
                if e.is_fatal() {
                    println!("Riscv exception: {}", e);
                    break;
                }
                continue;
            }
        };
    }
    cpu.dump_registers();
    cpu.csr.dump_csrs();
    cpu.dump_pc();
    Ok(())
}
