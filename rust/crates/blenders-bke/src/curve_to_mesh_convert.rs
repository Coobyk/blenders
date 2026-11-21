//! Port of `intern/curve_to_mesh_convert.cc` — `curve_to_mesh_sweep`
//! core: `fill_mesh_topology`, `mark_bezier_vector_edges_sharp`,
//! `build_point_matrix`(+miter scale), `fill_mesh_positions`,
//! `calculate_result_offsets`, `calculate_offsets`.
//! Attribute propagation injected via closures.

use blenders_math::matrix::Mat4;
use blenders_math::vector::{Vec3, Vec4};

/// `curves::segments_num`.
pub fn segments_num(points_num: i32, cyclic: bool) -> i32 {
    if points_num == 0 {
        return 0;
    }
    if cyclic {
        points_num
    } else {
        points_num - 1
    }
}

/// `segments_num_no_duplicate_edge`.
pub fn segments_num_no_duplicate_edge(points_num: i32, cyclic: bool) -> i32 {
    if points_num == 0 {
        return 0;
    }
    if points_num <= 2 {
        return segments_num(points_num, false);
    }
    segments_num(points_num, cyclic)
}

/// `has_caps`.
pub fn has_caps(main_cyclic: bool, profile_cyclic: bool, profile_segment_num: i32) -> bool {
    !main_cyclic && profile_cyclic && profile_segment_num > 2
}

/// `fill_mesh_topology` — write edges/corner_verts/corner_edges/
/// face_offsets for one (main, profile) pair. All output arrays are
/// indexed from 0; the `*_offset` params shift into them.
#[allow(clippy::too_many_arguments)]
pub fn fill_mesh_topology(
    vert_offset: usize,
    edge_offset: usize,
    face_offset: usize,
    loop_offset: usize,
    main_point_num: usize,
    profile_point_num: usize,
    main_cyclic: bool,
    profile_cyclic: bool,
    fill_caps: bool,
    edges: &mut [[usize; 2]],
    corner_verts: &mut [usize],
    corner_edges: &mut [usize],
    face_offsets: &mut [usize],
) {
    let main_segment_num = segments_num_no_duplicate_edge(main_point_num as i32, main_cyclic);
    let profile_segment_num = segments_num(profile_point_num as i32, profile_cyclic);

    if profile_point_num == 1 {
        for i in 0..main_point_num.saturating_sub(1) {
            edges[edge_offset + i] = [vert_offset + i, vert_offset + i + 1];
        }
        if main_cyclic && main_segment_num > 2 {
            edges[edge_offset + main_segment_num as usize - 1] =
                [vert_offset + main_point_num - 1, vert_offset];
        }
        return;
    }

    let main_edges_start = edge_offset;
    for i_profile in 0..profile_point_num {
        let profile_edge_offset = main_edges_start + i_profile * main_segment_num as usize;
        for i_ring in 0..main_segment_num as usize {
            let i_next_ring = if i_ring == main_point_num - 1 { 0 } else { i_ring + 1 };
            let ring_vert_offset = vert_offset + profile_point_num * i_ring;
            let next_ring_vert_offset = vert_offset + profile_point_num * i_next_ring;
            edges[profile_edge_offset + i_ring] =
                [ring_vert_offset + i_profile, next_ring_vert_offset + i_profile];
        }
    }

    let profile_edges_start =
        main_edges_start + profile_point_num * main_segment_num as usize;
    for i_ring in 0..main_point_num {
        let ring_vert_offset = vert_offset + profile_point_num * i_ring;
        let ring_edge_offset = profile_edges_start + i_ring * profile_segment_num as usize;
        for i_profile in 0..profile_segment_num as usize {
            let i_next_profile =
                if i_profile == profile_point_num - 1 { 0 } else { i_profile + 1 };
            edges[ring_edge_offset + i_profile] =
                [ring_vert_offset + i_profile, ring_vert_offset + i_next_profile];
        }
    }

    for i_ring in 0..main_segment_num as usize {
        let i_next_ring = if i_ring == main_point_num - 1 { 0 } else { i_ring + 1 };
        let ring_vert_offset = vert_offset + profile_point_num * i_ring;
        let next_ring_vert_offset = vert_offset + profile_point_num * i_next_ring;
        let ring_edge_start = profile_edges_start + profile_segment_num as usize * i_ring;
        let next_ring_edge_offset =
            profile_edges_start + profile_segment_num as usize * i_next_ring;
        let ring_face_offset = face_offset + i_ring * profile_segment_num as usize;
        let ring_loop_offset = loop_offset + i_ring * profile_segment_num as usize * 4;

        for i_profile in 0..profile_segment_num as usize {
            let ring_segment_loop_offset = ring_loop_offset + i_profile * 4;
            let i_next_profile =
                if i_profile == profile_point_num - 1 { 0 } else { i_profile + 1 };
            let main_edge_start = main_edges_start + main_segment_num as usize * i_profile;
            let next_main_edge_start =
                main_edges_start + main_segment_num as usize * i_next_profile;

            face_offsets[ring_face_offset + i_profile] = ring_segment_loop_offset;

            corner_verts[ring_segment_loop_offset] = ring_vert_offset + i_profile;
            corner_edges[ring_segment_loop_offset] = ring_edge_start + i_profile;

            corner_verts[ring_segment_loop_offset + 1] = ring_vert_offset + i_next_profile;
            corner_edges[ring_segment_loop_offset + 1] = next_main_edge_start + i_ring;

            corner_verts[ring_segment_loop_offset + 2] =
                next_ring_vert_offset + i_next_profile;
            corner_edges[ring_segment_loop_offset + 2] = next_ring_edge_offset + i_profile;

            corner_verts[ring_segment_loop_offset + 3] = next_ring_vert_offset + i_profile;
            corner_edges[ring_segment_loop_offset + 3] = main_edge_start + i_ring;
        }
    }

    if fill_caps && has_caps(main_cyclic, profile_cyclic, profile_segment_num) {
        let face_num = main_segment_num as usize * profile_segment_num as usize;
        let cap_loop_offset = loop_offset + face_num * 4;
        let cap_face_offset = face_offset + face_num;

        face_offsets[cap_face_offset] = cap_loop_offset;
        face_offsets[cap_face_offset + 1] = cap_loop_offset + profile_segment_num as usize;

        let last_ring_index = main_point_num - 1;
        let last_ring_vert_offset = vert_offset + profile_point_num * last_ring_index;
        let last_ring_edge_offset =
            profile_edges_start + profile_segment_num as usize * last_ring_index;

        for i in 0..profile_segment_num as usize {
            let i_inv = profile_segment_num as usize - i - 1;
            corner_verts[cap_loop_offset + i] = vert_offset + i_inv;
            corner_edges[cap_loop_offset + i] = profile_edges_start
                + if i == profile_segment_num as usize - 1 {
                    profile_segment_num as usize - 1
                } else {
                    i_inv - 1
                };
            corner_verts[cap_loop_offset + profile_segment_num as usize + i] =
                last_ring_vert_offset + i;
            corner_edges[cap_loop_offset + profile_segment_num as usize + i] =
                last_ring_edge_offset + i;
        }
    }
}

