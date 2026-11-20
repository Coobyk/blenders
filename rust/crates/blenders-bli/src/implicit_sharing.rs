//! Port of `BLI_implicit_sharing.hh` + `implicit_sharing.cc` —
//! `blender::implicit_sharing`: reference-counted shared data with
//! copy-on-write semantics (`ImplicitSharingInfo`,
//! `make_trivial_data_mutable_impl`, `resize_trivial_array_impl`,
//! `info_for_mem_free`).
//!
//! Raw allocations are byte buffers (`Box<[u8]>`) in place of `void *`.

use std::sync::atomic::{AtomicI64, Ordering};

/// `ImplicitSharingInfo` — base for the sharing handle of implicitly
/// shared data. Strong users call `add_user`/`remove_user_and_delete_if_last`;
/// weak users may outlive the data (`add_weak_user`/`delete_data_only`).
pub trait ImplicitSharingInfo: Send + Sync {
    /// `is_mutable` — true when this is the last strong user *and* no
    /// weak user has borrowed read access.
    fn is_mutable(&self) -> bool {
        self.strong_users() == 1 && !self.has_weak_borrowers()
    }
    /// `tag_ensured_mutable` — mark that data was written through this handle.
    fn tag_ensured_mutable(&self);
    /// `delete_self_with_data` — last strong user dropping: free data + self.
    fn delete_self_with_data(self: Box<Self>);
    /// `delete_data_only` — data freed but the info object stays for weak users.
    fn delete_data_only(&self);
    /// `has_weak_borrowers` — `weak_users > strong_users`.
    fn has_weak_borrowers(&self) -> bool {
        self.weak_users() > self.strong_users()
    }
    /// `has_strong_borrowers` — `strong_users > 1`.
    fn has_strong_borrowers(&self) -> bool {
        self.strong_users() > 1
    }
    fn strong_users(&self) -> i64;
    fn weak_users(&self) -> i64;
    fn add_user_impl(&self);
    fn remove_user_impl(&self);
    fn add_weak_user_impl(&self);
    fn remove_weak_user_impl(&self);

    /// `info->add_user()`.
    fn add_user(&self) {
        self.add_user_impl();
    }
    /// `info->add_weak_user()`.
    fn add_weak_user(&self) {
        self.add_weak_user_impl();
    }
}

/// `ImplicitSharingInfoAndData` — shareable byte buffer with the
/// canonical user counters.
pub struct ImplicitSharingInfoAndData {
    /// The shared data (None once `delete_data_only` ran).
    pub data: std::sync::RwLock<Option<Vec<u8>>>,
    /// Whether the buffer was allocated by us (mem-free style).
    mem_free: bool,
    strong_users: AtomicI64,
    weak_users: AtomicI64,
    ensured_mutable: AtomicI64,
}

impl ImplicitSharingInfoAndData {
    /// `info_for_mem_free` — wrap an owned buffer.
    pub fn new_mem_free(data: Vec<u8>) -> Box<Self> {
        Box::new(ImplicitSharingInfoAndData {
            data: std::sync::RwLock::new(Some(data)),
            mem_free: true,
            strong_users: AtomicI64::new(1),
            weak_users: AtomicI64::new(0),
            ensured_mutable: AtomicI64::new(0),
        })
    }
    /// Wrap a borrowed (not-owned) buffer.
    pub fn new_borrowed(data: Vec<u8>) -> Box<Self> {
        Box::new(ImplicitSharingInfoAndData {
            data: std::sync::RwLock::new(Some(data)),
            mem_free: false,
            strong_users: AtomicI64::new(1),
            weak_users: AtomicI64::new(0),
            ensured_mutable: AtomicI64::new(0),
        })
    }
}

impl ImplicitSharingInfo for ImplicitSharingInfoAndData {
    fn tag_ensured_mutable(&self) {
        self.ensured_mutable.store(1, Ordering::Relaxed);
    }
    fn delete_self_with_data(self: Box<Self>) {
        drop(self);
    }
    fn delete_data_only(&self) {
        *self.data.write().unwrap() = None;
    }
    fn strong_users(&self) -> i64 {
        self.strong_users.load(Ordering::Acquire)
    }
    fn weak_users(&self) -> i64 {
        self.weak_users.load(Ordering::Acquire)
    }
    fn add_user_impl(&self) {
        self.strong_users.fetch_add(1, Ordering::AcqRel);
    }
    fn remove_user_impl(&self) {
        self.strong_users.fetch_sub(1, Ordering::AcqRel);
    }
    fn add_weak_user_impl(&self) {
        self.weak_users.fetch_add(1, Ordering::AcqRel);
    }
    fn remove_weak_user_impl(&self) {
        self.weak_users.fetch_sub(1, Ordering::AcqRel);
    }
    fn is_mutable(&self) -> bool {
        self.strong_users() == 1 && !self.has_weak_borrowers() && self.mem_free
    }
}

/// `make_trivial_data_mutable_impl` — copy-on-write: if `info` reports
/// mutable, keep `old_data`; otherwise return a fresh copy.
pub fn make_trivial_data_mutable_impl(
    old_data: Option<Vec<u8>>,
    sharing_info: &dyn ImplicitSharingInfo,
) -> Vec<u8> {
    let Some(old) = old_data else {
        return Vec::new();
    };
    if sharing_info.is_mutable() {
        sharing_info.tag_ensured_mutable();
        old
    } else {
        sharing_info.remove_user_impl();
        old
    }
}

/// `resize_trivial_array_impl` — copy-on-write resize of a shared buffer.
pub fn resize_trivial_array_impl(
    old_data: Option<Vec<u8>>,
    new_size: usize,
    sharing_info: Option<&ImplicitSharingInfoAndData>,
) -> Option<Vec<u8>> {
    if new_size == 0 {
        if let Some(info) = sharing_info {
            info.remove_user_impl();
        }
        return None;
    }
    match (old_data, sharing_info) {
        (None, _) => Some(vec![0u8; new_size]),
        (Some(mut buf), Some(info)) if info.is_mutable() => {
            buf.resize(new_size, 0);
            info.tag_ensured_mutable();
            Some(buf)
        }
        (Some(buf), Some(info)) => {
            let mut v = vec![0u8; new_size];
            let n = buf.len().min(new_size);
            v[..n].copy_from_slice(&buf[..n]);
            info.remove_user_impl();
            Some(v)
        }
        (Some(mut buf), None) => {
            buf.resize(new_size, 0);
            Some(buf)
        }
    }
}
