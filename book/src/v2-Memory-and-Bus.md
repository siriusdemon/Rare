# 内存和总线


在上一节，我们把内存和CPU放在同一个结构体中，但在真实的硬件中，这两部分是分开的。如下图所示：

![bus](./images/bus.png)
<p class="comment">图片来自 Operation System: Three Easy Pieces</p>

CPU 和内存通过总线（bus）进行数据交换。

因此，我们定义以下结构：

<p class="filename">cpu.rs</p>

```rs
pub struct Cpu {
    pub regs: [u64; 32],
    pub pc: u64,
    pub bus: Bus,
}
```
CPU 现在不包含内存，而是连接了总线。总线上可能有多个 IO 设备，但目前我们只有一个（DRAM）。

<p class="filename">bus.rs</p>

```rs
pub struct Bus {
    dram: Dram,
}
```

<p class="filename">dram.rs</>

```rs
pub struct Dram {
    pub dram: Vec<u8>,
}
```


### 内存 API

内存（DRAM）只有两个功能：store，load。保存和读取的有效位数是 8，16，32，64。回顾上一节，我们采用的是小端字节序。实现如下

<p class="filename">dram.rs</>

```rs
impl Dram {
    pub fn new(code: Vec<u8>) -> Dram {
        let mut dram = vec![0; DRAM_SIZE as usize];
        dram.splice(..code.len(), code.into_iter());
        Self { dram }
    }

    // addr/size must be valid. Check in bus
    pub fn load(&self, addr: u64, size: u64) -> Result<u64, RvException> {
        let nbytes = size / 8;
        if (nbytes + addr - 1) > DRAM_END {
            return Err(RvException::InvalidAddress(addr));
        }

        let index = (addr - DRAM_BASE) as usize;
        let mut code = self.dram[index] as u64;
        // little-endian!
        for i in 1..nbytes {
            code |= (self.dram[index + i as usize] as u64) << (i * 8);
        }

        return Ok(code);
    }

    // addr/size must be valid. Check in bus
    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), RvException> {
        let nbytes = size / 8;
        if (nbytes + addr - 1) > DRAM_END {
            return Err(RvException::InvalidAddress(addr));
        }

        let index = (addr - DRAM_BASE) as usize;
        // little-endian!
        for i in 0..nbytes {
            let offset = 8 * i as usize;
            self.dram[index + i as usize] = ((value >> offset) & 0xff) as u8;
        }
        return Ok(())
    }
}
```
这里用到了一些全局变量和异常，定义如下

<p class="filename">param.rs</>

```rs
pub const DRAM_SIZE: u64 = 1024 * 1024 * 128;
pub const DRAM_BASE: u64 = 0x8000_0000;
pub const DRAM_END: u64 = DRAM_SIZE + DRAM_BASE - 1;
```

<p class="filename">exception.rs</p>

```rs
#[derive(Debug)]
pub enum RvException {
    InvalidAddress(u64),
    InvalidSize(u64),
    InvalidInstruction(u64),
}
```

### 总线 API

总线是 CPU 与各种 IO 设备（如键盘、鼠标、屏幕等）通信的渠道。总线上不同的地址范围对应了不同的设备。CPU 通过给总线发指令来间接操作其他的设备。

总线同样仅提供两个操作：store，load。

<p class="filename">bus.rs</p>

```rs
pub fn valid_size(size: u64) -> bool {
    [8, 16, 32, 64].contains(&size)
}

impl Bus {
    pub fn new(code: Vec<u8>) -> Bus {
        Self { dram: Dram::new(code) }
    }
    pub fn load(&self, addr: u64, size: u64) -> Result<u64, RvException> {
        if !valid_size(size) {
            return Err(RvException::InvalidSize(size));
        }
        match addr {
            DRAM_BASE..=DRAM_END => self.dram.load(addr, size),
            _ => Err(RvException::InvalidAddress(addr)),
        }
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), RvException> {
        if !valid_size(size) {
            return Err(RvException::InvalidSize(size));
        }
        match addr {
            DRAM_BASE..=DRAM_END => self.dram.store(addr, size, value),
            _ => Err(RvException::InvalidAddress(addr)),
        }
    }
}
```

这里，我们检查`addr`是否包含在`DRAM`的地址范围，否则报错。


### CPU API

现在 CPU 不直接读写内存，而是通过向总线发指令来读写内存。


<p class="filename">cpu.rs</p>

```rs
impl Cpu {
    pub fn new(code: Vec<u8>) -> Self {
        let mut regs = [0; 32];
        regs[2] = DRAM_END;

        let bus = Bus::new(code);

        Self {regs, pc: DRAM_BASE, bus}
    }

    pub fn load(&self, addr: u64, size: u64) -> Result<u64, RvException>{
        self.bus.load(addr, size)
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), RvException> {
        self.bus.store(addr, size, value)
    }

}
```