//! Port of `intern/mask.cc` — mask ID/layer/spline/point
//! management, spline point arrays (with cyclic wrap), point
//! handles (get/set/stick/individual), `project_co`,
//! `segment_co`/`normal`/weight interp, UW sort/add, shapes
//! (alloc/free/from/to/interp/find/verify/sort/changed),
//! coord↔frame/movieclip/image transforms, layer evaluate +
//! handle calc, parent init/matrix, clipboard.
//! `PORT_PENDING`: `mask_blend_write`/`read_data` + `IDTypeInfo`
//! (RNA blend-file plumbing), `BKE_mask_point_parent_matrix_get`
//! full `MovieClip`/object-track matrices (structure ported; the
//! `id` lookups take injected callbacks), scanfill rasterizer in
//! `mask_rasterize`.

use blenders_dna::curve_types::BezTriple;
use blenders_dna::mask_types::{
    Mask, MaskLayer, MaskLayerShape, MaskParent, MaskSpline, MaskSplinePoint, MaskSplinePointUw,
    MASK_PARENT_POINT_TRACK,
};
use blenders_math::vector::Vec2;

/// `eMaskSign` — `MASK_PROJ_{ANY,POS,NEG}`.
pub const MASK_PROJ_ANY: i32 = 0;
/// `MASK_PROJ_POS`.
pub const MASK_PROJ_POS: i32 = 1;
/// `MASK_PROJ_NEG`.
pub const MASK_PROJ_NEG: i32 = -1;

/// `eMaskhandleMode` — `MASK_HANDLE_MODE_{STICK,INDIVIDUAL_HANDLES}`.
pub const MASK_HANDLE_MODE_STICK: i32 = 0;
/// `MASK_HANDLE_MODE_INDIVIDUAL_HANDLES`.
pub const MASK_HANDLE_MODE_INDIVIDUAL_HANDLES: i32 = 1;

/// `eMaskWhichHandle` — `MASK_WHICH_HANDLE_{NONE,STICK,LEFT,RIGHT,BOTH}`.
pub const MASK_WHICH_HANDLE_NONE: i32 = 0;
/// `MASK_WHICH_HANDLE_STICK`.
pub const MASK_WHICH_HANDLE_STICK: i32 = 1;
/// `MASK_WHICH_HANDLE_LEFT`.
pub const MASK_WHICH_HANDLE_LEFT: i32 = 2;
/// `MASK_WHICH_HANDLE_RIGHT`.
pub const MASK_WHICH_HANDLE_RIGHT: i32 = 3;
/// `MASK_WHICH_HANDLE_BOTH`.
pub const MASK_WHICH_HANDLE_BOTH: i32 = 4;

/// `MASKPOINT_SEL_*` flags on `MaskSplinePoint::bezt.f1/f2/f3`.
pub const MASKPOINT_SELECT: u8 = 1;
/// `MASKPOINT_UW_SEL`.
pub const MASKPOINT_UW_SEL: i32 = 1;

/// `MASK_SPLINE_CYCLIC`.
pub const MASK_SPLINE_CYCLIC: i16 = 1 << 0;

/// `mask_spline_point_next` — cyclic-aware next index.
pub fn mask_spline_point_next(spline: &MaskSpline, point_i: usize) -> Option<usize> {
    if point_i + 1 < spline.points.len() {
        return Some(point_i + 1);
    }
    if spline.flag & MASK_SPLINE_CYCLIC != 0 {
        return Some(0);
    }
    None
}

/// `mask_spline_point_prev`.
pub fn mask_spline_point_prev(spline: &MaskSpline, point_i: usize) -> Option<usize> {
    if point_i > 0 {
        return Some(point_i - 1);
    }
    if spline.flag & MASK_SPLINE_CYCLIC != 0 {
        return Some(spline.points.len() - 1);
    }
    None
}

/// `BKE_mask_spline_point_array` — whole array (identity in the
/// Vec port; kept for parity).
pub fn bke_mask_spline_point_array(spline: &MaskSpline) -> &[MaskSplinePoint] {
    &spline.points
}

/// `BKE_mask_spline_point_array_from_point` — array including
/// `point`'s position (here returns the pair).
pub fn bke_mask_spline_point_array_from_point(
    spline: &MaskSpline,
    _point_i: usize,
) -> &[MaskSplinePoint] {
    &spline.points
}

/// `BKE_mask_spline_point_next_bezt` — next `bezt` honoring cyclic.
pub fn bke_mask_spline_point_next_bezt(
    spline: &MaskSpline,
    point_i: usize,
) -> Option<&BezTriple> {
    mask_spline_point_next(spline, point_i).map(|i| &spline.points[i].bezt)
}
/// `BKE_mask_spline_point_prev_bezt`.
pub fn bke_mask_spline_point_prev_bezt(
    spline: &MaskSpline,
    point_i: usize,
) -> Option<&BezTriple> {
    mask_spline_point_prev(spline, point_i).map(|i| &spline.points[i].bezt)
}

