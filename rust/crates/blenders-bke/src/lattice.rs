//! Port of `intern/lattice.cc` — `Lattice` index math (`uvw`
//! mapping/flip), `calc_lat_fudu`, `BKE_lattice_resize`,
//! `outside_lattice`, vert-coords get/apply, active point, centers,
//! bounds, transform/translate.
//!
//! `lt->def` is `Vec<BPoint>`; deform-vert and `lt_ob` deformation
//! during resize are injected.

use blenders_dna::curve_types::BPoint;
use blenders_dna::lattice_types::{Lattice, LT_ACTBP_NONE, LT_GRID, LT_OUTSIDE};
use blenders_dna::meshdata_types::MDeformVert;
use blenders_math::matrix_c::{mul_m4_v3, mul_v3_m4v3, mat4_to_scale};
use blenders_math::{Mat4, Vec3};

/// `BKE_lattice_index_from_uvw`.
pub fn bke_lattice_index_from_uvw(lt: &Lattice, u: i32, v: i32, w: i32) -> i32 {
    let totu = lt.pntsu as i32;
    let totv = lt.pntsv as i32;
    w * (totu * totv) + v * totu + u
}

/// `BKE_lattice_index_to_uvw` — returns `(u, v, w)`.
pub fn bke_lattice_index_to_uvw(lt: &Lattice, index: i32) -> (i32, i32, i32) {
    let totu = lt.pntsu as i32;
    let totv = lt.pntsv as i32;
    let w = index / (totu * totv);
    let rem = index - w * totu * totv;
    let v = rem / totu;
    let u = rem - v * totu;
    (u, v, w)
}

/// `BKE_lattice_index_flip` — mirror selected axes.
pub fn bke_lattice_index_flip(
    lt: &Lattice,
    index: i32,
    flip_u: bool,
    flip_v: bool,
    flip_w: bool,
) -> i32 {
    let (mut u, mut v, mut w) = bke_lattice_index_to_uvw(lt, index);
    if flip_u {
        u = (lt.pntsu as i32 - 1) - u;
    }
    if flip_v {
        v = (lt.pntsv as i32 - 1) - v;
    }
    if flip_w {
        w = (lt.pntsw as i32 - 1) - w;
    }
    bke_lattice_index_from_uvw(lt, u, v, w)
}

/// `BKE_lattice_bitmap_from_flag` — set bits where `f1 & flag`
/// (optionally skipping hidden points).
pub fn bke_lattice_bitmap_from_flag(
    lt: &Lattice,
    bitmap: &mut Vec<bool>,
    flag: u8,
    clear: bool,
    respecthide: bool,
) {
    let tot = lt.pntsu as usize * lt.pntsv as usize * lt.pntsw as usize;
    if bitmap.len() < tot {
        bitmap.resize(tot, false);
    }
    for (i, bp) in lt.def.iter().take(tot).enumerate() {
        if (bp.f1 & flag) != 0 && (!respecthide || bp.hide == 0) {
            bitmap[i] = true;
        } else if clear {
            bitmap[i] = false;
        }
    }
}

/// `calc_lat_fudu` — `(fu, du)` start and step for an axis.
pub fn calc_lat_fudu(flag: i16, res: i32) -> (f32, f32) {
    if res == 1 {
        (0.0, 0.0)
    } else if flag & LT_GRID != 0 {
        (-0.5 * (res - 1) as f32, 1.0)
    } else {
        (-1.0, 2.0 / (res - 1) as f32)
    }
}

