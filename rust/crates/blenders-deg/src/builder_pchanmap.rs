//! Port of `intern/builder/deg_builder_pchanmap.cc` —
/// `RootPChanMap`.

use std::collections::{HashMap, HashSet};

/// `RootPChanMap` — bone name → set of root names.
#[derive(Default)]
pub struct RootPChanMap {
    /// `map_`.
    pub map: HashMap<String, HashSet<String>>,
}

impl RootPChanMap {
    /// `add_bone` (verbatim).
    pub fn add_bone(&mut self, bone: &str, root: &str) {
        self.map
            .entry(bone.to_string())
            .or_default()
            .insert(root.to_string());
    }

    /// `has_common_root` (verbatim).
    pub fn has_common_root(&self, bone1: &str, bone2: &str) -> bool {
        let Some(bone1_roots) = self.map.get(bone1) else {
            return false;
        };
        let Some(bone2_roots) = self.map.get(bone2) else {
            return false;
        };
        !bone1_roots.is_disjoint(bone2_roots)
    }

    /// `print_debug` — verbatim format.
    pub fn print_debug(&self) {
        for (key, values) in &self.map {
            print!("  {} : {{ ", key);
            for val in values {
                print!("{}, ", val);
            }
            println!("}}");
        }
    }
}
