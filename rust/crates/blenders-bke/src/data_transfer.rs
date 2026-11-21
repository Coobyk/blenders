//! Port of `intern/data_transfer.cc` — `DT_TYPE_*` → `CD_*`
//! mapping, `dttypes_capacity`/`item_types`, `to_cdmask`,
//! `dttype_to_srcdst_index`, `data_transfer_interp_float_do`
//! (all mix modes), `CustomDataTransferLayerMap`, layer-mapping
//! helpers.
//! `PORT_PENDING`: `data_transfer_layersmapping_*` CustomData
//! layer resolution (needs `Mesh::customdata` + Attribute API),
//! `BKE_object_data_transfer_*` (MeshRemap engine + islands).

/// `eDTType`.
pub const DT_TYPE_MDEFORMVERT: i32 = 1 << 0;
/// `DT_TYPE_SKIN`.
pub const DT_TYPE_SKIN: i32 = 1 << 2;
/// `DT_TYPE_BWEIGHT_VERT`.
pub const DT_TYPE_BWEIGHT_VERT: i32 = 1 << 3;
/// `DT_TYPE_SHARP_EDGE`.
pub const DT_TYPE_SHARP_EDGE: i32 = 1 << 8;
/// `DT_TYPE_SEAM`.
pub const DT_TYPE_SEAM: i32 = 1 << 9;
/// `DT_TYPE_CREASE`.
pub const DT_TYPE_CREASE: i32 = 1 << 10;
/// `DT_TYPE_BWEIGHT_EDGE`.
pub const DT_TYPE_BWEIGHT_EDGE: i32 = 1 << 11;
/// `DT_TYPE_FREESTYLE_EDGE`.
pub const DT_TYPE_FREESTYLE_EDGE: i32 = 1 << 12;
/// `DT_TYPE_MPROPCOL_VERT`.
pub const DT_TYPE_MPROPCOL_VERT: i32 = 1 << 16;
/// `DT_TYPE_LNOR`.
pub const DT_TYPE_LNOR: i32 = 1 << 17;
/// `DT_TYPE_UV`.
pub const DT_TYPE_UV: i32 = 1 << 24;
/// `DT_TYPE_SHARP_FACE`.
pub const DT_TYPE_SHARP_FACE: i32 = 1 << 25;
/// `DT_TYPE_FREESTYLE_FACE`.
pub const DT_TYPE_FREESTYLE_FACE: i32 = 1 << 26;
/// `DT_TYPE_MLOOPCOL_VERT`.
pub const DT_TYPE_MLOOPCOL_VERT: i32 = 1 << 27;
/// `DT_TYPE_MPROPCOL_LOOP`.
pub const DT_TYPE_MPROPCOL_LOOP: i32 = 1 << 28;
/// `DT_TYPE_MLOOPCOL_LOOP`.
pub const DT_TYPE_MLOOPCOL_LOOP: i32 = 1 << 29;
/// `DT_TYPE_VCOL_ALL`.
pub const DT_TYPE_VCOL_ALL: i32 = (1 << 16) | (1 << 27) | (1 << 28) | (1 << 29);
/// `DT_TYPE_MAX`.
pub const DT_TYPE_MAX: usize = 30;
/// `DT_TYPE_VERT_ALL`.
pub const DT_TYPE_VERT_ALL: i32 = DT_TYPE_MDEFORMVERT
    | DT_TYPE_SKIN
    | DT_TYPE_BWEIGHT_VERT
    | DT_TYPE_MPROPCOL_VERT
    | DT_TYPE_MLOOPCOL_VERT;
/// `DT_TYPE_EDGE_ALL`.
pub const DT_TYPE_EDGE_ALL: i32 = DT_TYPE_SHARP_EDGE
    | DT_TYPE_SEAM
    | DT_TYPE_CREASE
    | DT_TYPE_BWEIGHT_EDGE
    | DT_TYPE_FREESTYLE_EDGE;
/// `DT_TYPE_LOOP_ALL`.
pub const DT_TYPE_LOOP_ALL: i32 =
    DT_TYPE_LNOR | DT_TYPE_UV | DT_TYPE_MPROPCOL_LOOP | DT_TYPE_MLOOPCOL_LOOP;
/// `DT_TYPE_POLY_ALL`.
pub const DT_TYPE_POLY_ALL: i32 = DT_TYPE_UV | DT_TYPE_SHARP_FACE | DT_TYPE_FREESTYLE_FACE;

