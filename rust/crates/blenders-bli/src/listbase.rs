//! Port of `listbase.cc` — intrusive doubly-linked `ListBase`/`Link`.
//! Nodes embed `Link` as their first member (like C structs with
//! `struct { Link *next, *prev; }` head). All ops are raw-pointer ops,
//! faithful to the C API; `unsafe` to deref.

#![allow(non_snake_case)]

/// `Link` — the embedded node header (next/prev).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Link {
    pub next: *mut Link,
    pub prev: *mut Link,
}

/// `LinkData` — `Link` + payload pointer.
#[repr(C)]
pub struct LinkData {
    pub next: *mut LinkData,
    pub prev: *mut LinkData,
    pub data: *mut core::ffi::c_void,
}

/// `ListBase`.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct ListBase {
    pub first: *mut Link,
    pub last: *mut Link,
}

impl ListBase {
    /// Zeroed `ListBase` (`{.first = NULL, .last = NULL}`).
    pub const fn new() -> Self {
        ListBase {
            first: std::ptr::null_mut(),
            last: std::ptr::null_mut(),
        }
    }
    /// `BLI_listbase_is_empty`.
    pub fn is_empty(&self) -> bool {
        self.first.is_null()
    }
}

/// `BLI_movelisttolist` — append src onto dst, emptying src.
///
/// # Safety
/// Pointers must belong to the respective lists.
pub unsafe fn movelisttolist(dst: *mut ListBase, src: *mut ListBase) {
    if unsafe { (*src).first.is_null() } {
        return;
    }
    unsafe {
        (*(*src).first).prev = (*dst).last;
        if !(*dst).last.is_null() {
            (*(*dst).last).next = (*src).first;
        } else {
            (*dst).first = (*src).first;
        }
        (*dst).last = (*src).last;
        (*src).first = std::ptr::null_mut();
        (*src).last = std::ptr::null_mut();
    }
}

/// `BLI_movelisttolist_reverse` — prepend src onto dst reversed order kept.
pub unsafe fn movelisttolist_reverse(dst: *mut ListBase, src: *mut ListBase) {
    if unsafe { (*src).first.is_null() } {
        return;
    }
    unsafe {
        (*(*src).first).prev = (*dst).last;
        if !(*dst).last.is_null() {
            (*(*dst).last).next = (*src).first;
        } else {
            (*dst).first = (*src).first;
        }
        /* NOTE: reversing linked list technically means we need to update
         * last and first indices... but for dst we can rely on knowing
         * what the last link is. */
        (*dst).last = (*src).first;
        (*src).first = std::ptr::null_mut();
        (*src).last = std::ptr::null_mut();
    }
}

/// `BLI_listbase_split_after` — everything after `vlink` moves to `split`.
pub unsafe fn listbase_split_after(
    original: *mut ListBase,
    split: *mut ListBase,
    vlink: *mut Link,
) {
    unsafe {
        if (*original).first.is_null() {
            *split = *original;
            return;
        }
        let link = if vlink.is_null() { (*original).first } else { vlink };
        (*split).first = (*link).next;
        (*split).last = (*original).last;
        if !(*split).first.is_null() {
            (*(*split).first).prev = std::ptr::null_mut();
        }
        (*original).last = link;
        (*link).next = std::ptr::null_mut();
    }
}

/// `BLI_addhead`.
pub unsafe fn addhead(listbase: *mut ListBase, vlink: *mut Link) {
    if vlink.is_null() {
        return;
    }
    unsafe {
        (*vlink).next = (*listbase).first;
        (*vlink).prev = std::ptr::null_mut();
        if (*listbase).first.is_null() {
            (*listbase).last = vlink;
        } else {
            (*(*listbase).first).prev = vlink;
        }
        (*listbase).first = vlink;
    }
}

/// `BLI_addtail`.
pub unsafe fn addtail(listbase: *mut ListBase, vlink: *mut Link) {
    if vlink.is_null() {
        return;
    }
    unsafe {
        (*vlink).next = std::ptr::null_mut();
        (*vlink).prev = (*listbase).last;
        if (*listbase).last.is_null() {
            (*listbase).first = vlink;
        } else {
            (*(*listbase).last).next = vlink;
        }
        (*listbase).last = vlink;
    }
}

