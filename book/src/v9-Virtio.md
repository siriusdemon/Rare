# Virtio

Virtio represents a virtual device family. In this chapter, we will focus on the *Block Device*, a virtual disk. The full documentation is available at [here](https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.html).

Section 2, 4 and 5 are mostly relevant and recommanded to read first.

In this chapter, we will implement the legacy interface of virtio block device, which is implemented by QEMU and our target OS xv6 provides a driver for it.

### 1. Block Device & MMIO Device Register Layout

Section 5.2 contains an introduction to the Block Device. We can find out many useful information there. The `Device ID` is 2 while there is only one virtqueue, whose index is 0. 

Section 4.2.2 & 4.2.4 provide the register layout for both current and legacy interfaces. Some registers are only explained in Section 4.2.2, that is why I mention it.  

I have listed most of these in `param.rs`.

### 2. Virtqueue

The mechanism for bulk data transport on virtio devices is pretentiously called a virtqueue. Each device can have zero or more virtqueues. The block device have only one virtqueue, namely requestq, indexed as 0. The `QueueNotify` should be initialized with the max number of virtqueues. It is 1 in this case.

Each virtqueue occupies two or more physically-contiguous pages (usually defined as 4096 bytes) and consists of three parts:

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


### 2.2 Available Ring
The available ring has the following layout structure:  

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


### 2.3 Used Ring
The used ring has the following layout structure:
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

### 2.4 Virtio Block Request

The driver queues requests to the virtqueue, and they are used by the device (not necessarily in order).  Each request is of form:
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

```c
#define VIRTIO_BLK_T_IN             0
#define VIRTIO_BLK_T_OUT            1
#define VIRTIO_BLK_T_FLUSH          4
#define VIRTIO_BLK_T_DISCARD        11
#define VIRTIO_BLK_T_WRITE_ZEROES   13
```

The sector number indicates the offset (multiplied by 512) where the read or write is to occur. This field is unused and set to 0 for commands other than read or write.

VIRTIO_BLK_T_IN requests populate data with the contents of sectors read from the block device (in multi- ples of 512 bytes). VIRTIO_BLK_T_OUT requests write the contents of data to the block device (in multiples of 512 bytes).

We will only support VIRTIO_BLK_T_IN and VIRTIO_BLK_T_OUT requests.

### 3. Virtio Block API

Our implementation is simplified but still contains enough details. And it won't be too difficult to improve it.

I have defined almost all of the structure we mentioned above in `virtqueue.rs` except the `virtq` itself. And the structure of request lacks two fields `data` and `status`. These structure are defined almost as same as [xv6's](https://github.com/mit-pdos/xv6-riscv/blob/riscv/kernel/virtio.h).

Let's define a virtio block device as following:


