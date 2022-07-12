// VirtQueue

use crate::param::*;

#[repr(C)]
pub struct VirtqDesc {
    pub addr: u64,
    pub len: u32,
    pub flags: u16,
    pub next: u16,
}

#[repr(C)]
pub struct VirtqAvail {
    pub flags: u16,
    pub idx: u16,
    pub ring: [u16; DESC_NUM],
    pub used_event: u16,
}

#[repr(C)]
pub struct VirtQUsedusedElem {
    pub id: u32,
    pub len: u32,
}

#[repr(C)]
pub struct VirtqUsed {
    pub flags: u16,
    pub idx: u16,
    pub ring: [VirtQUsedusedElem; DESC_NUM],
    pub avail_event: u16,
}


#[repr(C)]
pub struct VirtioBlkRequest {
    pub iotype: u32,
    pub reserved: u32,
    pub sector: u64,
}