/// `BLI_remlink`.
pub unsafe fn remlink(listbase: *mut ListBase, vlink: *mut Link) {
    if vlink.is_null() {
        return;
    }
    unsafe {
        let link = vlink;
        if !(*link).next.is_null() {
            (*(*link).next).prev = (*link).prev;
        }
        if !(*link).prev.is_null() {
            (*(*link).prev).next = (*link).next;
        }
        if (*listbase).last == link {
            (*listbase).last = (*link).prev;
        }
        if (*listbase).first == link {
            (*listbase).first = (*link).next;
        }
        (*link).next = std::ptr::null_mut();
        (*link).prev = std::ptr::null_mut();
    }
}

/// `BLI_remlink_safe` — only unlink when `vlink` is in `listbase`.
pub unsafe fn remlink_safe(listbase: *mut ListBase, vlink: *mut Link) -> bool {
    if unsafe { findindex(listbase, vlink) } != -1 {
        unsafe { remlink(listbase, vlink) };
        return true;
    }
    false
}

/// `BLI_listbase_swaplinks` — swap two links within one list.
pub unsafe fn listbase_swaplinks(listbase: *mut ListBase, vlinka: *mut Link, vlinkb: *mut Link) {
    unsafe {
        let linka = vlinka;
        let linkb = vlinkb;
        std::ptr::swap(&mut (*linka).next, &mut (*linkb).next);
        std::ptr::swap(&mut (*linka).prev, &mut (*linkb).prev);
        if !(*linka).next.is_null() {
            (*(*linka).next).prev = linka;
        }
        if !(*linka).prev.is_null() {
            (*(*linka).prev).next = linka;
        }
        if !(*linkb).next.is_null() {
            (*(*linkb).next).prev = linkb;
        }
        if !(*linkb).prev.is_null() {
            (*(*linkb).prev).next = linkb;
        }
        if (*listbase).first == linka {
            (*listbase).first = linkb;
        } else if (*listbase).first == linkb {
            (*listbase).first = linka;
        }
        if (*listbase).last == linka {
            (*listbase).last = linkb;
        } else if (*listbase).last == linkb {
            (*listbase).last = linka;
        }
    }
}

/// `BLI_pophead` — remove & return first link.
pub unsafe fn pophead(listbase: *mut ListBase) -> *mut Link {
    unsafe {
        let link = (*listbase).first;
        if link.is_null() {
            return std::ptr::null_mut();
        }
        remlink(listbase, link);
        link
    }
}

/// `BLI_poptail`.
pub unsafe fn poptail(listbase: *mut ListBase) -> *mut Link {
    unsafe {
        let link = (*listbase).last;
        if link.is_null() {
            return std::ptr::null_mut();
        }
        remlink(listbase, link);
        link
    }
}

/// `BLI_freelinkN` — remlink + free (Drop in Rust — caller frees).
pub unsafe fn freelinkN(listbase: *mut ListBase, vlink: *mut Link) {
    unsafe {
        remlink(listbase, vlink);
    }
}

/// `BLI_insertlinkafter`.
pub unsafe fn insertlinkafter(
    listbase: *mut ListBase,
    vprevlink: *mut Link,
    vnewlink: *mut Link,
) {
    unsafe {
        let newlink = vnewlink;
        let prevlink = vprevlink;
        if (*listbase).first.is_null() {
            (*listbase).first = newlink;
            (*listbase).last = newlink;
            return;
        }
        if prevlink.is_null() {
            (*newlink).prev = std::ptr::null_mut();
            (*newlink).next = (*listbase).first;
            (*(*listbase).first).prev = newlink;
            (*listbase).first = newlink;
            return;
        }
        (*newlink).next = (*prevlink).next;
        (*newlink).prev = prevlink;
        (*prevlink).next = newlink;
        if !(*newlink).next.is_null() {
            (*(*newlink).next).prev = newlink;
        }
        if (*listbase).last == prevlink {
            (*listbase).last = newlink;
        }
    }
}

