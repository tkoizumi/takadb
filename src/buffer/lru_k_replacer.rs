use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::Instant;

type FrameId = usize;

struct FrameEntry {
    access_history: VecDeque<Instant>, // Stores up to K timestamps
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

pub struct LruKReplacer {
    entries: HashMap<FrameId, FrameEntry>,
    replacer_size: usize,
    k: usize,
    // Using a Mutex here replicates the 'std::mutex latch_' in the C++ header
    latch: Mutex<LruKInternal>,
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
        }
    }
}

struct LruKInternal {
    // This tracks the actual frame data to keep the Mutex lock area small
    frames: HashMap<FrameId, FrameEntry>,
    current_size: usize,
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_lru_k_eviction() {
        let replacer = LruKReplacer::new(2, 10);
    }
}
