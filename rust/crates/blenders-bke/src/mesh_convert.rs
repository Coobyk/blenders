//! Port of `intern/mesh_convert.cc` — the curve-displist→mesh and
//! mesh→nurb-list conversion algorithms, verbatim index arithmetic.
//! Object-level orchestration (`BKE_mesh_to_curve`,
//! `BKE_mesh_new_from_object`, …) is injected via handles.

/// `DispList::type` values.
pub const DL_SEGM: i32 = 0;
/// `DL_POLY`.
pub const DL_POLY: i32 = 1;
/// `DL_SURF`.
pub const DL_SURF: i32 = 2;
/// `DL_INDEX3`.
pub const DL_INDEX3: i32 = 3;
/// `DL_INDEX4` (unused here).
pub const DL_INDEX4: i32 = 4;
/// `dl.flag` bits.
pub const DL_CYCL_U: i16 = 1;
/// `DL_CYCL_V`.
pub const DL_CYCL_V: i16 = 2;
/// `CU_SMOOTH` (`dl.rt` bit).
pub const CU_SMOOTH: i16 = 1;
/// `CU_NURB_ENDPOINT`.
pub const CU_NURB_ENDPOINT: u8 = 1 << 0;
/// `CU_NURB_CYCLIC`.
pub const CU_NURB_CYCLIC: u8 = 1 << 2;

/// `DispList`.
#[derive(Clone)]
pub struct DispList {
    /// `type`.
    pub dl_type: i32,
    /// `parts`.
    pub parts: i32,
    /// `nr`.
    pub nr: i32,
    /// `flag`.
    pub flag: i16,
    /// `rt`.
    pub rt: i16,
    /// `col`.
    pub col: i32,
    /// `verts` (xyz per vert).
    pub verts: Vec<f32>,
    /// `index` (for DL_INDEX3/4).
    pub index: Vec<i32>,
}

/// Output mesh of `mesh_nurbs_displist_to_mesh`.
#[derive(Default)]
pub struct ConvertedMesh {
    /// Vert positions.
    pub positions: Vec<[f32; 3]>,
    /// Edges (v1,v2).
    pub edges: Vec<[i32; 2]>,
    /// Face start offsets into `corner_verts`.
    pub face_offsets: Vec<i32>,
    /// Corner verts.
    pub corner_verts: Vec<i32>,
    /// Per-face `material_index`.
    pub material_indices: Vec<i32>,
    /// Per-face `sharp_face`.
    pub sharp_faces: Vec<bool>,
    /// Per-corner active UV map ("UVMap").
    pub uv_map: Vec<[f32; 2]>,
}

