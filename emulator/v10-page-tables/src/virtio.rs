use crate::exception::RvException;

use RvException::*;

pub struct Virtio {
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

const NOTIFY: u32 = 9999;

impl Virtio {
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
            queue_notify: NOTIFY, // insure? 
            status: 0,
            disk,
        }
    }

    pub fn is_interrupting(&mut self) -> bool {
        if self.queue_notify != NOTIFY {
            self.queue_notify = NOTIFY;
            return true;
        }
        return false;
    }
    
    pub fn load(&self, addr: u64, size: u64) -> Result<u64, RvException> {
        if size != 32 {
            return Err(LoadAccessFault(addr));
        }

        match addr {
            VIRTIO_MAGIC => Ok(0x74726976),
            VIRTIO_VERSION => Ok(0x1),
            VIRTIO_DEVICE_ID => Ok(0x2),
            VIRTIO_VENDOR_ID => Ok(0x554d4551),
            VIRTIO_DEVICE_FEATURES => Ok(0), // TODO: what should it return?
            VIRTIO_DRIVER_FEATURES => Ok(self.driver_features as u64),
            VIRTIO_QUEUE_NUM_MAX => Ok(8),
            VIRTIO_QUEUE_PFN => Ok(self.queue_pfn as u64),
            VIRTIO_STATUS => Ok(self.status as u64),
            _ => Err(LoadAccessFault(addr)),
        }
    }

    pub fn store(&mut self, addr: u64, size: u64, value: u64) -> Result<(), RvException> {
        if size != 32 {
            return Err(StoreOrAMOAccessFault(addr));
        }

        let value = value as u32;
        
        match addr {
            VIRTIO_DEVICE_FEATURES => Ok(self.driver_features = value),
            VIRTIO_GUEST_PAGE_SIZE => Ok(self.page_size = value),
            VIRTIO_QUEUE_SEL => Ok(self.queue_sel = value),
            VIRTIO_QUEUE_NUM => Ok(self.queue_num = value),
            VIRTIO_QUEUE_PFN => Ok(self.queue_pfn = value),
            VIRTIO_QUEUE_NOTIFY => Ok(self.queue_notify = value),
            VIRTIO_STATUS => Ok(self.status = value),
            _ => Err(StoreOrAMOAccessFault(addr)),
        }
    }

    pub fn get_new_id(&mut self) -> u64 {
        self.id = self.id.wrapping_add(1);
        return self.id;
    }

    pub fn desc_addr(&self) -> u64 {
        self.queue_pfn as u64 * self.page_size as u64
    }

    pub fn read_disk(&self, addr: u64) -> u8 {
        self.disk[addr as usize]
    }

    pub fn write_disk(&mut self, addr: u64, value: u8) {
        self.disk[addr as usize] = value;
    }
}