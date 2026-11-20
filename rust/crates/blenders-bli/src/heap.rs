//! Port of `BLI_heap.cc` — binary min-heap with stable `HeapNode` handles
//! (insert/remove/value-update by handle, like the chunked C version).

/// `HeapNode` — stable handle into the heap.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct HeapNode(usize);
impl HeapNode {
    /// `BLI_heap_node_value` / `BLI_heap_node_ptr` read via `Heap` methods.
    pub fn id(self) -> usize {
        self.0
    }
}

struct Node<T> {
    value: f32,
    ptr: T,
    /// index within `tree`.
    pos: usize,
}

/// `Heap` — min-heap of nodes ordered by `value`.
pub struct Heap<T> {
    tree: Vec<usize>, // node ids, heap-ordered
    nodes: Vec<Option<Node<T>>>,
    free: Vec<usize>,
}

impl<T> Heap<T> {
    /// `BLI_heap_new` / `BLI_heap_new_ex`.
    pub fn new() -> Self {
        Heap {
            tree: Vec::new(),
            nodes: Vec::new(),
            free: Vec::new(),
        }
    }
    pub fn with_capacity(n: usize) -> Self {
        Heap {
            tree: Vec::with_capacity(n),
            nodes: Vec::with_capacity(n),
            free: Vec::new(),
        }
    }

    /// `BLI_heap_len` / `BLI_heap_is_empty`.
    pub fn len(&self) -> usize {
        self.tree.len()
    }
    pub fn is_empty(&self) -> bool {
        self.tree.is_empty()
    }

    /// `BLI_heap_clear`.
    pub fn clear(&mut self) {
        self.tree.clear();
        self.nodes.clear();
        self.free.clear();
    }

    fn val(&self, tree_i: usize) -> f32 {
        self.nodes[self.tree[tree_i]].as_ref().unwrap().value
    }

    /// `heap_swap`.
    fn swap(&mut self, i: usize, j: usize) {
        self.tree.swap(i, j);
        let (a, b) = (self.tree[i], self.tree[j]);
        self.nodes[a].as_mut().unwrap().pos = i;
        self.nodes[b].as_mut().unwrap().pos = j;
    }

    /// `heap_down`.
    fn down(&mut self, mut i: usize) {
        loop {
            let (l, r) = (i * 2 + 1, i * 2 + 2);
            let mut s = i;
            if l < self.tree.len() && self.val(l) < self.val(s) {
                s = l;
            }
            if r < self.tree.len() && self.val(r) < self.val(s) {
                s = r;
            }
            if s == i {
                return;
            }
            self.swap(i, s);
            i = s;
        }
    }
    /// `heap_up`.
    fn up(&mut self, mut i: usize) {
        while i > 0 {
            let p = (i - 1) >> 1;
            if self.val(p) <= self.val(i) {
                return;
            }
            self.swap(i, p);
            i = p;
        }
    }

    /// `BLI_heap_insert`.
    pub fn insert(&mut self, value: f32, ptr: T) -> HeapNode {
        let pos = self.tree.len();
        let id = if let Some(id) = self.free.pop() {
            self.nodes[id] = Some(Node { value, ptr, pos });
            id
        } else {
            self.nodes.push(Some(Node { value, ptr, pos }));
            self.nodes.len() - 1
        };
        self.tree.push(id);
        self.up(pos);
        HeapNode(id)
    }

    /// `BLI_heap_insert_or_update` — insert when `*node_p` is None else update.
    pub fn insert_or_update(&mut self, node_p: &mut Option<HeapNode>, value: f32, ptr: T) {
        match *node_p {
            Some(n) => self.node_value_update_ptr(n, value, ptr),
            None => *node_p = Some(self.insert(value, ptr)),
        }
    }

    /// `BLI_heap_top` / `BLI_heap_top_value`.
    pub fn top(&self) -> Option<HeapNode> {
        self.tree.first().map(|&id| HeapNode(id))
    }
    pub fn top_value(&self) -> Option<f32> {
        self.tree.first().map(|&id| self.nodes[id].as_ref().unwrap().value)
    }

    /// `BLI_heap_pop_min` — remove & return min `ptr`.
    pub fn pop_min(&mut self) -> Option<T> {
        if self.tree.is_empty() {
            return None;
        }
        let last = self.tree.pop().unwrap();
        if !self.tree.is_empty() {
            let min_id = self.tree[0];
            self.tree[0] = last;
            self.nodes[last].as_mut().unwrap().pos = 0;
            self.down(0);
            let node = self.nodes[min_id].take().unwrap();
            self.free.push(min_id);
            Some(node.ptr)
        } else {
            let node = self.nodes[last].take().unwrap();
            self.free.push(last);
            Some(node.ptr)
        }
    }

    /// `BLI_heap_remove` — remove by handle.
    pub fn remove(&mut self, node: HeapNode) {
        let pos = self.nodes[node.0].as_ref().unwrap().pos;
        let last = self.tree.pop().unwrap();
        if pos < self.tree.len() {
            self.tree[pos] = last;
            self.nodes[last].as_mut().unwrap().pos = pos;
            self.down(pos);
            self.up(pos);
        }
        self.nodes[node.0] = None;
        self.free.push(node.0);
    }

    /// `BLI_heap_node_value_update`.
    pub fn node_value_update(&mut self, node: HeapNode, value: f32) {
        let pos = {
            let n = self.nodes[node.0].as_mut().unwrap();
            n.value = value;
            n.pos
        };
        self.up(pos);
        self.down(pos);
    }
    /// `BLI_heap_node_value_update_ptr`.
    pub fn node_value_update_ptr(&mut self, node: HeapNode, value: f32, ptr: T) {
        self.nodes[node.0].as_mut().unwrap().ptr = ptr;
        self.node_value_update(node, value);
    }

    /// `BLI_heap_node_value` / `BLI_heap_node_ptr`.
    pub fn node_value(&self, node: HeapNode) -> f32 {
        self.nodes[node.0].as_ref().unwrap().value
    }
    pub fn node_ptr(&self, node: HeapNode) -> &T {
        &self.nodes[node.0].as_ref().unwrap().ptr
    }

    /// `BLI_heap_is_valid`.
    pub fn is_valid(&self) -> bool {
        for i in (1..self.tree.len()).rev() {
            if self.val(i) < self.val((i - 1) >> 1) {
                return false;
            }
        }
        true
    }
}
