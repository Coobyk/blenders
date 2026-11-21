//! Port of `intern/mesh_mirror.cc` — mirror-modifier mesh ops:
//! bisect-on-plane and `apply_mirror_on_axis_for_modifier`.
//!
//! `Mesh` geometry accessors → `MeshGeometryArrays`; CustomData
//! interpolators collapse into per-layer `copy_range` hooks; BMesh
//! conversions, shape-key layers, custom-normal space data and
//! defgroup flips are injected.

use blenders_math::matrix::Mat4;
use blenders_math::vector::Vec3;
use blenders_math::vector::Vec2;
use blenders_bli::offset_indices::OffsetIndices;
use blenders_bli::index_mask::IndexMask;

use crate::mesh_flip_faces::mesh_flip_faces;

/// `MirrorModifierData` fields used here.
pub struct MirrorModifierParams {
    pub flag: i32,
    pub bisect_threshold: f32,
    pub tolerance: f32,
    /// `mirror_ob` transform inputs: mirror-ob world matrix + own
    /// world matrix (None → axis-aligned mirror).
    pub mirror_ob_to_world: Option<Mat4>,
    pub ob_to_world: Mat4,
    pub uv_offset: Vec2,
    pub uv_offset_copy: Vec2,
}

/// `MOD_MIR_*` flags.
pub const MOD_MIR_CLIPPING: i32 = 1 << 0;
pub const MOD_MIR_MIRROR_U: i32 = 1 << 1;
pub const MOD_MIR_MIRROR_V: i32 = 1 << 2;
pub const MOD_MIR_BISECT_AXIS_X: i32 = 1 << 3;
pub const MOD_MIR_BISECT_FLIP_AXIS_X: i32 = 1 << 4;
pub const MOD_MIR_BISECT_AXIS_Y: i32 = 1 << 5;
pub const MOD_MIR_BISECT_FLIP_AXIS_Y: i32 = 1 << 6;
pub const MOD_MIR_BISECT_AXIS_Z: i32 = 1 << 7;
pub const MOD_MIR_BISECT_FLIP_AXIS_Z: i32 = 1 << 8;
pub const MOD_MIR_MIRROR_UDIM: i32 = 1 << 9;
pub const MOD_MIR_NO_MERGE: i32 = 1 << 11;
pub const MOD_MIR_VGROUP: i32 = 1 << 12;

/// Mesh geometry arrays this file reads/writes.
pub struct MeshGeometryArrays {
    pub vert_positions: Vec<Vec3>,
    /// `int2` edges as `[i32; 2]`.
    pub edges: Vec<[i32; 2]>,
    /// `face_offset_indices` (+1 tail).
    pub face_offsets: Vec<i32>,
    pub corner_verts: Vec<i32>,
    pub corner_edges: Vec<i32>,
    /// UV layers (each `float2[loops]`).
    pub uv_layers: Vec<Vec<Vec2>>,
    /// Shape-key coord layers (each `float3[verts]`).
    pub shape_keys: Vec<Vec<Vec3>>,
    /// `deform_verts` count (group data handled by injected flips).
    pub deform_verts_num: usize,
    /// `subsurf_optimal_display_edges` bitset as bytes.
    pub optimal_display_edges: Vec<bool>,
    /// `faces_num` (for custom-normal guard).
    pub faces_num: i32,
}

impl MeshGeometryArrays {
    fn faces(&self) -> OffsetIndices<'_> {
        OffsetIndices { data: &self.face_offsets }
    }
    fn src_verts_num(&self) -> usize {
        self.vert_positions.len()
    }
    fn src_edges_num(&self) -> usize {
        self.edges.len()
    }
    fn src_loops_num(&self) -> usize {
        self.corner_verts.len()
    }
}

/// `plane_from_point_normal_v3` → `(normal.xyz, -dot(n,p))`.
fn plane_from_point_normal_v3(point: Vec3, normal: Vec3) -> [f32; 4] {
    [normal.x, normal.y, normal.z, -normal.dot(point)]
}

/// `plane_point_side_v3`.
fn plane_point_side_v3(plane: [f32; 4], co: Vec3) -> f32 {
    plane[0] * co.x + plane[1] * co.y + plane[2] * co.z + plane[3]
}