/// `DT_DATATYPE_IS_*` predicate family.
pub fn dt_datatype_is_vert(dt: i32) -> bool {
    matches!(
        dt,
        DT_TYPE_MDEFORMVERT
            | DT_TYPE_SKIN
            | DT_TYPE_BWEIGHT_VERT
            | DT_TYPE_MLOOPCOL_VERT
            | DT_TYPE_MPROPCOL_VERT
    ) || dt == DT_TYPE_MPROPCOL_VERT | DT_TYPE_MLOOPCOL_VERT
}
/// `IS_EDGE`.
pub fn dt_datatype_is_edge(dt: i32) -> bool {
    matches!(
        dt,
        DT_TYPE_CREASE | DT_TYPE_SHARP_EDGE | DT_TYPE_SEAM | DT_TYPE_BWEIGHT_EDGE
            | DT_TYPE_FREESTYLE_EDGE
    )
}
/// `IS_LOOP`.
pub fn dt_datatype_is_loop(dt: i32) -> bool {
    matches!(dt, DT_TYPE_UV | DT_TYPE_LNOR | DT_TYPE_MLOOPCOL_LOOP | DT_TYPE_MPROPCOL_LOOP)
        || dt == DT_TYPE_MPROPCOL_LOOP | DT_TYPE_MLOOPCOL_LOOP
}
/// `IS_FACE`.
pub fn dt_datatype_is_face(dt: i32) -> bool {
    matches!(dt, DT_TYPE_UV | DT_TYPE_SHARP_FACE | DT_TYPE_FREESTYLE_FACE)
}
/// `IS_MULTILAYERS`.
pub fn dt_datatype_is_multilayers(dt: i32) -> bool {
    matches!(
        dt,
        DT_TYPE_MDEFORMVERT | DT_TYPE_MPROPCOL_VERT | DT_TYPE_MLOOPCOL_VERT
            | DT_TYPE_MPROPCOL_LOOP | DT_TYPE_MLOOPCOL_LOOP | DT_TYPE_UV
    ) || dt == DT_TYPE_MPROPCOL_VERT | DT_TYPE_MLOOPCOL_VERT
        || dt == DT_TYPE_MPROPCOL_LOOP | DT_TYPE_MLOOPCOL_LOOP
}

/// `eDTMultilayerIndex`.
pub const DT_MULTILAYER_INDEX_INVALID: i32 = -1;
/// `MDEFORMVERT`.
pub const DT_MULTILAYER_INDEX_MDEFORMVERT: i32 = 0;
/// `VCOL_LOOP`.
pub const DT_MULTILAYER_INDEX_VCOL_LOOP: i32 = 2;
/// `UV`.
pub const DT_MULTILAYER_INDEX_UV: i32 = 3;
/// `VCOL_VERT`.
pub const DT_MULTILAYER_INDEX_VCOL_VERT: i32 = 4;
/// `MAX`.
pub const DT_MULTILAYER_INDEX_MAX: usize = 5;

/// `eDTLayersSrc`.
pub const DT_LAYERS_ACTIVE_SRC: i32 = -1;
/// `ALL_SRC`.
pub const DT_LAYERS_ALL_SRC: i32 = -2;
/// `VGROUP_SRC`.
pub const DT_LAYERS_VGROUP_SRC: i32 = 1 << 8;
/// `VGROUP_SRC_BONE_SELECT`.
pub const DT_LAYERS_VGROUP_SRC_BONE_SELECT: i32 = -(DT_LAYERS_VGROUP_SRC | 1);
/// `VGROUP_SRC_BONE_DEFORM`.
pub const DT_LAYERS_VGROUP_SRC_BONE_DEFORM: i32 = -(DT_LAYERS_VGROUP_SRC | 2);

/// `eDTLayersDst`.
pub const DT_LAYERS_ACTIVE_DST: i32 = -1;
/// `NAME_DST`.
pub const DT_LAYERS_NAME_DST: i32 = -2;
/// `INDEX_DST`.
pub const DT_LAYERS_INDEX_DST: i32 = -3;
/// `CREATE_DST`.
pub const DT_LAYERS_CREATE_DST: i32 = -4;
/// `REPLACE_DST`.
pub const DT_LAYERS_REPLACE_DST: i32 = -5;

