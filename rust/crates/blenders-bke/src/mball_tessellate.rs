//! Port of `intern/mball_tessellate.cc` — the Bloomenthal
//! polygonizer: `init_meta`, `build_bvh_spatial`, `densfunc`,
//! `metaball` BVH query, `setcorner`/`vertid`/`converge`,
//! `makecubetable`, `docube`, `find_first_points`, `polygonize`.
//! Linked lists → arenas + `HashMap`s; the cube table is a
//! `OnceLock<Vec>`.

use std::collections::HashMap;
use std::sync::OnceLock;

use blenders_math::matrix::Mat4;
use blenders_math::vector::Vec3;

/// `eMetaElem_Type`.
pub const MB_BALL: i32 = 0;
/// `MB_TUBEX` (deprecated).
pub const MB_TUBEX: i32 = 1;
/// `MB_TUBEY` (deprecated).
pub const MB_TUBEY: i32 = 2;
/// `MB_TUBEZ` (deprecated).
pub const MB_TUBEZ: i32 = 3;
/// `MB_TUBE`.
pub const MB_TUBE: i32 = 4;
/// `MB_PLANE`.
pub const MB_PLANE: i32 = 5;
/// `MB_ELLIPSOID`.
pub const MB_ELLIPSOID: i32 = 6;
/// `MB_CUBE`.
pub const MB_CUBE: i32 = 7;
/// `MB_NEGATIVE`.
pub const MB_NEGATIVE: i16 = 1 << 1;
/// `MB_HIDE`.
pub const MB_HIDE: i16 = 1 << 3;
/// `MB_UPDATE_*`.
pub const MB_UPDATE_ALWAYS: i32 = 0;
/// See above.
pub const MB_UPDATE_HALFRES: i32 = 1;
/// See above.
pub const MB_UPDATE_FAST: i32 = 2;
/// See above.
pub const MB_UPDATE_NEVER: i32 = 3;
/// `MBALL_ARRAY_LEN_INIT`.
pub const MBALL_ARRAY_LEN_INIT: usize = 512;

const L: i32 = 0;
const R: i32 = 1;
const B: i32 = 2;
const T: i32 = 3;
const N: i32 = 4;
const F: i32 = 5;
const LBN: i32 = 0;
const LBF: i32 = 1;
const LTN: i32 = 2;
const LTF: i32 = 3;
const RBN: i32 = 4;
const RBF: i32 = 5;
const RTN: i32 = 6;
const RTF: i32 = 7;

const LB: usize = 0;
const LT: usize = 1;
const LN: usize = 2;
const LF: usize = 3;
const RB: usize = 4;
const RT: usize = 5;
const RN: usize = 6;
const RF: usize = 7;
const BN: usize = 8;
const BF: usize = 9;
const TN: usize = 10;
const TF: usize = 11;

const CORNER1: [i32; 12] = [LBN, LTN, LBN, LBF, RBN, RTN, RBN, RBF, LBN, LBF, LTN, LTF];
const CORNER2: [i32; 12] = [LBF, LTF, LTN, LTF, RBF, RTF, RTN, RTF, RBN, RBF, RTN, RTF];
const LEFTFACE: [i32; 12] = [B, L, L, F, R, T, N, R, N, B, T, F];
const RIGHTFACE: [i32; 12] = [L, T, N, L, B, R, R, F, B, F, N, T];

fn mb_bit(i: i32, bit: i32) -> i32 {
    (i >> bit) & 1
}

/// `MetaElem` — the polygonizer's copy of a meta-element.
#[derive(Clone)]
pub struct MetaElem {
    /// `type`.
    pub ml_type: i32,
    /// `flag`.
    pub flag: i16,
    /// `x/y/z` (local position).
    pub pos: Vec3,
    /// `quat`.
    pub quat: [f32; 4],
    /// `expx/expy/expz` (primitive radius factors).
    pub expx: f32,
    /// See `expx`.
    pub expy: f32,
    /// See `expx`.
    pub expz: f32,
    /// `rad`.
    pub rad: f32,
    /// `rad2` = 1/rad².
    pub rad2: f32,
    /// `s` (stiffness).
    pub s: f32,
    /// `len`.
    pub len: f32,
    /// `mat` (object-space transform).
    pub mat: Mat4,
    /// `imat` (inverse).
    pub imat: Mat4,
    /// `bb` — world-space AABB stored as 8 corners (0=min, 6=max).
    pub bb: [[f32; 3]; 8],
}