/// `mesh_nurbs_displist_to_mesh` — `conv_polys` = `!CU_DO_2DFILL(cu)
/// || ob_type == OB_SURF` (the caller resolves the flag).
pub fn mesh_nurbs_displist_to_mesh(
    dispbase: &[DispList],
    conv_polys: bool,
) -> ConvertedMesh {
    let mut totvert = 0;
    let mut totedge = 0;
    let mut faces_num = 0;
    let mut totloop = 0;
    for dl in dispbase {
        if dl.dl_type == DL_SEGM {
            totvert += dl.parts * dl.nr;
            totedge += dl.parts * (dl.nr - 1);
        } else if dl.dl_type == DL_POLY {
            if conv_polys {
                totvert += dl.parts * dl.nr;
                totedge += dl.parts * dl.nr;
            }
        } else if dl.dl_type == DL_SURF {
            if dl.parts != 0 {
                totvert += dl.parts * dl.nr;
                let tot = (i32::from(dl.flag & DL_CYCL_U != 0) + dl.nr - 1)
                    * (i32::from(dl.flag & DL_CYCL_V != 0) + dl.parts - 1);
                faces_num += tot;
                totloop += tot * 4;
            }
        } else if dl.dl_type == DL_INDEX3 {
            totvert += dl.nr;
            let tot = dl.parts;
            faces_num += tot;
            totloop += tot * 3;
        }
    }

    let mut mesh = ConvertedMesh {
        positions: vec![[0.0; 3]; totvert.max(0) as usize],
        edges: vec![[0; 2]; totedge.max(0) as usize],
        face_offsets: vec![0; faces_num.max(0) as usize],
        corner_verts: vec![0; totloop.max(0) as usize],
        material_indices: vec![0; faces_num.max(0) as usize],
        sharp_faces: vec![false; faces_num.max(0) as usize],
        uv_map: vec![[0.0; 2]; totloop.max(0) as usize],
    };
    if totvert == 0 {
        return mesh;
    }

    let mut dst_vert = 0usize;
    let mut dst_edge = 0usize;
    let mut dst_poly = 0usize;
    let mut dst_loop = 0usize;
    for dl in dispbase {
        let is_smooth = dl.rt & CU_SMOOTH != 0;
        if dl.dl_type == DL_SEGM {
            let startvert = dst_vert as i32;
            for i in 0..(dl.parts * dl.nr) as usize {
                mesh.positions[dst_vert] =
                    [dl.verts[i * 3], dl.verts[i * 3 + 1], dl.verts[i * 3 + 2]];
                dst_vert += 1;
            }
            for a in 0..dl.parts {
                let ofs = a * dl.nr;
                for b in 1..dl.nr {
                    mesh.edges[dst_edge] = [startvert + ofs + b - 1, startvert + ofs + b];
                    dst_edge += 1;
                }
            }
        } else if dl.dl_type == DL_POLY {
            if conv_polys {
                let startvert = dst_vert as i32;
                for i in 0..(dl.parts * dl.nr) as usize {
                    mesh.positions[dst_vert] =
                        [dl.verts[i * 3], dl.verts[i * 3 + 1], dl.verts[i * 3 + 2]];
                    dst_vert += 1;
                }
                for a in 0..dl.parts {
                    let ofs = a * dl.nr;
                    for b in 0..dl.nr {
                        mesh.edges[dst_edge] = [
                            startvert + ofs + b,
                            if b == dl.nr - 1 {
                                startvert + ofs
                            } else {
                                startvert + ofs + b + 1
                            },
                        ];
                        dst_edge += 1;
                    }
                }
            }
        } else if dl.dl_type == DL_INDEX3 {
            let startvert = dst_vert as i32;
            for i in 0..dl.nr as usize {
                mesh.positions[dst_vert] =
                    [dl.verts[i * 3], dl.verts[i * 3 + 1], dl.verts[i * 3 + 2]];
                dst_vert += 1;
            }
            for a in 0..dl.parts as usize {
                let index = &dl.index[a * 3..a * 3 + 3];
                mesh.corner_verts[dst_loop] = startvert + index[0];
                mesh.corner_verts[dst_loop + 1] = startvert + index[2];
                mesh.corner_verts[dst_loop + 2] = startvert + index[1];
                mesh.face_offsets[dst_poly] = dst_loop as i32;
                mesh.material_indices[dst_poly] = dl.col;
                for i in 0..3 {
                    mesh.uv_map[dst_loop + i][0] =
                        (mesh.corner_verts[dst_loop + i] - startvert) as f32
                            / (dl.nr - 1) as f32;
                    mesh.uv_map[dst_loop + i][1] = 0.0;
                }
                mesh.sharp_faces[dst_poly] = !is_smooth;
                dst_poly += 1;
                dst_loop += 3;
            }
        } else if dl.dl_type == DL_SURF {
            let startvert = dst_vert as i32;
            for i in 0..(dl.parts * dl.nr) as usize {
                mesh.positions[dst_vert] =
                    [dl.verts[i * 3], dl.verts[i * 3 + 1], dl.verts[i * 3 + 2]];
                dst_vert += 1;
            }
            for a in 0..dl.parts {
                if (dl.flag & DL_CYCL_V) == 0 && a == dl.parts - 1 {
                    break;
                }
                let (mut p1, mut p2, mut p3, mut p4, mut b);
                if dl.flag & DL_CYCL_U != 0 {
                    p1 = startvert + dl.nr * a;
                    p2 = p1 + dl.nr - 1;
                    p3 = p1 + dl.nr;
                    p4 = p2 + dl.nr;
                    b = 0;
                } else {
                    p2 = startvert + dl.nr * a;
                    p1 = p2 + 1;
                    p4 = p2 + dl.nr;
                    p3 = p1 + dl.nr;
                    b = 1;
                }
                if (dl.flag & DL_CYCL_V) != 0 && a == dl.parts - 1 {
                    p3 -= dl.parts * dl.nr;
                    p4 -= dl.parts * dl.nr;
                }
                while b < dl.nr {
                    mesh.corner_verts[dst_loop] = p1;
                    mesh.corner_verts[dst_loop + 1] = p3;
                    mesh.corner_verts[dst_loop + 2] = p4;
                    mesh.corner_verts[dst_loop + 3] = p2;
                    mesh.face_offsets[dst_poly] = dst_loop as i32;
                    mesh.material_indices[dst_poly] = dl.col;

                    let mut orco_sizeu = dl.nr - 1;
                    let mut orco_sizev = dl.parts - 1;
                    if dl.flag & DL_CYCL_U != 0 {
                        orco_sizeu += 1;
                        if dl.flag & DL_CYCL_V != 0 {
                            orco_sizev += 1;
                        }
                    } else if dl.flag & DL_CYCL_V != 0 {
                        orco_sizev += 1;
                    }
                    for i in 0..4 {
                        let v = mesh.corner_verts[dst_loop + i] - startvert;
                        mesh.uv_map[dst_loop + i][0] =
                            (v / dl.nr) as f32 / orco_sizev as f32;
                        mesh.uv_map[dst_loop + i][1] =
                            (v % dl.nr) as f32 / orco_sizeu as f32;
                        if (i == 1 || i == 2) && mesh.uv_map[dst_loop + i][0] == 0.0 {
                            mesh.uv_map[dst_loop + i][0] = 1.0;
                        }
                        if (i == 0 || i == 1) && mesh.uv_map[dst_loop + i][1] == 0.0 {
                            mesh.uv_map[dst_loop + i][1] = 1.0;
                        }
                    }
                    mesh.sharp_faces[dst_poly] = !is_smooth;
                    dst_poly += 1;
                    dst_loop += 4;

                    p4 = p3;
                    p3 += 1;
                    p2 = p1;
                    p1 += 1;
                    b += 1;
                }
            }
        }
    }
    /* `mesh_calc_edges(mesh, true, false)` when faces exist — caller
     * regenerates loose edges. */
    mesh
}