/// `eCDTDataTransferMix` (`eDTMix`).
pub const CDT_MIX_NOMIX: i32 = -1;
/// `TRANSFER`.
pub const CDT_MIX_TRANSFER: i32 = 0;
/// `REPLACE_ABOVE_THRESHOLD`.
pub const CDT_MIX_REPLACE_ABOVE_THRESHOLD: i32 = 1;
/// `REPLACE_BELOW_THRESHOLD`.
pub const CDT_MIX_REPLACE_BELOW_THRESHOLD: i32 = 2;
/// `MIX`.
pub const CDT_MIX_MIX: i32 = 16;
/// `ADD`.
pub const CDT_MIX_ADD: i32 = 17;
/// `SUB`.
pub const CDT_MIX_SUB: i32 = 18;
/// `MUL`.
pub const CDT_MIX_MUL: i32 = 19;

/// `ME_VERT`/`ME_EDGE`/`ME_POLY`/`ME_LOOP` element-kind flags.
pub const ME_VERT: i32 = 1 << 0;
/// `ME_EDGE`.
pub const ME_EDGE: i32 = 1 << 1;
/// `ME_POLY`.
pub const ME_POLY: i32 = 1 << 2;
/// `ME_LOOP`.
pub const ME_LOOP: i32 = 1 << 3;

/// `CD_FAKE` base + fake types (`data_transfer_intern.hh`).
pub const CD_FAKE: i32 = 1 << 8;
/// `CD_FAKE_MDEFORMVERT` (CD_MDEFORMVERT=1).
pub const CD_FAKE_MDEFORMVERT: i32 = CD_FAKE | 1;
/// `CD_FAKE_SEAM`.
pub const CD_FAKE_SEAM: i32 = CD_FAKE | 100;
/// `CD_FAKE_UV`.
pub const CD_FAKE_UV: i32 = CD_FAKE | CD_PROP_FLOAT2;
/// `CD_FAKE_LNOR`.
pub const CD_FAKE_LNOR: i32 = CD_FAKE | 200;
/// `CD_FAKE_SHARP`.
pub const CD_FAKE_SHARP: i32 = CD_FAKE | 300;
/// `CD_FAKE_BWEIGHT`.
pub const CD_FAKE_BWEIGHT: i32 = CD_FAKE | 400;
/// `CD_FAKE_CREASE`.
pub const CD_FAKE_CREASE: i32 = CD_FAKE | 500;
/// `CD_FAKE_FREESTYLE_EDGE`.
pub const CD_FAKE_FREESTYLE_EDGE: i32 = CD_FAKE | 600;
/// `CD_FAKE_FREESTYLE_FACE`.
pub const CD_FAKE_FREESTYLE_FACE: i32 = CD_FAKE | 700;
/// `CD_FAKE_SKIN_RADIUS`.
pub const CD_FAKE_SKIN_RADIUS: i32 = CD_FAKE | 800;

/// `CD_MDEFORMVERT` (real type index used in the mask).
pub const CD_MDEFORMVERT: i32 = 1;
/// `CD_PROP_FLOAT2` mask index.
pub const CD_PROP_FLOAT2: i32 = 31;
/// `CD_PROP_BYTE_COLOR`.
pub const CD_PROP_BYTE_COLOR: i32 = 33;
/// `CD_PROP_COLOR`.
pub const CD_PROP_COLOR: i32 = 34;
/// `CD_MASK_MDEFORMVERT`.
pub const CD_MASK_MDEFORMVERT: i64 = 1 << CD_MDEFORMVERT;
/// `CD_MASK_PROP_FLOAT2`.
pub const CD_MASK_PROP_FLOAT2: i64 = 1i64 << CD_PROP_FLOAT2;

/// `CustomData_MeshMasks`.
#[derive(Clone, Copy, Default)]
pub struct CustomDataMeshMasks {
    /// `vmask`.
    pub vmask: i64,
    /// `emask`.
    pub emask: i64,
    /// `fmask`.
    pub fmask: i64,
    /// `pmask`.
    pub pmask: i64,
    /// `lmask`.
    pub lmask: i64,
}

