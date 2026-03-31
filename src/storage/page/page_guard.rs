use crate::buffer::buffer_pool_manager::{BufferPoolManager, FrameHeader};
use crate::buffer::lru_k_replacer::LruKReplacer;
use crate::storage::disk::disk_scheduler::{self, DiskScheduler};
use std::sync::{Arc, Mutex};

struct ReadPageGuard {
    page_id: usize,
    frame: Arc<FrameHeader>,
    replacer: Arc<LruKReplacer>,
    bpm_latch: Arc<Mutex<BufferPoolManager>>,
    disk_scheduler: Arc<DiskScheduler>,
    is_valid: bool,
}

impl ReadPageGuard {
    fn new(
        page_id: usize,
        frame: Arc<FrameHeader>,
        replacer: Arc<LruKReplacer>,
        bpm_latch: Arc<Mutex<BufferPoolManager>>,
        disk_scheduler: Arc<DiskScheduler>,
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
}

struct WritePageGuard {
    page_id: usize,
    frame: Arc<FrameHeader>,
    replacer: Arc<LruKReplacer>,
    bpm_latch: Arc<Mutex<BufferPoolManager>>,
    disk_scheduler: Arc<DiskScheduler>,
    is_valid: bool,
}

impl WritePageGuard {
    fn new(
        page_id: usize,
        frame: Arc<FrameHeader>,
        replacer: Arc<LruKReplacer>,
        bpm_latch: Arc<Mutex<BufferPoolManager>>,
        disk_scheduler: Arc<DiskScheduler>,
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
}
