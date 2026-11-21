//! Port of `intern/scene_context.cc` — `bke::scene_or_sequencer_scene_from_context`.
//!
//! Needs the `bContext` runtime (`CTX_wm_space_seq`/`CTX_data_*_scene`) —
//! PORT_PENDING: signature kept with a context handle until `blenders-ctx` exists.

use blenders_dna::id::NIL;

/// `scene_or_sequencer_scene_from_context(const bContext &C)` — PORT_PENDING.
/// `seq_space`, `sequencer_scene`, `scene` stand in for the CTX lookups:
/// returns `sequencer_scene` when a SpaceSeq is active, else `scene`.
pub fn scene_or_sequencer_scene_from_context(
    seq_space: usize,
    sequencer_scene: usize,
    scene: usize,
) -> usize {
    if seq_space != NIL && seq_space != 0 { sequencer_scene } else { scene }
}
