#![allow(unused_variables)]
#![allow(dead_code)]

use crate::buffer::lru_k_replacer::LruKReplacer;
use crate::constants::{NUM_NEW_PAGES, PAGE_SIZE};
use crate::storage::disk::disk_manager::DiskManager;
use crate::storage::disk::disk_scheduler::DiskScheduler;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize};

pub struct FrameHeader {
    pub frame_id: usize,
    pub page_id: usize,
    pub pin_count: AtomicUsize,
    pub is_dirty: AtomicBool,
    pub data: [u8; PAGE_SIZE],
}

const INVALID_PAGE_ID: usize = usize::MAX;

impl FrameHeader {
    fn new(frame_id: usize) -> Self {
        Self {
            frame_id,
            page_id: INVALID_PAGE_ID,
            pin_count: AtomicUsize::new(0),
            is_dirty: AtomicBool::new(false),
            data: [0u8; 4096],
        }
    }
    fn get_data(&self) -> &[u8] {
        &self.data
    }
    fn get_mut_data(&mut self) -> &mut [u8] {
        &mut self.data
    }
    fn reset(&mut self) {
        self.data.fill(0);
        self.pin_count = AtomicUsize::new(0);
        self.is_dirty = AtomicBool::new(false);
    }
}

type PageId = usize;
type FrameId = usize;
type NumFrames = usize;

pub struct BufferPoolManager {
    num_frames: usize,
    next_page_id: usize,
    frames: Vec<FrameHeader>,
    page_table: HashMap<PageId, FrameId>,
    free_frames: Vec<FrameId>,
    replacer: LruKReplacer,
    disk_scheduler: DiskScheduler,
}

impl BufferPoolManager {
    pub fn new(num_frames: usize, disk_manager: DiskManager) -> Self {
        let mut frames: Vec<FrameHeader> = Vec::with_capacity(num_frames);
        let mut free_frames: Vec<usize> = Vec::with_capacity(num_frames);
        for i in 0..num_frames {
            frames.push(FrameHeader::new(i));
            free_frames.push(i);
        }
        let replacer = LruKReplacer::new(num_frames, NUM_NEW_PAGES);
        let disk_scheduler = DiskScheduler::new(disk_manager);
        Self {
            num_frames,
            next_page_id: 0,
            frames,
            page_table: HashMap::new(),
            free_frames,
            replacer,
            disk_scheduler,
        }
    }
    pub fn size(self) -> NumFrames {
        self.num_frames
    }
}
