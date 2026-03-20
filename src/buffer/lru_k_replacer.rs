#![allow(unused_variables)]
#![allow(dead_code)]

use std::collections::{HashMap, VecDeque, hash_map::Entry};
use std::sync::Mutex;

use super::AccessType;

type FrameId = usize;

fn assert_valid_frame(frame_id: usize, replacer_size: usize) {
    assert!(
        frame_id < replacer_size,
        "frame_id {} is out of bounds (max: {})",
        frame_id,
        replacer_size
    );
}

#[derive(Debug)]
struct FrameEntry {
    access_history: VecDeque<usize>, // Stores up to K timestamps
    is_evictable: bool,
    k: usize,
}

impl FrameEntry {
    fn new(k: usize) -> Self {
        Self {
            access_history: VecDeque::with_capacity(k),
            is_evictable: false,
            k,
        }
    }
}

#[derive(Debug)]
pub struct LruKReplacer {
    replacer_size: usize,
    k: usize,
    latch: Mutex<LruKInternal>,
}

#[derive(Debug)]
struct LruKInternal {
    entries: HashMap<FrameId, FrameEntry>,
    current_size: usize,
    current_timestamp: usize,
}

impl LruKReplacer {
    fn new(k: usize, replacer_size: usize) -> Self {
        let lru_k_internal = LruKInternal {
            entries: HashMap::with_capacity(replacer_size),
            current_size: 0,
            current_timestamp: 0,
        };
        Self {
            replacer_size,
            k,
            latch: Mutex::new(lru_k_internal),
        }
    }

    fn evict(&mut self) -> Option<usize> {
        let mut internal = self.latch.lock().unwrap();
        let inf_frame_entry = internal
            .entries
            .iter()
            .filter(|(_, v)| v.is_evictable && v.access_history.len() < self.k)
            .min_by_key(|(_, v)| v.access_history.front());

        let f_frame_entry = internal
            .entries
            .iter()
            .filter(|(_, v)| v.is_evictable && v.access_history.len() >= self.k)
            .min_by_key(|(_, v)| v.access_history.front());

        let evict_id = inf_frame_entry.or(f_frame_entry).map(|(&id, _)| id);
        if let Some(id) = evict_id {
            internal.entries.remove(&id);
            Some(id)
        } else {
            None
        }
    }

    fn record_access(&mut self, frame_id: FrameId, access_type: AccessType) {
        assert_valid_frame(frame_id, self.replacer_size);

        let mut internal = self.latch.lock().unwrap();
        let curr_timestamp = internal.current_timestamp;

        let entry = internal
            .entries
            .entry(frame_id)
            .or_insert_with(|| FrameEntry::new(self.k));

        match access_type {
            AccessType::Scan => {
                if entry.access_history.len() < self.k {
                    entry.access_history.push_back(curr_timestamp);
                    internal.current_timestamp += 1;
                }
            }
            AccessType::Lookup | AccessType::Write => {
                if entry.access_history.len() >= self.k {
                    entry.access_history.pop_front();
                }
                entry.access_history.push_back(curr_timestamp);
                internal.current_timestamp += 1;
            }
        }
    }

    fn set_evictable(&mut self, frame_id: usize, set_evictable: bool) {
        assert_valid_frame(frame_id, self.replacer_size);
        let mut internal = self.latch.lock().unwrap();
        let mut delta: i32 = 0;

        if let Some(entry) = internal.entries.get_mut(&frame_id) {
            if set_evictable && !entry.is_evictable {
                delta = 1;
            } else if !set_evictable && entry.is_evictable {
                delta = -1;
            }
            entry.is_evictable = set_evictable;
        };
        if delta == 1 {
            internal.current_size += 1;
        } else if delta == -1 {
            internal.current_size -= 1;
        }
    }

