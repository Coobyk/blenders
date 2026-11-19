//! Port of Blender's core data layer — DNA structs, `Main` database,
//! `Object`/`ID` types (source/blender/blenkernel/, makesdna/).
//!
//! (`bmain` = Blender's `BKE_main`/`struct Main`; `main.rs` is reserved
//! for binaries in Rust.)
//!
//! Started with the `ID` base type and `Main` (BMain) container that every
//! datablock lives in; DNA structs are ported incrementally.

pub mod bmain;
pub mod id;
pub mod object;

pub use bmain::Main;
pub use id::{Id, IdKind, IdName};
pub use object::{Object, ObjectType};
