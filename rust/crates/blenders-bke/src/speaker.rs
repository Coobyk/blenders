//! Port of `intern/speaker.cc`.
//!
//! `speaker_init_data`/`speaker_foreach_id`/`speaker_blend_write` and
//! `BKE_speaker_add` need Main/LibraryForeachIDData/BlendWriter — PORT_PENDING.
//! The `IDType_ID_SPK` table itself is ported as data.

use crate::idtype::{IDTypeInfo, IDTYPE_FLAGS_APPEND_IS_REUSABLE};
use blenders_dna::id::{FILTER_ID_SO, FILTER_ID_SPK, IndexID};
use blenders_dna::id_enums::ID_SPK;

/// `BKE_id_new<Speaker>` default: `INIT_DEFAULT_STRUCT_AFTER(speaker, id)` —
/// Speaker has no extra defaults beyond the zeroed tail.
pub fn speaker_init_data() -> blenders_dna::speaker_types::Speaker {
    blenders_dna::speaker_types::Speaker::default()
}

/// `IDTypeInfo IDType_ID_SPK` — data table ported verbatim; callback slots
/// (`foreach_id`, `blend_write`) are PORT_PENDING markers (see idtype.rs).
pub static IDTYPE_ID_SPK: IDTypeInfo = {
    let mut t = IDTypeInfo::new(
        ID_SPK as i16,
        FILTER_ID_SPK,
        FILTER_ID_SO,
        IndexID::INDEX_ID_SPK as i32,
        core::mem::size_of::<blenders_dna::speaker_types::Speaker>() as u32,
        "Speaker",
        "speakers",
        "Speaker",
        IDTYPE_FLAGS_APPEND_IS_REUSABLE,
    );
    /* speaker_foreach_id / speaker_blend_write — PORT_PENDING. */
    let _ = &mut t;
    t
};

// `BKE_speaker_add(Main *bmain, const char *name)` — PORT_PENDING (needs Main).
