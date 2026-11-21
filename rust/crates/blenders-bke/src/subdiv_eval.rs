//! Port of `intern/subdiv_eval.cc` — the `bke::subdiv` evaluation
//! layer over OpenSubdiv: evaluator begin/refine, UV/orco upload,
//! limit-point / derivatives / vertex-data / face-varying /
//! displacement queries, degenerate-derivative fix, and the batched
//! variants.
//!
//! `OpenSubdiv_Evaluator` → the `EvalOutput` trait (the C-API
//! wrapper is `PORT_PENDING` behind it); topology refiner internals
//! for face-varying upload come through `FVarMapSource`.

use blenders_bli::index_mask::IndexMask;
use blenders_bli::offset_indices::OffsetIndices;
use blenders_math::vector::{Vec2, Vec3};

/// `eSubdivEvaluatorType`.
pub const SUBDIV_EVALUATOR_TYPE_CPU: i32 = 0;
pub const SUBDIV_EVALUATOR_TYPE_GPU: i32 = 1;

/// `eOpenSubdivEvaluator` (opensubdiv_capi_type.h).
pub const OPENSUBDIV_EVALUATOR_CPU: i32 = 0;
pub const OPENSUBDIV_EVALUATOR_GPU: i32 = 1;

/// `opensubdiv_evaluator_from_subdiv_evaluator_type`.
pub fn opensubdiv_evaluator_from_subdiv_evaluator_type(evaluator_type: i32) -> i32 {
    match evaluator_type {
        SUBDIV_EVALUATOR_TYPE_CPU => OPENSUBDIV_EVALUATOR_CPU,
        SUBDIV_EVALUATOR_TYPE_GPU => OPENSUBDIV_EVALUATOR_GPU,
        _ => OPENSUBDIV_EVALUATOR_CPU,
    }
}

/// `OpenSubdiv_EvaluatorSettings`.
#[derive(Default, Clone, Copy)]
pub struct OpenSubdivEvaluatorSettings {
    /// Total floats per vertex of vertex data (orco 3 + cloth_orco 3).
    pub num_vertex_data: i32,
}

/// `PtexCoord` — face + (u, v) query point.
#[derive(Clone, Copy, Default)]
pub struct PtexCoord {
    pub ptex_face_index: i32,
    pub uv: Vec2,
}

/// `OpenSubdiv_EvalOutput` — every evaluator call the C file makes.
pub trait EvalOutput {
    fn set_settings(&mut self, settings: &OpenSubdivEvaluatorSettings);
    fn set_coarse_positions(&mut self, positions: &[f32]);
    fn set_face_varying_data(&mut self, layer_index: i32, data: &[[f32; 2]]);
    fn set_vertex_data(&mut self, data: &[f32], start_vertex: i32, num_verts: i32);
    fn refine(&mut self);
    /// `evaluateLimit` — P (+ dPdu/dPdv when requested).
    fn evaluate_limit(
        &mut self,
        ptex_face_index: i32,
        u: f32,
        v: f32,
        with_derivatives: bool,
    ) -> (Vec3, Vec3, Vec3);
    fn evaluate_vertex_data(&mut self, ptex_face_index: i32, u: f32, v: f32, out: &mut [f32]);
    fn evaluate_face_varying(
        &mut self,
        channel: i32,
        ptex_face_index: i32,
        u: f32,
        v: f32,
    ) -> Vec2;
    fn evaluate_patches_limit(&mut self, coords: &[PtexCoord], p: &mut [Vec3], with_derivatives: Option<(&mut [Vec3], &mut [Vec3])>);
    fn evaluate_patches_vertex_data(&mut self, coords: &[PtexCoord], out: &mut [f32]);
    fn evaluate_patches_face_varying(&mut self, channel: i32, coords: &[PtexCoord], out: &mut [Vec2]);
}

/// `SubdivDisplacementEvaluator` — displacement fn pair.
pub trait DisplacementEvaluator {
    /// `initialize` — may be absent (C checks null).
    fn initialize(&mut self) {}
    /// `eval_displacement`.
    fn eval_displacement(
        &mut self,
        ptex_face_index: i32,
        u: f32,
        v: f32,
        d_pdu: Vec3,
        d_pdv: Vec3,
    ) -> Vec3;
}

