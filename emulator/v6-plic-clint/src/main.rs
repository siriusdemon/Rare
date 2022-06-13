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

pub use param::*;
use cpu::Cpu;


const ITERATION: usize = 10000;


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

    for _i in 0..ITERATION {
        let inst = match cpu.fetch() {
            Ok(inst) => inst,
            Err(e) => { 
                if e.is_fatal() {
                    println!("Riscv exception: {}", e);
                    break;
                } else {
                    cpu.handle_exception(e); 
                    continue;
                }
            }
        };
        match cpu.execute(inst) {
            Ok(_) => (),
            Err(e) => {
                if e.is_fatal() {
                    println!("Riscv exception: {}", e);
                    break;
                } else {
                    cpu.handle_exception(e);
                    continue;
                }
            }
        };
    }
    cpu.dump_registers();
    cpu.csr.dump_csrs();
    cpu.dump_pc();
    Ok(())
}
