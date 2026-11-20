//! Port of `stack.cc` — chunked LIFO stack (`BLI_Stack`), generic over `T`.

const CHUNK_SIZE_DEFAULT: usize = 1 << 16;
const CHUNK_ELEM_MIN: usize = 32;

struct StackChunk<T> {
    data: Vec<T>,
    len: usize,
}

fn stack_chunk_elem_max_calc(elem_size: usize, mut chunk_size: usize) -> usize {
    let elem_size_min = elem_size * CHUNK_ELEM_MIN;
    debug_assert!(elem_size != 0 && chunk_size != 0);
    while chunk_size <= elem_size_min {
        chunk_size <<= 1;
    }
    chunk_size / elem_size
}

/// `BLI_Stack`.
pub struct Stack<T> {
    chunks: Vec<StackChunk<T>>,
    chunk_free: Vec<StackChunk<T>>,
    chunk_index: usize,
    chunk_elem_max: usize,
}

impl<T: Default> Stack<T> {
    /// `BLI_stack_new_ex`.
    pub fn new_ex(chunk_size: usize) -> Self {
        let chunk_elem_max = stack_chunk_elem_max_calc(std::mem::size_of::<T>(), chunk_size);
        Stack {
            chunks: Vec::new(),
            chunk_free: Vec::new(),
            chunk_index: chunk_elem_max - 1,
            chunk_elem_max,
        }
    }
    /// `BLI_stack_new`.
    pub fn new() -> Self {
        Self::new_ex(CHUNK_SIZE_DEFAULT)
    }

    /// `BLI_stack_push_r` — grow and return slot (push via `push`).
    pub fn push(&mut self, item: T) {
        if self.chunks.is_empty() {
            self.chunks.push(StackChunk {
                data: Vec::with_capacity(self.chunk_elem_max),
                len: 0,
            });
            self.chunk_index = 0;
        } else if self.chunk_index + 1 >= self.chunk_elem_max {
            let mut chunk = self.chunk_free.pop().unwrap_or_else(|| StackChunk {
                data: Vec::with_capacity(self.chunk_elem_max),
                len: 0,
            });
            chunk.len = 0;
            self.chunks.push(chunk);
            self.chunk_index = 0;
        } else {
            self.chunk_index += 1;
        }
        let chunk = self.chunks.last_mut().unwrap();
        if chunk.len <= self.chunk_index {
            chunk.data.push(item);
            chunk.len += 1;
        } else {
            chunk.data[self.chunk_index] = item;
            chunk.len = self.chunk_index + 1;
        }
    }

    /// `BLI_stack_peek`.
    pub fn peek(&mut self) -> Option<&mut T> {
        if self.chunks.is_empty() {
            return None;
        }
        let chunk = self.chunks.last_mut().unwrap();
        if chunk.len == 0 {
            return None;
        }
        Some(&mut chunk.data[self.chunk_index])
    }

    /// `BLI_stack_pop`.
    pub fn pop(&mut self) -> Option<T> {
        if self.chunks.is_empty() {
            return None;
        }
        let chunk = self.chunks.last_mut().unwrap();
        let mut out = T::default();
        std::mem::swap(&mut out, &mut chunk.data[self.chunk_index]);
        if self.chunk_index == 0 {
            chunk.len = 0;
            let chunk = self.chunks.pop().unwrap();
            self.chunk_free.push(chunk);
            self.chunk_index = self.chunk_elem_max - 1;
        } else {
            self.chunk_index -= 1;
        }
        Some(out)
    }

    /// `BLI_stack_pop_n` — pop `n` into `dst` (reverse order).
    pub fn pop_n(&mut self, n: usize) -> Vec<T> {
        let mut out = Vec::with_capacity(n);
        for _ in 0..n {
            out.push(self.pop().expect("BLI_stack_pop_n underflow"));
        }
        out
    }
    /// `BLI_stack_pop_n_reverse` — pop `n`, same order as pushed.
    pub fn pop_n_reverse(&mut self, n: usize) -> Vec<T> {
        let mut v = self.pop_n(n);
        v.reverse();
        v
    }

    /// `BLI_stack_discard` — drop top without reading.
    pub fn discard(&mut self) {
        self.pop();
    }

    /// `BLI_stack_clear` (keeps one free chunk like the C impl).
    pub fn clear(&mut self) {
        self.chunk_free.extend(self.chunks.drain(..));
        self.chunk_index = self.chunk_elem_max - 1;
    }

    /// `BLI_stack_count`.
    pub fn count(&self) -> usize {
        let full = self.chunks.len().saturating_sub(1) * self.chunk_elem_max;
        full + self.chunks.last().map_or(0, |c| c.len)
    }

    /// `BLI_stack_is_empty`.
    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }
}