/// `FVarMapSource` — topology-refiner face-varying value indices per
/// face (what `GetFaceFVarValues` returns).
pub trait FVarMapSource {
    /// Number of base-level faces.
    fn num_faces(&self) -> i32;
    /// Number of fvar values for `layer_index`.
    fn num_fvar_values(&self, layer_index: i32) -> i32;
    /// `GetFaceFVarValues(face_index, layer_index)` → value indices
    /// in face-corner order.
    fn face_fvar_values(&self, face_index: i32, layer_index: i32) -> Vec<i32>;
}

/// `Subdiv` eval state.
pub struct SubdivEval<'a> {
    /// `topology_refiner` presence flag.
    pub has_topology_refiner: bool,
    /// `evaluator` — created lazily by `eval_begin`.
    pub evaluator: Option<Box<dyn EvalOutput>>,
    /// `displacement_evaluator` (may be absent).
    pub displacement_evaluator: Option<&'a mut dyn DisplacementEvaluator>,
    /// `stats_begin`/`stats_end` hooks (SUBDIV_STATS_*).
    pub stats: Option<&'a mut dyn FnMut(&'static str)>,
}

/// `eval_begin` — create evaluator if needed, apply settings, init
/// displacement. `create_evaluator` = `openSubdiv_createEvaluatorFromTopologyRefiner`.
pub fn eval_begin(
    subdiv: &mut SubdivEval,
    evaluator_type: i32,
    settings: &OpenSubdivEvaluatorSettings,
    create_evaluator: &mut dyn FnMut(i32 /*eOpenSubdivEvaluator*/) -> Option<Box<dyn EvalOutput>>,
) -> bool {
    if !subdiv.has_topology_refiner {
        /* Happens on input mesh with just loose geometry,
         * or when OpenSubdiv is disabled. */
        return false;
    }
    if subdiv.evaluator.is_none() {
        if let Some(s) = subdiv.stats.as_deref_mut() {
            s("SUBDIV_STATS_EVALUATOR_CREATE");
        }
        let osd_type = opensubdiv_evaluator_from_subdiv_evaluator_type(evaluator_type);
        subdiv.evaluator = create_evaluator(osd_type);
        if subdiv.evaluator.is_none() {
            return false;
        }
    }
    /* TODO(sergey): Check for topology change. */
    let ev = subdiv.evaluator.as_mut().unwrap();
    ev.set_settings(settings);
    eval_init_displacement(subdiv);
    true
}

/// `set_coarse_positions` — uploads positions minus `verts_no_face`.
pub fn set_coarse_positions(
    evaluator: &mut dyn EvalOutput,
    positions: &[Vec3],
    verts_no_face: &IndexMask,
    positions_len: i64,
) {
    let flat: Vec<f32> = if verts_no_face.is_empty() {
        positions.iter().flat_map(|v| [v.x, v.y, v.z]).collect()
    } else {
        let mut used = Vec::with_capacity(positions.len() - verts_no_face.size() as usize);
        let verts = verts_no_face.complement(&IndexMask::from_range(0, positions_len));
        for v in verts.iter() {
            let p = positions[v as usize];
            used.extend_from_slice(&[p.x, p.y, p.z]);
        }
        used.iter().flat_map(|v| [*v]).collect()
    };
    evaluator.set_coarse_positions(&flat);
}

/// `set_face_varying_data_from_uv` — gather UVs by fvar value
/// indices then upload (`TaskParallelRange` → serial).
pub fn set_face_varying_data_from_uv(
    evaluator: &mut dyn EvalOutput,
    fvar: &dyn FVarMapSource,
    faces: &OffsetIndices,
    uv_map: &[Vec2],
    layer_index: i32,
) {
    let num_fvar_values = fvar.num_fvar_values(layer_index);
    /* Use a temporary buffer so we do not upload UVs one at a time to
     * the GPU. */
    let mut buffer = vec![[0.0f32; 2]; num_fvar_values as usize];

    for face_index in 0..fvar.num_faces() {
        let face_uvs = &uv_map[faces.group(face_index as usize).start() as usize
            ..faces.group(face_index as usize).one_after_last() as usize];
        /* TODO(sergey): OpenSubdiv's C-API converter can change
         * winding of loops of a face, need to watch for that. */
        let uv_indices = fvar.face_fvar_values(face_index, layer_index);
        for (i, &uv_idx) in uv_indices.iter().enumerate() {
            buffer[uv_idx as usize] = [face_uvs[i].x, face_uvs[i].y];
        }
    }

    evaluator.set_face_varying_data(layer_index, &buffer);
}

