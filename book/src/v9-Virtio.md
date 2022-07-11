# Virtio

Virtio represents a virtual device family. In this chapter, we will focus on the *Block Device*, a virtual disk. The full documentation is available at [here](https://docs.oasis-open.org/virtio/virtio/v1.1/virtio-v1.1.html)

Section 2, 4 and 5 are mostly relevant and recommanded to read.

### 1. Block Device

Section 5.2 contains an introduction to the Block Device. We can find out many useful information there. The `Device ID` is 2 while there is only one virtqueue, whose index is 0. 

### 2. Important Structure

Virtqueue
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

Descriptor
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

Available ring.
```c
struct virtq_avail {
#define VIRTQ_AVAIL_F_NO_INTERRUPT 1
    le16 flags;
    le16 idx;
    le16 ring[ /* Queue Size */ ];
    le16 used_event; /* Only if VIRTIO_F_EVENT_IDX */
};
```

Used Ring
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

Block IO requests
```c
struct virtio_blk_req {
    le32 type;
    le32 reserved;
    le64 sector;
    u8 data[];
    u8 status;
};
```