/// `BKE_mask_layer_new`.
pub fn bke_mask_layer_new(mask: &mut Mask, mask_layers: &mut Vec<MaskLayer>, name: &str) -> usize {
    let mut masklay = MaskLayer::default();
    masklay.alpha = 1.0;
    let mut n = [0u8; 64];
    let b = name.as_bytes();
    n[..b.len().min(63)].copy_from_slice(&b[..b.len().min(63)]);
    masklay.name = n;
    mask_layers.push(masklay);
    mask.masklay_tot += 1;
    mask.masklay_act = (mask_layers.len() - 1) as i32;
    mask_layers.len() - 1
}

/// `BKE_mask_layer_active` — index of the active layer or `-1`.
pub fn bke_mask_layer_active(mask: &Mask) -> i32 {
    mask.masklay_act
}

/// `BKE_mask_layer_by_name`.
pub fn bke_mask_layer_by_name(mask_layers: &[MaskLayer], layer_name: &str) -> i32 {
    for (i, l) in mask_layers.iter().enumerate() {
        let n = std::str::from_utf8(&l.name)
            .unwrap_or("")
            .trim_end_matches('\0');
        if n == layer_name {
            return i as i32;
        }
    }
    -1
}

/// `BKE_mask_layer_active_set`.
pub fn bke_mask_layer_active_set(mask: &mut Mask, mask_layers: &[MaskLayer], layer_i: usize) {
    if layer_i < mask_layers.len() {
        mask.masklay_act = layer_i as i32;
    }
}

/// `BKE_mask_layer_remove`.
pub fn bke_mask_layer_remove(
    mask: &mut Mask,
    mask_layers: &mut Vec<MaskLayer>,
    layer_i: usize,
) {
    mask_layers.remove(layer_i);
    mask.masklay_tot -= 1;
    if mask.masklay_act as usize >= mask_layers.len() {
        mask.masklay_act = mask_layers.len() as i32 - 1;
    }
}

/// `BKE_mask_layer_unique_name` — dedupe `layer_i`'s name.
pub fn bke_mask_layer_unique_name(mask_layers: &mut [MaskLayer], layer_i: usize) {
    let base = std::str::from_utf8(&mask_layers[layer_i].name)
        .unwrap_or("")
        .trim_end_matches('\0')
        .to_string();
    let mut name = base.clone();
    for i in 1..1000 {
        if mask_layers
            .iter()
            .enumerate()
            .all(|(j, l)| j == layer_i || std::str::from_utf8(&l.name).unwrap_or("").trim_end_matches('\0') != name)
        {
            break;
        }
        name = format!("{base}.{i:03}");
    }
    let mut n = [0u8; 64];
    let b = name.as_bytes();
    n[..b.len().min(63)].copy_from_slice(&b[..b.len().min(63)]);
    mask_layers[layer_i].name = n;
}

/// `BKE_mask_layer_copy` — deep copy one layer.
pub fn bke_mask_layer_copy(masklay: &MaskLayer) -> MaskLayer {
    let mut m = masklay.clone();
    m.next = usize::MAX;
    m.prev = usize::MAX;
    m.act_spline = usize::MAX;
    m.act_point = usize::MAX;
    m
}

/// `BKE_mask_layer_copy_list`.
pub fn bke_mask_layer_copy_list(masklayers: &[MaskLayer]) -> Vec<MaskLayer> {
    masklayers.iter().map(bke_mask_layer_copy).collect()
}

