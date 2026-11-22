//! `blenders-animrig` — Rust port of
//! `source/blender/animrig/intern/*.cc`.
pub mod bonecolor;
pub mod action_selection;
pub mod driver;
pub mod armature;
pub mod action_runtime;
pub mod action_legacy;
pub mod visualkey;
pub mod nla;
pub mod action_iterators;
pub mod animdata;
pub mod keyframing_auto;
pub mod evaluation;
pub mod versioning;
pub mod pose;
pub mod keyingsets;
pub mod fcurve;
pub mod anim_rna;
pub mod keyframing;
pub mod bone_collections;
pub mod action;
// PORT_PENDING: *_test.cc files (action_test,
// action_legacy_test, action_iterators_test, anim_rna_test,
// bone_collections_test, evaluation_test, keyframing_test,
// nla_test, pose_test, versioning_test) — port as #[cfg(test)]
// modules once the test harness unifies.