impl MetaElem {
    /// `init_meta` per-element setup: clamp `s`, sign for negative
    /// elements, expand radii by type, build transformed AABB.
    /// `mat`/`imat` filled by the caller (ob⁻¹·bob·pos·rot).
    pub fn new_transformed(
        ml_type: i32,
        flag: i16,
        s: f32,
        rad: f32,
        expx: f32,
        expy: f32,
        expz: f32,
        len: f32,
        pos: Vec3,
        quat: [f32; 4],
        mat: Mat4,
        imat: Mat4,
    ) -> Self {
        let mut new_ml = MetaElem {
            ml_type,
            flag,
            pos,
            quat,
            expx,
            expy,
            expz,
            rad,
            rad2: 1.0 / (rad * rad),
            s: if s > 10.0 { 10.0 } else { s },
            len,
            mat,
            imat,
            bb: [[0.0; 3]; 8],
        };
        if new_ml.flag & MB_NEGATIVE != 0 {
            new_ml.s = -new_ml.s;
        }
        let mut expx = rad;
        let mut expy = rad;
        let mut expz = rad;
        match new_ml.ml_type {
            MB_BALL | MB_TUBEX | MB_TUBEY | MB_TUBEZ => {}
            MB_CUBE => {
                expz += new_ml.expz;
                expy += new_ml.expy;
                expx += new_ml.expx;
            }
            MB_PLANE => {
                expy += new_ml.expy;
                expx += new_ml.expx;
            }
            MB_TUBE => {
                expx += new_ml.expx;
            }
            MB_ELLIPSOID => {
                expx *= new_ml.expx;
                expy *= new_ml.expy;
                expz *= new_ml.expz;
            }
            _ => {}
        }
        new_ml.bb[0] = [-expx, -expy, -expz];
        new_ml.bb[1] = [expx, -expy, -expz];
        new_ml.bb[2] = [expx, expy, -expz];
        new_ml.bb[3] = [-expx, expy, -expz];
        new_ml.bb[4] = [-expx, -expy, expz];
        new_ml.bb[5] = [expx, -expy, expz];
        new_ml.bb[6] = [expx, expy, expz];
        new_ml.bb[7] = [-expx, expy, expz];
        let mut tempmin = [f32::MAX; 3];
        let mut tempmax = [f32::MIN; 3];
        for i in 0..8 {
            let c = new_ml.mat.transform_point(Vec3::from(new_ml.bb[i]));
            new_ml.bb[i] = [c.x, c.y, c.z];
            for a in 0..3 {
                tempmin[a] = tempmin[a].min(new_ml.bb[i][a]);
                tempmax[a] = tempmax[a].max(new_ml.bb[i][a]);
            }
        }
        new_ml.bb[0] = tempmin;
        new_ml.bb[6] = tempmax;
        new_ml
    }
}

/// `Box`.
#[derive(Clone, Default)]
pub struct BoxB {
    /// min/max.
    pub min: [f32; 3],
    /// See `min`.
    pub max: [f32; 3],
    /// leaf meta-elem index.
    pub ml: Option<usize>,
}

/// `make_box_union` (min = a->vec[0], max = a->vec[6]).
pub fn make_box_union(a_bb: &[[f32; 3]; 8], b: &BoxB) -> BoxB {
    BoxB {
        min: [
            a_bb[0][0].min(b.min[0]),
            a_bb[0][1].min(b.min[1]),
            a_bb[0][2].min(b.min[2]),
        ],
        max: [
            a_bb[6][0].max(b.max[0]),
            a_bb[6][1].max(b.max[1]),
            a_bb[6][2].max(b.max[2]),
        ],
        ml: b.ml,
    }
}

/// `make_box_from_metaelem`.
pub fn make_box_from_metaelem(ml_index: usize, ml: &MetaElem) -> BoxB {
    BoxB {
        min: ml.bb[0],
        max: ml.bb[6],
        ml: Some(ml_index),
    }
}