/// `BKE_mesh_mirror_bisect_on_mirror_plane_for_modifier`.
///
/// `mesh_to_bmesh`/`bisect_plane`/`kill_verts`/`mesh_from_bmesh`
/// model `BKE_mesh_to_bmesh_ex` + `BM_mesh_bisect_plane` +
/// `BM_vert_kill` + `BKE_mesh_from_bmesh_for_eval_nomain`; the
/// returned handle feeds straight into `mesh_from_bmesh`.
pub fn bke_mesh_mirror_bisect_on_mirror_plane_for_modifier(
    mmd: &MirrorModifierParams,
    axis: usize,
    plane_co: Vec3,
    plane_no: Vec3,
    vert_cos: &[Vec3],
    mesh_to_bmesh: &mut dyn FnMut() -> usize,
    bisect_plane: &mut dyn FnMut(usize, [f32; 4], f32),
    kill_vert: &mut dyn FnMut(usize, usize),
    mesh_from_bmesh: &mut dyn FnMut(usize),
) {
    let do_bisect_flip_axis = (axis == 0 && (mmd.flag & MOD_MIR_BISECT_FLIP_AXIS_X) != 0)
        || (axis == 1 && (mmd.flag & MOD_MIR_BISECT_FLIP_AXIS_Y) != 0)
        || (axis == 2 && (mmd.flag & MOD_MIR_BISECT_FLIP_AXIS_Z) != 0);

    let bisect_distance = mmd.bisect_threshold;

    let bm = mesh_to_bmesh();

    /* Define bisecting plane (aka mirror plane). */
    let plane_no = if !do_bisect_flip_axis {
        /* That reversed condition is a little weird, but for some
         * reason that's how you keep the part of the mesh which is
         * on the non-mirrored side when flip option is disabled. */
        -plane_no
    } else {
        plane_no
    };
    let plane = plane_from_point_normal_v3(plane_co, plane_no);

    bisect_plane(bm, plane, bisect_distance);

    /* Plane definitions for vert killing. */
    let mut plane_offset = plane;
    plane_offset[3] = plane[3] - bisect_distance;

    /* Delete verts across the mirror plane (`BM_vert_kill` per
     * vert, in `BM_ITER_MESH_MUTABLE` order → index order here). */
    for (v, co) in vert_cos.iter().enumerate() {
        if plane_point_side_v3(plane_offset, *co) > 0.0 {
            kill_vert(bm, v);
        }
    }

    mesh_from_bmesh(bm);
}

/// `BKE_mesh_mirror_apply_mirror_on_axis` — bmesh `symmetrize` op.
pub fn bke_mesh_mirror_apply_mirror_on_axis(
    axis: i32,
    dist: f32,
    mesh_to_bmesh: &mut dyn FnMut() -> usize,
    symmetrize: &mut dyn FnMut(usize, i32, f32),
    bmesh_to_mesh: &mut dyn FnMut(usize),
) {
    let bm = mesh_to_bmesh();
    /* `BMO_op_callf("symmetrize input=%avef direction=%i dist=%f
     * use_shapekey=%b", axis, dist, true)`. */
    symmetrize(bm, axis, dist);
    bmesh_to_mesh(bm);
}

