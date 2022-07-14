# Virtio

Virtio represents a virtual device family. In this chapter, we will focus on the *Block Device*, a virtual disk. The full documentation is available at [here](https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.html).

Section 2, 4 and 5 are mostly relevant and recommanded to read first.

In this chapter, we will implement the legacy interface of virtio block device, which is implemented by QEMU and our target OS xv6 provides a driver for it.

VirtIO 代表了一个虚拟设备家族。在本章中，我们将专注于虚拟磁盘（block device）。完整的文档可以在[这里](https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.html)获取。其中的第 2、4、5 节是最相关的，推荐先读一遍。

本章我们要实现的是虚拟磁盘的旧（legacy）接口，QEMU 实现了同样的接口，xv6 的驱动也与之适配。

### 1. Block Device & MMIO Device Register Layout

Section 5.2 contains an introduction to the Block Device. We can find out many useful information there. The `Device ID` is 2 while there is only one virtqueue, whose index is 0. 

Section 4.2.2 & 4.2.4 provide the register layout for both current and legacy interfaces. Some registers are only explained in Section 4.2.2, that is why I mention it.  

I have listed most of these in `param.rs`.

Virtio 文档的第 5.2 节是对虚拟磁盘的介绍，包括了设备ID，virtqueue 的数量等。4.2.2 和 4.2.4 节提供了旧接口的寄存器布局。一些寄存器仅在会 4.2.2 节中介绍，这两部分应该合起来看。

### 2. Virtqueue

The mechanism for bulk data transport on virtio devices is pretentiously called a virtqueue. Each device can have zero or more virtqueues. The block device have only one virtqueue, namely requestq, indexed as 0. The `QueueNotify` should be initialized with the max number of virtqueues. It is 1 in this case.

Each virtqueue occupies two or more physically-contiguous pages (usually defined as 4096 bytes) and consists of three parts:

CPU 与虚拟磁盘的数据传输主要依赖 virtqueue（虚拟队列）。每个设备可以有 0 或者多个 virtqueue。虚拟磁盘只有一个 virtqueue。寄存器 QueueNotify 初始化的时候被设置为 virtqueue 的数目。


![virtqueue-layout](./images/virtqueue-layout.png)

```c
struct virtq {
    // The actual descriptors (16 bytes each)
    struct virtq_desc desc[ Queue Size ];
    // A ring of available descriptor heads with free-running index.
    struct virtq_avail avail;
    // Padding to the next Queue Align boundary.
    u8 pad[ Padding ];
    // A ring of used descriptor heads with free-running index.
    struct virtq_used used;
};
```

### 2.1 Descriptor

The first field of a virtqueue is an array of descriptor, aka. descriptor table. 

The descriptor table refers to the buffers the driver is using for the device. `addr` is a physical address, and the buffers can be chained via `next`. Each descriptor describes a buffer which is read-only for the device (“device-readable”) or write-only for the device (“device-writable”), but a chain of descriptors can contain both device-readable and device-writable buffers.

virtqueue 的第一个字段是一个描述符数组，也叫描述符表。

描述符表告诉设备驱动正在使用的缓存。addr 是一个物理地址，缓存可以通过 next 字段进行衔接。描述符带有一个标志，用于标识该缓存对于设备来说是可读还是可写的。一串描述符中可以同时包含可读可写两种类型的缓存。

```c
struct virtq_desc {
    /* Address (guest-physical). */
    le64 addr;
    /* Length. */
    le32 len;
/* This marks a buffer as continuing via the next field. */
#define VIRTQ_DESC_F_NEXT 1
/* This marks a buffer as device write-only (otherwise device read-only). */
#define VIRTQ_DESC_F_WRITE 2
/* This means the buffer contains a list of buffer descriptors. */
#define VIRTQ_DESC_F_INDIRECT 4
    /* The flags as indicated above. */
    le16 flags;
    /* Next field if flags & NEXT */
    le16 next;
};
```

The actual contents of the memory offered to the device depends on the device type. Most common is to begin the data with a header (containing little-endian fields) for the device to read, and postfix it with a status tailer for the device to write.

In our emulator (also in QEMU), the block device request contains three descriptors. The first one (the header) contains some request information, the second one describes the data, and the last one contains a status for the device to write. (However, the code provided by the original author ignores the last one, so do I.)

设备的类型决定了提供给它的数据的实际内容。最常见的是做法是，数据带有一个只读的头部指示某些信息，接着则是实际的数据，最后再带一个可写的状态地址供设备去写。

