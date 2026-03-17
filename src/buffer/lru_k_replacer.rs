use std::collections::{HashMap, VecDeque};
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
    entries: HashMap<FrameId, FrameEntry>,
    replacer_size: usize,
    k: usize,
    latch: Mutex<LruKInternal>,
    current_timestamp: usize,
}

impl LruKReplacer {
    fn new(k: usize, replacer_size: usize) -> Self {
        let lru_k_internal = LruKInternal {
            frames: HashMap::with_capacity(replacer_size),
            current_size: 0,
        };
        Self {
            entries: HashMap::with_capacity(k),
            replacer_size: k,
            k,
            latch: Mutex::new(lru_k_internal),
            current_timestamp: 0,
        }
    }

    fn record_access(&mut self, frame_id: FrameId, access_type: AccessType) {
        assert_valid_frame(frame_id, self.replacer_size);

        let entry = self
            .entries
            .entry(frame_id)
            .or_insert_with(|| FrameEntry::new(self.k));

        match access_type {
            AccessType::Scan => {
                if entry.access_history.len() < self.k {
                    entry.access_history.push_back(self.current_timestamp);
                    self.current_timestamp += 1;
                }
            }
            AccessType::Lookup | AccessType::Write => {
                if entry.access_history.len() >= self.k {
                    entry.access_history.pop_front();
                }
                entry.access_history.push_back(self.current_timestamp);
                self.current_timestamp += 1;
            }
        }
    }

    fn set_evictable(frame_id: usize, set_evictable: bool) {}
}
#[derive(Debug)]
struct LruKInternal {
    // This tracks the actual frame data to keep the Mutex lock area small
    frames: HashMap<FrameId, FrameEntry>,
    current_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_access_internal_state() {
        // K=2, Capacity=10
        let mut replacer = LruKReplacer::new(2, 10);

        // 1. Test New Entry Creation
        replacer.record_access(1, AccessType::Lookup);
        assert!(replacer.entries.contains_key(&1));

        let entry = replacer.entries.get(&1).unwrap();
        assert_eq!(entry.access_history.len(), 1);
        assert_eq!(replacer.current_timestamp, 1);

        // 2. Test Reaching K
        replacer.record_access(1, AccessType::Lookup);
        let entry = replacer.entries.get(&1).unwrap();
        assert_eq!(entry.access_history.len(), 2);
        // Timestamps should be [0, 1]
        assert_eq!(entry.access_history[0], 0);
        assert_eq!(entry.access_history[1], 1);

        // 3. Test Sliding Window (Popping oldest)
        // Accessing again should drop timestamp 0 and add timestamp 2
        replacer.record_access(1, AccessType::Lookup);
        let entry = replacer.entries.get(&1).unwrap();
        println!("{:#?}", entry);
        assert_eq!(entry.access_history.len(), 2);
        assert_eq!(entry.access_history[0], 1);
        assert_eq!(entry.access_history[1], 2);

        // 4. Test Scan Resistance
        // Frame 1 already has 2 accesses (which is K).
        // A Scan should NOT add a new timestamp or increment the global clock.
        replacer.record_access(1, AccessType::Scan);
        let entry = replacer.entries.get(&1).unwrap();
        assert_eq!(entry.access_history.len(), 2);
        assert_eq!(entry.access_history[1], 2); // Still the old timestamp
        assert_eq!(replacer.current_timestamp, 3); // Clock stayed at 3
    }
}
