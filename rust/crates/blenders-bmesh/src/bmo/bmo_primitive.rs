//! Port of `bmo_primitive.cc` — procedural mesh creation ops:
//! create_grid/quadsphere/uvsphere/icosphere/monkey/circle/cone/cube.
//! `calc_uvs` paths are PORT_PENDING (CustomData loop UVs) — forced false.

use crate::core::*;
use crate::construct::*;
use crate::operators::*;
use crate::{BMesh, BM_EDGE, BM_VERT, BM_NIL};
use blenders_math::matrix::{Mat3, Mat4};
use blenders_math::quaternion::{axis_angle_to_mat3, sin_cos_from_fraction};
use blenders_math::vector::{Vec3, Vec4};
use super::primitive_data::*;

const VERT_MARK: u8 = 1;
const EDGE_ORIG: u8 = 1;
const EDGE_MARK: u8 = 2;
const FACE_MARK: u8 = 1;
const FACE_NEW: u8 = 2;
const MONKEYNV: usize = 271;
const MONKEYNF: usize = 250;
const MONKEYO: i32 = 4;

const QUADSPHERE_EQUI_ANGULAR_EVEN_AREA: i32 = 0;
const QUADSPHERE_EQUI_ANGULAR: i32 = 1;

fn mul_v3_m4v3(m: &Mat4, v: Vec3) -> Vec3 {
    let r = *m * Vec4::new(v.x, v.y, v.z, 1.0); Vec3::new(r.x, r.y, r.z)
}
fn mul_v3_m3v3(m: &Mat3, v: Vec3) -> Vec3 { *m * v }

/// `bmo_create_grid_exec`.
pub fn bmo_create_grid_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let dia = bmo_slot_float_get(op, false, "size");
    let xtot = bmo_slot_int_get(op, false, "x_segments").max(1);
    let ytot = bmo_slot_int_get(op, false, "y_segments").max(1);
    let xtot_inv2 = 2.0 / xtot as f32;
    let ytot_inv2 = 2.0 / ytot as f32;
    let mat = bmo_slot_mat4_get(op, false, "matrix");
    let mut varr: Vec<usize> = Vec::with_capacity(((xtot + 1) * (ytot + 1)) as usize);
    for y in 0..=ytot {
        let vy = (y as f32 * ytot_inv2 - 1.0) * dia;
        for x in 0..=xtot {
            let vx = (x as f32 * xtot_inv2 - 1.0) * dia;
            let tvec = mul_v3_m4v3(&mat, Vec3::new(vx, vy, 0.0));
            let vi = bm_vert_create(bm, tvec, None, 0);
            bm.vpool[vi].head.api_flag |= VERT_MARK;
            varr.push(vi);
        }
    }
    let xy = |x: i32, y: i32| (x + y * (xtot + 1)) as usize;
    for y in 1..=ytot {
        for x in 1..=xtot {
            let vquad = [varr[xy(x - 1, y - 1)], varr[xy(x, y - 1)],
                         varr[xy(x, y)], varr[xy(x - 1, y)]];
            bm_face_create_verts(bm, &vquad, None, 0, true, true);
        }
    }
}

/// `quadsphere_co_from_cube_equi_angular`.
fn qs_co_from_cube_equi_angular(co: Vec3, on_face: [bool; 3], angle: f32, angle_scale: f32) -> Vec3 {
    let mut co_sphere = Vec3::ZERO;
    for axis in 0..3 {
        co_sphere[axis] = if on_face[axis] { co[axis] } else { (co[axis] * angle).tan() * angle_scale };
    }
    co_sphere
}

/// `quadsphere_co_from_cube` (QuadSphereMethod).
fn qs_co_from_cube(co: Vec3, on_face: [bool; 3], method: i32) -> Vec3 {
    match method {
        QUADSPHERE_EQUI_ANGULAR_EVEN_AREA =>
            qs_co_from_cube_equi_angular(co, on_face, 0.8687, 0.845878184),
        QUADSPHERE_EQUI_ANGULAR =>
            qs_co_from_cube_equi_angular(co, on_face, std::f32::consts::FRAC_PI_4, 1.0),
        _ => co,
    }
}

