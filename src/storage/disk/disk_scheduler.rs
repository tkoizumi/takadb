#![allow(unused_variables)]
#![allow(dead_code)]

use crate::buffer::buffer_pool_manager::FrameHeader;
use crate::storage::disk::disk_manager::DiskManager;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::{self};

pub struct DiskRequest {
    is_write: bool,
    frame: Arc<FrameHeader>,
    page_id: usize,
    callback: Sender<()>,
}

impl DiskRequest {
    pub fn new(
        is_write: bool,
        frame: Arc<FrameHeader>,
        page_id: usize,
        callback: Sender<()>,
    ) -> Self {
        Self {
            is_write,
            frame,
            page_id,
            callback,
        }
    }
}

pub struct DiskScheduler {
    disk_manager: Arc<Mutex<DiskManager>>,
    request_queue: Sender<Option<DiskRequest>>,
    background_thread: Option<thread::JoinHandle<()>>,
}

impl DiskScheduler {
    pub fn new(disk_manager: DiskManager) -> Self {
        let arc_disk_manager = Arc::new(Mutex::new(disk_manager));
        let arc_disk_manager_clone = arc_disk_manager.clone();

        let (sender, receiver) = channel::<Option<DiskRequest>>();
        let background_thread_join_handle = thread::spawn(move || {
            Self::start_worker_thread(arc_disk_manager_clone, receiver);
        });
        Self {
            disk_manager: arc_disk_manager,
            request_queue: sender,
            background_thread: Some(background_thread_join_handle),
        }
    }
    pub fn schedule(&mut self, requests: Vec<DiskRequest>) {
        requests.into_iter().for_each(|request| {
            self.request_queue
                .send(Some(request))
                .expect("Failed to send request to disk worker")
        });
    }
    pub fn start_worker_thread(
        disk_manager: Arc<Mutex<DiskManager>>,
        receiver: Receiver<Option<DiskRequest>>,
    ) {
        while let Ok(request) = receiver.recv() {
            match request {
                Some(disk_request) => {
                    let mut manager = disk_manager.lock().unwrap();
                    let mut data = disk_request.frame.data.write().unwrap();
                    if disk_request.is_write {
                        manager
                            .write_page(disk_request.page_id, &*data)
                            .expect("Failed to write page.");
                    } else if manager.read_page(disk_request.page_id, &mut *data).is_err() {
                        data.fill(0);
                    }
                    disk_request.callback.send(()).unwrap();
                }
                None => break,
            }
        }
    }
}

#[test]
fn test_scheduler() {
    use crate::constants::PAGE_SIZE;
    use std::fs::remove_file;
    use std::path::PathBuf;

    let (writer_sender, writer_receiver) = channel::<()>();

    let frame = Arc::new(FrameHeader::new(1));
    frame.data.write().unwrap()[0..4].copy_from_slice(b"test");

    let writer_req = DiskRequest::new(true, frame.clone(), 1, writer_sender);
    let writer_requests: Vec<DiskRequest> = vec![writer_req];

    let file_name = PathBuf::from("test_scheduler.db");
    let disk_manager = DiskManager::new(file_name.clone()).expect("Failed to create DiskManager");

    let mut scheduler = DiskScheduler::new(disk_manager);
    scheduler.schedule(writer_requests);

    writer_receiver.recv().expect("Failed to write data");

    let (reader_sender, reader_receiver) = channel::<()>();
    let reader_data = vec![0u8; PAGE_SIZE];

    let reader_req = DiskRequest::new(false, frame.clone(), 1, reader_sender);
    let reader_requests: Vec<DiskRequest> = vec![reader_req];
    scheduler.schedule(reader_requests);

    let res = reader_receiver
        .recv()
        .expect("Failed to receive reader data");

    remove_file(&file_name).expect("File was not removed");

    assert_eq!(
        &frame.data.write().unwrap()[0..4],
        b"test",
        "Page 1 read data should be the same as writer data"
    );
}