/// `BLI_insertlinkbefore`.
pub unsafe fn insertlinkbefore(
    listbase: *mut ListBase,
    vnextlink: *mut Link,
    vnewlink: *mut Link,
) {
    unsafe {
        let newlink = vnewlink;
        let nextlink = vnextlink;
        if (*listbase).first.is_null() {
            (*listbase).first = newlink;
            (*listbase).last = newlink;
            return;
        }
        if nextlink.is_null() {
            (*newlink).next = std::ptr::null_mut();
            (*newlink).prev = (*listbase).last;
            if !(*listbase).last.is_null() {
                (*(*listbase).last).next = newlink;
            }
            (*listbase).last = newlink;
            return;
        }
        (*newlink).prev = (*nextlink).prev;
        (*newlink).next = nextlink;
        (*nextlink).prev = newlink;
        if !(*newlink).prev.is_null() {
            (*(*newlink).prev).next = newlink;
        }
        if (*listbase).first == nextlink {
            (*listbase).first = newlink;
        }
    }
}

/// `BLI_insertlinkreplace` — replace `vreplacedlink` with `vnewlink`.
pub unsafe fn insertlinkreplace(
    listbase: *mut ListBase,
    vreplacelink: *mut Link,
    vnewlink: *mut Link,
) {
    unsafe {
        let newlink = vnewlink;
        let lreplace = vreplacelink;
        if !(*lreplace).next.is_null() {
            (*(*lreplace).next).prev = newlink;
        }
        if !(*lreplace).prev.is_null() {
            (*(*lreplace).prev).next = newlink;
        }
        if (*listbase).last == lreplace {
            (*listbase).last = newlink;
        }
        if (*listbase).first == lreplace {
            (*listbase).first = newlink;
        }
        *newlink = *lreplace;
        (*lreplace).next = std::ptr::null_mut();
        (*lreplace).prev = std::ptr::null_mut();
    }
}

/// `BLI_listbase_link_move` — move `vlink` by `step` positions.
pub unsafe fn listbase_link_move(
    listbase: *mut ListBase,
    vlink: *mut Link,
    step: i32,
) -> bool {
    unsafe {
        let mut link = vlink;
        let first = (*listbase).first;
        let last = (*listbase).last;
        let mut find: *mut Link = std::ptr::null_mut();
        if step > 0 {
            let mut hook = link;
            for _ in 0..step {
                hook = (*hook).next;
                if hook.is_null() {
                    return false;
                }
            }
            find = hook;
        } else {
            let mut hook = link;
            for _ in 0..(-step) {
                hook = (*hook).prev;
                if hook.is_null() {
                    return false;
                }
            }
            find = hook;
        }
        if step > 0 {
            remlink(listbase, link);
            insertlinkafter(listbase, find, link);
        } else {
            remlink(listbase, link);
            insertlinkbefore(listbase, find, link);
        }
        let _ = link;
        link = find;
        let _ = (first, last, link);
        true
    }
}

/// `BLI_listbase_move_index`.
pub unsafe fn listbase_move_index(listbase: *mut ListBase, from: i32, to: i32) -> bool {
    if from == to {
        return false;
    }
    unsafe {
        let link = findlink(listbase, from);
        if link.is_null() {
            return false;
        }
        let link_from = if from < to {
            findlink(listbase, to + 1)
        } else {
            findlink(listbase, to)
        };
        if from < to {
            remlink(listbase, link);
            insertlinkbefore(listbase, link_from, link);
        } else {
            remlink(listbase, link);
            insertlinkbefore(listbase, link_from, link);
        }
        true
    }
}

/// `BLI_freelist` — walk and drop all nodes (Rust: caller passes a drop fn).
///
/// # Safety
/// `free_fn` is called once per node; nodes must be owned by the list.
pub unsafe fn freelist(listbase: *mut ListBase, mut free_fn: impl FnMut(*mut Link)) {
    unsafe {
        let mut link = (*listbase).first;
        while !link.is_null() {
            let next = (*link).next;
            free_fn(link);
            link = next;
        }
        (*listbase).first = std::ptr::null_mut();
        (*listbase).last = std::ptr::null_mut();
    }
}

/// `BLI_freelistN` — same via remlink.
pub unsafe fn freelistN(listbase: *mut ListBase, mut free_fn: impl FnMut(*mut Link)) {
    unsafe {
        loop {
            let link = pophead(listbase);
            if link.is_null() {
                break;
            }
            free_fn(link);
        }
    }
}