/// `bmo_create_quadsphere_exec`.
pub fn bmo_create_quadsphere_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let rad = bmo_slot_float_get(op, false, "radius");
    let seg = bmo_slot_int_get(op, false, "segments").max(1);
    let seg_vert = seg + 1;
    let method = bmo_slot_int_get(op, false, "method");
    let mat = bmo_slot_mat4_get(op, false, "matrix");
    let layer_full_len = seg_vert * seg_vert;
    let layer_ring_len = seg * 4;
    let vert_index = |g: [i32; 3]| -> usize {
        if g[2] == 0 { return (g[1] * seg_vert + g[0]) as usize; }
        if g[2] == seg {
            return (layer_full_len + layer_ring_len * (seg - 1) + g[1] * seg_vert + g[0]) as usize;
        }
        let ring_index = if g[1] == 0 { g[0] }
            else if g[0] == seg { seg + g[1] }
            else if g[1] == seg { seg * 3 - g[0] }
            else { seg * 4 - g[1] };
        (layer_full_len + layer_ring_len * (g[2] - 1) + ring_index) as usize
    };
    let cube_co = |i: i32| (i * 2 - seg) as f32 / seg as f32;
    let mut wrap_table = vec![0.0f32; seg_vert as usize];
    for i in 0..=seg {
        let on_face = [true, i == 0 || i == seg, false];
        wrap_table[i as usize] =
            qs_co_from_cube(Vec3::new(1.0, cube_co(i), 0.0), on_face, method).y;
    }
    let sphere_co = |g: [i32; 3]| -> Vec3 {
        let axis_face = if g[0] == 0 || g[0] == seg { 0 }
            else if g[1] == 0 || g[1] == seg { 1 } else { 2 };
        let axis_a = (axis_face + 1) % 3;
        let axis_b = (axis_face + 2) % 3;
        let mut co = Vec3::ZERO;
        co[axis_face] = cube_co(g[axis_face]);
        co[axis_a] = wrap_table[g[axis_a] as usize];
        co[axis_b] = wrap_table[g[axis_b] as usize];
        co
    };
    let total = (layer_full_len * 2 + layer_ring_len * (seg - 1)) as usize;
    let mut varr: Vec<usize> = Vec::with_capacity(total);
    let mut vert_add = |bm: &mut BMesh, g: [i32; 3]| {
        let co = sphere_co(g).normalized() * rad;
        let co = mul_v3_m4v3(&mat, co);
        let vi = bm_vert_create(bm, co, None, 0);
        varr.push(vi);
    };
    for z in 0..=seg {
        if z == 0 || z == seg {
            for y in 0..=seg { for x in 0..=seg { vert_add(bm, [x, y, z]); } }
        } else {
            for x in 0..=seg { vert_add(bm, [x, 0, z]); }
            for y in 1..=seg { vert_add(bm, [seg, y, z]); }
            for x in (0..=seg - 1).rev() { vert_add(bm, [x, seg, z]); }
            for y in (1..=seg - 1).rev() { vert_add(bm, [0, y, z]); }
        }
    }
    let corner = |cv: usize| [((cv >> 2) & 1) as i32, ((cv >> 1) & 1) as i32, (cv & 1) as i32];
    for f_index in 0..6 {
        let c = corner(CUBE_FACES[f_index][0]);
        let cu = corner(CUBE_FACES[f_index][1]);
        let cv = corner(CUBE_FACES[f_index][3]);
        let step_u = [cu[0] - c[0], cu[1] - c[1], cu[2] - c[2]];
        let step_v = [cv[0] - c[0], cv[1] - c[1], cv[2] - c[2]];
        let g0 = [c[0] * seg, c[1] * seg, c[2] * seg];
        for v in 0..seg {
            for u in 0..seg {
                let g = [g0[0] + step_u[0] * u + step_v[0] * v,
                         g0[1] + step_u[1] * u + step_v[1] * v,
                         g0[2] + step_u[2] * u + step_v[2] * v];
                let flip = (u * 2 + 1 < seg) != (v * 2 + 1 < seg);
                let gsu = [g[0] + step_u[0], g[1] + step_u[1], g[2] + step_u[2]];
                let gsv = [g[0] + step_v[0], g[1] + step_v[1], g[2] + step_v[2]];
                let gsuv = [gsu[0] + step_v[0], gsu[1] + step_v[1], gsu[2] + step_v[2]];
                let vquad = if !flip {
                    [varr[vert_index(g)], varr[vert_index(gsu)],
                     varr[vert_index(gsuv)], varr[vert_index(gsv)]]
                } else {
                    [varr[vert_index(gsu)], varr[vert_index(gsuv)],
                     varr[vert_index(gsv)], varr[vert_index(g)]]
                };
                bm_face_create_verts(bm, &vquad, None, 0, true, true);
            }
        }
    }
}