/// `BKE_object_data_transfer_dttypes_to_cdmask`.
pub fn bke_object_data_transfer_dttypes_to_cdmask(
    dtdata_types: i32,
    r_data_masks: &mut CustomDataMeshMasks,
) {
    for i in 0..DT_TYPE_MAX {
        let dtdata_type = 1 << i;
        if dtdata_types & dtdata_type == 0 {
            continue;
        }
        let cddata_type = bke_object_data_transfer_dttype_to_cdtype(dtdata_type);
        if cddata_type & CD_FAKE == 0 {
            if dt_datatype_is_vert(dtdata_type) {
                r_data_masks.vmask |= 1i64 << cddata_type;
            } else if dt_datatype_is_edge(dtdata_type) {
                r_data_masks.emask |= 1i64 << cddata_type;
            } else if dt_datatype_is_loop(dtdata_type) {
                r_data_masks.lmask |= 1i64 << cddata_type;
            } else if dt_datatype_is_face(dtdata_type) {
                r_data_masks.pmask |= 1i64 << cddata_type;
            }
        } else if cddata_type == CD_FAKE_MDEFORMVERT {
            r_data_masks.vmask |= CD_MASK_MDEFORMVERT;
        } else if cddata_type == CD_FAKE_UV {
            r_data_masks.lmask |= CD_MASK_PROP_FLOAT2;
        }
    }
}

/// `BKE_object_data_transfer_get_dttypes_capacity` — `(ret,
/// r_advanced_mixing, r_threshold)`.
pub fn bke_object_data_transfer_get_dttypes_capacity(
    dtdata_types: i32,
) -> (bool, bool, bool) {
    let mut ret = false;
    let mut advanced_mixing = false;
    let mut threshold = false;
    for i in 0..DT_TYPE_MAX {
        if ret && advanced_mixing && threshold {
            break;
        }
        let dtdata_type = 1 << i;
        if dtdata_types & dtdata_type == 0 {
            continue;
        }
        match dtdata_type {
            DT_TYPE_MDEFORMVERT => {
                advanced_mixing = true;
                threshold = true;
                ret = true;
            }
            DT_TYPE_SKIN | DT_TYPE_SHARP_EDGE | DT_TYPE_SEAM | DT_TYPE_FREESTYLE_EDGE
            | DT_TYPE_SHARP_FACE | DT_TYPE_FREESTYLE_FACE => {
                threshold = true;
                ret = true;
            }
            DT_TYPE_BWEIGHT_VERT | DT_TYPE_CREASE | DT_TYPE_BWEIGHT_EDGE | DT_TYPE_UV => {
                ret = true;
            }
            DT_TYPE_MPROPCOL_VERT | DT_TYPE_MLOOPCOL_VERT | DT_TYPE_MPROPCOL_LOOP
            | DT_TYPE_MLOOPCOL_LOOP => {
                advanced_mixing = true;
                threshold = true;
                ret = true;
            }
            DT_TYPE_LNOR => {
                advanced_mixing = true;
                ret = true;
            }
            _ => {}
        }
    }
    (ret, advanced_mixing, threshold)
}

/// `BKE_object_data_transfer_get_dttypes_item_types`.
pub fn bke_object_data_transfer_get_dttypes_item_types(dtdata_types: i32) -> i32 {
    let mut ret = 0;
    for i in 0..DT_TYPE_MAX {
        if ret ^ (ME_VERT | ME_EDGE | ME_LOOP | ME_POLY) == 0 {
            break;
        }
        let dtdata_type = 1 << i;
        if dtdata_types & dtdata_type == 0 {
            continue;
        }
        if dt_datatype_is_vert(dtdata_type) {
            ret |= ME_VERT;
        }
        if dt_datatype_is_edge(dtdata_type) {
            ret |= ME_EDGE;
        }
        if dt_datatype_is_loop(dtdata_type) {
            ret |= ME_LOOP;
        }
        if dt_datatype_is_face(dtdata_type) {
            ret |= ME_POLY;
        }
    }
    ret
}