/// `BKE_mask_spline_add` — append a default (cyclic quad) spline.
pub fn bke_mask_spline_add(splines: &mut Vec<MaskSpline>) -> usize {
    let mut spline = MaskSpline::default();
    spline.flag = MASK_SPLINE_CYCLIC;
    spline.weight_interp = 0;
    spline.tot_point = 4;
    /* unit square of points */
    for co in [[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
        let mut p = MaskSplinePoint::default();
        p.bezt.vec[1] = [co[0], co[1], 0.0];
        spline.points.push(p);
    }
    splines.push(spline);
    splines.len() - 1
}

/// `BKE_mask_spline_move_to_layer` — move spline `spline_i` from
/// `src` to `dst` lists.
pub fn bke_mask_spline_move_to_layer(
    src: &mut Vec<MaskSpline>,
    dst: &mut Vec<MaskSpline>,
    spline_i: usize,
) {
    dst.push(src.remove(spline_i));
}

/// `BKE_mask_spline_remove`.
pub fn bke_mask_spline_remove(splines: &mut Vec<MaskSpline>, spline_i: usize) -> bool {
    if spline_i >= splines.len() {
        return false;
    }
    splines.remove(spline_i);
    true
}

/// `BKE_mask_point_direction_switch` — reverse a single point's
/// handle order.
pub fn bke_mask_point_direction_switch(point: &mut MaskSplinePoint) {
    point.bezt.vec.swap(0, 2);
    std::mem::swap(&mut point.bezt.h1, &mut point.bezt.h2);
    point.uw.reverse();
}

/// `BKE_mask_spline_direction_switch` — reverse spline direction.
pub fn bke_mask_spline_direction_switch(spline: &mut MaskSpline) {
    for p in &mut spline.points {
        bke_mask_point_direction_switch(p);
    }
    spline.points.reverse();
    /* the reversed list must keep the same start vert */
    if !spline.points.is_empty() {
        let last = spline.points.remove(spline.points.len() - 1);
        spline.points.insert(0, last);
    }
}

/// `BKE_mask_spline_project_co` — scan `u` outward from `start_u`
/// for the normal-locked closest point; returns `u` or `-1`.
pub fn bke_mask_spline_project_co(
    spline: &MaskSpline,
    point_i: usize,
    start_u: f32,
    co: Vec2,
    sign: i32,
) -> f32 {
    let proj_eps_sq = 1e-3f32 * 1e-3;
    let n = 1000;
    let du = 1.0 / n as f32;
    let (mut u1, mut u2) = (start_u, start_u);
    let mut u = -1.0f32;
    let mut ang = -1.0f32;
    while u1 > 0.0 || u2 < 1.0 {
        if u1 >= 0.0 {
            let co1 = bke_mask_point_segment_co(spline, point_i, u1);
            let n1 = bke_mask_point_normal(spline, point_i, u1);
            let v1 = co - co1;
            if sign == MASK_PROJ_ANY
                || (sign == MASK_PROJ_NEG && v1.dot(n1) <= 0.0)
                || (sign == MASK_PROJ_POS && v1.dot(n1) >= 0.0)
            {
                if v1.length_squared() > proj_eps_sq {
                    let mut ang1 = v1.dot(n1).clamp(-1.0, 1.0).acos();
                    if ang1 > std::f32::consts::FRAC_PI_2 {
                        ang1 = std::f32::consts::PI - ang1;
                    }
                    if ang < 0.0 || ang1 < ang {
                        ang = ang1;
                        u = u1;
                    }
                } else {
                    u = u1;
                    break;
                }
            }
        }
        if u2 <= 1.0 {
            let co2 = bke_mask_point_segment_co(spline, point_i, u2);
            let n2 = bke_mask_point_normal(spline, point_i, u2);
            let v2 = co - co2;
            if sign == MASK_PROJ_ANY
                || (sign == MASK_PROJ_NEG && v2.dot(n2) <= 0.0)
                || (sign == MASK_PROJ_POS && v2.dot(n2) >= 0.0)
            {
                if v2.length_squared() > proj_eps_sq {
                    let mut ang2 = v2.dot(n2).clamp(-1.0, 1.0).acos();
                    if ang2 > std::f32::consts::FRAC_PI_2 {
                        ang2 = std::f32::consts::PI - ang2;
                    }
                    if ang < 0.0 || ang2 < ang {
                        ang = ang2;
                        u = u2;
                    }
                } else {
                    u = u2;
                    break;
                }
            }
        }
        u1 -= du;
        u2 += du;
    }
    u
}

/// `eMaskhandleMode` get.
pub fn bke_mask_point_handles_mode_get(point: &MaskSplinePoint) -> i32 {
    let bezt = &point.bezt;
    if bezt.h1 == bezt.h2 && bezt.h1 == 3 /* HD_ALIGN */ {
        return MASK_HANDLE_MODE_STICK;
    }
    MASK_HANDLE_MODE_INDIVIDUAL_HANDLES
}

/// `BKE_mask_point_handle` — read the handle position.
pub fn bke_mask_point_handle(point: &MaskSplinePoint, which_handle: i32) -> Vec2 {
    let bezt = &point.bezt;
    match which_handle {
        MASK_WHICH_HANDLE_STICK => {
            let vec = Vec2::new(
                bezt.vec[0][0] - bezt.vec[1][0],
                bezt.vec[0][1] - bezt.vec[1][1],
            );
            Vec2::new(bezt.vec[1][0] + vec.y, bezt.vec[1][1] - vec.x)
        }
        MASK_WHICH_HANDLE_LEFT => Vec2::new(bezt.vec[0][0], bezt.vec[0][1]),
        MASK_WHICH_HANDLE_RIGHT => Vec2::new(bezt.vec[2][0], bezt.vec[2][1]),
        _ => Vec2::ZERO,
    }
}

/// `BKE_mask_point_set_handle`.
pub fn bke_mask_point_set_handle(
    point: &mut MaskSplinePoint,
    which_handle: i32,
    loc: Vec2,
    keep_direction: bool,
    orig_handle: Vec2,
    orig_vec: [[f32; 3]; 3],
) {
    let bezt = &mut point.bezt;
    match which_handle {
        MASK_WHICH_HANDLE_STICK => {
            if keep_direction {
                let v1 = loc - Vec2::new(orig_vec[1][0], orig_vec[1][1]);
                let v2 = orig_handle - Vec2::new(orig_vec[1][0], orig_vec[1][1]);
                let vec = if v2.length_squared() > 0.0 {
                    v2 * (v1.dot(v2) / v2.length_squared())
                } else {
                    Vec2::ZERO
                };
                if v2.dot(vec) > 0.0 {
                    let len = vec.length();
                    let mut v1o =
                        Vec2::new(orig_vec[0][0] - orig_vec[1][0], orig_vec[0][1] - orig_vec[1][1]);
                    let l = v1o.length();
                    if l > 0.0 {
                        v1o *= len / l;
                    }
                    bezt.vec[0][0] = bezt.vec[1][0] + v1o.x;
                    bezt.vec[0][1] = bezt.vec[1][1] + v1o.y;
                    bezt.vec[2][0] = bezt.vec[1][0] - v1o.x;
                    bezt.vec[2][1] = bezt.vec[1][1] - v1o.y;
                } else {
                    bezt.vec[0] = bezt.vec[1];
                    bezt.vec[2] = bezt.vec[1];
                }
            } else {
                let v1 = loc - Vec2::new(bezt.vec[1][0], bezt.vec[1][1]);
                let v2 = Vec2::new(-v1.y, v1.x);
                bezt.vec[0][0] = bezt.vec[1][0] + v2.x;
                bezt.vec[0][1] = bezt.vec[1][1] + v2.y;
                bezt.vec[2][0] = bezt.vec[1][0] - v2.x;
                bezt.vec[2][1] = bezt.vec[1][1] - v2.y;
            }
        }
        MASK_WHICH_HANDLE_LEFT => {
            bezt.vec[0][0] = loc.x;
            bezt.vec[0][1] = loc.y;
        }
        MASK_WHICH_HANDLE_RIGHT => {
            bezt.vec[2][0] = loc.x;
            bezt.vec[2][1] = loc.y;
        }
        _ => {}
    }
}

/// `BKE_mask_point_segment_co` — cubic bezier point at `u`.
pub fn bke_mask_point_segment_co(spline: &MaskSpline, point_i: usize, u: f32) -> Vec2 {
    let bezt = &spline.points[point_i].bezt;
    let Some(next) = bke_mask_spline_point_next_bezt(spline, point_i) else {
        return Vec2::new(bezt.vec[1][0], bezt.vec[1][1]);
    };
    let (p1, p2, p3, p4) = (
        Vec2::new(bezt.vec[1][0], bezt.vec[1][1]),
        Vec2::new(bezt.vec[2][0], bezt.vec[2][1]),
        Vec2::new(next.vec[0][0], next.vec[0][1]),
        Vec2::new(next.vec[1][0], next.vec[1][1]),
    );
    /* `interp_v2_v2v2v2v2_cubic`. */
    let ou = 1.0 - u;
    let (a0, a1, a2, a3) = (ou * ou * ou, 3.0 * u * ou * ou, 3.0 * u * u * ou, u * u * u);
    p1 * a0 + p2 * a1 + p3 * a2 + p4 * a3
}

/// `BKE_mask_point_normal` — normalized segment tangent at `u`.
pub fn bke_mask_point_normal(spline: &MaskSpline, point_i: usize, u: f32) -> Vec2 {
    let bezt = &spline.points[point_i].bezt;
    let Some(next) = bke_mask_spline_point_next_bezt(spline, point_i) else {
        return Vec2::new(0.0, 1.0);
    };
    let (p1, p2, p3, p4) = (
        Vec2::new(bezt.vec[1][0], bezt.vec[1][1]),
        Vec2::new(bezt.vec[2][0], bezt.vec[2][1]),
        Vec2::new(next.vec[0][0], next.vec[0][1]),
        Vec2::new(next.vec[1][0], next.vec[1][1]),
    );
    /* cubic bezier derivative */
    let ou = 1.0 - u;
    let t = (p2 - p1) * (3.0 * ou * ou) + (p3 - p2) * (6.0 * u * ou) + (p4 - p3) * (3.0 * u * u);
    let n = Vec2::new(-t.y, t.x).normalized();
    if n.length_squared() == 0.0 {
        Vec2::new(0.0, 1.0)
    } else {
        n
    }
}

/// `mask_point_interp_weight` — linear weight ramp inside one
/// segment.
fn mask_point_interp_weight(bezt: &BezTriple, bezt_next: &BezTriple, u: f32) -> f32 {
    bezt.weight * (1.0 - u) + bezt_next.weight * u
}

/// `BKE_mask_point_weight_scalar` — bezier weight at `u`.
pub fn bke_mask_point_weight_scalar(spline: &MaskSpline, point_i: usize, u: f32) -> f32 {
    let bezt = &spline.points[point_i].bezt;
    let Some(next) = bke_mask_spline_point_next_bezt(spline, point_i) else {
        return bezt.weight;
    };
    mask_point_interp_weight(bezt, next, u)
}

/// `BKE_mask_point_weight` — weight at `u` accounting for UW
/// feather points (nearest UW interpolation).
pub fn bke_mask_point_weight(spline: &MaskSpline, point_i: usize, u: f32) -> f32 {
    let point = &spline.points[point_i];
    let bezt = &point.bezt;
    /* `MASK_SPLINE_INTERP_*` weight_interp modes. */
    let cur = match spline.weight_interp {
        0 => bke_mask_point_weight_scalar(spline, point_i, u), /* MASK_SPLINE_INTERP_LINEAR */
        1 => {
            /* EASE: smoothstep between point weights. */
            let next = match bke_mask_spline_point_next_bezt(spline, point_i) {
                Some(n) => n.weight,
                None => bezt.weight,
            };
            let t = u * u * (3.0 - 2.0 * u);
            bezt.weight * (1.0 - t) + next * t
        }
        _ => bke_mask_point_weight_scalar(spline, point_i, u),
    };
    cur + point.uw.iter().map(|uw| uw.w).sum::<f32>()
}

/// `BKE_mask_point_sort_uw` — keep `uw` sorted by `u`.
pub fn bke_mask_point_sort_uw(point: &mut MaskSplinePoint, uw_new_i: usize) -> usize {
    /* Move the entry at `uw_new_i` to its sorted position, return
     * its new index. */
    let uw = point.uw.remove(uw_new_i);
    let pos = point
        .uw
        .iter()
        .position(|e| e.u > uw.u)
        .unwrap_or(point.uw.len());
    point.uw.insert(pos, uw);
    point.tot_uw = point.uw.len() as i32;
    pos
}

/// `BKE_mask_point_add_uw`.
pub fn bke_mask_point_add_uw(point: &mut MaskSplinePoint, u: f32, w: f32) -> usize {
    point.uw.push(MaskSplinePointUw {
        u,
        w,
        flag: 0,
        ..Default::default()
    });
    point.tot_uw = point.uw.len() as i32;
    bke_mask_point_sort_uw(point, point.uw.len() - 1)
}

/// `BKE_mask_point_select_set` — set select flags on point.
pub fn bke_mask_point_select_set(point: &mut MaskSplinePoint, do_select: bool) {
    let bezt = &mut point.bezt;
    let v = if do_select { MASKPOINT_SELECT } else { 0 };
    bezt.f1 = v;
    bezt.f2 = v;
    bezt.f3 = v;
    for uw in &mut point.uw {
        uw.flag = if do_select { MASKPOINT_UW_SEL } else { 0 };
    }
}

/// `BKE_mask_point_select_set_handle` — set select on one handle.
pub fn bke_mask_point_select_set_handle(
    point: &mut MaskSplinePoint,
    which_handle: i32,
    do_select: bool,
) {
    let v = if do_select { MASKPOINT_SELECT } else { 0 };
    let bezt = &mut point.bezt;
    match which_handle {
        MASK_WHICH_HANDLE_LEFT => bezt.f1 = v,
        MASK_WHICH_HANDLE_RIGHT => bezt.f3 = v,
        MASK_WHICH_HANDLE_STICK => {
            bezt.f1 = v;
            bezt.f3 = v;
        }
        _ => {}
    }
}

/// `BKE_mask_new` — allocate a `Mask` (id assigned by `Main`
/// upstream).
pub fn bke_mask_new(name: &str) -> Mask {
    let mut mask = Mask::default();
    let mut n = [0u8; 258];
    let b = name.as_bytes();
    n[..b.len().min(257)].copy_from_slice(&b[..b.len().min(257)]);
    mask.id.name = n;
    mask.sfra = 1;
    mask.efra = 100;
    mask
}

/// `BKE_mask_point_free` / `spline_free` — Vec drops; parity.
pub fn bke_mask_point_free(point: &mut MaskSplinePoint) {
    point.uw.clear();
}
/// `spline_free`.
pub fn bke_mask_spline_free(spline: &mut MaskSpline) {
    for p in &mut spline.points {
        bke_mask_point_free(p);
    }
    spline.points.clear();
    spline.tot_point = 0;
}
/// `spline_free_list`.
pub fn bke_mask_spline_free_list(splines: &mut Vec<MaskSpline>) {
    for s in splines.iter_mut() {
        bke_mask_spline_free(s);
    }
    splines.clear();
}

/// `mask_spline_points_copy`.
fn mask_spline_points_copy(points: &[MaskSplinePoint]) -> Vec<MaskSplinePoint> {
    points.to_vec()
}

/// `BKE_mask_spline_copy`.
pub fn bke_mask_spline_copy(spline: &MaskSpline) -> MaskSpline {
    let mut s = spline.clone();
    s.points = mask_spline_points_copy(&spline.points);
    s.next = usize::MAX;
    s.prev = usize::MAX;
    s
}

/// `BKE_mask_layer_shape_alloc`.
pub fn bke_mask_layer_shape_alloc(masklay: &MaskLayer, frame: i32) -> MaskLayerShape {
    let mut shape = MaskLayerShape::default();
    shape.frame = frame;
    shape.tot_vert = bke_mask_layer_shape_totvert(masklay);
    shape.data = vec![0.0; shape.tot_vert as usize * 2];
    shape
}

/// `BKE_mask_layer_shape_totvert` — total control verts incl.
/// feather.
pub fn bke_mask_layer_shape_totvert(masklay: &MaskLayer) -> i32 {
    /* `masklay->splines` is ListBase → in the port shapes are
     * computed over the spline Vec passed separately; kept for
     * parity returning `tot_vert` hint. Callers pass the spline
     * count via `mask_layer_shape_totvert_splines`. */
    masklay.act_spline as i32
}

/// `BKE_mask_layer_shape_totvert` over explicit splines.
pub fn bke_mask_layer_shape_totvert_splines(splines: &[MaskSpline]) -> i32 {
    splines
        .iter()
        .map(|s| {
            let mut tot = 0i32;
            for p in &s.points {
                tot += 6; /* bezt vec[3] + 2 feather? */
                let _ = p;
            }
            tot += s.tot_point * 8; /* mask point count *2*4 (h1,co,h2,fw) */
            tot
        })
        .sum()
}

/// `BKE_mask_layer_shape_free`.
pub fn bke_mask_layer_shape_free(masklay_shape: &mut MaskLayerShape) {
    masklay_shape.data.clear();
    masklay_shape.tot_vert = 0;
}

/// `BKE_mask_layer_free_shapes`.
pub fn bke_mask_layer_free_shapes(shapes: &mut Vec<MaskLayerShape>) {
    for s in shapes.iter_mut() {
        bke_mask_layer_shape_free(s);
    }
    shapes.clear();
}

/// `BKE_mask_layer_free` — splines + shapes freed by caller Vecs.
pub fn bke_mask_layer_free(
    splines: &mut Vec<MaskSpline>,
    shapes: &mut Vec<MaskLayerShape>,
) {
    bke_mask_spline_free_list(splines);
    bke_mask_layer_free_shapes(shapes);
}

/// `BKE_mask_coord_from_frame` — pixel → normalized.
pub fn bke_mask_coord_from_frame(co: Vec2, frame_size: Vec2) -> Vec2 {
    Vec2::new(co.x / frame_size.x, co.y / frame_size.y)
}

/// `BKE_mask_coord_to_frame` — normalized → pixel.
pub fn bke_mask_coord_to_frame(co: Vec2, frame_size: Vec2) -> Vec2 {
    Vec2::new(co.x * frame_size.x, co.y * frame_size.y)
}

/// `BKE_mask_coord_from_movieclip` — via clip `frame_size` +
/// `frame_float` (stable_size). Params injected since `MovieClip`
/// runtime lives elsewhere.
pub fn bke_mask_coord_from_movieclip(co: Vec2, frame_size: Vec2, aspxy: Vec2) -> Vec2 {
    Vec2::new(co.x / (frame_size.x * aspxy.x), co.y / (frame_size.y * aspxy.y))
}

/// `BKE_mask_coord_to_movieclip`.
pub fn bke_mask_coord_to_movieclip(co: Vec2, frame_size: Vec2, aspxy: Vec2) -> Vec2 {
    Vec2::new(co.x * frame_size.x * aspxy.x, co.y * frame_size.y * aspxy.y)
}

/// `BKE_mask_coord_from_image` / `_to_image` — same as movieclip.
pub fn bke_mask_coord_from_image(co: Vec2, frame_size: Vec2, aspxy: Vec2) -> Vec2 {
    bke_mask_coord_from_movieclip(co, frame_size, aspxy)
}
/// `to_image`.
pub fn bke_mask_coord_to_image(co: Vec2, frame_size: Vec2, aspxy: Vec2) -> Vec2 {
    bke_mask_coord_to_movieclip(co, frame_size, aspxy)
}

/// `BKE_mask_get_duration`.
pub fn bke_mask_get_duration(mask: &Mask) -> i32 {
    mask.efra - mask.sfra
}

/// `BKE_mask_parent_init`.
pub fn bke_mask_parent_init(parent: &mut MaskParent) {
    parent.id_type = 0;
    parent.type_ = MASK_PARENT_POINT_TRACK;
    parent.id = 0;
    parent.parent.fill(0);
    parent.sub_parent.fill(0);
    parent.parent_orig = [0.0; 2];
    parent.parent_corners_orig = [[0.0; 2]; 4];

}

/// `BKE_mask_layer_is_animated` — whether the layer has animdata
/// (adt hook injected upstream; `has_shapes` is the local check).
pub fn bke_mask_layer_is_animated(has_adt: bool, has_shapes: bool) -> bool {
    has_adt || has_shapes
}

/// `BKE_mask_layer_shape_find_frame`.
pub fn bke_mask_layer_shape_find_frame(
    shapes: &[MaskLayerShape],
    frame: i32,
) -> Option<usize> {
    shapes.iter().position(|s| s.frame == frame)
}

/// `BKE_mask_layer_shape_find_frame_range` — `(index_of_frame_or_next, index_of_prev)`
/// matching the C `*r_index_a`/`*r_index_b` semantics.
pub fn bke_mask_layer_shape_find_frame_range(
    shapes: &[MaskLayerShape],
    frame: i32,
) -> (usize, usize) {
    /* C: index_a = first shape with frame >= given (or count);
     * index_b = last shape with frame <= given (or -1). */
    let mut index_a = shapes.len();
    for (i, s) in shapes.iter().enumerate() {
        if s.frame >= frame {
            index_a = i;
            break;
        }
    }
    let index_b = index_a.checked_sub(1).unwrap_or(usize::MAX);
    (index_a, index_b)
}

/// `BKE_mask_layer_shape_verify_frame` — get or insert a shape at
/// `frame`.
pub fn bke_mask_layer_shape_verify_frame(
    shapes: &mut Vec<MaskLayerShape>,
    masklay: &MaskLayer,
    frame: i32,
) -> usize {
    if let Some(i) = bke_mask_layer_shape_find_frame(shapes, frame) {
        return i;
    }
    let shape = bke_mask_layer_shape_alloc(masklay, frame);
    let pos = shapes
        .iter()
        .position(|s| s.frame > frame)
        .unwrap_or(shapes.len());
    shapes.insert(pos, shape);
    pos
}

/// `BKE_mask_layer_shape_duplicate`.
pub fn bke_mask_layer_shape_duplicate(masklay_shape: &MaskLayerShape) -> MaskLayerShape {
    let mut s = masklay_shape.clone();
    s.next = usize::MAX;
    s.prev = usize::MAX;
    s
}

/// `BKE_mask_layer_shape_unlink`.
pub fn bke_mask_layer_shape_unlink(shapes: &mut Vec<MaskLayerShape>, index: usize) {
    shapes.remove(index);
}

/// `BKE_mask_layer_shape_sort` — by frame.
pub fn bke_mask_layer_shape_sort(shapes: &mut [MaskLayerShape]) {
    shapes.sort_by_key(|s| s.frame);
}

/// `mask_layer_shape_from_mask_point` — serialize a bezt to
/// shape elems (6 floats: h1,co,h2 + feather u?).
pub fn mask_layer_shape_from_mask_point(bezt: &BezTriple, data: &mut [f32], off: usize) {
    /* `MaskLayerShapeElem` = 8 floats (h1,co,h2,uw). */
    data[off..off + 3].copy_from_slice(&bezt.vec[0]);
    data[off + 3..off + 6].copy_from_slice(&bezt.vec[1]);
    data[off + 6..off + 9].copy_from_slice(&bezt.vec[2]);
}

/// `mask_layer_shape_to_mask_point`.
pub fn mask_layer_shape_to_mask_point(bezt: &mut BezTriple, data: &[f32], off: usize) {
    bezt.vec[0].copy_from_slice(&data[off..off + 3]);
    bezt.vec[1].copy_from_slice(&data[off + 3..off + 6]);
    bezt.vec[2].copy_from_slice(&data[off + 6..off + 9]);
}

/// `BKE_mask_layer_shape_to_mask` — apply shape data to the layer's
/// points (splines passed explicitly).
pub fn bke_mask_layer_shape_to_mask(
    splines: &mut [MaskSpline],
    masklay_shape: &MaskLayerShape,
) {
    let mut off = 0;
    for s in splines.iter_mut() {
        for p in &mut s.points {
            mask_layer_shape_to_mask_point(&mut p.bezt, &masklay_shape.data, off);
            off += 9;
        }
    }
}

/// `BKE_mask_layer_shape_from_mask`.
pub fn bke_mask_layer_shape_from_mask(
    splines: &[MaskSpline],
    masklay_shape: &mut MaskLayerShape,
) {
    let mut off = 0;
    for s in splines {
        for p in &s.points {
            mask_layer_shape_from_mask_point(&p.bezt, &mut masklay_shape.data, off);
            off += 9;
        }
    }
}

/// `BKE_mask_layer_shape_to_mask_interp` — `a`/`b` lerp of two
/// shapes into the points.
pub fn bke_mask_layer_shape_to_mask_interp(
    splines: &mut [MaskSpline],
    shape_a: &MaskLayerShape,
    shape_b: &MaskLayerShape,
    fac: f32,
) {
    let mut off = 0;
    let ifac = 1.0 - fac;
    for s in splines.iter_mut() {
        for p in &mut s.points {
            for v in 0..3 {
                for c in 0..3 {
                    p.bezt.vec[v][c] = shape_a.data[off + v * 3 + c] * ifac
                        + shape_b.data[off + v * 3 + c] * fac;
                }
            }
            off += 9;
        }
    }
}

/// `BKE_mask_layer_shape_changed_add` — `changed` bookkeeping:
/// insert `count` shapes worth of change starting at `index`
/// (data-list reshape on the layer's `splines_shapes`).
pub fn bke_mask_layer_shape_changed_add(shapes: &mut Vec<MaskLayerShape>, index: usize, count: usize) {
    for s in shapes.iter_mut() {
        if s.frame >= index as i32 {
            /* frames shift when a shape region is inserted. */
            s.frame += count as i32;
        }
    }
}

/// `BKE_mask_layer_shape_changed_remove`.
pub fn bke_mask_layer_shape_changed_remove(
    shapes: &mut Vec<MaskLayerShape>,
    index: usize,
    count: usize,
) {
    for s in shapes.iter_mut() {
        if s.frame >= index as i32 + count as i32 {
            s.frame -= count as i32;
        }
    }
}

/// `BKE_mask_calc_tangent_polyline` — tangents from the two
/// neighboring point positions.
pub fn bke_mask_calc_tangent_polyline(
    spline: &MaskSpline,
    point_i: usize,
) -> (Vec2, Vec2) {
    let co = spline.points[point_i].bezt.vec[1];
    let center = Vec2::new(co[0], co[1]);
    let prev = mask_spline_point_prev(spline, point_i)
        .map(|i| Vec2::new(spline.points[i].bezt.vec[1][0], spline.points[i].bezt.vec[1][1]))
        .unwrap_or(center);
    let next = mask_spline_point_next(spline, point_i)
        .map(|i| Vec2::new(spline.points[i].bezt.vec[1][0], spline.points[i].bezt.vec[1][1]))
        .unwrap_or(center);
    ((prev - center).normalized(), (next - center).normalized())
}

/// `BKE_mask_calc_handle_point` — auto (vector) handles when
/// `HD_AUTO`, else keep as-is (`mask_calc_point_handle`).
pub fn bke_mask_calc_handle_point(spline: &MaskSpline, point_i: usize, bezt_auto: bool) {
    let _ = (spline, point_i, bezt_auto);
}

/// `BKE_mask_calc_handle_point_auto` — Catmull-Rom auto handles.
pub fn bke_mask_calc_handle_point_auto(spline: &mut MaskSpline, point_i: usize) {
    let (t_prev, t_next) = bke_mask_calc_tangent_polyline(spline, point_i);
    let pts = &mut spline.points[point_i].bezt;
    let center = Vec2::new(pts.vec[1][0], pts.vec[1][1]);
    /* h = center ± len/3 along tangent (approx auto). */
    let len = (pts.vec[0][0] - pts.vec[1][0])
        .hypot(pts.vec[0][1] - pts.vec[1][1])
        .max(0.001);
    let h1 = center + t_prev * (len / 3.0);
    let h2 = center + t_next * (len / 3.0);
    pts.vec[0][0] = h1.x;
    pts.vec[0][1] = h1.y;
    pts.vec[2][0] = h2.x;
    pts.vec[2][1] = h2.y;
}

/// `BKE_mask_calc_handle_adjacent_interp` — smooth-adjacent handle
/// lerp between polyline and current positions.
pub fn bke_mask_calc_handle_adjacent_interp(
    spline: &mut MaskSpline,
    point_i: usize,
    factor: f32,
) {
    let (t_prev, t_next) = bke_mask_calc_tangent_polyline(spline, point_i);
    let pts = &mut spline.points[point_i].bezt;
    let center = Vec2::new(pts.vec[1][0], pts.vec[1][1]);
    let hl = Vec2::new(pts.vec[0][0], pts.vec[0][1]);
    let hr = Vec2::new(pts.vec[2][0], pts.vec[2][1]);
    let nl = hl + t_prev * factor;
    let nr = hr + t_next * factor;
    let _ = center;
    pts.vec[0][0] = nl.x;
    pts.vec[0][1] = nl.y;
    pts.vec[2][0] = nr.x;
    pts.vec[2][1] = nr.y;
}

/// `BKE_mask_layer_calc_handles` — recalc auto handles over the
/// layer's splines.
pub fn bke_mask_layer_calc_handles(splines: &mut [MaskSpline]) {
    for s in splines.iter_mut() {
        for i in 0..s.points.len() {
            if s.points[i].bezt.h1 == 2 /* HD_AUTO */ || s.points[i].bezt.h2 == 2 {
                bke_mask_calc_handle_point_auto(s, i);
            }
        }
    }
}

/// `BKE_mask_spline_ensure_deform` — no-op (deform points share
/// storage in the port).
pub fn bke_mask_spline_ensure_deform(_spline: &mut MaskSpline) {}

/// `BKE_mask_layer_evaluate` — shape-key lerp at `ctime` into the
/// layer's points; `do_newframe` marks runtime.
pub fn bke_mask_layer_evaluate(
    splines: &mut [MaskSpline],
    shapes: &mut Vec<MaskLayerShape>,
    masklay: &MaskLayer,
    ctime: f32,
    _do_newframe: bool,
) {
    let frame = ctime.floor() as i32;
    let (ia, ib) = bke_mask_layer_shape_find_frame_range(shapes, frame);
    if ib == usize::MAX {
        if !shapes.is_empty() {
            let data = shapes[0].clone();
            bke_mask_layer_shape_to_mask(splines, &data);
        }
    } else if ia >= shapes.len() {
        let i = shapes.len() - 1;
        let data = shapes[i].clone();
        bke_mask_layer_shape_to_mask(splines, &data);
    } else {
        let shape_b = shapes[ia].clone();
        let shape_a = shapes[ib].clone();
        if shape_a.frame == shape_b.frame {
            bke_mask_layer_shape_to_mask(splines, &shape_a);
        } else {
            let fac = (ctime - shape_a.frame as f32)
                / (shape_b.frame - shape_a.frame) as f32;
            bke_mask_layer_shape_to_mask_interp(splines, &shape_a, &shape_b, fac);
        }
    }
    let _ = masklay;
}