/// `partition_mainb`.
pub fn partition_mainb(
    mainb: &mut [MetaElem],
    start: usize,
    end: usize,
    s: usize,
    div: f32,
) -> usize {
    let mut i = start;
    let mut j = end - 1;
    let div = div * 2.0;
    loop {
        while i < j && div > (mainb[i].bb[6][s] + mainb[i].bb[0][s]) {
            i += 1;
        }
        while j > i && div < (mainb[j].bb[6][s] + mainb[j].bb[0][s]) {
            j -= 1;
        }
        if i >= j {
            break;
        }
        mainb.swap(i, j);
        i += 1;
        j -= 1;
    }
    if i == start {
        i += 1;
    }
    i
}

/// `MetaballBVHNode` — 2-child binary node over `mainb`.
#[derive(Default)]
pub struct MetaballBVHNode {
    /// `bb[2]`.
    pub bb: [BoxB; 2],
    /// `child[2]` — node indices or None (leaf → bb.ml set).
    pub child: [Option<usize>; 2],
}
impl Clone for MetaballBVHNode {
    fn clone(&self) -> Self {
        Self {
            bb: self.bb.clone(),
            child: self.child,
        }
    }
}

/// `build_bvh_spatial` — returns (root_index, nodes arena, queue size).
pub fn build_bvh_spatial(
    mainb: &mut [MetaElem],
    allbb: &BoxB,
) -> (Vec<MetaballBVHNode>, usize) {
    let mut nodes: Vec<MetaballBVHNode> = Vec::new();
    nodes.push(MetaballBVHNode::default());
    let mut queue_size = 1usize;
    fn rec(
        mainb: &mut [MetaElem],
        nodes: &mut Vec<MetaballBVHNode>,
        node_idx: usize,
        start: usize,
        end: usize,
        allbox: &BoxB,
        queue_size: &mut usize,
    ) {
        *queue_size += 1;
        let dim = [
            allbox.max[0] - allbox.min[0],
            allbox.max[1] - allbox.min[1],
            allbox.max[2] - allbox.min[2],
        ];
        let mut s = 0;
        if dim[1] > dim[0] && dim[1] > dim[2] {
            s = 1;
        } else if dim[2] > dim[1] && dim[2] > dim[0] {
            s = 2;
        }
        let div = allbox.min[s] + (dim[s] / 2.0);
        let part = partition_mainb(mainb, start, end, s, div);

        nodes[node_idx].bb[0] = make_box_from_metaelem(start, &mainb[start]);
        nodes[node_idx].child[0] = None;
        if part > start + 1 {
            for j in start..part {
                nodes[node_idx].bb[0] =
                    make_box_union(&mainb[j].bb, &nodes[node_idx].bb[0]);
            }
            let child_idx = nodes.len();
            nodes.push(MetaballBVHNode::default());
            nodes[node_idx].child[0] = Some(child_idx);
            let bb0 = nodes[node_idx].bb[0].clone();
            rec(mainb, nodes, child_idx, start, part, &bb0, queue_size);
        }

        nodes[node_idx].child[1] = None;
        if part < end {
            nodes[node_idx].bb[1] = make_box_from_metaelem(part, &mainb[part]);
            if part < end - 1 {
                for j in part..end {
                    nodes[node_idx].bb[1] =
                        make_box_union(&mainb[j].bb, &nodes[node_idx].bb[1]);
                }
                let child_idx = nodes.len();
                nodes.push(MetaballBVHNode::default());
                nodes[node_idx].child[1] = Some(child_idx);
                let bb1 = nodes[node_idx].bb[1].clone();
                rec(mainb, nodes, child_idx, part, end, &bb1, queue_size);
            }
        } else {
            nodes[node_idx].bb[1] = BoxB {
                min: [f32::MAX; 3],
                max: [f32::MIN; 3],
                ml: None,
            };
        }
    }
    rec(mainb, &mut nodes, 0, 0, mainb.len(), allbb, &mut queue_size);
    (nodes, queue_size)
}