/// `mark_bezier_vector_edges_sharp`.
pub fn mark_bezier_vector_edges_sharp(
    profile_point_num: usize,
    main_segment_num: usize,
    control_point_offsets: &[i32],
    handle_types_left: &[i8],
    handle_types_right: &[i8],
    sharp_edges: &mut [bool],
) {
    let main_edges_start = 0;
    if crate::curve_bezier::point_is_sharp(handle_types_left, handle_types_right, 0) {
        for e in &mut sharp_edges[main_edges_start..main_edges_start + main_segment_num] {
            *e = true;
        }
    }
    for i in 1..profile_point_num {
        if crate::curve_bezier::point_is_sharp(handle_types_left, handle_types_right, i) {
            let offset = main_edges_start
                + main_segment_num * control_point_offsets[i] as usize;
            for e in &mut sharp_edges[offset..offset + main_segment_num] {
                *e = true;
            }
        }
    }
}

/// `build_point_matrix`.
pub fn build_point_matrix(location: Vec3, tangent: Vec3, normal: Vec3) -> Mat4 {
    let x = tangent;
    let y = normal.cross(tangent).normalized();
    let z = normal;
    Mat4::from_cols(
        Vec4::new(x.x, x.y, x.z, 0.0),
        Vec4::new(y.x, y.y, y.z, 0.0),
        Vec4::new(z.x, z.y, z.z, 0.0),
        Vec4::new(location.x, location.y, location.z, 1.0),
    )
}

