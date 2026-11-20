//! Port of `session_uid.cc` — `BLI_session_uid_generate` monotonic UIDs.

use std::sync::atomic::{AtomicU64, Ordering};

/// `SESSION_UID_NONE` / the generated ids.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SessionUid(pub u64);

/// `detail::SESSION_UID_NONE`.
pub const SESSION_UID_NONE: SessionUid = SessionUid(0);

impl SessionUid {
    /// `uid.is_generated()` — true unless NONE.
    pub fn is_generated(&self) -> bool {
        self.0 != 0
    }
}

static GLOBAL_SESSION_UID: AtomicU64 = AtomicU64::new(0);

/// `BLI_session_uid_generate` — returns successive non-zero uids.
pub fn session_uid_generate() -> SessionUid {
    let mut uid = SessionUid(GLOBAL_SESSION_UID.fetch_add(1, Ordering::Relaxed) + 1);
    if !uid.is_generated() {
        uid = SessionUid(GLOBAL_SESSION_UID.fetch_add(1, Ordering::Relaxed) + 1);
    }
    uid
}