/// `bmo_create_uvsphere_exec` — segment strip + per-seg extrude/rotate.
pub fn bmo_create_uvsphere_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let rad = bmo_slot_float_get(op, false, "radius");
    let seg = bmo_slot_int_get(op, false, "u_segments").max(1);
    let tot = bmo_slot_int_get(op, false, "v_segments").max(1);
    let mat = bmo_slot_mat4_get(op, false, "matrix");
    let phid = std::f32::consts::PI / tot as f32;
    /* one segment first (cross-section on YZ, x=0) */
    let mut preveve = BM_NIL;
    for a in 0..=tot {
        let (sin_phi, cos_phi) = sin_cos_from_fraction(a, 2 * tot);
        let vec = Vec3::new(0.0, rad * sin_phi, rad * cos_phi);
        let eve = bm_vert_create(bm, vec, None, 0);
        bm.vpool[eve].head.api_flag |= VERT_MARK;
        if a != 0 {
            let e = crate::core::bm_edge_create(bm, preveve, eve, None, 0);
            bm.epool[e].head.api_flag |= EDGE_ORIG;
        }
        preveve = eve;
    }
    /* extrude and rotate; negative phi for outward normals */
    let cmat = axis_angle_to_mat3(Vec3::Z, -(std::f32::consts::PI * 2.0) / seg as f32);
    let mut prevop: Option<BMOperator> = None;
    for a in 0..seg {
        let mut bmop = if a != 0 {
            let src = prevop.take().unwrap();
            let mut o = bmo_op_init(bm, &crate::opdefines::OPDEF_EXTRUDE_EDGE_ONLY, op.flag);
            let src_buf = o.slots_in.iter().position(|s| s.name == "edges")
                .map(|si| { let _ = si; src.slots_out.iter().find(|s| s.name == "geom.out")
                    .map(|s| s.buf.clone()) }).flatten();
            if let Some(buf) = src_buf {
                let si = o.slots_in.iter().position(|s| s.name == "edges").unwrap();
                o.slots_in[si].buf = buf;
            }
            bmo_op_finish(bm, src);
            bmo_op_exec(bm, &mut o, crate::bmo::bmo_extrude::bmo_extrude_edge_only_exec);
            o
        } else {
            let mut o = bmo_op_init(bm, &crate::opdefines::OPDEF_EXTRUDE_EDGE_ONLY, op.flag);
            let si = o.slots_in.iter().position(|s| s.name == "edges").unwrap();
            o.slots_in[si].buf =
                bmo_slot_buffer_from_enabled_hflag_temp(bm, 1 << BM_EDGE, EDGE_ORIG);
            bmo_op_exec(bm, &mut o, crate::bmo::bmo_extrude::bmo_extrude_edge_only_exec);
            o
        };
        bmo_slot_buffer_flag_enable(bm, &mut bmop, true, "geom.out", VERT_MARK);
        let geom_buf = bmop.slots_out.iter().find(|s| s.name == "geom.out")
            .map(|s| s.buf.clone()).unwrap_or_default();
        let mut rop = bmo_op_init(bm, &crate::opdefines::OPDEF_ROTATE, op.flag);
        if let Some(si) = rop.slots_in.iter().position(|s| s.name == "verts") {
            rop.slots_in[si].buf = geom_buf;
        }
        if let Some(si) = rop.slots_in.iter().position(|s| s.name == "cent") {
            rop.slots_in[si].vec = Vec3::ZERO;
        }
        if let Some(si) = rop.slots_in.iter().position(|s| s.name == "matrix") {
            rop.slots_in[si].mat4 = mat3_to_mat4(cmat);
        }
        bmo_op_exec(bm, &mut rop, crate::bmo::bmo_utils::bmo_rotate_exec);
        bmo_op_finish(bm, rop);
        prevop = Some(bmop);
    }
    if seg > 0 {
        if let Some(o) = prevop.take() { bmo_op_finish(bm, o); }
    }
    /* merge verts with remove_doubles */
    let len = 2.0 * rad * (phid / 2.0).sin();
    let vec = Vec3::new(rad * phid.sin(), 0.0, rad * phid.cos());
    let vec2 = mul_v3_m3v3(&cmat, vec);
    let len2 = (vec - vec2).length();
    let _ = bmo_op_callf(bm, op.flag, "remove_doubles",
        &[CallfArg::OFlag("verts".into(), 1 << BM_VERT, VERT_MARK),
          CallfArg::Float("dist".into(), len.min(len2) / 3.0)]);
    /* apply inverse matrix (created verts are in local space) */
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.vert(v).head.api_flag & VERT_MARK != 0 {
            let co = mul_v3_m4v3(&mat, bm.vert(v).co);
            bm.vpool[v].co = co;
        }
    }
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "verts.out", 1 << BM_VERT, VERT_MARK);
}

