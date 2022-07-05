# UART

UART (Universal Asynchronous Receiver-Transmitter) is a computer hardware device for asynchronous serial communication in which the data format and transmission speeds are configurable. It sends data bits one by one. (quote from WikiPedia.) 

UART in our emulator is used to receive data from standard input and transmit data to standard output. The Spec is available at [here](http://byterunner.com/16550.html). It is recommanded to read it first.

The registers we need are RHR, THR and LSR.

### 1. Uart Structure

I am not sure whether the implementation in this emulator is correct. Nevertheless, it works.

The constants of UART are defined in `param.rs`. Let's take a look at the `uart.rs` here.

<p class="filename">uart.rs</p>

```rs
use std::io;
use std::io::prelude::*;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Condvar, Mutex,
};
use std::thread;

pub struct Uart {
    /// Pair of an array for UART buffer and a conditional variable.
    uart: Arc<(Mutex<[u8; UART_SIZE as usize]>, Condvar)>,
    /// Bit if an interrupt happens.
    interrupt: Arc<AtomicBool>,
}
```

The array of `u8` wrapped by `Mutex` is the address space of UART. We wrap it with `Mutex` and `Arc` because we are going to share the same `Uart` with two threads. We need condition variables for thread synchronization.

### 2. Initialization

We will use one UART to transfer data between the emulator and the host computer. So when we initialize the UART, we spawn a new thread to run in a loop, waiting for the input from console. When the uart receive a char (u8), it firstly check whether the data in buffer have been transferred (RX bit is cleared). If so, it places the new data in the buffer and set the RX bit, otherwise it wait.

<p class="filename">uart.rs</p>

```rs
impl Uart {
    /// Create a new `Uart` object.
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
        
        Self { uart, interrupt }
    }
}
```

In our implementation, we will emit every char we receive to standard output immediately. So `LSR_TX` is always on. 


### 3. UART API

Similarly, the UART provides two function `load` and `store` to manipulate data in its address space. Additionally, we also provide a function to check whether it is interrupting.

<p class="filename">uart.rs</p>

```rs
impl Uart {
    // ...
    pub fn load(&mut self, addr: u64, size: u64) -> Result<u64, Exception> {
        if size != 8 {
            return Err(Exception::LoadAccessFault(addr));
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
}
```

In `load`, we check for the size first since UART register bit-width is 8. Then, we lock the uart and check the index. If it equals to RHR, we wake up the sleeping thread. Since we have held the lock, the value in buffer will not be overwrited before we go to the end of this function. Before we return, we also clear the RX bit so another thread is able to write new data in the buffer.

The process in `store` is quite similar. The differece is when we store a value to THR, we deliver it to standard output immediately.

<p class="filename">uart.rs</p>

```rs
impl Uart {
    // ...
    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), Exception> {
        if size != 8 {
            return Err(Exception::StoreAMOAccessFault(addr));
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
```

Uart has provided another function for atomically swap out the current state.

<p class="filename">uart.rs</p>

```rs
impl Uart {
    // ...
    pub fn is_interrupting(&self) -> bool {
        self.interrupt.swap(false, Ordering::Acquire)
    }
}
```

I deliberately ignore the `Ordering` we use here since this topic is a little distracting. Please refer to Chapter 8 of `The Rustonomicon` if you are interested in it.


### 4. Testing

The original author have provided two inspired tests for us. They are `helloworld` and `echoback`.

```c
int main() {
    volatile char *uart = (volatile char *) 0x10000000;
    uart[0] = 'H';
    uart[0] = 'e';
    uart[0] = 'l';
    uart[0] = 'l';
    uart[0] = 'o';
    uart[0] = ',';
    uart[0] = ' ';
    uart[0] = 'w';
    uart[0] = 'o';
    uart[0] = 'r';
    uart[0] = 'l';
    uart[0] = 'd';
    uart[0] = '!';
    uart[0] = '\n';
    return 0;
}";
```

```c
int main() {
    while (1) {
        volatile char *uart = (volatile char *) 0x10000000;
        while ((uart[5] & 0x01) == 0);
        char c = uart[0];
        if ('a' <= c && c <= 'z') {
            c = c + 'A' - 'a';
        }
        uart[0] = c;
    }
}";
```

I have added both as test cases. When you run `cargo test`, two binary file named `test_helloworld.bin` and `test_echoback.bin` will be created. Then you can run `cargo run xx.bin` to play with them.


### 5. Conclusion

We have defined UART in this chapter. And we have two threads to share the same uart structure with the help of some sync utilities of Rust. UART is able to generate interrupts so we provide a method to swap out its status atomically. On next chapter, we will complete our tour about interrupt in RISC-V.