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
    fn get_data(&self) -> &[u8] {
        &self.data
    }
    fn get_mut_data(&mut self) -> &mut [u8] {
        &mut self.data
    }
    fn reset(&mut self) {
        self.data.fill(0);
        self.pin_count = 0;
        self.is_dirty = false
    }
}
struct BufferPoolManager {}
