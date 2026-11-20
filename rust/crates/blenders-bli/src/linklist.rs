//! Port of `BLI_linklist.cc` — singly linked list (`LinkNode`).
//! Arena/pool allocator variants collapse into the boxed-node storage.

/// `LinkNode`.
pub struct LinkNode<T> {
    pub next: Option<Box<LinkNode<T>>>,
    pub data: T,
}

/// `LinkNodePair` — head/tail handle used by append APIs.
#[derive(Default)]
pub struct LinkNodePair<T> {
    pub list: Option<Box<LinkNode<T>>>,
    pub last: *mut LinkNode<T>,
}
impl<T> Clone for LinkNodePair<T> {
    fn clone(&self) -> Self {
        LinkNodePair { list: None, last: std::ptr::null_mut() }
    }
}

/// `BLI_linklist_find`.
pub fn linklist_find<'a, T: PartialEq>(mut list: Option<&'a LinkNode<T>>, ptr: &T) -> Option<&'a LinkNode<T>> {
    while let Some(n) = list {
        if &n.data == ptr {
            return Some(n);
        }
        list = n.next.as_deref();
    }
    None
}

/// `BLI_linklist_find_last`.
pub fn linklist_find_last<'a, T>(mut list: Option<&'a LinkNode<T>>, cmp: impl Fn(&T) -> bool) -> Option<&'a LinkNode<T>> {
    let mut found = None;
    while let Some(n) = list {
        if cmp(&n.data) {
            found = Some(n);
        }
        list = n.next.as_deref();
    }
    found
}

/// `BLI_linklist_apply` — apply `fp` to each node (may delete own node).
pub fn linklist_apply<T>(mut list: Option<Box<LinkNode<T>>>, mut fp: impl FnMut(&mut LinkNode<T>)) -> Option<Box<LinkNode<T>>> {
    let mut cur = list.as_deref_mut();
    while let Some(n) = cur {
        fp(n);
        cur = n.next.as_deref_mut();
    }
    list
}

/// `BLI_linklist_count`.
pub fn linklist_count<T>(mut list: Option<&LinkNode<T>>) -> usize {
    let mut n = 0;
    while let Some(node) = list {
        n += 1;
        list = node.next.as_deref();
    }
    n
}

/// `BLI_linklist_index` — index of `ptr`, -1 when absent.
pub fn linklist_index<T: PartialEq>(mut list: Option<&LinkNode<T>>, ptr: &T) -> i32 {
    let mut i = 0;
    while let Some(n) = list {
        if &n.data == ptr {
            return i;
        }
        i += 1;
        list = n.next.as_deref();
    }
    -1
}

/// `BLI_linklist_reverse`.
pub fn linklist_reverse<T>(mut list: Option<Box<LinkNode<T>>>) -> Option<Box<LinkNode<T>>> {
    let mut prev = None;
    while let Some(mut n) = list {
        list = n.next.take();
        n.next = prev;
        prev = Some(n);
    }
    prev
}

/// `BLI_linklist_prepend` / `_append` (plus arena/pool/nlink variants).
pub fn linklist_prepend<T>(pair: &mut LinkNodePair<T>, data: T) {
    let node = Box::new(LinkNode { next: pair.list.take(), data });
    let raw = Box::into_raw(node);
    pair.list = unsafe { Some(Box::from_raw(raw)) };
    if pair.last.is_null() {
        pair.last = raw;
    }
}
pub fn linklist_append<T>(pair: &mut LinkNodePair<T>, data: T) {
    let node = Box::new(LinkNode { next: None, data });
    let raw = Box::into_raw(node);
    if pair.last.is_null() {
        pair.list = unsafe { Some(Box::from_raw(raw)) };
    } else {
        unsafe {
            (*pair.last).next = Some(Box::from_raw(raw));
        }
    }
    pair.last = raw;
}
/// `BLI_linklist_prepend_nlink` / `_append_nlink` — caller-allocated node.
pub fn linklist_prepend_nlink<T>(pair: &mut LinkNodePair<T>, mut node: Box<LinkNode<T>>) {
    node.next = pair.list.take();
    let raw = Box::into_raw(node);
    pair.list = unsafe { Some(Box::from_raw(raw)) };
    if pair.last.is_null() {
        pair.last = raw;
    }
}
pub fn linklist_append_nlink<T>(pair: &mut LinkNodePair<T>, node: Box<LinkNode<T>>) {
    let raw = Box::into_raw(node);
    if pair.last.is_null() {
        pair.list = unsafe { Some(Box::from_raw(raw)) };
    } else {
        unsafe {
            (*pair.last).next = Some(Box::from_raw(raw));
        }
    }
    pair.last = raw;
}

/// `BLI_linklist_insert_after`.
pub fn linklist_insert_after<T>(nlink: &mut LinkNode<T>, node: Box<LinkNode<T>>) {
    let mut node = node;
    node.next = nlink.next.take();
    nlink.next = Some(node);
}

/// `BLI_linklist_pop`.
pub fn linklist_pop<T>(pair: &mut LinkNodePair<T>) -> Option<T> {
    let node = pair.list.take()?;
    pair.list = node.next;
    if pair.list.is_none() {
        pair.last = std::ptr::null_mut();
    }
    Some(node.data)
}

/// `BLI_linklist_move_item` — move `rank`th node to `pos`.
pub fn linklist_move_item<T>(pair: &mut LinkNodePair<T>, rank: usize, pos: usize) -> bool {
    if rank == pos {
        return false;
    }
    let mut v: Vec<T> = Vec::new();
    let mut cur = pair.list.take();
    while let Some(mut n) = cur {
        cur = n.next.take();
        v.push(n.data);
    }
    if rank >= v.len() || pos > v.len() {
        // rebuild and bail
        for d in v.into_iter().rev() {
            linklist_prepend(pair, d);
        }
        return false;
    }
    let item = v.remove(rank);
    v.insert(pos, item);
    for d in v.into_iter().rev() {
        linklist_prepend(pair, d);
    }
    true
}

/// `BLI_linklist_sort` / `_r` — merge sort with `cmp(a,b)->bool` meaning a<b.
pub fn linklist_sort<T>(list: Option<Box<LinkNode<T>>>, cmp: impl Fn(&T, &T) -> std::cmp::Ordering + Copy) -> Option<Box<LinkNode<T>>> {
    let mut v: Vec<T> = Vec::new();
    let mut cur = list;
    while let Some(mut n) = cur {
        cur = n.next.take();
        v.push(n.data);
    }
    v.sort_by(|a, b| cmp(a, b));
    let mut out = None;
    for d in v.into_iter().rev() {
        out = Some(Box::new(LinkNode { next: out, data: d }));
    }
    out
}