/// `densfunc` — meta-elem density at world point (x,y,z).
pub fn densfunc(ball: &MetaElem, x: f32, y: f32, z: f32) -> f32 {
    let mut dvec = [x, y, z];
    /* `mul_m4_v3(imat, dvec)` — transform point. */
    let v = ball.imat.transform_point(Vec3::new(dvec[0], dvec[1], dvec[2]));
    dvec = [v.x, v.y, v.z];

    match ball.ml_type {
        MB_BALL => {}
        MB_CUBE => {
            if dvec[2] > ball.expz {
                dvec[2] -= ball.expz;
            } else if dvec[2] < -ball.expz {
                dvec[2] += ball.expz;
            } else {
                dvec[2] = 0.0;
            }
            if dvec[1] > ball.expy {
                dvec[1] -= ball.expy;
            } else if dvec[1] < -ball.expy {
                dvec[1] += ball.expy;
            } else {
                dvec[1] = 0.0;
            }
            if dvec[0] > ball.expx {
                dvec[0] -= ball.expx;
            } else if dvec[0] < -ball.expx {
                dvec[0] += ball.expx;
            } else {
                dvec[0] = 0.0;
            }
        }
        MB_PLANE => {
            if dvec[1] > ball.expy {
                dvec[1] -= ball.expy;
            } else if dvec[1] < -ball.expy {
                dvec[1] += ball.expy;
            } else {
                dvec[1] = 0.0;
            }
            if dvec[0] > ball.expx {
                dvec[0] -= ball.expx;
            } else if dvec[0] < -ball.expx {
                dvec[0] += ball.expx;
            } else {
                dvec[0] = 0.0;
            }
        }
        MB_TUBE => {
            if dvec[0] > ball.expx {
                dvec[0] -= ball.expx;
            } else if dvec[0] < -ball.expx {
                dvec[0] += ball.expx;
            } else {
                dvec[0] = 0.0;
            }
        }
        MB_ELLIPSOID => {
            dvec[0] /= ball.expx;
            dvec[1] /= ball.expy;
            dvec[2] /= ball.expz;
        }
        MB_TUBEX => {
            if dvec[0] > ball.len {
                dvec[0] -= ball.len;
            } else if dvec[0] < -ball.len {
                dvec[0] += ball.len;
            } else {
                dvec[0] = 0.0;
            }
        }
        MB_TUBEY => {
            if dvec[1] > ball.len {
                dvec[1] -= ball.len;
            } else if dvec[1] < -ball.len {
                dvec[1] += ball.len;
            } else {
                dvec[1] = 0.0;
            }
        }
        MB_TUBEZ => {
            if dvec[2] > ball.len {
                dvec[2] -= ball.len;
            } else if dvec[2] < -ball.len {
                dvec[2] += ball.len;
            } else {
                dvec[2] = 0.0;
            }
        }
        _ => {}
    }

    let dist2 = 1.0 - (dvec[0] * dvec[0] + dvec[1] * dvec[1] + dvec[2] * dvec[2]) * ball.rad2;
    if dist2 < 0.0 {
        0.0
    } else {
        ball.s * dist2 * dist2 * dist2
    }
}

/// `CORNER`.
#[derive(Clone)]
pub struct Corner {
    /// `i`,`j`,`k`.
    pub i: i32,
    /// See `i`.
    pub j: i32,
    /// See `i`.
    pub k: i32,
    /// `co`.
    pub co: Vec3,
    /// `value`.
    pub value: f32,
}

/// The polygonizer process state (index-arena version of `PROCESS`).
#[derive(Default)]
pub struct Process {
    /// `thresh`.
    pub thresh: f32,
    /// `size`.
    pub size: f32,
    /// `delta`.
    pub delta: f32,
    /// `converge_res`.
    pub converge_res: u32,
    /// `mainb`.
    pub mainb: Vec<MetaElem>,
    /// `allbb`.
    pub allbb: Option<BoxB>,
    /// BVH nodes (root = 0).
    pub bvh_nodes: Vec<MetaballBVHNode>,
    /// `centers` hash → dedup set.
    pub centers: HashMap<(i32, i32, i32), ()>,
    /// `corners` hash → corner arena.
    pub corners: HashMap<(i32, i32, i32), usize>,
    /// corner arena.
    pub corner_list: Vec<Corner>,
    /// `edges` hash → (sorted corner pair) → vert id.
    pub edges: HashMap<(i32, i32, i32, i32, i32, i32), i32>,
    /// `indices` — faces (i1,i2,i3,i4; i3==i4 → tri).
    pub indices: Vec<[i32; 4]>,
    /// `co`.
    pub co: Vec<Vec3>,
    /// `no`.
    pub no: Vec<Vec3>,
    /// pending cubes (stack).
    pub cubes: Vec<Cube>,
}

