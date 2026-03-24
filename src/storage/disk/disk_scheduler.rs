use crate::storage::disk::disk_manager::DiskManager;
use std::sync::mpsc::{Sender, channel};
use std::sync::{Arc, Mutex};
use std::thread::{self};

struct DiskRequest {
    is_write: bool,
    data: Vec<u8>,
    page_id: usize,
}

impl DiskRequest {
    fn new(is_write: bool, data: Vec<u8>, page_id: usize) -> Self {
        Self {
            is_write,
            data,
            page_id,
        }
    }
}

struct DiskScheduler {
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
            while let Ok(request) = receiver.recv() {
                match request {
                    Some(mut disk_request) => {
                        let mut manager = arc_disk_manager_clone.lock().unwrap();
                        if disk_request.is_write {
                            manager
                                .write_page(disk_request.page_id, &disk_request.data)
                                .expect("Failed to write page.");
                        } else {
                            manager
                                .read_page(disk_request.page_id, &mut disk_request.data)
                                .expect("Failed to read page.");
                        }
                    }
                    None => break,
                }
            }
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
}