/// `set_vert_data_from_orco` — upload orco (+cloth_orco) vertex data.
pub fn set_vert_data_from_orco(
    evaluator: &mut dyn EvalOutput,
    num_verts: i32,
    orco: Option<&[[f32; 3]]>,
    cloth_orco: Option<&[[f32; 3]]>,
) {
    match (orco, cloth_orco) {
        (Some(orco), Some(cloth_orco)) => {
            /* Set one by one if have both. */
            for i in 0..num_verts as usize {
                let mut data = [0.0f32; 6];
                data[..3].copy_from_slice(&orco[i]);
                data[3..].copy_from_slice(&cloth_orco[i]);
                evaluator.set_vertex_data(&data, i as i32, 1);
            }
        }
        (Some(orco), None) => {
            let flat: Vec<f32> = orco.iter().flatten().copied().collect();
            evaluator.set_vertex_data(&flat, 0, num_verts);
        }
        (None, Some(cloth_orco)) => {
            let flat: Vec<f32> = cloth_orco.iter().flatten().copied().collect();
            evaluator.set_vertex_data(&flat, 0, num_verts);
        }
        (None, None) => {}
    }
}

/// `get_mesh_evaluator_settings`.
pub fn get_mesh_evaluator_settings(has_orco: bool, has_cloth_orco: bool) -> OpenSubdivEvaluatorSettings {
    OpenSubdivEvaluatorSettings {
        num_vertex_data: (if has_orco { 3 } else { 0 }) + (if has_cloth_orco { 3 } else { 0 }),
    }
}

/// `eval_refine_from_mesh` — upload positions/UV/orco then refine.
#[allow(clippy::too_many_arguments)]
pub fn eval_refine_from_mesh(
    subdiv: &mut SubdivEval,
    positions: &[Vec3],
    verts_no_face: &IndexMask,
    fvar: Option<&dyn FVarMapSource>,
    faces: &OffsetIndices,
    uv_maps: &[&[Vec2]],
    orco: Option<&[[f32; 3]]>,
    cloth_orco: Option<&[[f32; 3]]>,
    num_verts: i32,
) -> bool {
    if subdiv.evaluator.is_none() {
        /* NOTE: This situation is supposed to be handled by begin(). */
        debug_assert!(false, "Is not supposed to happen");
        return false;
    }
    let ev = subdiv.evaluator.as_mut().unwrap().as_mut();
    /* Set coordinates of base mesh vertices. */
    set_coarse_positions(ev, positions, verts_no_face, positions.len() as i64);
    /* Set face-varying data to UV maps. */
    if let Some(fvar) = fvar {
        for (i, uv_map) in uv_maps.iter().enumerate() {
            set_face_varying_data_from_uv(ev, fvar, faces, uv_map, i as i32);
        }
    }
    /* Set vertex data to orco. */
    set_vert_data_from_orco(ev, num_verts, orco, cloth_orco);
    /* Update evaluator to the new coarse geometry. */
    if let Some(s) = subdiv.stats.as_deref_mut() {
        s("SUBDIV_STATS_EVALUATOR_REFINE");
    }
    ev.refine();
    true
}

/// `eval_init_displacement`.
pub fn eval_init_displacement(subdiv: &mut SubdivEval) {
    if let Some(disp) = subdiv.displacement_evaluator.as_deref_mut() {
        disp.initialize();
    }
}

/// `eval_limit_point`.
pub fn eval_limit_point(subdiv: &mut SubdivEval, ptex_face_index: i32, u: f32, v: f32) -> Vec3 {
    match subdiv.evaluator.as_mut() {
        Some(ev) => ev.evaluate_limit(ptex_face_index, u, v, false).0,
        None => Vec3::splat(0.0),
    }
}

