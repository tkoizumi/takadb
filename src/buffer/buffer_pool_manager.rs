use crate::constants::PAGE_SIZE;

struct FrameHeader {
    frame_id: usize,
    pin_count: usize,
    is_dirty: bool,
    data: [u8; PAGE_SIZE],
}

impl FrameHeader {
    fn new(frame_id: usize) -> Self {
        Self {
            frame_id,
            pin_count: 0,
            is_dirty: false,
            data: [0u8; 4096],
        }
    }
}
struct BufferPoolManager {}
