# UART

UART (Universal Asynchronous Receiver-Transmitter) is a computer hardware device for asynchronous serial communication in which the data format and transmission speeds are configurable. It sends data bits one by one. (quote from WikiPedia.) 

UART in our emulator is used to receive data from standard input and transmit data to standard output. The Spec is available at [here](http://byterunner.com/16550.html). It is recommanded to read it first.

The registers we need are RHR, THR and LSR.

UART 是通用异步收发器的缩写，是一种用于异步通信的硬件设备。它逐位发送数据，且其通信速率是可调的。（来自维基百科）。知乎上有不少介绍 UART 的文章，可以看看。

在我们的模拟器中，UART 是用于与宿主机的标准输入输出流进行通信。这里用的型号是 16550，其标准在[这里](http://byterunner.com/16550.html)。 我们只用到 RHR，THR，LSR 这三个寄存器。

### 1. Uart Structure

I am not sure whether the implementation in this emulator is correct. Nevertheless, it works.

The constants of UART are defined in `param.rs`. Let's take a look at the `uart.rs` here.

我不确定这里的实现是否是对的，但不管怎样，跑得通！UART 的常量定义在 param.rs 中，我们先看下 uart.rs 的内容。

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

Condition variables are synchronization primitives that enable threads to wait until a particular condition occurs. They enable threads to atomically release a lock and enter the sleeping state. Condition variables support operations that "wake one" or "wake all" waiting threads. After a thread is woken, it re-acquires the lock it released when the thread entered the sleeping state. 

UART 的地址空间是一个用 Mutex 打包好的 u8 数组。由于 UART 要用于多线程，所以我们得 Mutex 和 Arc 来封装。此外，我们需要条件变量来做线程同步。

条件变量是用于线程同步的原语，可以使线程释放一个锁并进入休眠，直到某一条件满足时将线程唤醒。线程苏醒时会重新获取它进入休眠前释放的锁。条件变量通常支持只唤醒一个线程或者唤醒所有线程这两种操作。


### 2. Initialization

We will use one UART to transfer data between the emulator and the host computer. So when we initialize the UART, we spawn a new thread to run in a loop, waiting for the input from console. When the uart receive a char (u8), it firstly check whether the data in buffer have been transferred (RX bit is cleared). If so, it places the new data in the buffer and set the RX bit, otherwise it wait.

我们将采用 UART 在模拟器和宿主机间进行通信。因此，在初始化 UART 的时候，我们创建了一个新的线程。这个线程跑在一个死循环中，等待控制台的输入。当它接到一个字符时，首先检查缓存（buffer）中的数据是否已经被取走了，也就是说 RX 是否被清零了。如果是，就把刚收到的数据放在缓存中，并置 RX 为 1，否则，进入休眠。

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

在我们的实现中，我们每接到到一个字符，都会立即打印到标准输出。因此，LSR_TX 总是 1.

### 3. UART API

Similarly, the UART provides two function `load` and `store` to manipulate data in its address space. Additionally, we also provide a function to check whether it is interrupting.

与其他设备，UART 也提供了 loal 和 store 两个函数来操作其地址空间中的数据。此外，我们提供了一个函数用于判断其是否发生了中断。

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

在 load 函数中，我们首先检查 size，因为 UART 寄存器位宽为 8。然后，我们给 uart 加锁，再对 index 进行匹配，如果等于 RHR，则唤醒休眠的线程。因为当前线程有 uart 的锁，在其释放锁之前（即函数执行完毕之前），另一个线程是无法覆盖缓存中的数据的。在返回之前，当前线程将 RX 置 0 以便让另一线程可以在缓存中写入新的数据。

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

UART 还提供了另一个函数用于将当前的中断状态读出来且置之为 false。

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

我有意忽略了关于 Ordering 的内容。想了解的话可以看下 The Rustonomicon 的第 8 章。

### 4. Testing

The original author have provided two inspired tests for us. They are `helloworld` and `echoback`.

原作者提供了两个非常有启发性的测试用例：helloworld 和 echoback。

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

这两个我都添加到测试用例了。当执行 cargo test 时，会相应生成两个 bin 文件。再用 cargo run xx.bin 的方式来跑。


### 5. Conclusion

We have defined UART in this chapter. And we have two threads to share the same uart structure with the help of some sync utilities of Rust. UART is able to generate interrupts so we provide a method to swap out its status atomically. On next chapter, we will complete our tour about interrupt in RISC-V.

本章中我们定义了 UART。同时，用两个线程共享一个 uart 的方式来完成宿主机与模拟器的通信。在下一章，我们将会完整实现 RISC-V 的中断机制。