/// `BKE_object_data_transfer_dttype_to_cdtype`.
pub fn bke_object_data_transfer_dttype_to_cdtype(dtdata_type: i32) -> i32 {
    match dtdata_type {
        DT_TYPE_MDEFORMVERT => CD_FAKE_MDEFORMVERT,
        DT_TYPE_SKIN => CD_FAKE_SKIN_RADIUS,
        DT_TYPE_BWEIGHT_VERT => CD_FAKE_BWEIGHT,
        DT_TYPE_SHARP_EDGE => CD_FAKE_SHARP,
        DT_TYPE_SEAM => CD_FAKE_SEAM,
        DT_TYPE_CREASE => CD_FAKE_CREASE,
        DT_TYPE_BWEIGHT_EDGE => CD_FAKE_BWEIGHT,
        DT_TYPE_FREESTYLE_EDGE => CD_FAKE_FREESTYLE_EDGE,
        DT_TYPE_UV => CD_FAKE_UV,
        DT_TYPE_SHARP_FACE => CD_FAKE_SHARP,
        DT_TYPE_FREESTYLE_FACE => CD_FAKE_FREESTYLE_FACE,
        DT_TYPE_LNOR => CD_FAKE_LNOR,
        DT_TYPE_MLOOPCOL_VERT | DT_TYPE_MLOOPCOL_LOOP => CD_PROP_BYTE_COLOR,
        DT_TYPE_MPROPCOL_VERT | DT_TYPE_MPROPCOL_LOOP => CD_PROP_COLOR,
        _ => 0,
    }
}

/// `BKE_object_data_transfer_dttype_to_srcdst_index`.
pub fn bke_object_data_transfer_dttype_to_srcdst_index(dtdata_type: i32) -> i32 {
    match dtdata_type {
        DT_TYPE_MDEFORMVERT => DT_MULTILAYER_INDEX_MDEFORMVERT,
        DT_TYPE_UV => DT_MULTILAYER_INDEX_UV,
        DT_TYPE_MPROPCOL_VERT | DT_TYPE_MLOOPCOL_VERT => DT_MULTILAYER_INDEX_VCOL_VERT,
        DT_TYPE_MPROPCOL_LOOP | DT_TYPE_MLOOPCOL_LOOP => DT_MULTILAYER_INDEX_VCOL_LOOP,
        _ => DT_MULTILAYER_INDEX_INVALID,
    }
}

/// `data_transfer_interp_float_do`.
pub fn data_transfer_interp_float_do(
    mix_mode: i32,
    val_dst: f32,
    val_src: f32,
    mix_factor: f32,
) -> f32 {
    if (mix_mode == CDT_MIX_REPLACE_ABOVE_THRESHOLD && val_dst < mix_factor)
        || (mix_mode == CDT_MIX_REPLACE_BELOW_THRESHOLD && val_dst > mix_factor)
    {
        return val_dst;
    }
    let val_ret = match mix_mode {
        CDT_MIX_REPLACE_ABOVE_THRESHOLD | CDT_MIX_REPLACE_BELOW_THRESHOLD => val_src,
        CDT_MIX_MIX => (val_dst + val_src) * 0.5,
        CDT_MIX_ADD => val_dst + val_src,
        CDT_MIX_SUB => val_dst - val_src,
        CDT_MIX_MUL => val_dst * val_src,
        _ => val_src,
    };
    val_dst + (val_ret - val_dst) * mix_factor
}

/// `CustomDataTransferLayerMap` — one source→dest layer mapping.
#[derive(Clone)]
pub struct CustomDataTransferLayerMap {
    /// `data_type` (CD_*).
    pub data_type: i32,
    /// `mix_mode`.
    pub mix_mode: i32,
    /// `mix_factor`.
    pub mix_factor: f32,
    /// `mix_weights` (per-vert deform weights, injected).
    pub mix_weights: usize,
    /// `data_src_n` / `data_dst_n` layer indexes.
    pub data_src_n: i32,
    /// See above.
    pub data_dst_n: i32,
    /// `elem_src_n`/`elem_dst_n` (multilayer sub-index).
    pub elem_src_n: i32,
    /// See above.
    pub elem_dst_n: i32,
    /// `size`/`elem_size`.
    pub size: usize,
    /// See above.
    pub elem_size: usize,
    /// `data_src`/`data_dst` (injected storage handles).
    pub data_src: usize,
    /// See above.
    pub data_dst: usize,
    /// `interp` callback id (per-CDT interp fn).
    pub interp: Option<usize>,
    /// `copy_data` flag.
    pub copy_data: bool,
}

/// `data_transfer_get_loop_islands_generator` — for `CD_FAKE_UV`,
/// island seams come from `BKE_mesh_calc_islands_loop_face_edgeseam`.
pub fn data_transfer_get_loop_islands_generator(cddata_type: i32) -> Option<i32> {
    if cddata_type == CD_FAKE_UV {
        return Some(0);
    }
    None
}