/// `build_point_matrix_miter_scale`.
pub fn build_point_matrix_miter_scale(
    location_prev: Vec3,
    location: Vec3,
    location_next: Vec3,
    tangent: Vec3,
    normal: Vec3,
    miter_scale_limit: f32,
) -> Mat4 {
    let din = location - location_prev;
    let dout = location_next - location;
    let length_in = din.length();
    let length_out = dout.length();
    if length_in < 1e-10 || length_out < 1e-10 {
        return build_point_matrix(location, tangent, normal);
    }
    let dir_in = din / length_in;
    let dir_out = dout / length_out;
    let cos_turn = dir_in.dot(dir_out);
    if cos_turn >= 1.0 || cos_turn == -1.0 {
        return build_point_matrix(location, tangent, normal);
    }
    let mut matrix = build_point_matrix(location, tangent, normal);
    let scale = (2.0f32 / (1.0 + cos_turn)).sqrt().min(miter_scale_limit);
    let bisector = (dir_out - dir_in).normalized();
    let x = Vec3::new(matrix[0][0], matrix[0][1], matrix[0][2]);
    let y = Vec3::new(matrix[1][0], matrix[1][1], matrix[1][2]);
    let new_x = x + bisector * ((scale - 1.0) * x.dot(bisector));
    let new_y = y + bisector * ((scale - 1.0) * y.dot(bisector));
    matrix[0] = Vec4::new(new_x.x, new_x.y, new_x.z, 0.0);
    matrix[1] = Vec4::new(new_y.x, new_y.y, new_y.z, 0.0);
    matrix
}

/// `build_point_matrix` dispatching variant (ring index).
#[allow(clippy::too_many_arguments)]
pub fn build_point_matrix_ring(
    main_positions: &[Vec3],
    tangents: &[Vec3],
    normals: &[Vec3],
    scales: &[f32],
    main_cyclic: bool,
    miter_scale_limit: Option<f32>,
    i_ring: usize,
) -> Mat4 {
    let is_end = !main_cyclic && (i_ring == 0 || i_ring == main_positions.len() - 1);
    let mut point_matrix = if miter_scale_limit.is_none() || is_end {
        build_point_matrix(main_positions[i_ring], normals[i_ring], tangents[i_ring])
    } else {
        let i_prev = if i_ring == 0 { main_positions.len() - 1 } else { i_ring - 1 };
        let i_next = if i_ring == main_positions.len() - 1 { 0 } else { i_ring + 1 };
        build_point_matrix_miter_scale(
            main_positions[i_prev],
            main_positions[i_ring],
            main_positions[i_next],
            normals[i_ring],
            tangents[i_ring],
            miter_scale_limit.unwrap(),
        )
    };
    if !scales.is_empty() {
        let s = scales[i_ring];
        for c in 0..3 {
            point_matrix[c] = point_matrix[c] * s;
        }
    }
    point_matrix
}

/// `fill_mesh_positions`.
#[allow(clippy::too_many_arguments)]
pub fn fill_mesh_positions(
    main_positions: &[Vec3],
    profile_positions: &[Vec3],
    tangents: &[Vec3],
    normals: &[Vec3],
    scales: &[f32],
    main_cyclic: bool,
    miter_scale_limit: Option<f32>,
    mesh_positions: &mut [Vec3],
) {
    if profile_positions.len() == 1 {
        for i_ring in 0..main_positions.len() {
            let m = build_point_matrix_ring(
                main_positions,
                tangents,
                normals,
                scales,
                main_cyclic,
                miter_scale_limit,
                i_ring,
            );
            mesh_positions[i_ring] = m.transform_point(profile_positions[0]);
        }
    } else {
        for i_ring in 0..main_positions.len() {
            let m = build_point_matrix_ring(
                main_positions,
                tangents,
                normals,
                scales,
                main_cyclic,
                miter_scale_limit,
                i_ring,
            );
            let ring_vert_start = i_ring * profile_positions.len();
            for (i_profile, p) in profile_positions.iter().enumerate() {
                mesh_positions[ring_vert_start + i_profile] = m.transform_point(*p);
            }
        }
    }
}

/// `ResultOffsets` — per (main,profile)-mesh result counts.
#[derive(Default, Clone)]
pub struct ResultOffsets {
    /// total result meshes.
    pub total: usize,
    /// vert/edge/loop/face prefix offsets.
    pub vert: Vec<i32>,
    /// See `vert`.
    pub edge: Vec<i32>,
    /// See `vert`.
    pub lp: Vec<i32>,
    /// See `vert`.
    pub face: Vec<i32>,
    /// main/profile curve index per result.
    pub main_indices: Vec<i32>,
    /// See `main_indices`.
    pub profile_indices: Vec<i32>,
    /// any single-point curves.
    pub any_single_point_main: bool,
    /// See `any_single_point_main`.
    pub any_single_point_profile: bool,
}

