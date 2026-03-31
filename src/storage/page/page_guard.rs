use crate::buffer::buffer_pool_manager::{BufferPoolManager, FrameHeader};
use crate::buffer::lru_k_replacer::LruKReplacer;
use crate::storage::disk::disk_scheduler::DiskScheduler;
use std::sync::{Arc, Mutex};

struct ReadPageGuard {
    page_id: usize,
    frame: Arc<FrameHeader>,
    replacer: Arc<LruKReplacer>,
    bpm_latch: Arc<Mutex<BufferPoolManager>>,
    disk_scheduler: Arc<DiskScheduler>,
    is_valid: bool,
}

struct WritePageGuard {
    page_id: usize,
    frame: Arc<FrameHeader>,
    replacer: Arc<LruKReplacer>,
    bpm_latch: Arc<Mutex<BufferPoolManager>>,
    disk_scheduler: Arc<DiskScheduler>,
    is_valid: bool,
}
