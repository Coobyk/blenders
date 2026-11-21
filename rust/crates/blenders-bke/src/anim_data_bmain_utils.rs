//! Port of `intern/anim_data_bmain_utils.cc` —
//! `BKE_animdata_main_cb` (iterate all ID lists, callback on each
//! `AnimData`-bearing ID + its nodetree where present).
//!
//! IDs are `usize` handles; `animdata_from_id` is `BKE_animdata_from_id`
//! and `nodetree_of` resolves `->nodetree` for `Tex`/`Light`/`Material`/
//! `World`/`Scene` (all PORT_PENDING the ID/AnimData ports).

/// All `bmain` ListBases the C iterates, in iteration order. The
/// five `ANIMDATA_NODETREE_IDS_CB` lists also visit each ID's nodetree.
#[derive(Default)]
pub struct MainIdLists {
    pub nodetrees: Vec<usize>,
    pub textures: Vec<usize>,
    pub lights: Vec<usize>,
    pub materials: Vec<usize>,
    pub cameras: Vec<usize>,
    pub shapekeys: Vec<usize>,
    pub metaballs: Vec<usize>,
    pub curves: Vec<usize>,
    pub armatures: Vec<usize>,
    pub lattices: Vec<usize>,
    pub meshes: Vec<usize>,
    pub particles: Vec<usize>,
    pub speakers: Vec<usize>,
    pub movieclips: Vec<usize>,
    pub objects: Vec<usize>,
    pub masks: Vec<usize>,
    pub worlds: Vec<usize>,
    pub scenes: Vec<usize>,
    pub linestyles: Vec<usize>,
    pub gpencils: Vec<usize>,
    pub grease_pencils: Vec<usize>,
    pub palettes: Vec<usize>,
    pub cachefiles: Vec<usize>,
    pub hair_curves: Vec<usize>,
    pub pointclouds: Vec<usize>,
    pub volumes: Vec<usize>,
}

/// `BKE_animdata_main_cb` — `func(id, adt)` for each ID with AnimData.
/// Nodetree-bearing lists additionally emit `(nodetree_id, adt2)`.
pub fn bke_animdata_main_cb(
    lists: &MainIdLists,
    animdata_from_id: &dyn Fn(usize) -> Option<usize>,
    nodetree_of: &dyn Fn(usize) -> usize,
    func: &mut dyn FnMut(usize, usize),
) {
    /* `ANIMDATA_IDS_CB`. */
    macro_rules! ids_cb {
        ($list:expr) => {
            for &id in $list {
                if let Some(adt) = animdata_from_id(id) {
                    func(id, adt);
                }
            }
        };
    }
    /* `ANIMDATA_NODETREE_IDS_CB` — nodetree adt first, then own adt. */
    macro_rules! nodetree_ids_cb {
        ($list:expr) => {
            for &id in $list {
                let adt = animdata_from_id(id);
                let ntp = nodetree_of(id);
                if ntp != 0 {
                    if let Some(adt2) = animdata_from_id(ntp) {
                        func(ntp, adt2);
                    }
                }
                if let Some(adt) = adt {
                    func(id, adt);
                }
            }
        };
    }
    ids_cb!(&lists.nodetrees);
    nodetree_ids_cb!(&lists.textures);
    nodetree_ids_cb!(&lists.lights);
    nodetree_ids_cb!(&lists.materials);
    ids_cb!(&lists.cameras);
    ids_cb!(&lists.shapekeys);
    ids_cb!(&lists.metaballs);
    ids_cb!(&lists.curves);
    ids_cb!(&lists.armatures);
    ids_cb!(&lists.lattices);
    ids_cb!(&lists.meshes);
    ids_cb!(&lists.particles);
    ids_cb!(&lists.speakers);
    ids_cb!(&lists.movieclips);
    ids_cb!(&lists.objects);
    ids_cb!(&lists.masks);
    nodetree_ids_cb!(&lists.worlds);
    nodetree_ids_cb!(&lists.scenes);
    ids_cb!(&lists.linestyles);
    ids_cb!(&lists.gpencils);
    ids_cb!(&lists.grease_pencils);
    ids_cb!(&lists.palettes);
    ids_cb!(&lists.cachefiles);
    ids_cb!(&lists.hair_curves);
    ids_cb!(&lists.pointclouds);
    ids_cb!(&lists.volumes);
}
