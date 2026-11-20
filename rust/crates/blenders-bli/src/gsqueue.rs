//! Port of `gsqueue.cc` — chunked queue, generic over `T` instead of raw bytes.
//! `CHUNK_SIZE_DEFAULT` (1<<16) and `CHUNK_ELEM_MIN` (32) preserved;
//! chunk allocation granularity follows `queue_chunk_elem_max_calc`.

const CHUNK_SIZE_DEFAULT: usize = 1 << 16;
const CHUNK_ELEM_MIN: usize = 32;

struct QueueChunk<T> {
    data: Vec<T>,
    len: usize,
}

/// `queue_chunk_elem_max_calc` — with `MEM_SIZE_OVERHEAD` folded out.
fn chunk_elem_max_calc(elem_size: usize, mut chunk_size: usize) -> usize {
    let elem_size_min = elem_size * CHUNK_ELEM_MIN;
    debug_assert!(elem_size != 0 && chunk_size != 0);
    while chunk_size <= elem_size_min {
        chunk_size <<= 1;
    }
    chunk_size / elem_size
}

/// `GSQueue`.
pub struct GsQueue<T> {
    /// `chunk_first..` — deque of chunks (front = pop side).
    chunks: std::collections::VecDeque<QueueChunk<T>>,
    chunk_free: Vec<QueueChunk<T>>,
    chunk_first_index: usize,
    chunk_last_index: usize,
    chunk_elem_max: usize,
    elem_num: usize,
}

impl<T: Default> GsQueue<T> {
    /// `BLI_gsqueue_new`.
    pub fn new() -> Self {
        let chunk_elem_max = chunk_elem_max_calc(std::mem::size_of::<T>(), CHUNK_SIZE_DEFAULT);
        GsQueue {
            chunks: std::collections::VecDeque::new(),
            chunk_free: Vec::new(),
            chunk_first_index: 0,
            chunk_last_index: chunk_elem_max - 1,
            chunk_elem_max,
            elem_num: 0,
        }
    }

    /// `BLI_gsqueue_push`.
    pub fn push(&mut self, item: T) {
        self.chunk_last_index += 1;
        self.elem_num += 1;
        if self.chunk_last_index == self.chunk_elem_max {
            let chunk = self.chunk_free.pop().unwrap_or_else(|| QueueChunk {
                data: Vec::with_capacity(self.chunk_elem_max),
                len: 0,
            });
            self.chunks.push_back(chunk);
            self.chunk_last_index = 0;
        }
        let chunk = self.chunks.back_mut().unwrap();
        if chunk.len <= self.chunk_last_index {
            chunk.data.push(item);
            chunk.len += 1;
        } else {
            chunk.data[self.chunk_last_index] = item;
        }
    }

    /// `BLI_gsqueue_pop`.
    pub fn pop(&mut self) -> T {
        debug_assert!(!self.is_empty());
        let mut out = T::default();
        {
            let chunk = self.chunks.front_mut().unwrap();
            std::mem::swap(&mut out, &mut chunk.data[self.chunk_first_index]);
        }
        self.chunk_first_index += 1;
        self.elem_num -= 1;
        if self.chunk_first_index == self.chunk_elem_max || self.elem_num == 0 {
            let mut chunk_free = self.chunks.pop_front().unwrap();
            chunk_free.len = 0;
            self.chunk_first_index = 0;
            if self.chunks.is_empty() {
                self.chunk_last_index = self.chunk_elem_max - 1;
            }
            self.chunk_free.push(chunk_free);
        }
        out
    }

    /// `BLI_gsqueue_len`.
    pub fn len(&self) -> usize {
        self.elem_num
    }

    /// `BLI_gsqueue_is_empty`.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}