/// `CUBE`.
pub struct Cube {
    /// `i`,`j`,`k`.
    pub i: i32,
    /// See `i`.
    pub j: i32,
    /// See `i`.
    pub k: i32,
    /// `corners[8]` → corner arena index.
    pub corners: [usize; 8],
}

impl Process {
    /// `metaball` — BVH traversal density.
    pub fn metaball(&self, x: f32, y: f32, z: f32) -> f32 {
        let mut dens = 0.0f32;
        let mut stack = vec![0usize];
        while let Some(node_idx) = stack.pop() {
            let node = &self.bvh_nodes[node_idx];
            for i in 0..2 {
                let bb = &node.bb[i];
                if bb.min[0] <= x
                    && bb.max[0] >= x
                    && bb.min[1] <= y
                    && bb.max[1] >= y
                    && bb.min[2] <= z
                    && bb.max[2] >= z
                {
                    if let Some(child) = node.child[i] {
                        stack.push(child);
                    } else if let Some(ml) = bb.ml {
                        dens += densfunc(&self.mainb[ml], x, y, z);
                    }
                }
            }
        }
        self.thresh - dens
    }

    /// `setcenter` — insert into dedup set, true if already present.
    pub fn setcenter(&mut self, i: i32, j: i32, k: i32) -> bool {
        !self.centers.insert((i, j, k), ()).is_none()
    }

    /// `setcorner` — get-or-create the corner at (i,j,k).
    pub fn setcorner(&mut self, i: i32, j: i32, k: i32) -> usize {
        if let Some(&idx) = self.corners.get(&(i, j, k)) {
            return idx;
        }
        let co = Vec3::new(
            (i as f32 - 0.5) * self.size,
            (j as f32 - 0.5) * self.size,
            (k as f32 - 0.5) * self.size,
        );
        let value = self.metaball(co.x, co.y, co.z);
        let idx = self.corner_list.len();
        self.corner_list.push(Corner { i, j, k, co, value });
        self.corners.insert((i, j, k), idx);
        idx
    }

    /// `setedge`.
    fn setedge(&mut self, c1: &Corner, c2: &Corner, vid: i32) {
        let (mut a, mut b) = ((c1.i, c1.j, c1.k), (c2.i, c2.j, c2.k));
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        self.edges.insert((a.0, a.1, a.2, b.0, b.1, b.2), vid);
    }

    /// `getedge`.
    fn getedge(&self, c1: &Corner, c2: &Corner) -> i32 {
        let (mut a, mut b) = ((c1.i, c1.j, c1.k), (c2.i, c2.j, c2.k));
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        self.edges.get(&(a.0, a.1, a.2, b.0, b.1, b.2)).copied().unwrap_or(-1)
    }

    /// `add_cube`.
    pub fn add_cube(&mut self, i: i32, j: i32, k: i32) {
        if !self.setcenter(i, j, k) {
            let mut corners = [0usize; 8];
            for n in 0..8 {
                corners[n] = self.setcorner(
                    i + mb_bit(n as i32, 2),
                    j + mb_bit(n as i32, 1),
                    k + mb_bit(n as i32, 0),
                );
            }
            self.cubes.push(Cube { i, j, k, corners });
        }
    }

    /// `vnormal` — forward-difference normal (`USE_ACCUM_NORMAL` off
    /// in the port; `make_face` accumulates normals verbatim below).
    fn vnormal(&self, point: Vec3) -> Vec3 {
        let delta = self.delta;
        let f = self.metaball(point.x, point.y, point.z);
        Vec3::new(
            self.metaball(point.x + delta, point.y, point.z) - f,
            self.metaball(point.x, point.y + delta, point.z) - f,
            self.metaball(point.x, point.y, point.z + delta) - f,
        )
    }