/// `fix_degenerate_derivatives` — step inside the face a little when
/// derivatives are zero or equal (see #160589-class degenerate
/// tangent spaces).
fn fix_degenerate_derivatives(
    subdiv: &mut SubdivEval,
    ptex_face_index: i32,
    u: f32,
    v: f32,
    r_p: &mut Vec3,
    r_dpdu: &mut Vec3,
    r_dpdv: &mut Vec3,
) {
    let is_zero = |v: Vec3| v.length_squared() == 0.0;
    if is_zero(*r_dpdu) || is_zero(*r_dpdv) || *r_dpdu == *r_dpdv {
        if let Some(ev) = subdiv.evaluator.as_mut() {
            let (p, du, dv) =
                ev.evaluate_limit(ptex_face_index, u * 0.999 + 0.0005, v * 0.999 + 0.0005, true);
            *r_p = p;
            *r_dpdu = du;
            *r_dpdv = dv;
        }
    }
}

/// `eval_limit_point_and_derivatives`.
pub fn eval_limit_point_and_derivatives(
    subdiv: &mut SubdivEval,
    ptex_face_index: i32,
    u: f32,
    v: f32,
) -> (Vec3, Vec3, Vec3) {
    let (mut p, mut dpdu, mut dpdv) = match subdiv.evaluator.as_mut() {
        Some(ev) => ev.evaluate_limit(ptex_face_index, u, v, true),
        None => (Vec3::splat(0.0), Vec3::splat(0.0), Vec3::splat(0.0)),
    };
    fix_degenerate_derivatives(subdiv, ptex_face_index, u, v, &mut p, &mut dpdu, &mut dpdv);
    (p, dpdu, dpdv)
}

/// `eval_limit_point_and_normal`.
pub fn eval_limit_point_and_normal(
    subdiv: &mut SubdivEval,
    ptex_face_index: i32,
    u: f32,
    v: f32,
) -> (Vec3, Vec3) {
    let (p, dpdu, dpdv) = eval_limit_point_and_derivatives(subdiv, ptex_face_index, u, v);
    (p, dpdu.cross(dpdv).normalized())
}

/// `eval_vert_data` (single point).
pub fn eval_vert_data(
    subdiv: &mut SubdivEval,
    ptex_face_index: i32,
    u: f32,
    v: f32,
    out: &mut [f32],
) {
    if let Some(ev) = subdiv.evaluator.as_mut() {
        ev.evaluate_vertex_data(ptex_face_index, u, v, out);
    }
}

/// `eval_face_varying` (single point).
pub fn eval_face_varying(
    subdiv: &mut SubdivEval,
    face_varying_channel: i32,
    ptex_face_index: i32,
    u: f32,
    v: f32,
) -> Vec2 {
    match subdiv.evaluator.as_mut() {
        Some(ev) => ev.evaluate_face_varying(face_varying_channel, ptex_face_index, u, v),
        None => Vec2::splat(0.0),
    }
}

/// `eval_displacement` (single point).
pub fn eval_displacement(
    subdiv: &mut SubdivEval,
    ptex_face_index: i32,
    u: f32,
    v: f32,
    d_pdu: Vec3,
    d_pdv: Vec3,
) -> Vec3 {
    match subdiv.displacement_evaluator.as_deref_mut() {
        Some(disp) => disp.eval_displacement(ptex_face_index, u, v, d_pdu, d_pdv),
        None => Vec3::splat(0.0),
    }
}

/// `eval_final_point` — limit + displacement when present.
pub fn eval_final_point(subdiv: &mut SubdivEval, ptex_face_index: i32, u: f32, v: f32) -> Vec3 {
    if subdiv.displacement_evaluator.is_some() {
        let (p, dpdu, dpdv) = eval_limit_point_and_derivatives(subdiv, ptex_face_index, u, v);
        p + eval_displacement(subdiv, ptex_face_index, u, v, dpdu, dpdv)
    } else {
        eval_limit_point(subdiv, ptex_face_index, u, v)
    }
}

/// `eval_limit_points` (batched).
pub fn eval_limit_points(subdiv: &mut SubdivEval, coords: &[PtexCoord], r_p: &mut [Vec3]) {
    debug_assert_eq!(coords.len(), r_p.len());
    if coords.is_empty() {
        return;
    }
    match subdiv.evaluator.as_mut() {
        Some(ev) => ev.evaluate_patches_limit(coords, r_p, None),
        None => r_p.fill(Vec3::splat(0.0)),
    }
}

