//! Port of `BLI_session_uid.cc` — process-unique 64-bit IDs.

use std::sync::atomic::{AtomicU64, Ordering};

/// `SessionUID` — wraps the raw uid; `uid_ == 0` means "not generated".
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct SessionUID(pub u64);

impl SessionUID {
    /// `detail::SESSION_UID_NONE`.
    pub const NONE: SessionUID = SessionUID(0);
    /// `SessionUID::is_generated`.
    pub fn is_generated(&self) -> bool {
        self.0 != 0
    }
}

static GLOBAL_SESSION_UID: AtomicU64 = AtomicU64::new(0);

/// `BLI_session_uid_generate` — skips 0 (reserve for NONE).
pub fn session_uid_generate() -> SessionUID {
    let mut uid = GLOBAL_SESSION_UID.fetch_add(1, Ordering::Relaxed) + 1;
    if !SessionUID(uid).is_generated() {
        uid = GLOBAL_SESSION_UID.fetch_add(1, Ordering::Relaxed) + 1;
    }
    SessionUID(uid)
}