    /// `converge` — bisection between two corners.
    fn converge(&self, c1: &Corner, c2: &Corner) -> Vec3 {
        let (mut c1_value, mut c1_co, mut c2_value, mut c2_co);
        if c1.value < c2.value {
            c1_value = c2.value;
            c1_co = c2.co;
            c2_value = c1.value;
            c2_co = c1.co;
        } else {
            c1_value = c1.value;
            c1_co = c1.co;
            c2_value = c2.value;
            c2_co = c2.co;
        }
        let mut r_p = Vec3::ZERO;
        for _ in 0..self.converge_res {
            r_p = c1_co + (c2_co - c1_co) * 0.5;
            let dens = self.metaball(r_p.x, r_p.y, r_p.z);
            if dens > 0.0 {
                c1_value = dens;
                c1_co = r_p;
            } else {
                c2_value = dens;
                c2_co = r_p;
            }
        }
        let tmp = -c1_value / (c2_value - c1_value);
        c1_co + (c2_co - c1_co) * tmp
    }

    /// `vertid` — shared vertex id for the edge (c1,c2).
    fn vertid(&mut self, c1_idx: usize, c2_idx: usize) -> i32 {
        let c1 = self.corner_list[c1_idx].clone();
        let c2 = self.corner_list[c2_idx].clone();
        let vid = self.getedge(&c1, &c2);
        if vid != -1 {
            return vid;
        }
        let v = self.converge(&c1, &c2);
        let no = self.vnormal(v);
        self.co.push(v);
        self.no.push(no);
        let vid = self.co.len() as i32 - 1;
        self.setedge(&c1, &c2, vid);
        vid
    }

    /// `make_face` (+ `USE_ACCUM_NORMAL` variant accumulation —
    /// `accumulate_vertex_normals_v3` injected by caller in the
    /// non-accum port normals come from `vnormal`).
    pub fn make_face(&mut self, i1: i32, i2: i32, i3: i32, i4: i32) {
        self.indices.push([i1, i2, i3, i4]);
        /* accumulate quad/tri normal into `no` */
        let (a, b, c, d) = (
            self.co[i1 as usize],
            self.co[i2 as usize],
            self.co[i3 as usize],
            self.co[i4 as usize],
        );
        let n = if i4 == i3 {
            (b - a).cross(c - a)
        } else {
            (c - a).cross(d - b)
        };
        let idx = [i1, i2, i3, i4];
        for (k, &vi) in idx.iter().enumerate() {
            if k == 3 && i4 == i3 {
                break;
            }
            self.no[vi as usize] = self.no[vi as usize] + n;
        }
    }

    /// `docube` — emit faces + spawn neighbor cubes per `faces[index]`.
    pub fn docube(&mut self, cube: Cube) {
        let mut index = 0i32;
        for i in 0..8 {
            if self.corner_list[cube.corners[i]].value > 0.0 {
                index += 1 << i;
            }
        }
        let faces = faces_table();
        let fi = faces[index as usize];
        let (ci, cj, ck) = (cube.i, cube.j, cube.k);
        if mb_bit(fi, 0) != 0 {
            self.add_cube(ci - 1, cj, ck);
        }
        if mb_bit(fi, 1) != 0 {
            self.add_cube(ci + 1, cj, ck);
        }
        if mb_bit(fi, 2) != 0 {
            self.add_cube(ci, cj - 1, ck);
        }
        if mb_bit(fi, 3) != 0 {
            self.add_cube(ci, cj + 1, ck);
        }
        if mb_bit(fi, 4) != 0 {
            self.add_cube(ci, cj, ck - 1);
        }
        if mb_bit(fi, 5) != 0 {
            self.add_cube(ci, cj, ck + 1);
        }

        for poly in &cube_table()[index as usize] {
            let mut indexar = [0i32; 8];
            let mut count = 0;
            for &e in poly {
                let c1 = cube.corners[CORNER1[e] as usize];
                let c2 = cube.corners[CORNER2[e] as usize];
                indexar[count] = self.vertid(c1, c2);
                count += 1;
            }
            if count > 2 {
                match count {
                    3 => self.make_face(indexar[2], indexar[1], indexar[0], indexar[0]),
                    4 => self.make_face(indexar[3], indexar[2], indexar[1], indexar[0]),
                    5 => {
                        self.make_face(indexar[3], indexar[2], indexar[1], indexar[0]);
                        self.make_face(indexar[4], indexar[3], indexar[0], indexar[0]);
                    }
                    6 => {
                        self.make_face(indexar[3], indexar[2], indexar[1], indexar[0]);
                        self.make_face(indexar[5], indexar[4], indexar[3], indexar[0]);
                    }
                    7 => {
                        self.make_face(indexar[3], indexar[2], indexar[1], indexar[0]);
                        self.make_face(indexar[5], indexar[4], indexar[3], indexar[0]);
                        self.make_face(indexar[6], indexar[5], indexar[0], indexar[0]);
                    }
                    _ => {}
                }
            }
        }
    }