/// `eval_limit_points_and_derivatives` (batched, + fix per point).
pub fn eval_limit_points_and_derivatives(
    subdiv: &mut SubdivEval,
    coords: &[PtexCoord],
    r_p: &mut [Vec3],
    r_dpdu: &mut [Vec3],
    r_dpdv: &mut [Vec3],
) {
    debug_assert_eq!(coords.len(), r_p.len());
    debug_assert_eq!(coords.len(), r_dpdu.len());
    debug_assert_eq!(coords.len(), r_dpdv.len());
    if coords.is_empty() {
        return;
    }
    if let Some(ev) = subdiv.evaluator.as_mut() {
        ev.evaluate_patches_limit(coords, r_p, Some((r_dpdu, r_dpdv)));
    }
    for i in 0..coords.len() {
        let coord = &coords[i];
        /* Fix each in place — borrow ends before the loop. */
        let (p, du, dv) = (r_p[i], r_dpdu[i], r_dpdv[i]);
        let (mut p2, mut du2, mut dv2) = (p, du, dv);
        fix_degenerate_derivatives(
            subdiv,
            coord.ptex_face_index,
            coord.uv.x,
            coord.uv.y,
            &mut p2,
            &mut du2,
            &mut dv2,
        );
        r_p[i] = p2;
        r_dpdu[i] = du2;
        r_dpdv[i] = dv2;
    }
}

/// `eval_final_points` (batched).
pub fn eval_final_points(subdiv: &mut SubdivEval, coords: &[PtexCoord], r_p: &mut [Vec3]) {
    if subdiv.displacement_evaluator.is_none() {
        eval_limit_points(subdiv, coords, r_p);
        return;
    }
    let n = coords.len();
    let mut dpdu = vec![Vec3::splat(0.0); n];
    let mut dpdv = vec![Vec3::splat(0.0); n];
    eval_limit_points_and_derivatives(subdiv, coords, r_p, &mut dpdu, &mut dpdv);
    for i in 0..n {
        let coord = &coords[i];
        let d = eval_displacement(
            subdiv,
            coord.ptex_face_index,
            coord.uv.x,
            coord.uv.y,
            dpdu[i],
            dpdv[i],
        );
        r_p[i] += d;
    }
}

/// `eval_displacement_averaged` — per-group mean displacement.
pub fn eval_displacement_averaged(
    subdiv: &mut SubdivEval,
    coords: &[PtexCoord],
    groups: &OffsetIndices,
    r_d: &mut [Vec3],
) {
    debug_assert_eq!(groups.total_size() as i64, coords.len() as i64);
    debug_assert_eq!(groups.len() as usize, r_d.len());
    let n = coords.len();
    let mut p = vec![Vec3::splat(0.0); n];
    let mut dpdu = vec![Vec3::splat(0.0); n];
    let mut dpdv = vec![Vec3::splat(0.0); n];
    eval_limit_points_and_derivatives(subdiv, coords, &mut p, &mut dpdu, &mut dpdv);
    for i in 0..groups.len() {
        let group = groups.group(i);
        let mut d_sum = Vec3::splat(0.0);
        for j in group.start()..group.one_after_last() {
            let coord = &coords[j as usize];
            d_sum += eval_displacement(
                subdiv,
                coord.ptex_face_index,
                coord.uv.x,
                coord.uv.y,
                dpdu[j as usize],
                dpdv[j as usize],
            );
        }
        r_d[i as usize] = d_sum * (1.0 / group.size() as f32);
    }
}

/// `eval_vert_data` (batched).
pub fn eval_vert_data_batched(subdiv: &mut SubdivEval, coords: &[PtexCoord], out: &mut [f32]) {
    if coords.is_empty() {
        return;
    }
    if let Some(ev) = subdiv.evaluator.as_mut() {
        ev.evaluate_patches_vertex_data(coords, out);
    }
}

/// `eval_face_varying` (batched).
pub fn eval_face_varying_batched(
    subdiv: &mut SubdivEval,
    face_varying_channel: i32,
    coords: &[PtexCoord],
    out: &mut [Vec2],
) {
    debug_assert_eq!(coords.len(), out.len());
    if coords.is_empty() {
        return;
    }
    match subdiv.evaluator.as_mut() {
        Some(ev) => ev.evaluate_patches_face_varying(face_varying_channel, coords, out),
        None => out.fill(Vec2::splat(0.0)),
    }
}
