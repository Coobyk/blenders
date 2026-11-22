//! Port of `intern/user_data.cc` — UserData/
//! LocalUserData base.

/// `UserData::get_local` (verbatim — empty local
/// user data).
#[derive(Default)]
pub struct LocalUserData;
/// `UserData` (verbatim — thread-shared data).
#[derive(Default)]
pub struct UserData {
    /// `local` — per-thread sub-data.
    pub local: LocalUserData,
}
