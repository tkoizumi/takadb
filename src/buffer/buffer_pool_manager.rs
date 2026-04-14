#![allow(unused_variables)]
#![allow(dead_code)]

use crate::buffer::AccessType;
use crate::buffer::AccessType::{Read, Write};
use crate::buffer::lru_k_replacer::LruKReplacer;
use crate::constants::{NUM_NEW_PAGES, PAGE_SIZE};
use crate::storage::disk::disk_manager::DiskManager;
use crate::storage::disk::disk_scheduler::{DiskRequest, DiskScheduler};
use crate::storage::page::page_guard::{ReadPageGuard, WritePageGuard};

use std::collections::HashMap;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub struct FrameHeader {
    pub frame_id: usize,
    pub page_id: AtomicUsize,
    pub pin_count: AtomicUsize,
    pub is_dirty: AtomicBool,
    pub data: RwLock<[u8; PAGE_SIZE]>,
}

const INVALID_PAGE_ID: usize = usize::MAX;

impl FrameHeader {
    pub fn new(frame_id: usize) -> Self {
        Self {
            frame_id,
            page_id: AtomicUsize::new(INVALID_PAGE_ID),
            pin_count: AtomicUsize::new(0),
            is_dirty: AtomicBool::new(false),
            data: RwLock::new([0u8; 4096]),
        }
    }
    pub fn get_data(&self) -> RwLockReadGuard<'_, [u8; PAGE_SIZE]> {
        self.data.read().unwrap()
    }
    pub fn get_mut_data(&self) -> RwLockWriteGuard<'_, [u8; PAGE_SIZE]> {
        self.data.write().unwrap()
    }

    pub fn reset(&mut self) {
        let mut data_guard = self.data.write().expect("Unable to acquire lock");
        data_guard.fill(0);
        self.pin_count.store(0, SeqCst);
        self.is_dirty.store(false, SeqCst);
    }
}

type PageId = usize;
type FrameId = usize;
type NumFrames = usize;

pub struct BufferPoolManager {
    num_frames: usize,
    next_page_id: AtomicUsize,
    frames: Vec<Arc<FrameHeader>>,
    page_table: HashMap<PageId, FrameId>,
    free_frames: Vec<FrameId>,
    replacer: Arc<LruKReplacer>,
    disk_scheduler: Arc<Mutex<DiskScheduler>>,
}

