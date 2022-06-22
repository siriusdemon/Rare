// UART 
// -- See the spec: http://byterunner.com/16550.html
// RUst atomic module
// -- Atomic types present operations that, when used correctly, synchronize updates between threads.
// Memory ordering
// -- https://en.cppreference.com/w/c/atomic/memory_order
use std::io;
use std::io::prelude::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::thread;



use crate::param::*;
use crate::exception::RvException;

use RvException::*;



pub struct Uart {
    /// Pair of an array for UART buffer and a conditional variable.
    uart: Arc<(Mutex<[u8; UART_SIZE as usize]>, Condvar)>,
    /// Bit if an interrupt happens.
    interrupt: Arc<AtomicBool>,
}

impl Uart {
    pub fn new() -> Self {
        let mut array = [0; UART_SIZE as usize];
        array[UART_LSR as usize] |= MASK_UART_LSR_TX;

        let uart = Arc::new(((Mutex::new(array)), Condvar::new()));
        let interrupt = Arc::new(AtomicBool::new(false));

        // receive part
        let read_uart = Arc::clone(&uart);
        let read_interrupt = Arc::clone(&interrupt);
        let mut byte = [0];
        thread::spawn(move || loop {
            match io::stdin().read(&mut byte) {
                Ok(_) => {
                    let (uart, cvar) = &*read_uart;
                    let mut array = uart.lock().unwrap();
                    // if data have been received but not yet be transferred.
                    // this thread wait for it to be transferred.
                    while (array[UART_LSR as usize] & MASK_UART_LSR_RX) == 1 {
                        array = cvar.wait(array).unwrap();
                    }
                    // data have been transferred, so receive next one.
                    array[UART_RHR as usize] = byte[0];
                    read_interrupt.store(true, Ordering::Release);
                    array[UART_LSR as usize] |= MASK_UART_LSR_RX;
                }
                Err(e) => println!("{}", e),
            }
        });
        
        Self {uart, interrupt}
    }

    pub fn is_interrupting(&self) -> bool {
        self.interrupt.swap(false, Ordering::Acquire)
    }

    pub fn load(&mut self, addr: u64, size: u64) -> Result<u64, RvException> {
        if size != 8 {
            return Err(LoadAccessFault(addr));
        }
        let (uart, cvar) = &*self.uart;
        let mut array = uart.lock().unwrap(); 
        let index = addr - UART_BASE;
        // a read happens
        match index {
            UART_RHR => {
                cvar.notify_one();
                array[UART_LSR as usize] &= !MASK_UART_LSR_RX;
                Ok(array[UART_RHR as usize] as u64)
            }
            _ => Ok(array[index as usize] as u64),
        } 
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), RvException> {
        if size != 8 {
            return Err(StoreOrAMOAccessFault(addr));
        }
        let (uart, cvar) = &*self.uart;
        let mut array = uart.lock().unwrap();
        let index = addr - UART_BASE;
        match index {
            UART_THR => {
                print!("{}", value as u8 as char);
                io::stdout().flush().unwrap();
                return Ok(());
            }
            _ => {
                array[index as usize] = value as u8;
                return Ok(());
            }
        }
    }
}