/// `BKE_lattice_resize` — clamp to 32000 points, generate the new
/// grid coords, deform them through `lt_ob` when provided (injected),
/// then rebuild `def` and the fu/du params.
pub fn bke_lattice_resize(
    lt: &mut Lattice,
    mut u_new: i32,
    mut v_new: i32,
    mut w_new: i32,
    lt_ob: Option<&mut dyn FnMut(&mut [Vec3])>,
) {
    lt.dvert = Vec::<MDeformVert>::new();

    while u_new * v_new * w_new > 32000 {
        if u_new >= v_new && u_new >= w_new {
            u_new -= 1;
        } else if v_new >= u_new && v_new >= w_new {
            v_new -= 1;
        } else {
            w_new -= 1;
        }
    }

    let (mut fu, mut du) = calc_lat_fudu(lt.flag, u_new);
    let (mut fv, mut dv) = calc_lat_fudu(lt.flag, v_new);
    let (mut fw, mut dw) = calc_lat_fudu(lt.flag, w_new);

    if lt_ob.is_some() {
        let default_size = 1.0f32;
        if u_new != 1 {
            fu = -default_size / 2.0;
            du = default_size / (u_new - 1) as f32;
        }
        if v_new != 1 {
            fv = -default_size / 2.0;
            dv = default_size / (v_new - 1) as f32;
        }
        if w_new != 1 {
            fw = -default_size / 2.0;
            dw = default_size / (w_new - 1) as f32;
        }
    }

    let mut vert_coords = Vec::with_capacity((u_new * v_new * w_new) as usize);
    let mut wc = fw;
    for _ in 0..w_new {
        let mut vc = fv;
        for _ in 0..v_new {
            let mut uc = fu;
            for _ in 0..u_new {
                vert_coords.push(Vec3::new(uc, vc, wc));
                uc += du;
            }
            vc += dv;
        }
        wc += dw;
    }

    if let Some(deform) = lt_ob {
        /* C swaps the lattice interp types to LINEAR, resets the
         * object's transform, runs `BKE_lattice_deform_coords` on the
         * generated grid, and restores both — injected as one call
         * over the coord buffer. */
        deform(&mut vert_coords);
    }

    lt.fu = fu;
    lt.fv = fv;
    lt.fw = fw;
    lt.du = du;
    lt.dv = dv;
    lt.dw = dw;

    lt.pntsu = u_new as i16;
    lt.pntsv = v_new as i16;
    lt.pntsw = w_new as i16;

    lt.actbp = LT_ACTBP_NONE;
    lt.def = vert_coords
        .into_iter()
        .map(|c| BPoint {
            vec: [c.x, c.y, c.z, 0.0],
            ..Default::default()
        })
        .collect();
}

/// `latt_bp`.
fn latt_bp(lt: &Lattice, u: i32, v: i32, w: i32) -> usize {
    bke_lattice_index_from_uvw(lt, u, v, w) as usize
}

/// `outside_lattice` — with `LT_OUTSIDE`, inner points hide and get
/// projected to the average of the 6 face positions; without it all
/// points unhide.
pub fn outside_lattice(lt: &mut Lattice) {
    let (u_n, v_n, w_n) = (
        lt.pntsu as i32,
        lt.pntsv as i32,
        lt.pntsw as i32,
    );
    if lt.flag & LT_OUTSIDE == 0 {
        for bp in lt.def.iter_mut() {
            bp.hide = 0;
        }
        return;
    }

    let du = if u_n > 1 { 1.0 / (u_n - 1) as f32 } else { 0.0 };
    let dv = if v_n > 1 { 1.0 / (v_n - 1) as f32 } else { 0.0 };
    let dw = if w_n > 1 { 1.0 / (w_n - 1) as f32 } else { 0.0 };

    for w in 0..w_n {
        for v in 0..v_n {
            for u in 0..u_n {
                let inner = !(u == 0 || v == 0 || w == 0
                    || u == u_n - 1
                    || v == v_n - 1
                    || w == w_n - 1);
                if !inner {
                    continue;
                }
                let i = latt_bp(lt, u, v, w);
                lt.def[i].hide = 1;
                lt.def[i].f1 &= !1; /* ~SELECT */

                let mut vec = [0.0f32; 3];
                let (a, b) = (latt_bp(lt, 0, v, w), latt_bp(lt, u_n - 1, v, w));
                let fac = du * u as f32;
                for k in 0..3 {
                    vec[k] += (1.0 - fac) * lt.def[a].vec[k] + fac * lt.def[b].vec[k];
                }
                let (a, b) = (latt_bp(lt, u, 0, w), latt_bp(lt, u, v_n - 1, w));
                let fac = dv * v as f32;
                for k in 0..3 {
                    vec[k] += (1.0 - fac) * lt.def[a].vec[k] + fac * lt.def[b].vec[k];
                }
                let (a, b) = (latt_bp(lt, u, v, 0), latt_bp(lt, u, v, w_n - 1));
                let fac = dw * w as f32;
                for k in 0..3 {
                    vec[k] += (1.0 - fac) * lt.def[a].vec[k] + fac * lt.def[b].vec[k];
                }
                for k in 0..3 {
                    lt.def[i].vec[k] = vec[k] / 3.0;
                }
            }
        }
    }
}

