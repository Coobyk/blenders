//! Port of `BLI_memblock.cc` — chunked bump allocator
//! (`BLI_memblock`/`BLI_memblock_iter`). Elements are raw bytes;
//! `free_callback` receives each live element's pointer.

const CHUNK_LIST_SIZE: usize = 16;

/// `BLI_memblock` — bump allocator over growable list of fixed-size chunks.
pub struct MemBlock {
    chunk_list: Vec<Option<Vec<u8>>>,
    elem_size: usize,
    /// index of the last allocated element.
    elem_last: isize,
    /// index of the next allocation slot.
    elem_next: usize,
    /// byte offset inside `chunk_list[chunk_next]`.
    elem_next_ofs: usize,
    /// max byte offset per chunk.
    chunk_max_ofs: usize,
    chunk_next: usize,
    chunk_size: usize,
}

/// `MemblockValFreeFP`.
pub type MemblockValFreeFP<'a> = &'a mut dyn FnMut(*mut u8);

impl MemBlock {
    /// `BLI_memblock_create` — default 1<<16 chunk.
    pub fn new(elem_size: usize) -> Self {
        Self::create_ex(elem_size, 1 << 16)
    }
    /// `BLI_memblock_create_ex`.
    pub fn create_ex(elem_size: usize, chunk_size: usize) -> Self {
        assert!(elem_size < chunk_size);
        let mut chunk_list = Vec::with_capacity(CHUNK_LIST_SIZE);
        chunk_list.push(Some(vec![0u8; chunk_size]));
        for _ in 1..CHUNK_LIST_SIZE {
            chunk_list.push(None);
        }
        MemBlock {
            chunk_list,
            elem_size,
            elem_last: -1,
            elem_next: 0,
            elem_next_ofs: 0,
            chunk_max_ofs: (chunk_size / elem_size) * elem_size,
            chunk_next: 0,
            chunk_size,
        }
    }
    fn elem_per_chunk(&self) -> usize {
        self.chunk_size / self.elem_size
    }
    /// `BLI_memblock_alloc` — returns a raw pointer to `elem_size` bytes.
    pub fn alloc(&mut self) -> *mut u8 {
        self.elem_last = self.elem_last.max(self.elem_next as isize);
        self.elem_next += 1;
        let ptr = self.chunk_list[self.chunk_next]
            .as_mut()
            .unwrap()
            .as_mut_ptr();
        let ptr = unsafe { ptr.add(self.elem_next_ofs) };
        self.elem_next_ofs += self.elem_size;
        if self.elem_next_ofs == self.chunk_max_ofs {
            self.elem_next_ofs = 0;
            self.chunk_next += 1;
            if self.chunk_next >= self.chunk_list.len() {
                self.chunk_list.push(Some(vec![0u8; self.chunk_size]));
                self.chunk_list.resize(self.chunk_list.len() + CHUNK_LIST_SIZE - 1, None);
            }
            if self.chunk_list[self.chunk_next].is_none() {
                self.chunk_list[self.chunk_next] = Some(vec![0u8; self.chunk_size]);
            }
        }
        ptr
    }
    /// `BLI_memblock_destroy` — call `free_callback` on live elements, free all.
    pub fn destroy(&mut self, mut free_callback: Option<MemblockValFreeFP>) {
        if let Some(f) = free_callback.as_mut() {
            let epc = self.elem_per_chunk();
            for i in 0..=self.elem_last.max(-1) as usize {
                if self.elem_last < 0 {
                    break;
                }
                let chunk_idx = i / epc;
                let elem_idx = i - epc * chunk_idx;
                let val = unsafe {
                    self.chunk_list[chunk_idx]
                        .as_mut()
                        .unwrap()
                        .as_mut_ptr()
                        .add(self.elem_size * elem_idx)
                };
                f(val);
            }
        }
        self.chunk_list.clear();
    }
    /// `BLI_memblock_clear` — free elements after `elem_next`, keep first chunks.
    pub fn clear(&mut self, mut free_callback: Option<MemblockValFreeFP>) {
        let epc = self.elem_per_chunk();
        let last_used_chunk = self.elem_next / epc;
        if let Some(f) = free_callback.as_mut() {
            let mut i = self.elem_last;
            while i >= self.elem_next as isize {
                let chunk_idx = i as usize / epc;
                let elem_idx = i as usize - epc * chunk_idx;
                let val = unsafe {
                    self.chunk_list[chunk_idx]
                        .as_mut()
                        .unwrap()
                        .as_mut_ptr()
                        .add(self.elem_size * elem_idx)
                };
                f(val);
                i -= 1;
            }
        }
        self.chunk_list.truncate(last_used_chunk + 1);
        if self.chunk_list.len() < CHUNK_LIST_SIZE {
            self.chunk_list.resize(CHUNK_LIST_SIZE, None);
        }
        self.elem_last = self.elem_next as isize - 1;
        self.elem_next = 0;
        self.elem_next_ofs = 0;
        self.chunk_next = 0;
    }
    /// `BLI_memblock_iternew`.
    pub fn iter(&self) -> MemblockIter {
        MemblockIter {
            cur_index: 0,
            end_index: self.elem_next,
        }
    }
    /// `BLI_memblock_elem_get` — raw pointer to `elem`-th allocated element.
    pub fn elem_get(&mut self, chunk: usize, elem: usize) -> *mut u8 {
        let epc = self.elem_per_chunk();
        let chunk = chunk + elem / epc;
        let elem = elem % epc;
        assert!(chunk < self.chunk_list.len());
        unsafe {
            self.chunk_list[chunk]
                .as_mut()
                .unwrap()
                .as_mut_ptr()
                .add(self.elem_size * elem)
        }
    }
    /// `iterstep` for iterator index → pointer.
    pub fn iter_elem(&mut self, index: usize) -> *mut u8 {
        let epc = self.elem_per_chunk();
        let chunk_idx = index / epc;
        let elem_idx = index - epc * chunk_idx;
        unsafe {
            self.chunk_list[chunk_idx]
                .as_mut()
                .unwrap()
                .as_mut_ptr()
                .add(self.elem_size * elem_idx)
        }
    }
}

/// `BLI_memblock_iter`.
pub struct MemblockIter {
    cur_index: usize,
    end_index: usize,
}

impl MemblockIter {
    /// `BLI_memblock_iterstep` — next index or None.
    pub fn step(&mut self) -> Option<usize> {
        if self.cur_index == self.end_index {
            return None;
        }
        let i = self.cur_index;
        self.cur_index += 1;
        Some(i)
    }
}
