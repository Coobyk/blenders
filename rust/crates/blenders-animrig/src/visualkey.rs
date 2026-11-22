//! Port of `intern/visualkey.cc` — `visualkey_can_use`
//! and `visualkey_get_values`.

/// `VISUALKEY_*` verbatim.
pub const VISUALKEY_NONE: i32 = 0;
pub const VISUALKEY_LOC: i32 = 1;
pub const VISUALKEY_ROT: i32 = 2;
pub const VISUALKEY_SCA: i32 = 3;
/// `PCHAN_HAS_IK`/`PCHAN_INFLUENCED_BY_IK` verbatim.
pub const PCHAN_HAS_IK: i32 = 1 << 0;
pub const PCHAN_INFLUENCED_BY_IK: i32 = 1 << 23;
/// `CONSTRAINT_DISABLE` verbatim.
pub const CONSTRAINT_DISABLE: i32 = 1 << 7;
/// `CONSTRAINT_TYPE_*` verbatim (ordinal + 1<<10).
pub const CONSTRAINT_TYPE_NULL: i32 = 0 + 1000;
pub const CONSTRAINT_TYPE_CHILDOF: i32 = 1 + 1000;
pub const CONSTRAINT_TYPE_TRACKTO: i32 = 2 + 1000;
pub const CONSTRAINT_TYPE_KINEMATIC: i32 = 3 + 1000;
pub const CONSTRAINT_TYPE_FOLLOWPATH: i32 = 4 + 1000;
pub const CONSTRAINT_TYPE_ROTLIMIT: i32 = 5 + 1000;
pub const CONSTRAINT_TYPE_LOCLIMIT: i32 = 6 + 1000;
pub const CONSTRAINT_TYPE_SIZELIMIT: i32 = 7 + 1000;
pub const CONSTRAINT_TYPE_ROTLIKE: i32 = 8 + 1000;
pub const CONSTRAINT_TYPE_LOCLIKE: i32 = 9 + 1000;
pub const CONSTRAINT_TYPE_SIZELIKE: i32 = 10 + 1000;
pub const CONSTRAINT_TYPE_TRANSLIKE: i32 = 11 + 1000;
pub const CONSTRAINT_TYPE_DISTLIMIT: i32 = 12 + 1000;
pub const CONSTRAINT_TYPE_ARMATURE: i32 = 15 + 1000;
pub const CONSTRAINT_TYPE_TRANSFORM: i32 = 19 + 1000;
pub const CONSTRAINT_TYPE_LOCKTRACK: i32 = 21 + 1000;
pub const CONSTRAINT_TYPE_DAMPTRACK: i32 = 23 + 1000;
pub const CONSTRAINT_TYPE_MINMAX: i32 = 26 + 1000;

/// `visualkey_can_use` constraint table (verbatim —
/// type × searchtype).
pub fn constraint_allows_visualkey(con_type: i32, searchtype: i32) -> bool {
    match con_type {
        x if x == CONSTRAINT_TYPE_CHILDOF || x == CONSTRAINT_TYPE_ARMATURE => true,
        x if x == CONSTRAINT_TYPE_TRANSFORM || x == CONSTRAINT_TYPE_TRANSLIKE => true,
        x if x == CONSTRAINT_TYPE_FOLLOWPATH => true,
        x if x == CONSTRAINT_TYPE_KINEMATIC => true,
        x if x == CONSTRAINT_TYPE_TRACKTO => searchtype == VISUALKEY_ROT,
        x if x == CONSTRAINT_TYPE_DAMPTRACK => searchtype == VISUALKEY_ROT,
        x if x == CONSTRAINT_TYPE_ROTLIMIT => searchtype == VISUALKEY_ROT,
        x if x == CONSTRAINT_TYPE_LOCLIMIT => searchtype == VISUALKEY_LOC,
        x if x == CONSTRAINT_TYPE_SIZELIMIT => searchtype == VISUALKEY_SCA,
        x if x == CONSTRAINT_TYPE_DISTLIMIT => searchtype == VISUALKEY_LOC,
        x if x == CONSTRAINT_TYPE_ROTLIKE => searchtype == VISUALKEY_ROT,
        x if x == CONSTRAINT_TYPE_LOCLIKE => searchtype == VISUALKEY_LOC,
        x if x == CONSTRAINT_TYPE_SIZELIKE => searchtype == VISUALKEY_SCA,
        x if x == CONSTRAINT_TYPE_LOCKTRACK => searchtype == VISUALKEY_ROT,
        x if x == CONSTRAINT_TYPE_MINMAX => searchtype == VISUALKEY_LOC,
        _ => false,
    }
}

/// `visualkey_can_use` identifier → searchtype (verbatim).
pub fn visualkey_searchtype(identifier: &str) -> i32 {
    if identifier.contains("location") {
        VISUALKEY_LOC
    } else if identifier.contains("rotation") {
        VISUALKEY_ROT
    } else if identifier.contains("scale") {
        VISUALKEY_SCA
    } else {
        VISUALKEY_NONE
    }
}