/// `calculate_result_offsets` — serial port of the parallel_invoke
/// two-phase count-then-pair enumeration.
#[allow(clippy::too_many_arguments)]
pub fn calculate_result_offsets(
    main_curve_points: &[usize],
    profile_curve_points: &[usize],
    main_cyclic: &[bool],
    profile_cyclic: &[bool],
    fill_caps: bool,
) -> ResultOffsets {
    let mut result = ResultOffsets {
        total: main_curve_points.len() * profile_curve_points.len(),
        ..Default::default()
    };
    result.vert.resize(result.total + 1, 0);
    result.edge.resize(result.total + 1, 0);
    result.lp.resize(result.total + 1, 0);
    result.face.resize(result.total + 1, 0);

    let mut mesh_index = 0usize;
    let mut vert_offset = 0i32;
    let mut edge_offset = 0i32;
    let mut loop_offset = 0i32;
    let mut face_offset = 0i32;
    for (i_main, &main_point_num_u) in main_curve_points.iter().enumerate() {
        let main_point_num = main_point_num_u as i32;
        let mc = main_cyclic[i_main];
        let main_segment_num = segments_num_no_duplicate_edge(main_point_num, mc);
        for (i_profile, &profile_point_num_u) in profile_curve_points.iter().enumerate() {
            result.vert[mesh_index] = vert_offset;
            result.edge[mesh_index] = edge_offset;
            result.lp[mesh_index] = loop_offset;
            result.face[mesh_index] = face_offset;

            let profile_point_num = profile_point_num_u as i32;
            let pc = profile_cyclic[i_profile];
            if profile_point_num != 0 {
                let profile_segment_num = segments_num(profile_point_num, pc);
                let caps = fill_caps && has_caps(mc, pc, profile_segment_num);
                let tube_face_num = main_segment_num * profile_segment_num;

                if profile_point_num == 1 {
                    vert_offset += main_point_num;
                    edge_offset += main_segment_num;
                } else {
                    vert_offset += main_point_num * profile_point_num;
                    edge_offset += main_segment_num * profile_point_num
                        + profile_segment_num * main_point_num;
                    loop_offset += tube_face_num * 4;
                    face_offset += tube_face_num;
                    if caps {
                        loop_offset += profile_segment_num * 2;
                        face_offset += 2;
                    }
                }
            }
            result.main_indices.push(i_main as i32);
            result.profile_indices.push(i_profile as i32);
            mesh_index += 1;
        }
        if main_point_num == 1 {
            result.any_single_point_main = true;
        }
    }
    result.vert[result.total] = vert_offset;
    result.edge[result.total] = edge_offset;
    result.lp[result.total] = loop_offset;
    result.face[result.total] = face_offset;
    result.any_single_point_profile =
        profile_curve_points.iter().any(|&n| n == 1);
    result
}

/// `CombinationInfo` for `foreach_curve_combination`.
pub struct CombinationInfo {
    /// Range start offsets.
    pub vert_start: usize,
    /// See `vert_start`.
    pub edge_start: usize,
    /// See `vert_start`.
    pub loop_start: usize,
    /// See `vert_start`.
    pub face_start: usize,
    /// Curve indices.
    pub main_index: usize,
    /// See `main_index`.
    pub profile_index: usize,
    /// Cyclic flags.
    pub main_cyclic: bool,
    /// See `main_cyclic`.
    pub profile_cyclic: bool,
    /// Segment counts.
    pub main_segment_num: i32,
    /// See `main_segment_num`.
    pub profile_segment_num: i32,
}

/// `foreach_curve_combination` — iterate (main,profile) pairs.
pub fn foreach_curve_combination(
    main_curve_points: &[usize],
    profile_curve_points: &[usize],
    main_cyclic: &[bool],
    profile_cyclic: &[bool],
    offsets: &ResultOffsets,
    mut f: impl FnMut(&CombinationInfo),
) {
    let mut mesh_index = 0usize;
    for (i_main, &mp) in main_curve_points.iter().enumerate() {
        for (i_profile, &pp) in profile_curve_points.iter().enumerate() {
            if pp == 0 {
                mesh_index += 1;
                continue;
            }
            let info = CombinationInfo {
                vert_start: offsets.vert[mesh_index] as usize,
                edge_start: offsets.edge[mesh_index] as usize,
                loop_start: offsets.lp[mesh_index] as usize,
                face_start: offsets.face[mesh_index] as usize,
                main_index: i_main,
                profile_index: i_profile,
                main_cyclic: main_cyclic[i_main],
                profile_cyclic: profile_cyclic[i_profile],
                main_segment_num: segments_num_no_duplicate_edge(mp as i32, main_cyclic[i_main]),
                profile_segment_num: segments_num(pp as i32, profile_cyclic[i_profile]),
            };
            f(&info);
            mesh_index += 1;
        }
    }
}