fn mat3_to_mat4(m: Mat3) -> Mat4 {
    Mat4 { cols: [
        Vec4::new(m.cols[0].x, m.cols[0].y, m.cols[0].z, 0.0),
        Vec4::new(m.cols[1].x, m.cols[1].y, m.cols[1].z, 0.0),
        Vec4::new(m.cols[2].x, m.cols[2].y, m.cols[2].z, 0.0),
        Vec4::new(0.0, 0.0, 0.0, 1.0)] }
}

fn bmo_slot_buffer_from_enabled_hflag_temp(bm: &mut BMesh, htype_mask: u8, oflag: u8)
    -> Vec<(u8, usize)>
{
    let mut out = Vec::new();
    if htype_mask & (1 << crate::BM_VERT) != 0 {
        for v in crate::iterators::VertsOfMesh::new(bm) {
            if bm.vert(v).head.api_flag & oflag != 0 { out.push((crate::BM_VERT, v)); }
        }
    }
    if htype_mask & (1 << crate::BM_EDGE) != 0 {
        for e in crate::iterators::EdgesOfMesh::new(bm) {
            if bm.edge(e).head.api_flag & oflag != 0 { out.push((crate::BM_EDGE, e)); }
        }
    }
    out
}

/// `bmo_create_icosphere_exec` — icosphere base + optional subdivide.
pub fn bmo_create_icosphere_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let rad = bmo_slot_float_get(op, false, "radius");
    let rad_div = rad / 200.0;
    let subdiv = bmo_slot_int_get(op, false, "subdivisions");
    let mat = bmo_slot_mat4_get(op, false, "matrix");
    let mut eva = [BM_NIL; 12];
    for a in 0..12 {
        let vec = Vec3::new(rad_div * ICOVERT[a][0], rad_div * ICOVERT[a][1],
                            rad_div * ICOVERT[a][2]);
        eva[a] = bm_vert_create(bm, vec, None, 0);
        bm.vpool[eva[a]].head.api_flag |= VERT_MARK;
    }
    for a in 0..20 {
        let (v1, v2, v3) = (eva[ICOFACE[a][0]], eva[ICOFACE[a][1]], eva[ICOFACE[a][2]]);
        let f = bm_face_create_quad_tri(bm, v1, v2, v3, BM_NIL, None, 0);
        let l_first = bm.face(f).l_first;
        let mut l = l_first;
        loop {
            let le = bm.loo(l).e; bm.epool[le].head.api_flag |= EDGE_MARK;
            l = bm.loo(l).next;
            if l == l_first { break; }
        }
    }
    if subdiv > 1 {
        /* subdivide_edges op — PORT_PENDING until ported; run if wired. */
        let mut bmop = bmo_op_init(bm, &crate::opdefines::OPDEF_SUBDIVIDE_EDGES, op.flag);
        if let Some(si) = bmop.slots_in.iter().position(|s| s.name == "edges") {
            bmop.slots_in[si].buf =
                bmo_slot_buffer_from_enabled_hflag_temp(bm, 1 << BM_EDGE, EDGE_MARK);
        }
        if let Some(si) = bmop.slots_in.iter().position(|s| s.name == "smooth") {
            bmop.slots_in[si].f = rad;
        }
        if let Some(si) = bmop.slots_in.iter().position(|s| s.name == "cuts") {
            bmop.slots_in[si].i = (1 << (subdiv - 1)) - 1;
        }
        if let Some(si) = bmop.slots_in.iter().position(|s| s.name == "use_grid_fill") {
            bmop.slots_in[si].b = true;
        }
        if let Some(si) = bmop.slots_in.iter().position(|s| s.name == "use_sphere") {
            bmop.slots_in[si].b = true;
        }
        bmo_op_exec(bm, &mut bmop, |bm, op| {
            if let Some(f) = crate::opdefines::OPDEF_SUBDIVIDE_EDGES.exec { f(bm, op) }
        });
        bmo_slot_buffer_flag_enable(bm, &mut bmop, true, "geom.out", VERT_MARK);
        bmo_op_finish(bm, bmop);
    }
    for v in crate::iterators::VertsOfMesh::new(bm).collect::<Vec<_>>() {
        if bm.vert(v).head.api_flag & VERT_MARK != 0 {
            let co = mul_v3_m4v3(&mat, bm.vert(v).co);
            bm.vpool[v].co = co;
        }
    }
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "verts.out", 1 << BM_VERT, VERT_MARK);
}