/// `BKE_lattice_vert_coords_get`.
pub fn bke_lattice_vert_coords_get(lt: &Lattice) -> Vec<Vec3> {
    let n = lt.pntsu as usize * lt.pntsv as usize * lt.pntsw as usize;
    lt.def
        .iter()
        .take(n)
        .map(|bp| Vec3::new(bp.vec[0], bp.vec[1], bp.vec[2]))
        .collect()
}

/// `BKE_lattice_vert_coords_apply_with_mat4`.
pub fn bke_lattice_vert_coords_apply_with_mat4(
    lt: &mut Lattice,
    vert_coords: &[Vec3],
    transform: Mat4,
) {
    let n = lt.pntsu as usize * lt.pntsv as usize * lt.pntsw as usize;
    for (i, co) in vert_coords.iter().take(n).enumerate() {
        let v = mul_v3_m4v3(transform, *co);
        lt.def[i].vec = [v.x, v.y, v.z, lt.def[i].vec[3]];
    }
}

/// `BKE_lattice_vert_coords_apply`.
pub fn bke_lattice_vert_coords_apply(lt: &mut Lattice, vert_coords: &[Vec3]) {
    let n = lt.pntsu as usize * lt.pntsv as usize * lt.pntsw as usize;
    for (i, co) in vert_coords.iter().take(n).enumerate() {
        lt.def[i].vec[0] = co.x;
        lt.def[i].vec[1] = co.y;
        lt.def[i].vec[2] = co.z;
    }
}

/// `BKE_lattice_active_point_get` — index or None when `actbp`
/// is unset/out of range.
pub fn bke_lattice_active_point_get(lt: &Lattice) -> Option<usize> {
    let n = lt.pntsu as i32 * lt.pntsv as i32 * lt.pntsw as i32;
    if lt.actbp >= 0 && lt.actbp < n {
        Some(lt.actbp as usize)
    } else {
        None
    }
}

/// `BKE_lattice_center_median`.
pub fn bke_lattice_center_median(lt: &Lattice) -> Option<[f32; 3]> {
    let n = lt.pntsu as usize * lt.pntsv as usize * lt.pntsw as usize;
    if n == 0 {
        return None;
    }
    let mut cent = Vec3::ZERO;
    for bp in lt.def.iter().take(n) {
        cent += Vec3::new(bp.vec[0], bp.vec[1], bp.vec[2]);
    }
    Some((cent / n as f32).into())
}

/// `BKE_lattice_minmax`.
pub fn bke_lattice_minmax(lt: &Lattice) -> Option<([f32; 3], [f32; 3])> {
    let n = lt.pntsu as usize * lt.pntsv as usize * lt.pntsw as usize;
    if n == 0 {
        return None;
    }
    let mut min = Vec3::splat(f32::INFINITY);
    let mut max = Vec3::splat(f32::NEG_INFINITY);
    for bp in lt.def.iter().take(n) {
        blenders_math::vector::minmax_v3(&mut min, &mut max, Vec3::new(bp.vec[0], bp.vec[1], bp.vec[2]));
    }
    Some((min.into(), max.into()))
}

/// `BKE_lattice_transform` — `do_keys` path is `PORT_PENDING`
/// (KeyBlock data isn't in the crate).
pub fn bke_lattice_transform(lt: &mut Lattice, mat: Mat4, _do_keys: bool) {
    let n = lt.pntsu as usize * lt.pntsv as usize * lt.pntsw as usize;
    for bp in lt.def.iter_mut().take(n) {
        let mut v = Vec3::new(bp.vec[0], bp.vec[1], bp.vec[2]);
        mul_m4_v3(mat, &mut v);
        bp.vec[0] = v.x;
        bp.vec[1] = v.y;
        bp.vec[2] = v.z;
    }
}

/// `BKE_lattice_translate` — `do_keys` path `PORT_PENDING`.
pub fn bke_lattice_translate(lt: &mut Lattice, offset: [f32; 3], _do_keys: bool) {
    let n = lt.pntsu as usize * lt.pntsv as usize * lt.pntsw as usize;
    for bp in lt.def.iter_mut().take(n) {
        bp.vec[0] += offset[0];
        bp.vec[1] += offset[1];
        bp.vec[2] += offset[2];
    }
}

/// `BKE_lattice_scale` helper parity — scale in `BKE_lattice_transform`
/// callers use `mat4_to_scale`.
pub fn lattice_scale_of(mat: Mat4) -> f32 {
    mat4_to_scale(mat)
}
