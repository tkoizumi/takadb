use crate::buffer::buffer_pool_manager::{BufferPoolManager, FrameHeader};
use crate::buffer::lru_k_replacer::LruKReplacer;
use crate::constants::PAGE_SIZE;
use crate::storage::disk::disk_scheduler::{DiskRequest, DiskScheduler};
use std::sync::atomic::Ordering::SeqCst;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex, RwLockReadGuard, RwLockWriteGuard};

pub struct ReadPageGuard {
    pub page_id: usize,
    pub frame: Arc<FrameHeader>,
    pub replacer: Arc<LruKReplacer>,
    pub bpm_latch: Arc<Mutex<BufferPoolManager>>,
    pub disk_scheduler: Arc<Mutex<DiskScheduler>>,
    pub is_valid: bool,
}

impl ReadPageGuard {
    pub fn new(
        page_id: usize,
        frame: Arc<FrameHeader>,
        replacer: Arc<LruKReplacer>,
        bpm_latch: Arc<Mutex<BufferPoolManager>>,
        disk_scheduler: Arc<Mutex<DiskScheduler>>,
    ) -> Self {
        Self {
            page_id,
            frame,
            replacer,
            bpm_latch,
            disk_scheduler,
            is_valid: false,
        }
    }
    pub fn get_data(&self) -> RwLockReadGuard<'_, [u8; PAGE_SIZE]> {
        self.frame.get_data()
    }

    pub fn get_page_id(&self) -> usize {
        self.frame.page_id
    }

    pub fn is_dirty(&self) -> bool {
        self.frame.is_dirty.load(SeqCst)
    }

    pub fn flush(&self) {
        let (send, _) = channel();
        let request: DiskRequest =
            DiskRequest::new(true, self.get_data().to_vec(), self.get_page_id(), send);
        let requests = vec![request];
        let mut scheduler_lock = self.disk_scheduler.lock().unwrap();
        scheduler_lock.schedule(requests);
    }
}

impl Drop for ReadPageGuard {
    fn drop(&mut self) {
        if self.is_valid {
            let old_pin = self.frame.pin_count.fetch_sub(1, SeqCst);
            if old_pin == 1 {
                let replacer = &self.replacer;
                let frame = &self.frame;
                replacer.set_evictable(frame.frame_id, true);
            }
        }
        self.is_valid = false;
    }
}

pub struct WritePageGuard {
    pub page_id: usize,
    pub frame: Arc<FrameHeader>,
    pub replacer: Arc<LruKReplacer>,
    pub bpm_latch: Arc<Mutex<BufferPoolManager>>,
    pub disk_scheduler: Arc<Mutex<DiskScheduler>>,
    pub is_valid: bool,
}

impl WritePageGuard {
    pub fn new(
        page_id: usize,
        frame: Arc<FrameHeader>,
        replacer: Arc<LruKReplacer>,
        bpm_latch: Arc<Mutex<BufferPoolManager>>,
        disk_scheduler: Arc<Mutex<DiskScheduler>>,
    ) -> Self {
        Self {
            page_id,
            frame,
            replacer,
            bpm_latch,
            disk_scheduler,
            is_valid: false,
        }
    }

    pub fn get_data(&self) -> RwLockReadGuard<'_, [u8; PAGE_SIZE]> {
        self.frame.get_data()
    }

    pub fn get_mut_data(&self) -> RwLockWriteGuard<'_, [u8; PAGE_SIZE]> {
        self.frame.get_mut_data()
    }

    pub fn get_page_id(&self) -> usize {
        self.frame.page_id
    }

    pub fn is_dirty(&self) -> bool {
        self.frame.is_dirty.load(SeqCst)
    }

    pub fn flush(&self) {
        let (send, _) = channel();
        let request: DiskRequest =
            DiskRequest::new(true, self.get_data().to_vec(), self.get_page_id(), send);
        let requests = vec![request];
        let mut scheduler_lock = self.disk_scheduler.lock().unwrap();
        scheduler_lock.schedule(requests);
    }
}

impl Drop for WritePageGuard {
    fn drop(&mut self) {
        if self.is_valid {
            let old_pin = self.frame.pin_count.fetch_sub(1, SeqCst);
            if old_pin == 1 {
                let replacer = &self.replacer;
                let frame = &self.frame;
                replacer.set_evictable(frame.frame_id, true);
                self.is_valid = false;
            }
        }
    }
}