/// `bmo_create_monkey_exec` — mirrored monkey head from data tables.
pub fn bmo_create_monkey_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let mat = bmo_slot_mat4_get(op, false, "matrix");
    let mut tv = vec![BM_NIL; MONKEYNV * 2];
    for i in 0..MONKEYNV {
        let v = Vec3::new(
            (MONKEYV[i][0] as f32 + 127.0) / 128.0,
            MONKEYV[i][2] as f32 / -128.0,
            MONKEYV[i][1] as f32 / 128.0);
        tv[i] = bm_vert_create(bm, v, None, 0);
        bm.vpool[tv[i]].head.api_flag |= VERT_MARK;
        let vm = Vec3::new(-v.x, v.y, v.z);
        if vm.x.abs() < 0.001 {
            tv[MONKEYNV + i] = tv[i];
        } else {
            let eve = bm_vert_create(bm, vm, None, 0);
            let co = mul_v3_m4v3(&mat, bm.vert(eve).co);
            bm.vpool[eve].co = co;
            tv[MONKEYNV + i] = eve;
        }
        bm.vpool[tv[MONKEYNV + i]].head.api_flag |= VERT_MARK;
        let co = mul_v3_m4v3(&mat, bm.vert(tv[i]).co);
        bm.vpool[tv[i]].co = co;
    }
    for i in 0..MONKEYNF {
        let mv = MONKEYF[i];
        let f_a = bm_face_create_quad_tri(
            bm,
            tv[(mv[0] as i32 + i as i32 - MONKEYO) as usize],
            tv[(mv[1] as i32 + i as i32 - MONKEYO) as usize],
            tv[(mv[2] as i32 + i as i32 - MONKEYO) as usize],
            if mv[3] != mv[2] { tv[(mv[3] as i32 + i as i32 - MONKEYO) as usize] } else { BM_NIL },
            None, 0);
        let _ = f_a;
        bm_face_create_quad_tri(
            bm,
            tv[MONKEYNV + (mv[2] as i32 + i as i32 - MONKEYO) as usize],
            tv[MONKEYNV + (mv[1] as i32 + i as i32 - MONKEYO) as usize],
            tv[MONKEYNV + (mv[0] as i32 + i as i32 - MONKEYO) as usize],
            if mv[3] != mv[2] {
                tv[MONKEYNV + (mv[3] as i32 + i as i32 - MONKEYO) as usize]
            } else { BM_NIL },
            None, 0);
    }
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "verts.out", 1 << BM_VERT, VERT_MARK);
}

