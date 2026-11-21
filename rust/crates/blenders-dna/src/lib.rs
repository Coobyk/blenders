//! Ports of `source/blender/makesdna/DNA_*.h` — the serialized data-block
//! structs every blenkernel file builds on. Pointers become `usize` arena
//! indices / `Option<Box<>>` per the porting conventions.
#![allow(dead_code)]

pub mod id;
