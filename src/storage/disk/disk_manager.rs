#![allow(unused_variables)]
#![allow(dead_code)]

use crate::constants::{NUM_NEW_PAGES, PAGE_SIZE};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Error, Result as io_Result};
use std::path::PathBuf;

pub struct DiskManager {
    db_io: File,
    num_flushes: usize,
    num_writes: usize,
    num_deletes: usize,
    db_file_name: PathBuf,
    pages: HashMap<usize, usize>, //records the offset of each page in the db file
    page_count: usize,
    file_size: usize,
    free_slots: Vec<usize>, //records the free slots in the db file if pages are deleted indicated
                            //by offset
}

impl DiskManager {
    fn new(db_file_name: PathBuf) -> io_Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&db_file_name)?;

        let file_size = file.metadata()?.len() as usize;

        Ok(Self {
            db_io: file,
            num_flushes: 0,
            num_writes: 0,
            num_deletes: 0,
            db_file_name,
            pages: HashMap::new(),
            page_count: 0,
            file_size,
            free_slots: vec![],
        })
    }
    fn read_page() {}
    fn write_page() {}
    fn allocate_page(&mut self) -> Result<usize, Error> {
        if let Some(offset) = self.free_slots.pop() {
            return Ok(offset);
        }

        let orig_file_size = self.file_size;
        let new_file_size = self.file_size + NUM_NEW_PAGES * PAGE_SIZE;
        self.db_io.set_len(new_file_size as u64)?;

        self.page_count += 1;
        self.pages.insert(self.page_count, orig_file_size);

        let mut curr_file_size = orig_file_size;

        for _ in 1..NUM_NEW_PAGES {
            curr_file_size += PAGE_SIZE;
            self.free_slots.push(curr_file_size);
        }
        self.file_size = new_file_size;
        Ok(orig_file_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let file_name = PathBuf::from("test_file");
        if let Ok(disk_manager) = DiskManager::new(file_name) {
            assert_eq!(
                disk_manager.num_flushes, 0,
                "There should be 0 num_flushes when initialized."
            );
        }
    }
}