/// `bmo_create_circle_exec`.
pub fn bmo_create_circle_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let radius = bmo_slot_float_get(op, false, "radius");
    let segs = bmo_slot_int_get(op, false, "segments");
    let cap_ends = bmo_slot_bool_get(op, false, "cap_ends");
    let cap_tris = bmo_slot_bool_get(op, false, "cap_tris");
    if segs == 0 { return; }
    let mat = bmo_slot_mat4_get(op, false, "matrix");
    let mut cent1 = BM_NIL;
    if cap_ends {
        cent1 = bm_vert_create(bm, mul_v3_m4v3(&mat, Vec3::ZERO), None, 0);
        bm.vpool[cent1].head.api_flag |= VERT_MARK;
    }
    let mut lastv1 = BM_NIL;
    let mut firstv1 = BM_NIL;
    let mut v1 = BM_NIL;
    for a in 0..segs {
        let (s, c) = sin_cos_from_fraction(a, segs);
        let vec = mul_v3_m4v3(&mat, Vec3::new(-radius * s, radius * c, 0.0));
        v1 = bm_vert_create(bm, vec, None, 0);
        bm.vpool[v1].head.api_flag |= VERT_MARK;
        if lastv1 != BM_NIL {
            bm_edge_create(bm, v1, lastv1, None, 0);
        }
        if a != 0 && cap_ends {
            let f = bm_face_create_quad_tri(bm, cent1, lastv1, v1, BM_NIL, None, 0);
            bm.fpool[f].head.api_flag |= FACE_NEW;
        }
        if firstv1 == BM_NIL { firstv1 = v1; }
        lastv1 = v1;
    }
    bm_edge_create(bm, firstv1, lastv1, None, 0);
    if cap_ends {
        let f = bm_face_create_quad_tri(bm, cent1, v1, firstv1, BM_NIL, None, 0);
        bm.fpool[f].head.api_flag |= FACE_NEW;
    }
    if !cap_tris {
        bmo_op_callf(bm, op.flag, "dissolve_faces",
            &[CallfArg::OFlag("faces".into(), 1 << crate::BM_FACE, FACE_NEW)]);
    }
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "verts.out", 1 << BM_VERT, VERT_MARK);
}