/// `curve_to_mesh_sweep` — mesh topology driver. Takes per-curve
/// point counts + evaluated positions/tangents/normals arrays per
/// (main, profile) curve; outputs edges/corner/face_offsets/positions.
/// Curve data access via flat slices: `main_curve_points` (pts/curve),
/// `main_eval_positions` flattened, etc.
#[allow(clippy::too_many_arguments)]
pub fn curve_to_mesh_sweep(
    main_curve_points: &[usize],
    profile_curve_points: &[usize],
    main_cyclic: &[bool],
    profile_cyclic: &[bool],
    main_eval_positions: &[Vec<Vec3>],
    main_eval_tangents: &[Vec<Vec3>],
    main_eval_normals: &[Vec<Vec3>],
    profile_eval_positions: &[Vec<Vec3>],
    scales: &[f32],
    fill_caps: bool,
    miter_limit_angle: Option<f32>,
) -> Option<(Vec<[usize; 2]>, Vec<usize>, Vec<usize>, Vec<usize>, Vec<Vec3>, Vec<bool>)> {
    let miter_scale_limit = miter_limit_angle.map(|a| {
        if a >= std::f32::consts::PI {
            f32::MAX
        } else {
            1.0 / (a / 2.0).cos()
        }
    });
    let offsets = calculate_result_offsets(
        main_curve_points,
        profile_curve_points,
        main_cyclic,
        profile_cyclic,
        fill_caps,
    );
    let total_verts = *offsets.vert.last().unwrap() as usize;
    if total_verts == 0 {
        return None;
    }
    let total_edges = *offsets.edge.last().unwrap() as usize;
    let total_faces = *offsets.face.last().unwrap() as usize;
    let total_loops = *offsets.lp.last().unwrap() as usize;

    let mut edges = vec![[0usize; 2]; total_edges];
    let mut face_offsets = vec![0usize; total_faces];
    let mut corner_verts = vec![0usize; total_loops];
    let mut corner_edges = vec![0usize; total_loops];
    let mut positions = vec![Vec3::ZERO; total_verts];
    let mut sharp_faces = vec![false; total_faces];

    foreach_curve_combination(
        main_curve_points,
        profile_curve_points,
        main_cyclic,
        profile_cyclic,
        &offsets,
        |info| {
            fill_mesh_topology(
                info.vert_start,
                info.edge_start,
                info.face_start,
                info.loop_start,
                main_curve_points[info.main_index],
                profile_curve_points[info.profile_index],
                info.main_cyclic,
                info.profile_cyclic,
                fill_caps,
                &mut edges,
                &mut corner_verts,
                &mut corner_edges,
                &mut face_offsets,
            );
            if fill_caps
                && has_caps(info.main_cyclic, info.profile_cyclic, info.profile_segment_num)
            {
                let face_num = (info.main_segment_num * info.profile_segment_num) as usize;
                let cap_face_offset = info.face_start + face_num;
                if cap_face_offset < sharp_faces.len() {
                    sharp_faces[cap_face_offset] = true;
                }
                if cap_face_offset + 1 < sharp_faces.len() {
                    sharp_faces[cap_face_offset + 1] = true;
                }
            }
            let mp = main_eval_positions[info.main_index].as_slice();
            let tp = main_eval_tangents[info.main_index].as_slice();
            let np = main_eval_normals[info.main_index].as_slice();
            let pp = profile_eval_positions[info.profile_index].as_slice();
            let vert_start = info.vert_start;
            if pp.len() == 1 {
                for i_ring in 0..mp.len() {
                    let m = build_point_matrix_ring(
                        mp,
                        tp,
                        np,
                        scales,
                        info.main_cyclic,
                        miter_scale_limit,
                        i_ring,
                    );
                    positions[vert_start + i_ring] = m.transform_point(pp[0]);
                }
            } else {
                for i_ring in 0..mp.len() {
                    let m = build_point_matrix_ring(
                        mp,
                        tp,
                        np,
                        scales,
                        info.main_cyclic,
                        miter_scale_limit,
                        i_ring,
                    );
                    let ring_vert_start = vert_start + i_ring * pp.len();
                    for (i_profile, p) in pp.iter().enumerate() {
                        positions[ring_vert_start + i_profile] = m.transform_point(*p);
                    }
                }
            }
        },
    );

    Some((edges, face_offsets, corner_verts, corner_edges, positions, sharp_faces))
}