    /// `find_first_points` — seed cubes from each meta-elem.
    pub fn find_first_points(&mut self, em: usize) {
        let ml_bb = self.mainb[em].bb;
        let center = closest_lattice(&[
            (ml_bb[0][0] + ml_bb[6][0]) / 2.0,
            (ml_bb[0][1] + ml_bb[6][1]) / 2.0,
            (ml_bb[0][2] + ml_bb[6][2]) / 2.0,
        ], self.size);
        let lbn = prev_lattice(&ml_bb[0], self.size);
        let rtf = next_lattice(&ml_bb[6], self.size);
        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    if dx == 0 && dy == 0 && dz == 0 {
                        continue;
                    }
                    let dir = [dx, dy, dz];
                    let mut it = center;
                    let mut b = self.setcorner(it[0], it[1], it[2]);
                    let mut b_val = self.corner_list[b].value;
                    loop {
                        it[0] += dir[0];
                        it[1] += dir[1];
                        it[2] += dir[2];
                        let a_val = b_val;
                        b = self.setcorner(it[0], it[1], it[2]);
                        b_val = self.corner_list[b].value;
                        if a_val * b_val < 0.0 {
                            let mut add = [it[0] - dir[0], it[1] - dir[1], it[2] - dir[2]];
                            for a in 0..3 {
                                add[a] = add[a].min(it[a]);
                            }
                            self.add_cube(add[0], add[1], add[2]);
                            break;
                        }
                        if !(it[0] > lbn[0]
                            && it[1] > lbn[1]
                            && it[2] > lbn[2]
                            && it[0] < rtf[0]
                            && it[1] < rtf[1]
                            && it[2] < rtf[2])
                        {
                            break;
                        }
                    }
                }
            }
        }
    }

    /// `polygonize`.
    pub fn polygonize(&mut self) {
        ensure_cube_table();
        for i in 0..self.mainb.len() {
            self.find_first_points(i);
        }
        while let Some(cube) = self.cubes.pop() {
            self.docube(cube);
        }
    }
}

fn next_lattice(pos: &[f32; 3], size: f32) -> [i32; 3] {
    [
        (pos[0] / size + 0.5).ceil() as i32,
        (pos[1] / size + 0.5).ceil() as i32,
        (pos[2] / size + 0.5).ceil() as i32,
    ]
}
fn prev_lattice(pos: &[f32; 3], size: f32) -> [i32; 3] {
    let mut r = next_lattice(pos, size);
    r[0] -= 1;
    r[1] -= 1;
    r[2] -= 1;
    r
}
fn closest_lattice(pos: &[f32; 3], size: f32) -> [i32; 3] {
    [
        (pos[0] / size + 1.0).floor() as i32,
        (pos[1] / size + 1.0).floor() as i32,
        (pos[2] / size + 1.0).floor() as i32,
    ]
}

/// `nextcwedge`.
fn nextcwedge(edge: usize, face: i32) -> usize {
    match edge {
        LB => if face == L { LF } else { BN },
        LT => if face == L { LN } else { TF },
        LN => if face == L { LB } else { TN },
        LF => if face == L { LT } else { BF },
        RB => if face == R { RN } else { BF },
        RT => if face == R { RF } else { TN },
        RN => if face == R { RT } else { BN },
        RF => if face == R { RB } else { TF },
        BN => if face == B { RB } else { LN },
        BF => if face == B { LB } else { RF },
        TN => if face == T { LT } else { RN },
        TF => if face == T { RT } else { LF },
        _ => 0,
    }
}

/// `otherface`.
fn otherface(edge: usize, face: i32) -> i32 {
    let other = LEFTFACE[edge];
    if face == other {
        RIGHTFACE[edge]
    } else {
        other
    }
}

