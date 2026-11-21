//! Port of `DNA_session_uid_types.h` — `SessionUID`.

/// `SessionUID`.
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct SessionUid {
    pub uid: u64,
}
impl SessionUid {
    /// `SESSION_UID_NONE`.
    pub const NONE: SessionUid = SessionUid { uid: 0 };
    /// `is_generated()`.
    pub fn is_generated(&self) -> bool {
        *self != Self::NONE
    }
    /// `hash()`.
    pub fn hash(&self) -> u64 {
        self.uid
    }
}