    fn remove(&mut self, frame_id: usize) {
        assert_valid_frame(frame_id, self.replacer_size);
        let mut internal = self.latch.lock().unwrap();
        if let Entry::Occupied(entry) = internal.entries.entry(frame_id) {
            let frame = entry.get();
            assert!(
                frame.is_evictable,
                "frame_id {} is not evictable (evictable: {})",
                frame_id, frame.is_evictable
            );
            if frame.is_evictable {
                entry.remove_entry();
                internal.current_size -= 1;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_access_internal_state() {
        let mut replacer = LruKReplacer::new(2, 10);
        replacer.record_access(1, AccessType::Lookup);
        {
            let internal = replacer.latch.lock().unwrap();
            // 1. Test New Entry Creation
            assert!(internal.entries.contains_key(&1));

            let entry = internal.entries.get(&1).unwrap();
            assert_eq!(entry.access_history.len(), 1);
            assert_eq!(internal.current_timestamp, 1);
        }

        // 2. Test Reaching K
        replacer.record_access(1, AccessType::Lookup);
        {
            let internal = replacer.latch.lock().unwrap();
            let entry = internal.entries.get(&1).unwrap();
            assert_eq!(entry.access_history.len(), 2);
            // Timestamps should be [0, 1]
            assert_eq!(entry.access_history[0], 0);
            assert_eq!(entry.access_history[1], 1);
        }

        // 3. Test Sliding Window (Popping oldest)
        // Accessing again should drop timestamp 0 and add timestamp 2
        replacer.record_access(1, AccessType::Lookup);
        {
            let internal = replacer.latch.lock().unwrap();
            let entry = internal.entries.get(&1).unwrap();
            println!("{:#?}", entry);
            assert_eq!(entry.access_history.len(), 2);
            assert_eq!(entry.access_history[0], 1);
            assert_eq!(entry.access_history[1], 2);
        }

        // 4. Test Scan Resistance
        // Frame 1 already has 2 accesses (which is K).
        // A Scan should NOT add a new timestamp or increment the global clock.
        replacer.record_access(1, AccessType::Scan);
        {
            let internal = replacer.latch.lock().unwrap();
            let entry = internal.entries.get(&1).unwrap();
            assert_eq!(entry.access_history.len(), 2);
            assert_eq!(entry.access_history[1], 2); // Still the old timestamp
            assert_eq!(internal.current_timestamp, 3); // Clock stayed at 3
        }
    }
}

#[test]
fn test_set_evictable_internal_state() {
    let mut replacer = LruKReplacer::new(2, 10);

    // 1. Setup: Access two frames. They start as NOT evictable.
    replacer.record_access(1, AccessType::Lookup);
    replacer.record_access(2, AccessType::Lookup);

    {
        let internal = replacer.latch.lock().unwrap();
        assert_eq!(internal.current_size, 0, "Initial size should be 0");
    }

    // 2. Set Frame 1 to evictable
    replacer.set_evictable(1, true);

    {
        let internal = replacer.latch.lock().unwrap();
        assert_eq!(internal.current_size, 1, "Size should increment to 1");
        assert!(internal.entries.get(&1).unwrap().is_evictable);
    }

    // 3. Test Idempotency: Set Frame 1 to evictable AGAIN
    replacer.set_evictable(1, true);

    {
        let internal = replacer.latch.lock().unwrap();
        assert_eq!(
            internal.current_size, 1,
            "Size should NOT increment on redundant call"
        );
    }

    // 4. Set Frame 2 to evictable
    replacer.set_evictable(2, true);

    {
        let internal = replacer.latch.lock().unwrap();
        assert_eq!(internal.current_size, 2, "Size should be 2");
    }

    // 5. Pin Frame 1 (Set non-evictable)
    replacer.set_evictable(1, false);

    {
        let internal = replacer.latch.lock().unwrap();
        assert_eq!(internal.current_size, 1, "Size should decrement to 1");
        assert!(!internal.entries.get(&1).unwrap().is_evictable);
    }
}

#[test]
fn test_removal() {
    let mut replacer = LruKReplacer::new(2, 10);
    replacer.record_access(1, AccessType::Lookup);
    replacer.record_access(2, AccessType::Lookup);
    replacer.set_evictable(1, true);
    replacer.set_evictable(2, true);
    replacer.remove(1);

    {
        let internal = replacer.latch.lock().unwrap();
        assert_eq!(
            internal.current_size, 1,
            "Size should be 1 as there are two evicable and one was removed"
        );
    }
}

#[test]
fn test_evict_1() {
    let mut replacer = LruKReplacer::new(2, 10);
    replacer.record_access(1, AccessType::Lookup);
    replacer.record_access(2, AccessType::Lookup);
    replacer.set_evictable(1, true);
    replacer.set_evictable(2, false);

    let evicted_id = replacer.evict();

    {
        let internal = replacer.latch.lock().unwrap();
        assert_eq!(
            evicted_id.unwrap(),
            1,
            "Frame 1 should be evicted as 2 is set not to be evicable"
        );
    }
}

#[test]
fn test_evict_2() {
    let mut replacer = LruKReplacer::new(2, 10);
    replacer.record_access(1, AccessType::Lookup);
    replacer.record_access(2, AccessType::Lookup);
    replacer.set_evictable(1, true);
    replacer.set_evictable(2, true);

    let evicted_id = replacer.evict();

    {
        let internal = replacer.latch.lock().unwrap();
        assert_eq!(
            evicted_id.unwrap(),
            1,
            "Frame 1 should be evicted as 1 and 2 have inf backward k distance but 1 has the oldest timestamp"
        );
    }
}

#[test]
fn test_evict_3() {
    let mut replacer = LruKReplacer::new(2, 10);
    replacer.record_access(2, AccessType::Lookup);
    replacer.record_access(1, AccessType::Lookup);
    replacer.record_access(1, AccessType::Lookup);
    replacer.record_access(2, AccessType::Lookup);
    replacer.set_evictable(1, true);
    replacer.set_evictable(2, true);

    let evicted_id = replacer.evict();

    {
        let internal = replacer.latch.lock().unwrap();
        assert_eq!(
            evicted_id.unwrap(),
            2,
            "Frame 2 should be evicted as it has the biggest backward K distance"
        );
    }
}