在我们的模拟器中，磁盘设备收到的数据包含三个描述符。第一个是头部，包含了请求的信息，第二个是实际的数据，第三个是一个可写的状态。（然而。原作者忽略了对状态的处理，我这也是如此。）

### 2.2 Available Ring
The available ring has the following layout structure:  

available ring 的结构如下：

```c
struct virtq_avail {
#define VIRTQ_AVAIL_F_NO_INTERRUPT 1
    le16 flags;
    le16 idx;
    le16 ring[ /* Queue Size */ ];
    le16 used_event; /* Only if VIRTIO_F_EVENT_IDX */
};
```

The driver uses the available ring to offer buffers to the device: each ring entry refers to the head of a descriptor chain. It is only written by the driver and read by the device.

`idx` field indicates where the driver would put the next descriptor entry in the ring (modulo the queue size).  This starts at 0, and increases.

驱动程序使用 available ring 来提供数据给设备。每个入口代表了一个描述符串的头部。available 只能由驱动程序来写，由设备来读。used ring 与之相反。

字段 idx 指定了驱动程序放置数据到 available ring 的索引，

### 2.3 Used Ring
The used ring has the following layout structure:

used ring 的结构如下：

```c
struct virtq_used {
#define VIRTQ_USED_F_NO_NOTIFY 1
    le16 flags;
    le16 idx;
    struct virtq_used_elem ring[ /* Queue Size */];
    le16 avail_event; /* Only if VIRTIO_F_EVENT_IDX */
};
/* le32 is used here for ids for padding reasons. */
struct virtq_used_elem {
    /* Index of start of used descriptor chain. */
    le32 id;
    /* Total length of the descriptor chain which was used (written to) */
    le32 len;
};
```

The used ring is where the device returns buffers once it is done with them: it is only written to by the device, and read by the driver.

Each entry in the ring is a pair: `id` indicates the head entry of the descriptor chain describing the buffer (this matches an entry placed in the available ring by the guest earlier), and `len` the total of bytes written into the buffer.

Historically, many drivers ignored the len value, as a result, many devices set len incorrectly. Thus, when using the legacy interface, it is generally a good idea to ignore the len value in used ring entries if possible.

设备处理完 buffer 之后，就会在 used ring 写入一个新的描述符 id 指向该 buffer。表示它已经处理完了。

Ring 的每个入口都有两个字段，id 指向了一个该 buffer 的头部描述符。len 表示这个 buffer 的长度。

由于历史原因，许多驱动程序会忽略 len 的值，因此，很多设备也没有正确地设置这个值。当使用旧接口时，最好忽略这个字段。

### 2.4 Virtio Block Request

The driver queues requests to the virtqueue, and they are used by the device (not necessarily in order).  Each request is of form:

驱动程序将请求排队发给 virtqueue，然后由设备去使用。请求的格式如下：

```c
struct virtio_blk_req {
    le32 type;
    le32 reserved;
    le64 sector;
    u8 data[];
    u8 status;
};
```

The type of the request is either a read (VIRTIO_BLK_T_IN), a write (VIRTIO_BLK_T_OUT), a discard (VIRTIO_BLK_T_DISCARD), a write zeroes (VIRTIO_BLK_T_WRITE_ZEROES) or a flush (VIRTIO_BLK_T_FLUSH)

请求类型要么是读，要么是写。或者丢弃，写入 0，或者刷新。

```c
#define VIRTIO_BLK_T_IN             0
#define VIRTIO_BLK_T_OUT            1
#define VIRTIO_BLK_T_FLUSH          4
#define VIRTIO_BLK_T_DISCARD        11
#define VIRTIO_BLK_T_WRITE_ZEROES   13
```

The sector number indicates the offset (multiplied by 512) where the read or write is to occur. This field is unused and set to 0 for commands other than read or write.

VIRTIO_BLK_T_IN requests populate data with the contents of sectors read from the block device (in multiples of 512 bytes). VIRTIO_BLK_T_OUT requests write the contents of data to the block device (in multiples of 512 bytes).

We will only support VIRTIO_BLK_T_IN and VIRTIO_BLK_T_OUT requests.

字段 sector 指明了要读写的扇区（每个扇区为 512 个字节，因此该值应该乘以 512）。VIRTIO_BLK_T_IN 表示将指定扇区的数据读到缓存中。VIRTIO_BLK_T_OUT 表示将缓存的数据写到磁盘里。我们仅支持这两种类型的请求。

