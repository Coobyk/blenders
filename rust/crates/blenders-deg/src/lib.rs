//! Port of `source/blender/depsgraph/intern` —
//! `DEG_*` dependency-graph core.

pub mod depsgraph_type;
pub mod depsgraph_update;
pub mod eval_stats;
pub mod node_factory;
pub mod node_time;
pub mod relation;
pub mod runtime_backup;
pub mod writeback_sync;
pub mod builder_key;
pub mod builder_map;
pub mod builder_pchanmap;
pub mod registry;
pub mod eval;
pub mod builder_stack;
pub mod pipelines;
pub mod builder_remove_noop;
pub mod builder_transitive;
pub mod node_id;