/// `BLI_listbase_count_at_most`.
pub unsafe fn listbase_count_at_most(listbase: *const ListBase, count_max: i32) -> i32 {
    let mut count = 0;
    let mut link = unsafe { (*listbase).first };
    while !link.is_null() && count < count_max {
        link = unsafe { (*link).next };
        count += 1;
    }
    count
}

/// `BLI_listbase_count`.
pub unsafe fn listbase_count(listbase: *const ListBase) -> i32 {
    let mut count = 0;
    let mut link = unsafe { (*listbase).first };
    while !link.is_null() {
        link = unsafe { (*link).next };
        count += 1;
    }
    count
}

/// `BLI_findlink` — link at index `number`.
pub unsafe fn findlink(listbase: *const ListBase, number: i32) -> *mut Link {
    let mut link = unsafe { (*listbase).first };
    let mut i = 0;
    while !link.is_null() {
        if i == number {
            return link;
        }
        i += 1;
        link = unsafe { (*link).next };
    }
    std::ptr::null_mut()
}

/// `BLI_rfindlink` — link at `number`-from-end.
pub unsafe fn rfindlink(listbase: *const ListBase, number: i32) -> *mut Link {
    let mut link = unsafe { (*listbase).last };
    let mut i = 0;
    while !link.is_null() {
        if i == number {
            return link;
        }
        i += 1;
        link = unsafe { (*link).prev };
    }
    std::ptr::null_mut()
}

/// `BLI_findlinkfrom` — nth link starting at `start`.
pub unsafe fn findlinkfrom(start: *mut Link, step: i32) -> *mut Link {
    let mut link = start;
    let mut i = 0;
    while !link.is_null() && i != step {
        link = unsafe { (*link).next };
        i += 1;
    }
    link
}

/// `BLI_findindex` — index of `vlink`, -1 when not found.
pub unsafe fn findindex(listbase: *const ListBase, vlink: *const Link) -> i32 {
    let mut link = unsafe { (*listbase).first };
    let mut i = 0;
    while !link.is_null() {
        if link == vlink as *mut Link {
            return i;
        }
        i += 1;
        link = unsafe { (*link).next };
    }
    -1
}

/// `BLI_listbase_reverse`.
pub unsafe fn listbase_reverse(lb: *mut ListBase) {
    unsafe {
        if (*lb).first.is_null() {
            return;
        }
        let mut link = (*lb).first;
        while !link.is_null() {
            std::ptr::swap(&mut (*link).next, &mut (*link).prev);
            link = (*link).prev;
        }
        std::ptr::swap(&mut (*lb).first, &mut (*lb).last);
    }
}

/// `BLI_listbase_rotate_first` — make `vlink` the new head.
pub unsafe fn listbase_rotate_first(lb: *mut ListBase, vlink: *mut Link) {
    unsafe {
        if vlink == (*lb).first {
            return;
        }
        remlink(lb, vlink);
        addhead(lb, vlink);
    }
}

/// `BLI_listbase_rotate_last`.
pub unsafe fn listbase_rotate_last(lb: *mut ListBase, vlink: *mut Link) {
    unsafe {
        if vlink == (*lb).last {
            return;
        }
        remlink(lb, vlink);
        addtail(lb, vlink);
    }
}

/// `BLI_listbase_validate` — check next/prev chains agree.
pub unsafe fn listbase_validate(lb: *mut ListBase) -> bool {
    unsafe {
        let mut link = (*lb).first;
        let mut prev: *mut Link = std::ptr::null_mut();
        while !link.is_null() {
            if (*link).prev != prev {
                return false;
            }
            prev = link;
            link = (*link).next;
        }
        prev == (*lb).last
    }
}

/// `BLI_genericNodeN` — wrap data in a `LinkData` node (boxed).
pub fn generic_node_n(data: *mut core::ffi::c_void) -> *mut LinkData {
    let node = Box::new(LinkData {
        next: std::ptr::null_mut(),
        prev: std::ptr::null_mut(),
        data,
    });
    Box::into_raw(node)
}
