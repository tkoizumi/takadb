#![allow(unused_variables)]
#![allow(dead_code)]

use crate::constants::{NUM_NEW_PAGES, PAGE_SIZE};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Result as io_Result, Seek, SeekFrom, Write};
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
    fn write_page(&mut self, page_id: usize, data: &[u8]) -> io_Result<()> {
        if data.len() > PAGE_SIZE {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                "Data is bigger then page size",
            ));
        }
        let offset = if let Some(&existing_offset) = self.pages.get(&page_id) {
            existing_offset
        } else {
            let new_offset = self.allocate_page()?;
            self.pages.insert(page_id, new_offset);
            self.page_count += 1;
            new_offset
        };
        self.db_io.seek(SeekFrom::Start(offset as u64))?;
        self.db_io.write_all(data)?;
        self.num_writes += 1;
        self.db_io.flush()?;
        self.num_flushes += 1;
        Ok(())
    }
    fn allocate_page(&mut self) -> Result<usize, Error> {
        if let Some(offset) = self.free_slots.pop() {
            return Ok(offset);
        }

        let orig_file_size = self.file_size;
        let new_file_size = self.file_size + NUM_NEW_PAGES * PAGE_SIZE;
        self.db_io.set_len(new_file_size as u64)?;

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
        let file_name = PathBuf::from("test_file_new");
        if let Ok(disk_manager) = DiskManager::new(file_name.clone()) {
            assert_eq!(
                disk_manager.num_flushes, 0,
                "There should be 0 num_flushes when initialized."
            );
        }
        std::fs::remove_file(file_name).expect("Failed to remove file.");
    }

    #[test]
    fn test_allocate() {
        let file_name = PathBuf::from("test_file_write.db");
        let Ok(mut disk_manager) = DiskManager::new(file_name.clone()) else {
            return;
        };
        let Ok(offset) = disk_manager.allocate_page() else {
            return;
        };

        assert_eq!(offset, 0, "Offset is 0 as this is the first page on disk");
        std::fs::remove_file(file_name).expect("Failed to remove file.");
    }

    #[test]
    fn test_write() {
        let file_name = PathBuf::from("test_file_allocate");
        let Ok(mut disk_manager) = DiskManager::new(file_name.clone()) else {
            return;
        };
        let page_id = 1;
        let mut data = vec![0u8; PAGE_SIZE];
        data[0..4].copy_from_slice(b"test");

        disk_manager
            .write_page(page_id, &data)
            .expect("Failed to write page");

        assert_eq!(
            disk_manager.num_flushes, 1,
            "Should have incremented flush by 1"
        );
        assert_eq!(
            disk_manager.num_writes, 1,
            "Should have incremented write by 1"
        );
        assert!(
            disk_manager.pages.contains_key(&page_id),
            "Page id should be in the hashmap"
        );
        std::fs::remove_file(file_name).expect("Failed to remove file.");
    }
}