### 3. Virtio Block API

Our implementation is simplified but still contains enough details. And it won't be too difficult to improve it.

I have defined almost all of the structure we mentioned above in `virtqueue.rs` except the `virtq` itself. And the structure of request lacks two fields `data` and `status`. These structure are defined almost as same as [xv6's](https://github.com/mit-pdos/xv6-riscv/blob/riscv/kernel/virtio.h).

Let's define a virtio block device as follows:

虽然我们的实现是简化过的，但依旧包含了足够的细节，后续改进应该也不会太难。

以上所提到的数据结构，除了 virtq 之外，其他的都定义在 virtqueue.rs 当中。这些数据结构与 xv6 的基本相同。

我们定义虚拟磁盘的数据结构如下：

<p class=filename>virtio.rs</p>

```rs
pub struct VirtioBlock {
    id: u64,
    driver_features: u32,
    page_size: u32,
    queue_sel: u32,
    queue_num: u32,
    queue_pfn: u32,
    queue_notify: u32,
    status: u32,
    disk: Vec<u8>,
}

const MAX_BLOCK_QUEUE: u32 = 1;

impl VirtioBlock {
    pub fn new(disk_image: Vec<u8>) -> Self {
        let mut disk = Vec::new();
        disk.extend(disk_image.into_iter());

        Self {
            id: 0, 
            driver_features: 0,
            page_size: 0,
            queue_sel: 0,
            queue_num: 0,
            queue_pfn: 0,
            queue_notify: MAX_BLOCK_QUEUE,
            status: 0,
            disk,
        }
    }
}
```

When we create a virtio block device, we initialize its NOTIFY as maximum number of virtqueue. When the device is interrupting, NOTIFY contains the index of the virtqueue needed to process.

The virtio block device provide several APIs:

+ interrupting: whether the device is interrupting
+ load: load the value of certain MMIO registers
+ store: store some value into certain MMIO registers
+ get_new_id: get the next id of used ring.
+ desc_addr: get the base address of the virtqueue.
+ read_disk: read data from disk and store into data buffer.
+ write_disk: write the data contained in buffer into disk.

The implementation is straightforward. Please stop to read the code in `virtio.rs`. You also need to add this module into `main.rs` and `bus.rs`.

当我们初始化磁盘时，NOTIFY 设置为 virtqueue 的数量。当设备发生中断时，NOTIFY 中包含了要处理的 virtqueue 的索引。

虚拟磁盘提供了以下几个 API

+ interrupting: 表示该设备是否发生了中断
+ load: 加载某个寄存器的值
+ store: 将值写入某个寄存器
+ get_new_id: 获取下一个 used ring 的索引
+ desc_addr: 获取 virtqueue 的地址
+ read_disk: 从磁盘中读取数据
+ write_disk: 写数据到磁盘。

### 4. Data Transfer

We will implement the `data_access` in `cpu.rs`. When an virtio block interrupt arrives, we call this function to perform disk IO.

The first step is to compute the address of the descriptor table, available ring and the used ring.  We also cast the address to a type reference to ease field access.

我们在 cpu.rs 中实现 data_access，当一个磁盘中断到达时，我们调用这个函数处理磁盘 IO。

第一步是计算出描述符表、available ring 和 used ring 的内存地址。我们将之转换为一个类型引用，以方便我们读取字段的值。

<p class="filename">cpu.rs</p>

```rs
impl Cpu {
    pub fn disk_access(&mut self) {
        const desc_size: u64 = size_of::<VirtqDesc>() as u64;
        // 2.6.2 Legacy Interfaces: A Note on Virtqueue Layout
        // ------------------------------------------------------------------
        // Descriptor Table  | Available Ring | (...padding...) | Used Ring
        // ------------------------------------------------------------------
        let desc_addr = self.bus.virtio_blk.desc_addr();
        let avail_addr = desc_addr + DESC_NUM as u64 * desc_size;
        let used_addr = desc_addr + PAGE_SIZE;
        // cast address to reference
        let virtq_avail = unsafe { &(*(avail_addr as *const VirtqAvail)) };
        let virtq_used  = unsafe { &(*(used_addr  as *const VirtqUsed)) };
        // ... TO BE CONTINUE ...
    }
}
```

The pattern to cast an address to an immutable type reference is as following. We will repeat it many times.

将一个地址转换为一个不可变类型引用的代码模式如下。我们会多次重复这个模式。

```rs
let obj = unsafe { &(*(memaddr as *const YourType))};
```



The idx field of `virtq_avail` should be indexed into available ring to get the index of descriptor we need to process.

virtq_avail 的 idx 字段可用于从 available ring 中找到我们要处理的描述符的索引。

<p class="filename">cpu.rs</p>

```rs
impl Cpu {
    pub fn disk_access(&mut self) {
        // ...
        let idx = self.bus.load(&virtq_avail.idx as *const _ as u64, 16).unwrap() as usize;
        let index = self.bus.load(&virtq_avail.ring[idx % DESC_NUM] as *const _ as u64, 16).unwrap();
        // ... TO BE CONTINUE ...
    }
}
```

As we have mentioned above, a block device request use three descriptors. One for the header, one for the data, and one for the status. The header descriptor contains the request. We use only first two descriptors.

The first descriptor contains the request information and a pointer to the data descriptor. The `addr` field points to a virtio block request. We need two fields in the request: the sector number stored in the `sector` field tells us where to perform IO and the `iotype` tells us whether to read or write. The `next` field points to the second descriptor. (data descriptor)

正如我们上面所提到的，一个块设备请求使用了三个描述符。一个用于头部，一个用于数据，一个用于状态回写。头部描述符包含了请求的信息，我们只使用前两个描述符。

第一个描述符包含了请求的信息以及一个指向数据描述符的指针。addr 字符指向了该请求。我们需要两个字段，sector 告诉我们相应的扇区，iotype 告诉这个请求是要读还是要写。next 指向了要读/写的数据描述符。

<p class="filename">cpu.rs</p>

```rs
impl Cpu {
    pub fn disk_access(&mut self) {
        // ...
        let desc_addr0 = desc_addr + desc_size * index;
        let virtq_desc0 = unsafe { &(*(desc_addr0 as *const VirtqDesc)) };
        let next0  = self.bus.load(&virtq_desc0.next  as *const _ as u64, 16).unwrap();

        let req_addr = self.bus.load(&virtq_desc0.addr as *const _ as u64, 64).unwrap();
        let virtq_blk_req = unsafe { &(*(req_addr as *const VirtioBlkRequest)) };
        let blk_sector = self.bus.load(&virtq_blk_req.sector as *const _ as u64, 64).unwrap();
        let iotype = self.bus.load(&virtq_blk_req.iotype as *const _ as u64, 32).unwrap() as u32;
        // ... TO BE CONTINUE ...
    }
}
```

We use the `next0` of first descriptor to compute the address of the second descriptor. To perform disk IO, we need the `addr` field and the `len` field. The `addr` field points to the data to read or write while the `len` donates the size of the data. And we perform disk IO based on the `iotype`.

我们使用第一个描述符的 next0 计算出第二个描述符的索引。为了执行磁盘 IO，我们需要 addr 和 len 字段。addr 字段指向了数据的内存地址。len 则表明该数据的大小。iotype 决定是读还是写。

<p class="filename">cpu.rs</p>

```rs
impl Cpu {
    pub fn disk_access(&mut self) {
        // ...
        let desc_addr1 = desc_addr + desc_size * next0;
        let virtq_desc1 = unsafe { &(*(desc_addr1 as *const VirtqDesc)) };
        let addr1 = self.bus.load(&virtq_desc1.addr as *const _ as u64, 64).unwrap();
        let len1  = self.bus.load(&virtq_desc1.len  as *const _ as u64, 32).unwrap();

        match iotype {
            VIRTIO_BLK_T_OUT => {
                for i in 0..len1 {
                    let data = self.bus.load(addr1 + i, 8).unwrap();
                    self.bus.virtio_blk.write_disk(blk_sector * SECTOR_SIZE + i, data);
                }
            }
            VIRTIO_BLK_T_IN => {
                for i in 0..len1 {
                    let data = self.bus.virtio_blk.read_disk(blk_sector * SECTOR_SIZE + i);
                    self.bus.store(addr1 + i, 8, data as u64).unwrap();
                }
            } 
            _ => unreachable!(),
        }       
        // ... TO BE CONTINUE ...
    }
}
```

Finally, we need to update used ring to tell driver we are done.

最后，我们更新 used ring 通知驱动程序。

<p class="filename">cpu.rs</p>

```rs
impl Cpu {
    pub fn disk_access(&mut self) {
        // ...
        let new_id = self.bus.virtio_blk.get_new_id();
        self.bus.store(&virtq_used.idx as *const _ as u64, 16, new_id % 8).unwrap();
    }
}
```


The whole function is as follows: 完整的函数代码如下

<p class="filename">cpu.rs</p>

```rs
impl Cpu {
    pub fn disk_access(&mut self) {
        const desc_size: u64 = size_of::<VirtqDesc>() as u64;
        // 2.6.2 Legacy Interfaces: A Note on Virtqueue Layout
        // ------------------------------------------------------------------
        // Descriptor Table  | Available Ring | (...padding...) | Used Ring
        // ------------------------------------------------------------------
        let desc_addr = self.bus.virtio_blk.desc_addr();
        let avail_addr = desc_addr + DESC_NUM as u64 * desc_size;
        let used_addr = desc_addr + PAGE_SIZE;

        // cast addr to a reference to ease field access.
        let virtq_avail = unsafe { &(*(avail_addr as *const VirtqAvail)) };
        let virtq_used  = unsafe { &(*(used_addr  as *const VirtqUsed)) };

        // The idx field of virtq_avail should be indexed into available ring to get the
        // index of descriptor we need to process.
        let idx = self.bus.load(&virtq_avail.idx as *const _ as u64, 16).unwrap() as usize;
        let index = self.bus.load(&virtq_avail.ring[idx % DESC_NUM] as *const _ as u64, 16).unwrap();

        // The first descriptor:
        // which contains the request information and a pointer to the data descriptor.
        let desc_addr0 = desc_addr + desc_size * index;
        let virtq_desc0 = unsafe { &(*(desc_addr0 as *const VirtqDesc)) };
        // The addr field points to a virtio block request. We need the sector number stored 
        // in the sector field. The iotype tells us whether to read or write.
        let req_addr = self.bus.load(&virtq_desc0.addr as *const _ as u64, 64).unwrap();
        let virtq_blk_req = unsafe { &(*(req_addr as *const VirtioBlkRequest)) };
        let blk_sector = self.bus.load(&virtq_blk_req.sector as *const _ as u64, 64).unwrap();
        let iotype = self.bus.load(&virtq_blk_req.iotype as *const _ as u64, 32).unwrap() as u32;
        // The next field points to the second descriptor. (data descriptor)
        let next0  = self.bus.load(&virtq_desc0.next  as *const _ as u64, 16).unwrap();

        // the second descriptor. 
        let desc_addr1 = desc_addr + desc_size * next0;
        let virtq_desc1 = unsafe { &(*(desc_addr1 as *const VirtqDesc)) };
        // The addr field points to the data to read or write
        let addr1  = self.bus.load(&virtq_desc1.addr  as *const _ as u64, 64).unwrap();
        // the len donates the size of the data
        let len1   = self.bus.load(&virtq_desc1.len   as *const _ as u64, 32).unwrap();

        match iotype {
            VIRTIO_BLK_T_OUT => {
                for i in 0..len1 {
                    let data = self.bus.load(addr1 + i, 8).unwrap();
                    self.bus.virtio_blk.write_disk(blk_sector * SECTOR_SIZE + i, data);
                }
            }
            VIRTIO_BLK_T_IN => {
                for i in 0..len1 {
                    let data = self.bus.virtio_blk.read_disk(blk_sector * SECTOR_SIZE + i);
                    self.bus.store(addr1 + i, 8, data as u64).unwrap();
                }
            } 
            _ => unreachable!(),
        }       

        let new_id = self.bus.virtio_blk.get_new_id();
        self.bus.store(&virtq_used.idx as *const _ as u64, 16, new_id % 8).unwrap();
    }
}
```

### 5. Conclusion

The implementation we provide here is not optimal. Ideally, we are supposed to check the flags field in the descriptor to follow the chain until the NEXT flag bit is not set. We can do such a simplification since we already know how xv6 deliver disk IO. Nevertheless, disk IO is hard and error-prone. And I had spent one week to figure out what happen in it. Next comes our final chapter, we will arm our emulator with a page table. 

我们这里提供的实现并非是最优的。理想情况下，我们应该逐个松查描述符链中的每个描述符，直到没有 NEXT 标志为止。我们做了简化是因为我们知道 xv6 是如何进行磁盘 IO 的。不管怎么样，磁盘 IO 很容易出错。我好了一周的时间才搞清楚里面的细节。我们还有最后一章，我们将实现一个虚拟地址系统。