impl BufferPoolManager {
    pub fn new(num_frames: usize, disk_manager: DiskManager) -> Self {
        let mut frames: Vec<Arc<FrameHeader>> = Vec::with_capacity(num_frames);
        let mut free_frames: Vec<usize> = Vec::with_capacity(num_frames);
        for i in 0..num_frames {
            frames.push(Arc::new(FrameHeader::new(i)));
            free_frames.push(i);
        }
        let replacer = LruKReplacer::new(num_frames, NUM_NEW_PAGES);
        let disk_scheduler = DiskScheduler::new(disk_manager);
        Self {
            num_frames,
            next_page_id: AtomicUsize::new(0),
            frames,
            page_table: HashMap::new(),
            free_frames,
            replacer: Arc::new(replacer),
            disk_scheduler: Arc::new(Mutex::new(disk_scheduler)),
        }
    }
    pub fn size(self) -> NumFrames {
        self.num_frames
    }
    pub fn new_page(&self) -> PageId {
        self.next_page_id.fetch_add(1, SeqCst)
    }
    pub fn get_pin_count(&self, page_id: PageId) -> Option<usize> {
        if let Some(&frame_id) = self.page_table.get(&page_id) {
            let frame_guard = &self.frames[frame_id];
            Some(frame_guard.pin_count.load(SeqCst))
        } else {
            None
        }
    }
    fn pin_frame(&self, frame_id: usize) -> Arc<FrameHeader> {
        let frame = self.frames[frame_id].clone();
        frame.pin_count.fetch_add(1, SeqCst);
        frame
    }
    fn register_with_repacer(&self, frame_id: usize, access_type: AccessType) {
        self.replacer.record_access(frame_id, access_type);
        self.replacer.set_evictable(frame_id, false);
    }
    fn load_page_from_disk(&self, frame: &Arc<FrameHeader>, page_id: PageId) {
        let (tx, rx) = channel::<()>();
        let req = DiskRequest::new(false, frame.clone(), page_id, tx);
        let requests: Vec<DiskRequest> = vec![req];

        self.disk_scheduler.lock().unwrap().schedule(requests);

        rx.recv().unwrap();
    }
    fn reset_frame(&self, frame: &Arc<FrameHeader>, page_id: PageId) {
        frame.page_id.store(page_id, SeqCst);
        frame.is_dirty.store(false, SeqCst);
    }
    pub fn checked_read_page(&mut self, page_id: PageId) -> Option<ReadPageGuard> {
        if let Some(&frame_id) = self.page_table.get(&page_id) {
            let frame = self.pin_frame(frame_id);

            self.register_with_repacer(frame_id, Read);
            Some(ReadPageGuard::new(
                page_id,
                frame,
                self.replacer.clone(),
                self.disk_scheduler.clone(),
            ))
        } else if let Some(frame_id) = self.free_frames.pop() {
            let frame = self.pin_frame(frame_id);
            self.reset_frame(&frame, page_id);

            self.load_page_from_disk(&frame, page_id);
            self.page_table.insert(page_id, frame_id);

            self.register_with_repacer(frame_id, Read);
            Some(ReadPageGuard::new(
                page_id,
                frame,
                self.replacer.clone(),
                self.disk_scheduler.clone(),
            ))
        } else if let Some(frame_id) = self.replacer.evict() {
            let frame = self.pin_frame(frame_id);
            if frame.is_dirty.load(SeqCst) {
                //flush page
            }
            self.page_table.remove(&frame.page_id.load(SeqCst));

            self.reset_frame(&frame, page_id);

            self.load_page_from_disk(&frame, page_id);
            self.page_table.insert(page_id, frame_id);

            self.register_with_repacer(frame_id, Read);
            Some(ReadPageGuard::new(
                page_id,
                frame,
                self.replacer.clone(),
                self.disk_scheduler.clone(),
            ))
        } else {
            None
        }
    }
    pub fn checked_write_page(&mut self, page_id: PageId) -> Option<WritePageGuard> {
        if let Some(&frame_id) = self.page_table.get(&page_id) {
            let frame = self.pin_frame(frame_id);

            self.register_with_repacer(frame_id, Write);
            Some(WritePageGuard::new(
                page_id,
                frame,
                self.replacer.clone(),
                self.disk_scheduler.clone(),
            ))
        } else if let Some(frame_id) = self.free_frames.pop() {
            let frame = self.pin_frame(frame_id);
            self.reset_frame(&frame, page_id);

            self.load_page_from_disk(&frame, page_id);
            self.page_table.insert(page_id, frame_id);

            self.register_with_repacer(frame_id, Write);
            Some(WritePageGuard::new(
                page_id,
                frame,
                self.replacer.clone(),
                self.disk_scheduler.clone(),
            ))
        } else if let Some(frame_id) = self.replacer.evict() {
            let frame = self.pin_frame(frame_id);
            if frame.is_dirty.load(SeqCst) {
                //flush page
            }
            self.page_table.remove(&frame.page_id.load(SeqCst));

            self.reset_frame(&frame, page_id);

            self.load_page_from_disk(&frame, page_id);
            self.page_table.insert(page_id, frame_id);

            self.register_with_repacer(frame_id, Write);
            Some(WritePageGuard::new(
                page_id,
                frame,
                self.replacer.clone(),
                self.disk_scheduler.clone(),
            ))
        } else {
            None
        }
    }
}
