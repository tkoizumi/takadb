use crate::buffer::buffer_pool_manager::{BufferPoolManager, FrameHeader};
use crate::buffer::lru_k_replacer::LruKReplacer;
use crate::storage::disk::disk_scheduler::DiskScheduler;
use std::sync::{Arc, Mutex};

struct ReadPageGuard {
    page_id: usize,
    frame: FrameHeader,
    replacer: LruKReplacer,
    bpm_latch: Arc<Mutex<BufferPoolManager>>,
    disk_scheduler: DiskScheduler,
    is_valid: bool,
}

struct WritePageGuard {
    page_id: usize,
    frame: FrameHeader,
    replacer: LruKReplacer,
    bpm_latch: Arc<Mutex<BufferPoolManager>>,
    disk_scheduler: DiskScheduler,
    is_valid: bool,
}