/// Cube table: per corner-configuration → list of polys → edge ids.
static CUBE_TABLE: OnceLock<Vec<Vec<Vec<usize>>>> = OnceLock::new();
/// `faces[256]` — neighbor-cube spawn mask.
static FACES_TABLE: OnceLock<Vec<i32>> = OnceLock::new();

/// `makecubetable`.
pub fn ensure_cube_table() {
    CUBE_TABLE.get_or_init(|| {
        let mut table: Vec<Vec<Vec<usize>>> = vec![Vec::new(); 256];
        for i in 0..256i32 {
            let mut done = [false; 12];
            let mut pos = [0i32; 8];
            for c in 0..8 {
                pos[c] = mb_bit(i, c as i32);
            }
            for e in 0..12usize {
                if !done[e] && pos[CORNER1[e] as usize] != pos[CORNER2[e] as usize] {
                    let mut ints: Vec<usize> = Vec::new();
                    let start = e;
                    let mut edge = e;
                    let mut face = if pos[CORNER1[e] as usize] != 0 {
                        RIGHTFACE[e]
                    } else {
                        LEFTFACE[e]
                    };
                    loop {
                        edge = nextcwedge(edge, face);
                        done[edge] = true;
                        if pos[CORNER1[edge] as usize] != pos[CORNER2[edge] as usize] {
                            ints.insert(0, edge);
                            if edge == start {
                                break;
                            }
                            face = otherface(edge, face);
                        }
                    }
                    table[i as usize].insert(0, ints);
                }
            }
        }
        table
    });
    FACES_TABLE.get_or_init(|| {
        let table = CUBE_TABLE.get().unwrap();
        let mut faces = vec![0i32; 256];
        for i in 0..256 {
            for poly in &table[i] {
                for &e in poly {
                    if [LB, LT, LN, LF].contains(&e) {
                        faces[i] |= 1 << L;
                    }
                    if [RB, RT, RN, RF].contains(&e) {
                        faces[i] |= 1 << R;
                    }
                    if [LB, RB, BN, BF].contains(&e) {
                        faces[i] |= 1 << B;
                    }
                    if [LT, RT, TN, TF].contains(&e) {
                        faces[i] |= 1 << T;
                    }
                    if [LN, RN, BN, TN].contains(&e) {
                        faces[i] |= 1 << N;
                    }
                    if [LF, RF, BF, TF].contains(&e) {
                        faces[i] |= 1 << F;
                    }
                }
            }
        }
        faces
    });
}

fn cube_table() -> &'static Vec<Vec<Vec<usize>>> {
    ensure_cube_table();
    CUBE_TABLE.get().unwrap()
}
fn faces_table() -> &'static Vec<i32> {
    ensure_cube_table();
    FACES_TABLE.get().unwrap()
}

/// `BKE_mball_polygonize` — converge_res + size selection + eps check
/// verbatim; caller supplies `mainb`/`allbb` via `init_meta`-equivalent
/// (object-scene iteration injected).
pub fn bke_mball_polygonize(
    mainb: Vec<MetaElem>,
    thresh: f32,
    size: f32,
) -> Option<(Vec<Vec3>, Vec<Vec3>, Vec<[i32; 4]>)> {
    if mainb.is_empty() {
        return None;
    }
    let converge_res = if thresh < 0.001 {
        16
    } else if thresh < 0.01 {
        8
    } else if thresh < 0.1 {
        4
    } else {
        2
    };
    let mut allbb = BoxB {
        min: mainb[0].bb[0],
        max: mainb[0].bb[6],
        ml: None,
    };
    let mut mainb = mainb;
    for ml in mainb.iter().skip(1) {
        allbb = make_box_union(&ml.bb, &allbb);
    }
    let (bvh_nodes, _queue_size) = build_bvh_spatial(&mut mainb, &allbb);
    let mut process = Process {
        thresh,
        size,
        delta: size * 0.001,
        converge_res,
        mainb,
        allbb: Some(allbb),
        bvh_nodes,
        ..Default::default()
    };
    process.polygonize();
    if process.indices.is_empty() {
        return None;
    }
    /* Normalize accumulated normals. */
    for n in process.no.iter_mut() {
        let l = n.length();
        if l > 0.0 {
            *n = *n / l;
        }
    }
    Some((process.co, process.no, process.indices))
}