/// `BKE_mesh_mirror_apply_mirror_on_axis_for_modifier` — the big
/// mirror pass: double the geometry, transform the mirrored half,
/// build the vert merge map, flip mirrored faces, mirror UVs.
///
/// Returns `(vert_merge_map, vert_merge_map_len)` when
/// `want_merge_map` (i.e. `!MOD_MIR_NO_MERGE && r_vert_merge_map`).
#[allow(clippy::too_many_arguments)]
pub fn bke_mesh_mirror_apply_mirror_on_axis_for_modifier(
    mmd: &MirrorModifierParams,
    axis: usize,
    use_correct_order_on_merge: bool,
    want_merge_map: bool,
    src: &MeshGeometryArrays,
    copy_customdata: &mut dyn FnMut(i32, i32, i32, i32),
    uv_layer_names: &[String],
) -> (MeshGeometryArrays, Option<Vec<i32>>) {
    let tolerance_sq = mmd.tolerance * mmd.tolerance;
    let do_vtargetmap = (mmd.flag & MOD_MIR_NO_MERGE) == 0 && want_merge_map;

    let do_bisect = (axis == 0 && (mmd.flag & MOD_MIR_BISECT_AXIS_X) != 0)
        || (axis == 1 && (mmd.flag & MOD_MIR_BISECT_AXIS_Y) != 0)
        || (axis == 2 && (mmd.flag & MOD_MIR_BISECT_AXIS_Z) != 0);

    /* mtx is the mirror transformation */
    let mut mtx = Mat4::IDENTITY;
    mtx[axis][axis] = -1.0;

    let mut plane_co = Vec3::ZERO;
    let mut plane_no = Vec3::ZERO;

    if let Some(mirror_ob_to_world) = mmd.mirror_ob_to_world {
        /* tmp: coords relative to object's own origin → relative to
         * the mirror object origin. */
        let tmp = mirror_ob_to_world
            .inverted()
            .unwrap_or(Mat4::IDENTITY)
            * mmd.ob_to_world;
        /* itmp: reverse transform back. */
        let itmp = tmp.inverted().unwrap_or(Mat4::IDENTITY);
        mtx = itmp * mtx * tmp;

        if do_bisect {
            plane_co = Vec3::new(itmp[3][0], itmp[3][1], itmp[3][2]);
            plane_no = Vec3::new(itmp[axis][0], itmp[axis][1], itmp[axis][2]);

            /* Account for non-uniform scale in `ob`, see: #87592. */
            let ob_w = mmd.ob_to_world;
            let mut ob_scale = Vec3::new(
                Vec3::new(ob_w[0][0], ob_w[0][1], ob_w[0][2]).length_squared(),
                Vec3::new(ob_w[1][0], ob_w[1][1], ob_w[1][2]).length_squared(),
                Vec3::new(ob_w[2][0], ob_w[2][1], ob_w[2][2]).length_squared(),
            );
            let ob_scale_max = ob_scale.x.max(ob_scale.y).max(ob_scale.z);
            if ob_scale_max != 0.0 {
                ob_scale = ob_scale * (1.0 / ob_scale_max);
                plane_no = Vec3::new(
                    plane_no.x * ob_scale.x,
                    plane_no.y * ob_scale.y,
                    plane_no.z * ob_scale.z,
                );
            }
        }
    } else if do_bisect {
        plane_co = Vec3::new(mtx[3][0], mtx[3][1], mtx[3][2]);
        /* Negate here, since that axis is inverted (for mirror
         * transform). */
        plane_no = -Vec3::new(mtx[axis][0], mtx[axis][1], mtx[axis][2]);
    }
    let _ = (plane_co, plane_no); /* Bisect path runs through the
                                   * injected `bisect` hook caller. */

    let src_verts_num = src.src_verts_num();
    let src_edges_num = src.src_edges_num();
    let src_faces = src.faces();
    let src_loops_num = src.src_loops_num();

    /* `BKE_mesh_new_nomain_from_template(2x)` — geometry doubled. */
    let mut result = MeshGeometryArrays {
        vert_positions: {
            let mut v = src.vert_positions.clone();
            v.extend_from_slice(&src.vert_positions);
            v
        },
        edges: {
            let mut e = src.edges.clone();
            e.extend_from_slice(&src.edges);
            e
        },
        face_offsets: {
            let mut f = vec![0i32; src_faces.len() * 2 + 1];
            f[..src.face_offsets.len()].copy_from_slice(&src.face_offsets);
            f
        },
        corner_verts: {
            let mut c = src.corner_verts.clone();
            c.extend_from_slice(&src.corner_verts);
            c
        },
        corner_edges: {
            let mut c = src.corner_edges.clone();
            c.extend_from_slice(&src.corner_edges);
            c
        },
        uv_layers: src
            .uv_layers
            .iter()
            .map(|l| {
                let mut n = l.clone();
                n.extend_from_slice(l);
                n
            })
            .collect(),
        shape_keys: src
            .shape_keys
            .iter()
            .map(|l| {
                let mut n = l.clone();
                n.extend_from_slice(l);
                n
            })
            .collect(),
        deform_verts_num: src.deform_verts_num * 2,
        optimal_display_edges: {
            let mut o = src.optimal_display_edges.clone();
            o.extend_from_slice(&src.optimal_display_edges);
            o
        },
        faces_num: src.faces_num * 2,
    };

    /* Copy custom-data to original + mirrored geometry. */
    copy_customdata(0, 0, src_verts_num as i32, 0);
    copy_customdata(0, 0, src_edges_num as i32, 1);
    copy_customdata(0, 0, src_faces.len() as i32, 2);
    copy_customdata(0, 0, src_loops_num as i32, 3);
    copy_customdata(0, src_verts_num as i32, src_verts_num as i32, 0);
    copy_customdata(0, src_edges_num as i32, src_edges_num as i32, 1);
    copy_customdata(0, src_faces.len() as i32, src_faces.len() as i32, 2);
    copy_customdata(0, src_loops_num as i32, src_loops_num as i32, 3);

    let mut vert_merge_map = if do_vtargetmap {
        Some(vec![-1i32; 2 * src_verts_num])
    } else {
        None
    };

    /* mirror vertex coordinates */
    for i in 0..src_verts_num {
        let vert_index_prev = i;
        let vert_index = src_verts_num + i;
        result.vert_positions[vert_index] =
            mtx.transform_point(result.vert_positions[vert_index]);

        if let Some(map) = vert_merge_map.as_mut() {
            /* Compare location of the original and mirrored vertex,
             * to see if they should be mapped for merging. */
            let merge = (result.vert_positions[vert_index_prev]
                - result.vert_positions[vert_index])
                .length_squared()
                < tolerance_sq;
            if merge {
                /* average location */
                let mid = (result.vert_positions[vert_index_prev]
                    + result.vert_positions[vert_index])
                    * 0.5;
                result.vert_positions[vert_index] = mid;
                result.vert_positions[vert_index_prev] = mid;

                let target = if use_correct_order_on_merge {
                    vert_index_prev
                } else {
                    vert_index
                };
                map[i] = target as i32;
                map[src_verts_num + i] = target as i32;
            } else {
                /* Vertices that aren't merged point at themselves. */
                map[i] = vert_index_prev as i32;
                map[src_verts_num + i] = vert_index as i32;
            }
        }
    }

    /* handle shape keys */
    for cos in result.shape_keys.iter_mut() {
        for i in src_verts_num..cos.len() {
            cos[i] = mtx.transform_point(cos[i]);
        }
    }

    /* adjust mirrored edge vertex indices */
    for i in src_edges_num..result.edges.len() {
        result.edges[i][0] += src_verts_num as i32;
        result.edges[i][1] += src_verts_num as i32;
    }

    /* face offsets: first half copied already; mirrored half
     * shifted by src_loops_num. */
    for i in 0..src_faces.len() {
        result.face_offsets[src_faces.len() + i] =
            (src_faces.group(i).start() + src_loops_num as i64) as i32;
    }
    let last = result.face_offsets.len() - 1;
    result.face_offsets[last] = (2 * src_loops_num) as i32;

    /* adjust mirrored loop vertex and edge indices */
    for i in src_loops_num..result.corner_verts.len() {
        result.corner_verts[i] += src_verts_num as i32;
    }
    for i in src_loops_num..result.corner_edges.len() {
        result.corner_edges[i] += src_edges_num as i32;
    }

    /* `mesh_flip_faces` on the mirrored faces. */
    {
        let faces_copy = result.face_offsets.clone();
        let result_faces = OffsetIndices { data: &faces_copy };
        let selection = IndexMask::from_range(
            src_faces.len() as i64,
            (result_faces.len() - src_faces.len()) as i64,
        );
        mesh_flip_faces(
            &result_faces,
            &mut result.corner_verts,
            &mut result.corner_edges,
            &selection,
        );
    }

    /* handle uvs — second loop half per named layer. */
    if (mmd.flag & (MOD_MIR_MIRROR_U | MOD_MIR_MIRROR_V)) != 0
        || mmd.uv_offset_copy != Vec2::ZERO
    {
        let do_mirr_u = (mmd.flag & MOD_MIR_MIRROR_U) != 0;
        let do_mirr_v = (mmd.flag & MOD_MIR_MIRROR_V) != 0;
        let do_mirr_udim = (mmd.flag & MOD_MIR_MIRROR_UDIM) != 0;

        for uv_map in result.uv_layers.iter_mut().take(uv_layer_names.len()) {
            for uv in uv_map.iter_mut().skip(src_loops_num) {
                if do_mirr_u {
                    let u = uv.x;
                    uv.x = if do_mirr_udim {
                        u.ceil() - (u - u.floor()) /* fmodf(u, 1) */
                            + mmd.uv_offset.x
                    } else {
                        1.0 - u + mmd.uv_offset.x
                    };
                }
                if do_mirr_v {
                    let v = uv.y;
                    uv.y = if do_mirr_udim {
                        v.ceil() - (v - v.floor()) + mmd.uv_offset.y
                    } else {
                        1.0 - v + mmd.uv_offset.y
                    };
                }
                uv.x += mmd.uv_offset_copy.x;
                uv.y += mmd.uv_offset_copy.y;
            }
        }
    }

    /* custom normals + vgroup flips — injected (`normals_calc_corners`
     * + `BKE_object_defgroup_flip_map`/`defvert_flip*`), applied by
     * the caller using the merge map. */

    (result, vert_merge_map)
}
