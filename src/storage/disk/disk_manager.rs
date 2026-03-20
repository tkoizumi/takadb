use std::collections::HashMap;
use std::fs::File;

pub struct DiskManager {
    db_io: File,
    num_flushes: usize,
    num_writes: usize,
    num_deletes: usize,
    db_file_name: String,
    pages: HashMap<usize, usize>,
    page_count: usize,
    file_size: usize,
    free_slots: Vec<usize>,
}

impl DiskManager {
    fn new() -> Self {}
    fn read_page() {}
    fn write_page() {}
}
