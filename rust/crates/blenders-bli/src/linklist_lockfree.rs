//! Port of `BLI_linklist_lockfree.cc` — lock-free single-linked list
//! (`LockfreeLinkList`/`LockfreeLinkNode`).

use std::sync::atomic::{AtomicPtr, Ordering};

/// `LockfreeLinkNode`.
pub struct LockfreeLinkNode {
    pub next: AtomicPtr<LockfreeLinkNode>,
}

/// `LockfreeLinkList` — head/tail over raw node pointers.
pub struct LockfreeLinkList {
    dummy_node: Box<LockfreeLinkNode>,
    head: *mut LockfreeLinkNode,
    tail: AtomicPtr<LockfreeLinkNode>,
}

impl LockfreeLinkList {
    /// `BLI_linklist_lockfree_init`.
    pub fn new() -> Self {
        let mut dummy_node = Box::new(LockfreeLinkNode {
            next: AtomicPtr::new(std::ptr::null_mut()),
        });
        let p: *mut LockfreeLinkNode = dummy_node.as_mut();
        LockfreeLinkList {
            dummy_node,
            head: p,
            tail: AtomicPtr::new(p),
        }
    }
    /// `BLI_linklist_lockfree_insert` — CAS tail-append.
    pub fn insert(&self, node: *mut LockfreeLinkNode) {
        unsafe {
            (*node).next.store(std::ptr::null_mut(), Ordering::Release);
            let mut tail_node;
            loop {
                tail_node = self.tail.load(Ordering::Acquire);
                if (*tail_node)
                    .next
                    .compare_exchange(
                        std::ptr::null_mut(),
                        node,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    )
                    .is_err()
                {
                    // help move the tail forward
                    let next = (*tail_node).next.load(Ordering::Acquire);
                    let _ = self.tail.compare_exchange(
                        tail_node,
                        next,
                        Ordering::AcqRel,
                        Ordering::Acquire,
                    );
                    continue;
                }
                break;
            }
            let _ = self.tail.compare_exchange(
                tail_node,
                node,
                Ordering::AcqRel,
                Ordering::Acquire,
            );
        }
    }
    /// `BLI_linklist_lockfree_begin` — first real node (after dummy).
    pub fn begin(&self) -> *mut LockfreeLinkNode {
        unsafe { (*self.head).next.load(Ordering::Acquire) }
    }
    /// `BLI_linklist_lockfree_free` — run `free_func` on every node, reset.
    pub fn free(&self, mut free_func: impl FnMut(*mut LockfreeLinkNode)) {
        let mut node = self.begin();
        while !node.is_null() {
            let next = unsafe { (*node).next.load(Ordering::Acquire) };
            free_func(node);
            node = next;
        }
    }
    /// `BLI_linklist_lockfree_clear`.
    pub fn clear(&self, free_func: impl FnMut(*mut LockfreeLinkNode)) {
        self.free(free_func);
    }
}