/// `BPoint` fields filled by `BKE_mesh_to_curve_nurblist`.
#[derive(Clone)]
pub struct BPointOut {
    /// `vec` (co xyz + w).
    pub vec: [f32; 4],
    /// `f1` = SELECT.
    pub f1: u8,
    /// `radius`.
    pub radius: f32,
    /// `weight`.
    pub weight: f32,
}
/// `SELECT`.
pub const SELECT: u8 = 1;

/// `Nurb` fields filled by `BKE_mesh_to_curve_nurblist`.
pub struct NurbOut {
    /// `pntsu`.
    pub pntsu: i32,
    /// `pntsv`.
    pub pntsv: i32,
    /// `orderu`.
    pub orderu: i16,
    /// `flagu`.
    pub flagu: u8,
    /// `resolu`.
    pub resolu: i16,
    /// `bp`.
    pub bp: Vec<BPointOut>,
}

/// `BKE_mesh_to_curve_nurblist` — edges with exactly `edge_users_test`
/// face-users are chained into nurb polylines. `face_offsets` are
/// face start offsets; `corner_edges` the corner-edge table.
pub fn bke_mesh_to_curve_nurblist(
    positions: &[[f32; 3]],
    mesh_edges: &[[i32; 2]],
    face_offsets: &[i32],
    corner_edges: &[i32],
    edge_users_test: i32,
) -> Vec<NurbOut> {
    let mut edge_users = vec![0i32; mesh_edges.len()];
    for i in 0..face_offsets.len() {
        let start = face_offsets[i] as usize;
        let end = if i + 1 < face_offsets.len() {
            face_offsets[i + 1] as usize
        } else {
            corner_edges.len()
        };
        for &e in &corner_edges[start..end] {
            edge_users[e as usize] += 1;
        }
    }
    let mut edges: Vec<i32> = (0..mesh_edges.len() as i32)
        .filter(|&i| edge_users[i as usize] == edge_users_test)
        .collect();

    let mut nurbs = Vec::new();
    while !edges.is_empty() {
        let mut polyline: Vec<i32> = Vec::new();
        let mut closed = false;
        let edge_current = mesh_edges[*edges.last().unwrap() as usize];
        let mut start_vert = edge_current[0];
        let mut end_vert = edge_current[1];
        let mut faces_num = 0;
        polyline.push(start_vert);
        faces_num += 1;
        polyline.push(end_vert);
        faces_num += 1;
        edges.pop();

        let mut ok = true;
        while ok {
            ok = false;
            let mut j = edges.len();
            while j > 0 {
                j -= 1;
                let edge = mesh_edges[edges[j] as usize];
                if edge[0] == end_vert {
                    end_vert = edge[1];
                    polyline.push(end_vert);
                    faces_num += 1;
                    edges.remove(j);
                    ok = true;
                } else if edge[1] == end_vert {
                    end_vert = edge[0];
                    polyline.push(end_vert);
                    faces_num += 1;
                    edges.remove(j);
                    ok = true;
                } else if edge[0] == start_vert {
                    start_vert = edge[1];
                    polyline.insert(0, start_vert);
                    faces_num += 1;
                    edges.remove(j);
                    ok = true;
                } else if edge[1] == start_vert {
                    start_vert = edge[0];
                    polyline.insert(0, start_vert);
                    faces_num += 1;
                    edges.remove(j);
                    ok = true;
                }
            }
        }

        if start_vert == end_vert {
            polyline.pop();
            faces_num -= 1;
            closed = true;
        }

        let mut nu = NurbOut {
            pntsu: faces_num,
            pntsv: 1,
            orderu: 4,
            flagu: CU_NURB_ENDPOINT | if closed { CU_NURB_CYCLIC } else { 0 },
            resolu: 12,
            bp: Vec::with_capacity(faces_num as usize),
        };
        for &idx in polyline.iter().take(faces_num as usize) {
            let p = positions[idx as usize];
            nu.bp.push(BPointOut {
                vec: [p[0], p[1], p[2], 1.0],
                f1: SELECT,
                radius: 1.0,
                weight: 1.0,
            });
        }
        nurbs.push(nu);
    }
    nurbs
}