/// `bmo_create_cone_exec`.
pub fn bmo_create_cone_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let rad1 = bmo_slot_float_get(op, false, "radius1");
    let rad2 = bmo_slot_float_get(op, false, "radius2");
    let depth_half = 0.5 * bmo_slot_float_get(op, false, "depth");
    let segs = bmo_slot_int_get(op, false, "segments");
    let cap_ends = bmo_slot_bool_get(op, false, "cap_ends");
    let cap_tris = bmo_slot_bool_get(op, false, "cap_tris");
    if segs == 0 { return; }
    let mat = bmo_slot_mat4_get(op, false, "matrix");
    let (mut cent1, mut cent2) = (BM_NIL, BM_NIL);
    if cap_ends {
        cent1 = bm_vert_create(bm, mul_v3_m4v3(&mat, Vec3::new(0.0, 0.0, -depth_half)), None, 0);
        cent2 = bm_vert_create(bm, mul_v3_m4v3(&mat, Vec3::new(0.0, 0.0, depth_half)), None, 0);
        bm.vpool[cent1].head.api_flag |= VERT_MARK;
        bm.vpool[cent2].head.api_flag |= VERT_MARK;
    }
    let side_faces_len = segs - 1;
    let mut side_faces: Vec<usize> = Vec::with_capacity(side_faces_len as usize);
    let (mut lastv1, mut lastv2) = (BM_NIL, BM_NIL);
    let (mut firstv1, mut firstv2) = (BM_NIL, BM_NIL);
    let (mut v1, mut v2) = (BM_NIL, BM_NIL);
    for i in 0..segs {
        let (sin_phi, cos_phi) = sin_cos_from_fraction(i, segs);
        v1 = bm_vert_create(bm,
            mul_v3_m4v3(&mat, Vec3::new(rad1 * sin_phi, rad1 * cos_phi, -depth_half)),
            None, 0);
        v2 = bm_vert_create(bm,
            mul_v3_m4v3(&mat, Vec3::new(rad2 * sin_phi, rad2 * cos_phi, depth_half)),
            None, 0);
        bm.vpool[v1].head.api_flag |= VERT_MARK;
        bm.vpool[v2].head.api_flag |= VERT_MARK;
        if i != 0 {
            if cap_ends {
                let f = bm_face_create_quad_tri(bm, cent1, lastv1, v1, BM_NIL, None, 0);
                bm.fpool[f].head.api_flag |= FACE_NEW;
                let f = bm_face_create_quad_tri(bm, cent2, v2, lastv2, BM_NIL, None, 0);
                bm.fpool[f].head.api_flag |= FACE_NEW;
            }
            let f = bm_face_create_quad_tri(bm, lastv1, lastv2, v2, v1, None, 0);
            side_faces.push(f);
        } else {
            firstv1 = v1;
            firstv2 = v2;
        }
        lastv1 = v1;
        lastv2 = v2;
    }
    if cap_ends {
        let f = bm_face_create_quad_tri(bm, cent1, v1, firstv1, BM_NIL, None, 0);
        bm.fpool[f].head.api_flag |= FACE_NEW;
        let f = bm_face_create_quad_tri(bm, cent2, firstv2, v2, BM_NIL, None, 0);
        bm.fpool[f].head.api_flag |= FACE_NEW;
    }
    let _ = bm_face_create_quad_tri(bm, v1, v2, firstv2, firstv1, None, 0);
    /* collapse tip verts */
    if rad1 == 0.0 {
        if cap_ends { bm_vert_kill(bm, cent1); }
        for &f in &side_faces {
            let l = bm.face(f).l_first;
            let lp = bm.loo(l).prev;
            crate::mods::bm_edge_collapse(bm, bm.loo(lp).e, bm.loo(lp).v, true, true);
        }
    }
    if rad2 == 0.0 {
        if cap_ends { bm_vert_kill(bm, cent2); }
        for &f in &side_faces {
            let l = bm.face(f).l_first;
            let ln = bm.loo(l).next;
            crate::mods::bm_edge_collapse(bm, bm.loo(ln).e, bm.loo(ln).v, true, true);
        }
    }
    if !cap_tris {
        bmo_op_callf(bm, op.flag, "dissolve_faces",
            &[CallfArg::OFlag("faces".into(), 1 << crate::BM_FACE, FACE_NEW)]);
    }
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "verts.out", 1 << BM_VERT, VERT_MARK);
}

/// `bmo_create_cube_exec`.
pub fn bmo_create_cube_exec(bm: &mut BMesh, op: &mut BMOperator) {
    let mut off = bmo_slot_float_get(op, false, "size") / 2.0;
    let mat = bmo_slot_mat4_get(op, false, "matrix");
    if off == 0.0 { off = 0.5; }
    let mut verts = [BM_NIL; 8];
    let mut i = 0;
    for x in [-1, 1] {
        for y in [-1, 1] {
            for z in [-1, 1] {
                let vec = mul_v3_m4v3(&mat,
                    Vec3::new(x as f32 * off, y as f32 * off, z as f32 * off));
                verts[i] = bm_vert_create(bm, vec, None, 0);
                bm.vpool[verts[i]].head.api_flag |= VERT_MARK;
                i += 1;
            }
        }
    }
    for i in 0..6 {
        let quad = [verts[CUBE_FACES[i][0]], verts[CUBE_FACES[i][1]],
                    verts[CUBE_FACES[i][2]], verts[CUBE_FACES[i][3]]];
        bm_face_create_verts(bm, &quad, None, 0, true, true);
    }
    bmo_slot_buffer_from_enabled_flag(bm, op, true, "verts.out", 1 << BM_VERT, VERT_MARK);
}
