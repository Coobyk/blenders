//! Port of `mesh_intersect.cc` — exact-arithmetic triangle mesh self- and
//! n-ary intersection (`WITH_GMP` build; `mpq_class` -> `Mpq`/BigRational,
//! `mpq2`/`mpq3` -> `Mpq2`/`Mpq3`).
//!
//! C++ `Vert *`/`Face *`/`Plane *` pointers become usize indices into
//! `IMeshArena` pools. `parallel_for`/`task_parallel_range` become serial
//! loops (see PORTING.md). Debug `std::cout`/`PERFDEBUG` blocks omitted.

use crate::delaunay_2d::{delaunay_2d_calc, CdtInput, CdtResult, CdtScalar};
use crate::delaunay_2d::{
    CDT_ORIG_EDGES, CDT_ORIG_FACES, CDT_ORIG_VERTS,
};
use crate::kdopbvh::{
    bvhtree_balance, bvhtree_insert, bvhtree_new, bvhtree_overlap, bvhtree_overlap_self,
    BVHTreeOverlap,
};
use crate::polyfill_2d::polyfill_calc_pub;
use blenders_math::boolean::orient3d_mpq;
use blenders_math::geom::axis_dominant_to_m3_negate;
use blenders_math::mpq::{mpq, mpq_abs, mpq_i, mpq_sgn, mpq_to_f64, Mpq, Mpq2, Mpq3};
use num_traits::Signed;
use std::collections::HashMap;

/// `NO_INDEX`.
pub const NO_INDEX: i32 = -1;
const NIL: usize = usize::MAX;
const DBL_EPSILON: f64 = f64::EPSILON;
const FLT_EPSILON: f64 = f32::EPSILON as f64;

/* `index_*` constants — supremum/index error analysis from Burnikel et al. */
const INDEX_DOT_PLANE_COORDS: i32 = 15;
const INDEX_DOT_CROSS: i32 = 11;
const INDEX_PLANE_SIDE: i32 = 3 + 2 * INDEX_DOT_PLANE_COORDS;

/// `meshintersect::Vert`.
#[derive(Clone)]
pub struct Vert {
    pub co_exact: Mpq3,
    pub co: [f64; 3],
    pub id: i32,
    pub orig: i32,
}
impl Vert {
    pub fn new(mco: Mpq3, dco: [f64; 3], id: i32, orig: i32) -> Self {
        Vert {
            co_exact: mco,
            co: dco,
            id,
            orig,
        }
    }
    /// `Vert::operator==` — compares co_exact only.
    pub fn eq_co(&self, other: &Vert) -> bool {
        self.co_exact == other.co_exact
    }
    /// `Vert::hash` — double-bits hash.
    pub fn hash(&self) -> u64 {
        let mut x = self.co[0].to_bits();
        let mut y = self.co[1].to_bits();
        let mut z = self.co[2].to_bits();
        x = (x >> 56) ^ (x >> 46) ^ x;
        y = (y >> 55) ^ (y >> 45) ^ y;
        z = (z >> 54) ^ (z >> 44) ^ z;
        x ^ y ^ z
    }
}

/// `meshintersect::Plane`.
#[derive(Clone)]
pub struct Plane {
    pub norm_exact: Mpq3,
    pub d_exact: Mpq,
    pub norm: [f64; 3],
    pub d: f64,
}
impl Plane {
    /// `Plane(mpq3 norm_exact, mpq d_exact)`.
    pub fn new_exact(norm_exact: Mpq3, d_exact: Mpq) -> Self {
        let norm = norm_exact.get_d();
        let d = mpq_to_f64(&d_exact);
        Plane {
            norm_exact,
            d_exact,
            norm,
            d,
        }
    }
    /// `Plane(double3 norm, double d)` — exact part marked unpopulated.
    pub fn new_approx(norm: [f64; 3], d: f64) -> Self {
        Plane {
            norm_exact: Mpq3::zero(),
            d_exact: Mpq::default(),
            norm,
            d,
        }
    }
    pub fn exact_populated(&self) -> bool {
        !self.norm_exact.x.is_zero() || !self.norm_exact.y.is_zero() || !self.norm_exact.z.is_zero()
    }
    /// `Plane::make_canonical` — normalize by first nonzero norm component.
    pub fn make_canonical(&mut self) {
        if !self.norm_exact.x.is_zero() {
            let den = self.norm_exact.x.clone();
            self.norm_exact = Mpq3::new(
                mpq_i(1),
                &self.norm_exact.y / &den,
                &self.norm_exact.z / &den,
            );
            self.d_exact = &self.d_exact / &den;
        } else if !self.norm_exact.y.is_zero() {
            let den = self.norm_exact.y.clone();
            self.norm_exact = Mpq3::new(mpq_i(0), mpq_i(1), &self.norm_exact.z / &den);
            self.d_exact = &self.d_exact / &den;
        } else if !self.norm_exact.z.is_zero() {
            let den = self.norm_exact.z.clone();
            self.norm_exact = Mpq3::new(mpq_i(0), mpq_i(0), mpq_i(1));
            self.d_exact = &self.d_exact / &den;
        } else {
            self.d_exact = mpq_i(0); /* Degenerate plane. */
        }
        self.norm = self.norm_exact.get_d();
        self.d = mpq_to_f64(&self.d_exact);
    }
    /// `Plane::operator==` — canonical forms compare by exact parts.
    pub fn eq_plane(&self, other: &Plane) -> bool {
        self.norm_exact == other.norm_exact && self.d_exact == other.d_exact
    }
    /// `Plane::hash` — double-bits hash.
    pub fn hash(&self) -> u64 {
        let mut x = self.norm[0].to_bits();
        let mut y = self.norm[1].to_bits();
        let mut z = self.norm[2].to_bits();
        let mut d = self.d.to_bits();
        x = (x >> 56) ^ (x >> 46) ^ x;
        y = (y >> 55) ^ (y >> 45) ^ y;
        z = (z >> 54) ^ (z >> 44) ^ z;
        d = (d >> 53) ^ (d >> 43) ^ d;
        x ^ y ^ z ^ d
    }
}

/// `meshintersect::Face` — verts/edge_orig/is_intersect parallel arrays;
/// `plane` is an index into `IMeshArena::planes`.
#[derive(Clone)]
pub struct Face {
    pub vert: Vec<usize>,
    pub edge_orig: Vec<i32>,
    pub is_intersect: Vec<bool>,
    pub plane: Option<usize>,
    pub id: i32,
    pub orig: i32,
}
impl Face {
    pub fn new(verts: Vec<usize>, id: i32, orig: i32, edge_origs: Vec<i32>, is_intersect: Vec<bool>) -> Self {
        Face {
            vert: verts,
            edge_orig: edge_origs,
            is_intersect,
            plane: None,
            id,
            orig,
        }
    }
    pub fn size(&self) -> usize {
        self.vert.len()
    }
    pub fn is_tri(&self) -> bool {
        self.vert.len() == 3
    }
    /// `Face::populate_plane` (arena supplies vert/plane storage).
    pub fn populate_plane(&mut self, arena: &mut IMeshArena, need_exact: bool) {
        if self.plane.is_some() {
            let p = &arena.planes[self.plane.unwrap()];
            if !need_exact || p.exact_populated() {
                return;
            }
        }
        if need_exact {
            let normal_exact = if self.vert.len() > 3 {
                let co: Vec<Mpq3> = self
                    .vert
                    .iter()
                    .map(|&v| arena.verts[v].co_exact.clone())
                    .collect();
                cross_poly_mpq(&co)
            } else {
                let tr02 = &arena.verts[self.vert[0]].co_exact - &arena.verts[self.vert[2]].co_exact;
                let tr12 = &arena.verts[self.vert[1]].co_exact - &arena.verts[self.vert[2]].co_exact;
                Mpq3::cross(&tr02, &tr12)
            };
            let d_exact = -normal_exact.dot(&arena.verts[self.vert[0]].co_exact);
            self.plane = Some(arena.add_plane(Plane::new_exact(normal_exact, d_exact)));
        } else {
            let normal = if self.vert.len() > 3 {
                let co: Vec<[f64; 3]> = self.vert.iter().map(|&v| arena.verts[v].co).collect();
                cross_poly_d(&co)
            } else {
                let tr02 = sub_d(arena.verts[self.vert[0]].co, arena.verts[self.vert[2]].co);
                let tr12 = sub_d(arena.verts[self.vert[1]].co, arena.verts[self.vert[2]].co);
                cross_d(tr02, tr12)
            };
            let d = -dot_d(normal, arena.verts[self.vert[0]].co);
            self.plane = Some(arena.add_plane(Plane::new_approx(normal, d)));
        }
    }
    /// `Face::cyclic_equal` — verts may be cyclically shifted.
    pub fn cyclic_equal(&self, arena: &IMeshArena, other: &Face) -> bool {
        if self.size() != other.size() {
            return false;
        }
        let flen = self.size();
        for start in 0..flen {
            for start_other in 0..flen {
                let mut ok = true;
                for i in 0..flen {
                    let p = (start + i) % flen;
                    let p_other = (start_other + i) % flen;
                    if arena.verts[self.vert[p]].co_exact != arena.verts[other.vert[p_other]].co_exact
                    {
                        ok = false;
                        break;
                    }
                }
                if ok {
                    return true;
                }
            }
        }
        false
    }
}

/* ---- small double3 helpers (math::cross/dot/sub on double3) ---- */
fn sub_d(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn dot_d(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross_d(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}
fn abs_d(a: [f64; 3]) -> [f64; 3] {
    [a[0].abs(), a[1].abs(), a[2].abs()]
}
/// `math::cross_poly` for double3 (Newell).
fn cross_poly_d(poly: &[[f64; 3]]) -> [f64; 3] {
    let n = poly.len();
    let mut ans = [0.0; 3];
    for i in 0..n {
        let prev = poly[(i + n - 1) % n];
        let cur = poly[i];
        ans[0] += (prev[1] - cur[1]) * (prev[2] + cur[2]);
        ans[1] += (prev[2] - cur[2]) * (prev[0] + cur[0]);
        ans[2] += (prev[0] - cur[0]) * (prev[1] + cur[1]);
    }
    ans
}
/// `math::cross_poly` for mpq3.
fn cross_poly_mpq(poly: &[Mpq3]) -> Mpq3 {
    let n = poly.len();
    let mut ans = Mpq3::zero();
    for i in 0..n {
        let prev = &poly[(i + n - 1) % n];
        let cur = &poly[i];
        ans.x += &(&prev.y - &cur.y) * &(&prev.z + &cur.z);
        ans.y += &(&prev.z - &cur.z) * &(&prev.x + &cur.x);
        ans.z += &(&prev.x - &cur.x) * &(&prev.y + &cur.y);
    }
    ans
}
/// `math::dominant_axis` for Mpq3.
fn dominant_axis_mpq(a: &Mpq3) -> usize {
    let bx = mpq_abs(&a.x);
    let by = mpq_abs(&a.y);
    let bz = mpq_abs(&a.z);
    if bx > by {
        if bx > bz {
            0
        } else {
            2
        }
    } else if by > bz {
        1
    } else {
        2
    }
}
/// `math::dominant_axis` for double3.
fn dominant_axis_d(a: [f64; 3]) -> usize {
    let b = abs_d(a);
    if b[0] > b[1] {
        if b[0] > b[2] {
            0
        } else {
            2
        }
    } else if b[1] > b[2] {
        1
    } else {
        2
    }
}

/// `IMeshArena` — owns all Vert/Face/Plane storage; dedups Verts by co_exact.
pub struct IMeshArena {
    pub verts: Vec<Vert>,
    pub faces: Vec<Face>,
    pub planes: Vec<Plane>,
    /// hash -> candidate vert indices (VSetKey set, keyed on Vert::hash).
    vset: HashMap<u64, Vec<usize>>,
    next_vert_id: i32,
    next_face_id: i32,
}
impl Default for IMeshArena {
    fn default() -> Self {
        Self::new()
    }
}
impl IMeshArena {
    pub fn new() -> Self {
        IMeshArena {
            verts: Vec::new(),
            faces: Vec::new(),
            planes: Vec::new(),
            vset: HashMap::new(),
            next_vert_id: 0,
            next_face_id: 0,
        }
    }
    pub fn reserve(&mut self, vert_num_hint: usize, face_num_hint: usize) {
        self.verts.reserve(vert_num_hint);
        self.faces.reserve(face_num_hint);
        self.vset.reserve(vert_num_hint);
    }
    pub fn tot_allocated_verts(&self) -> usize {
        self.verts.len()
    }
    pub fn tot_allocated_faces(&self) -> usize {
        self.faces.len()
    }
    fn add_plane(&mut self, p: Plane) -> usize {
        self.planes.push(p);
        self.planes.len() - 1
    }
    /// `add_or_find_vert(mpq3, orig)`.
    pub fn add_or_find_vert_mpq(&mut self, co: &Mpq3, orig: i32) -> usize {
        let dco = co.get_d();
        let vtry = Vert::new(co.clone(), dco, NO_INDEX, orig);
        self.add_or_find_vert_v(vtry)
    }
    /// `add_or_find_vert(double3, orig)`.
    pub fn add_or_find_vert_d(&mut self, co: [f64; 3], orig: i32) -> usize {
        let mco = Mpq3::from_f64s(co);
        let vtry = Vert::new(mco, co, NO_INDEX, orig);
        self.add_or_find_vert_v(vtry)
    }
    /// `add_or_find_vert(Vert*)`.
    pub fn add_or_find_vert_v(&mut self, vtry: Vert) -> usize {
        let h = vtry.hash();
        if let Some(bucket) = self.vset.get(&h) {
            for &i in bucket {
                if self.verts[i].co_exact == vtry.co_exact {
                    return i;
                }
            }
        }
        let index = self.verts.len();
        let mut v = vtry;
        v.id = self.next_vert_id;
        self.next_vert_id += 1;
        self.verts.push(v);
        self.vset.entry(h).or_default().push(index);
        index
    }
    /// `find_vert(mpq3)` — returns vert index or None.
    pub fn find_vert(&self, co: &Mpq3) -> Option<usize> {
        let dco = co.get_d();
        let vtry = Vert::new(co.clone(), dco, NO_INDEX, NO_INDEX);
        let h = vtry.hash();
        self.vset.get(&h).and_then(|bucket| {
            bucket
                .iter()
                .copied()
                .find(|&i| self.verts[i].co_exact == *co)
        })
    }
    /// `add_face` variants.
    pub fn add_face_full(
        &mut self,
        verts: Vec<usize>,
        orig: i32,
        edge_origs: Vec<i32>,
        is_intersect: Vec<bool>,
    ) -> usize {
        let f = Face::new(verts, self.next_face_id, orig, edge_origs, is_intersect);
        self.next_face_id += 1;
        self.faces.push(f);
        self.faces.len() - 1
    }
    pub fn add_face_eo(&mut self, verts: Vec<usize>, orig: i32, edge_origs: Vec<i32>) -> usize {
        let n = verts.len();
        self.add_face_full(verts, orig, edge_origs, vec![false; n])
    }
    pub fn add_face(&mut self, verts: Vec<usize>, orig: i32) -> usize {
        let n = verts.len();
        self.add_face_full(verts, orig, vec![NO_INDEX; n], vec![false; n])
    }
    /// `find_face` — slow; only for tests. `vs` may be a cyclic shift.
    pub fn find_face(&self, vs: &[usize]) -> Option<usize> {
        for (i, f) in self.faces.iter().enumerate() {
            if f.vert.len() == vs.len() && f.cyclic_equal(self, &Face {
                vert: vs.to_vec(),
                edge_orig: Vec::new(),
                is_intersect: Vec::new(),
                plane: None,
                id: NO_INDEX,
                orig: NO_INDEX,
            }) {
                return Some(i);
            }
        }
        None
    }
}

/// `IMesh` — face indices into an arena; vert array populated on demand.
#[derive(Clone, Default)]
pub struct IMesh {
    pub face: Vec<usize>, /* usize::MAX marks a removed face. */
    pub vert: Vec<usize>,
    vert_to_index: HashMap<usize, usize>,
    vert_populated: bool,
}
impl IMesh {
    pub fn new() -> Self {
        IMesh::default()
    }
    pub fn from_faces(faces: Vec<usize>) -> Self {
        IMesh {
            face: faces,
            ..IMesh::default()
        }
    }
    pub fn set_faces(&mut self, faces: Vec<usize>) {
        self.face = faces;
    }
    pub fn face_size(&self) -> usize {
        self.face.len()
    }
    pub fn faces(&self) -> &[usize] {
        &self.face
    }
    pub fn has_verts(&self) -> bool {
        self.vert_populated
    }
    pub fn populate_vert(&mut self, arena: &IMeshArena) {
        self.populate_vert_hint(arena, 4 * self.face.len());
    }
    /// `IMesh::populate_vert` — builds vert list from faces, sorted by orig
    /// then id (upstream `fix_order` behavior).
    pub fn populate_vert_hint(&mut self, arena: &IMeshArena, _max_verts: usize) {
        if self.vert_populated {
            return;
        }
        let mut next_index = 0usize;
        for &f in &self.face {
            for &v in &arena.faces[f].vert {
                if !self.vert_to_index.contains_key(&v) {
                    self.vert_to_index.insert(v, next_index);
                    next_index += 1;
                }
            }
        }
        let mut vert: Vec<usize> = vec![0; next_index];
        for (v, i) in &self.vert_to_index {
            vert[*i] = *v;
        }
        vert.sort_by_key(|&v| {
            let vv = &arena.verts[v];
            (vv.orig == NO_INDEX, vv.orig.max(0), vv.id)
        });
        /* Wait: upstream sort is by orig (only when both != NO_INDEX), then id.
         * The key above orders non-NO_INDEX origs first then ids — equivalent. */
        for (i, &v) in vert.iter().enumerate() {
            self.vert_to_index.insert(v, i);
        }
        self.vert = vert;
        self.vert_populated = true;
    }
    /// `lookup_vert` — index of v in populated vert list.
    pub fn lookup_vert(&self, v: usize) -> i32 {
        self.vert_to_index.get(&v).copied().map(|i| i as i32).unwrap_or(NO_INDEX)
    }
    /// `IMesh::erase_face_positions`.
    pub fn erase_face_positions(
        &mut self,
        f_index: usize,
        face_pos_erase: &[bool],
        arena: &mut IMeshArena,
    ) -> bool {
        let cur_f = &arena.faces[self.face[f_index]];
        let cur_len = cur_f.size();
        let to_erase_num = face_pos_erase.iter().filter(|&&e| e).count();
        if to_erase_num == 0 {
            return false;
        }
        let new_len = cur_len - to_erase_num;
        if new_len < 3 {
            self.face[f_index] = NIL;
            return true;
        }
        let mut new_vert = Vec::with_capacity(new_len);
        let mut new_edge_orig = Vec::with_capacity(new_len);
        let mut new_is_intersect = Vec::with_capacity(new_len);
        for i in 0..cur_len {
            if !face_pos_erase[i] {
                new_vert.push(cur_f.vert[i]);
                new_edge_orig.push(cur_f.edge_orig[i]);
                new_is_intersect.push(cur_f.is_intersect[i]);
            }
        }
        let orig = cur_f.orig;
        self.face[f_index] = arena.add_face_full(new_vert, orig, new_edge_orig, new_is_intersect);
        false
    }
    /// `IMesh::set_dirty_verts`.
    pub fn set_dirty_verts(&mut self) {
        self.vert.clear();
        self.vert_to_index.clear();
        self.vert_populated = false;
    }
    /// `IMesh::remove_null_faces`.
    pub fn remove_null_faces(&mut self) {
        self.face.retain(|&f| f != NIL);
    }
}

/// `BoundingBox` — float3 min/max as in kdopbvh but on doubles upstream
/// (upstream uses `BoundingBox` with float3). We keep f64.
#[derive(Clone, Copy)]
pub struct BoundingBox {
    pub min: [f64; 3],
    pub max: [f64; 3],
}
impl BoundingBox {
    pub fn new() -> Self {
        BoundingBox {
            min: [f64::MAX; 3],
            max: [f64::MIN; 3],
        }
    }
    pub fn combine(&mut self, co: [f64; 3]) {
        for i in 0..3 {
            self.min[i] = self.min[i].min(co[i]);
            self.max[i] = self.max[i].max(co[i]);
        }
    }
    pub fn combine_bb(&mut self, bb: &BoundingBox) {
        for i in 0..3 {
            self.min[i] = self.min[i].min(bb.min[i]);
            self.max[i] = self.max[i].max(bb.max[i]);
        }
    }
    pub fn expand(&mut self, pad: f64) {
        for i in 0..3 {
            self.min[i] -= pad;
            self.max[i] += pad;
        }
    }
}

/// `bbs_might_intersect` = `isect_aabb_aabb_v3`.
pub fn bbs_might_intersect(a: &BoundingBox, b: &BoundingBox) -> bool {
    blenders_math::geom::isect_aabb_aabb(
        blenders_math::vector::Vec3::new(a.min[0] as f32, a.min[1] as f32, a.min[2] as f32),
        blenders_math::vector::Vec3::new(a.max[0] as f32, a.max[1] as f32, a.max[2] as f32),
        blenders_math::vector::Vec3::new(b.min[0] as f32, b.min[1] as f32, b.min[2] as f32),
        blenders_math::vector::Vec3::new(b.max[0] as f32, b.max[1] as f32, b.max[2] as f32),
    )
}

/// `calc_face_bounding_boxes` (serial version).
fn calc_face_bounding_boxes(arena: &IMeshArena, m: &IMesh) -> Vec<BoundingBox> {
    let n = m.face_size();
    let mut ans = vec![BoundingBox::new(); n];
    let mut max_abs_val = 0.0f64;
    for (iter, bb) in ans.iter_mut().enumerate() {
        for &v in &arena.faces[m.face[iter]].vert {
            let co = arena.verts[v].co;
            bb.combine(co);
            for i in 0..3 {
                max_abs_val = max_abs_val.max(co[i].abs());
            }
        }
    }
    let pad_factor = 10.0f64;
    let mut pad = if max_abs_val == 0.0 {
        FLT_EPSILON
    } else {
        2.0 * FLT_EPSILON * max_abs_val
    };
    pad *= pad_factor;
    for bb in &mut ans {
        bb.expand(pad);
    }
    ans
}

/* ---- CoplanarCluster / CoplanarClusterInfo ---- */

/// `CoplanarCluster` — a cluster of coplanar triangles, by index.
#[derive(Clone, Default)]
struct CoplanarCluster {
    tris: Vec<usize>,
    bb: Option<BoundingBox>,
}
impl CoplanarCluster {
    fn add_tri(&mut self, t: usize, bb: BoundingBox) {
        self.tris.push(t);
        match &mut self.bb {
            Some(b) => b.combine_bb(&bb),
            None => self.bb = Some(bb),
        }
    }
    fn tot_tri(&self) -> usize {
        self.tris.len()
    }
    fn bounding_box(&self) -> BoundingBox {
        self.bb.unwrap_or_else(BoundingBox::new)
    }
}

/// `CoplanarClusterInfo` — indexed set of clusters + tri -> cluster map.
#[derive(Clone)]
struct CoplanarClusterInfo {
    clusters: Vec<CoplanarCluster>,
    tri_cluster: Vec<i32>,
}
impl CoplanarClusterInfo {
    fn new(numtri: usize) -> Self {
        CoplanarClusterInfo {
            clusters: Vec::new(),
            tri_cluster: vec![NO_INDEX; numtri],
        }
    }
    fn tri_cluster(&self, t: usize) -> i32 {
        self.tri_cluster[t]
    }
    fn add_cluster(&mut self, cl: CoplanarCluster) -> usize {
        let c_index = self.clusters.len();
        for &t in &cl.tris {
            self.tri_cluster[t] = c_index as i32;
        }
        self.clusters.push(cl);
        c_index
    }
    fn tot_cluster(&self) -> usize {
        self.clusters.len()
    }
    fn cluster(&self, c: usize) -> &CoplanarCluster {
        &self.clusters[c]
    }
}

/* ---- ITT_value / Guigue-Devillers tri-tri intersection ---- */

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum IttKind {
    None,
    Point,
    Segment,
    Coplanar,
}
/// `ITT_value`.
#[derive(Clone)]
pub struct IttValue {
    pub p1: Mpq3,
    pub p2: Mpq3,
    pub t_source: i32,
    pub kind: IttKind,
}
impl IttValue {
    fn none() -> Self {
        IttValue {
            p1: Mpq3::zero(),
            p2: Mpq3::zero(),
            t_source: -1,
            kind: IttKind::None,
        }
    }
    fn point(p1: Mpq3) -> Self {
        IttValue {
            p1,
            p2: Mpq3::zero(),
            t_source: -1,
            kind: IttKind::Point,
        }
    }
    fn segment(p1: Mpq3, p2: Mpq3) -> Self {
        IttValue {
            p1,
            p2,
            t_source: -1,
            kind: IttKind::Segment,
        }
    }
    fn coplanar(tsrc: i32) -> Self {
        IttValue {
            p1: Mpq3::zero(),
            p2: Mpq3::zero(),
            t_source: tsrc,
            kind: IttKind::Coplanar,
        }
    }
}

/// `project_3d_to_2d` — elide proj_axis.
fn project_3d_to_2d(p3d: &Mpq3, proj_axis: usize) -> Mpq2 {
    match proj_axis {
        0 => Mpq2::new(p3d.y.clone(), p3d.z.clone()),
        1 => Mpq2::new(p3d.x.clone(), p3d.z.clone()),
        _ => Mpq2::new(p3d.x.clone(), p3d.y.clone()),
    }
}

/// `supremum_dot_cross` — error bound machinery (Burnikel et al).
fn supremum_dot_cross(a: [f64; 3], b: [f64; 3]) -> f64 {
    let aa = abs_d(a);
    let ab = abs_d(b);
    let c = [
        aa[1] * ab[2] + aa[2] * ab[1],
        aa[2] * ab[0] + aa[0] * ab[2],
        aa[0] * ab[1] + aa[1] * ab[0],
    ];
    dot_d(c, c)
}

/// `filter_plane_side` — fast double plane-side test with error bound.
fn filter_plane_side(
    p: [f64; 3],
    plane_p: [f64; 3],
    plane_no: [f64; 3],
    abs_p: [f64; 3],
    abs_plane_p: [f64; 3],
    abs_plane_no: [f64; 3],
) -> i32 {
    let d = dot_d(sub_d(p, plane_p), plane_no);
    if d == 0.0 {
        return 0;
    }
    let supremum = dot_d(
        [
            abs_p[0] + abs_plane_p[0],
            abs_p[1] + abs_plane_p[1],
            abs_p[2] + abs_plane_p[2],
        ],
        abs_plane_no,
    );
    let err_bound = supremum * (INDEX_PLANE_SIDE as f64) * DBL_EPSILON;
    if d.abs() > err_bound {
        if d > 0.0 {
            1
        } else {
            -1
        }
    } else {
        0
    }
}

/// `tti_interp` — point on ab where plane (normal n, through c) intersects it.
fn tti_interp(a: &Mpq3, b: &Mpq3, c: &Mpq3, n: &Mpq3) -> Mpq3 {
    let ab = a - b;
    let ac = a - c;
    let den = ab.dot(n);
    debug_assert!(!den.is_zero());
    let alpha = ac.dot(n) / &den;
    a - &(&alpha * &ab)
}

/// `tti_above` — sign of dot(ad, cross(b-a, c-a)).
fn tti_above(a: &Mpq3, b: &Mpq3, c: &Mpq3, ad: &Mpq3) -> i32 {
    let ba = b - a;
    let ca = c - a;
    let n = Mpq3::cross(&ba, &ca);
    mpq_sgn(&ad.dot(&n))
}

/// `itt_canon2` — Guigue-Devillers interval-overlap classification.
fn itt_canon2(
    p1: &Mpq3,
    q1: &Mpq3,
    r1: &Mpq3,
    p2: &Mpq3,
    q2: &Mpq3,
    r2: &Mpq3,
    n1: &Mpq3,
    n2: &Mpq3,
) -> IttValue {
    let p1p2 = p2 - p1;
    let intersect_1;
    let intersect_2;
    let mut no_overlap = false;
    if tti_above(p1, q1, r2, &p1p2) > 0 {
        if tti_above(p1, r1, r2, &p1p2) <= 0 {
            if tti_above(p1, r1, q2, &p1p2) > 0 {
                /* Overlap [k [i l] j]. */
                intersect_1 = tti_interp(p1, r1, p2, n2);
                intersect_2 = tti_interp(p2, r2, p1, n1);
            } else {
                /* Overlap [i [k l] j]. */
                intersect_1 = tti_interp(p2, q2, p1, n1);
                intersect_2 = tti_interp(p2, r2, p1, n1);
            }
        } else {
            no_overlap = true; /* [k l] [i j] */
            intersect_1 = Mpq3::zero();
            intersect_2 = Mpq3::zero();
        }
    } else if tti_above(p1, q1, q2, &p1p2) < 0 {
        no_overlap = true; /* [i j] [k l] */
        intersect_1 = Mpq3::zero();
        intersect_2 = Mpq3::zero();
    } else if tti_above(p1, r1, q2, &p1p2) >= 0 {
        /* Overlap [k [i j] l]. */
        intersect_1 = tti_interp(p1, r1, p2, n2);
        intersect_2 = tti_interp(p1, q1, p2, n2);
    } else {
        /* Overlap [i [k j] l]. */
        intersect_1 = tti_interp(p2, q2, p1, n1);
        intersect_2 = tti_interp(p1, q1, p2, n2);
    }
    if no_overlap {
        return IttValue::none();
    }
    if intersect_1 == intersect_2 {
        return IttValue::point(intersect_1);
    }
    IttValue::segment(intersect_1, intersect_2)
}

/// `itt_canon1` — canonicalization layer over itt_canon2.
#[allow(clippy::too_many_arguments)]
fn itt_canon1(
    p1: &Mpq3,
    q1: &Mpq3,
    r1: &Mpq3,
    p2: &Mpq3,
    q2: &Mpq3,
    r2: &Mpq3,
    n1: &Mpq3,
    n2: &Mpq3,
    sp2: i32,
    sq2: i32,
    sr2: i32,
) -> IttValue {
    if sp2 > 0 {
        if sq2 > 0 {
            return itt_canon2(p1, r1, q1, r2, p2, q2, n1, n2);
        }
        if sr2 > 0 {
            return itt_canon2(p1, r1, q1, q2, r2, p2, n1, n2);
        }
        return itt_canon2(p1, q1, r1, p2, q2, r2, n1, n2);
    }
    if sp2 < 0 {
        if sq2 < 0 {
            return itt_canon2(p1, q1, r1, r2, p2, q2, n1, n2);
        }
        if sr2 < 0 {
            return itt_canon2(p1, q1, r1, q2, r2, p2, n1, n2);
        }
        return itt_canon2(p1, r1, q1, p2, q2, r2, n1, n2);
    }
    if sq2 < 0 {
        if sr2 >= 0 {
            return itt_canon2(p1, r1, q1, q2, r2, p2, n1, n2);
        }
        return itt_canon2(p1, q1, r1, p2, q2, r2, n1, n2);
    }
    if sq2 > 0 {
        if sr2 > 0 {
            return itt_canon2(p1, r1, q1, p2, q2, r2, n1, n2);
        }
        return itt_canon2(p1, q1, r1, q2, r2, p2, n1, n2);
    }
    if sr2 > 0 {
        return itt_canon2(p1, q1, r1, r2, p2, q2, n1, n2);
    }
    if sr2 < 0 {
        return itt_canon2(p1, r1, q1, r2, p2, q2, n1, n2);
    }
    IttValue::coplanar(-1)
}

/// `intersect_tri_tri` — Guigue-Devillers triangle-triangle intersection.
fn intersect_tri_tri(arena: &IMeshArena, tm: &IMesh, t1: usize, t2: usize) -> IttValue {
    let tri1 = &arena.faces[tm.face[t1]];
    let tri2 = &arena.faces[tm.face[t2]];
    let plane2 = tri2.plane.map(|p| &arena.planes[p]);
    let (vp1, vq1, vr1) = (tri1.vert[0], tri1.vert[1], tri1.vert[2]);
    let (vp2, vq2, vr2) = (tri2.vert[0], tri2.vert[1], tri2.vert[2]);
    let plane2 = plane2.expect("plane populated");
    let d_n2 = plane2.norm;
    let d_p1 = arena.verts[vp1].co;
    let d_q1 = arena.verts[vq1].co;
    let d_r1 = arena.verts[vr1].co;
    let d_p2 = arena.verts[vp2].co;
    let d_q2 = arena.verts[vq2].co;
    let d_r2 = arena.verts[vr2].co;
    let abs_d_r2 = abs_d(d_r2);
    let abs_d_n2 = abs_d(d_n2);
    let mut sp1 = filter_plane_side(d_p1, d_r2, d_n2, abs_d(d_p1), abs_d_r2, abs_d_n2);
    let mut sq1 = filter_plane_side(d_q1, d_r2, d_n2, abs_d(d_q1), abs_d_r2, abs_d_n2);
    let mut sr1 = filter_plane_side(d_r1, d_r2, d_n2, abs_d(d_r1), abs_d_r2, abs_d_n2);
    if (sp1 > 0 && sq1 > 0 && sr1 > 0) || (sp1 < 0 && sq1 < 0 && sr1 < 0) {
        return IttValue::none();
    }
    let plane1 = tri1.plane.map(|p| &arena.planes[p]).expect("plane populated");
    let d_n1 = plane1.norm;
    let abs_d_r1 = abs_d(d_r1);
    let abs_d_n1 = abs_d(d_n1);
    let mut sp2 = filter_plane_side(d_p2, d_r1, d_n1, abs_d(d_p2), abs_d_r1, abs_d_n1);
    let mut sq2 = filter_plane_side(d_q2, d_r1, d_n1, abs_d(d_q2), abs_d_r1, abs_d_n1);
    let mut sr2 = filter_plane_side(d_r2, d_r1, d_n1, abs_d(d_r2), abs_d_r1, abs_d_n1);
    if (sp2 > 0 && sq2 > 0 && sr2 > 0) || (sp2 < 0 && sq2 < 0 && sr2 < 0) {
        return IttValue::none();
    }
    /* Exact fallback for zero signs. */
    let p1 = &arena.verts[vp1].co_exact;
    let q1 = &arena.verts[vq1].co_exact;
    let r1 = &arena.verts[vr1].co_exact;
    let p2 = &arena.verts[vp2].co_exact;
    let q2 = &arena.verts[vq2].co_exact;
    let r2 = &arena.verts[vr2].co_exact;
    let n2 = &plane2.norm_exact;
    if sp1 == 0 {
        sp1 = mpq_sgn(&(p1 - r2).dot(n2));
    }
    if sq1 == 0 {
        sq1 = mpq_sgn(&(q1 - r2).dot(n2));
    }
    if sr1 == 0 {
        sr1 = mpq_sgn(&(r1 - r2).dot(n2));
    }
    if (sp1 * sq1 > 0) && (sp1 * sr1 > 0) {
        return IttValue::none();
    }
    let n1 = &plane1.norm_exact;
    if sp2 == 0 {
        sp2 = mpq_sgn(&(p2 - r1).dot(n1));
    }
    if sq2 == 0 {
        sq2 = mpq_sgn(&(q2 - r1).dot(n1));
    }
    if sr2 == 0 {
        sr2 = mpq_sgn(&(r2 - r1).dot(n1));
    }
    if (sp2 * sq2 > 0) && (sp2 * sr2 > 0) {
        return IttValue::none();
    }
    /* Canonical-order dispatch. */
    let mut ans = if sp1 > 0 {
        if sq1 > 0 {
            itt_canon1(r1, p1, q1, p2, r2, q2, n1, n2, sp2, sr2, sq2)
        } else if sr1 > 0 {
            itt_canon1(q1, r1, p1, p2, r2, q2, n1, n2, sp2, sr2, sq2)
        } else {
            itt_canon1(p1, q1, r1, p2, q2, r2, n1, n2, sp2, sq2, sr2)
        }
    } else if sp1 < 0 {
        if sq1 < 0 {
            itt_canon1(r1, p1, q1, p2, q2, r2, n1, n2, sp2, sq2, sr2)
        } else if sr1 < 0 {
            itt_canon1(q1, r1, p1, p2, q2, r2, n1, n2, sp2, sq2, sr2)
        } else {
            itt_canon1(p1, q1, r1, p2, r2, q2, n1, n2, sp2, sr2, sq2)
        }
    } else if sq1 < 0 {
        if sr1 >= 0 {
            itt_canon1(q1, r1, p1, p2, r2, q2, n1, n2, sp2, sr2, sq2)
        } else {
            itt_canon1(p1, q1, r1, p2, q2, r2, n1, n2, sp2, sq2, sr2)
        }
    } else if sq1 > 0 {
        if sr1 > 0 {
            itt_canon1(p1, q1, r1, p2, r2, q2, n1, n2, sp2, sr2, sq2)
        } else {
            itt_canon1(q1, r1, p1, p2, q2, r2, n1, n2, sp2, sq2, sr2)
        }
    } else if sr1 > 0 {
        itt_canon1(r1, p1, q1, p2, q2, r2, n1, n2, sp2, sq2, sr2)
    } else if sr1 < 0 {
        itt_canon1(r1, p1, q1, p2, r2, q2, n1, n2, sp2, sr2, sq2)
    } else {
        IttValue::coplanar(t2 as i32)
    };
    if ans.kind == IttKind::Coplanar {
        ans.t_source = t2 as i32;
    }
    ans
}

/* ---- CDT_data: input assembly for delaunay_2d ---- */

/// `CDT_data`.
struct CdtData {
    t_plane: usize, /* arena plane index */
    vert: Vec<Mpq2>,
    edge: Vec<[i32; 2]>,
    face_offsets: Vec<usize>,
    face_vert_indices: Vec<i32>,
    input_face: Vec<usize>,
    is_reversed: Vec<bool>,
    cdt_out: CdtResult<Mpq>,
    verts_to_edge: HashMap<(i32, i32), i32>,
    proj_axis: usize,
}

fn prepare_need_vert(cd: &mut CdtData, p3d: &Mpq3) -> i32 {
    let p2d = project_3d_to_2d(p3d, cd.proj_axis);
    cd.vert.push(p2d);
    (cd.vert.len() - 1) as i32
}

/// `unproject_cdt_vert` — lift 2d point back to the 3d plane.
fn unproject_cdt_vert(cd: &CdtData, arena: &IMeshArena, p2d: &Mpq2) -> Mpq3 {
    let plane = &arena.planes[cd.t_plane];
    debug_assert!(plane.exact_populated());
    let n = &plane.norm_exact;
    let d = &plane.d_exact;
    let mut p3d = Mpq3::zero();
    match cd.proj_axis {
        0 => {
            let num = -(n[1].clone() * &p2d.x + n[2].clone() * &p2d.y + d.clone());
            p3d.x = num / &n[0];
            p3d.y = p2d.x.clone();
            p3d.z = p2d.y.clone();
        }
        1 => {
            p3d.x = p2d.x.clone();
            let num = -(n[0].clone() * &p2d.x + n[2].clone() * &p2d.y + d.clone());
            p3d.y = num / &n[1];
            p3d.z = p2d.y.clone();
        }
        _ => {
            p3d.x = p2d.x.clone();
            p3d.y = p2d.y.clone();
            let num = -(n[0].clone() * &p2d.x + n[1].clone() * &p2d.y + d.clone());
            p3d.z = num / &n[2];
        }
    }
    p3d
}

fn prepare_need_edge(cd: &mut CdtData, p1: &Mpq3, p2: &Mpq3) {
    let v1 = prepare_need_vert(cd, p1);
    let v2 = prepare_need_vert(cd, p2);
    cd.edge.push([v1, v2]);
}

fn prepare_need_tri(cd: &mut CdtData, arena: &IMeshArena, tm: &IMesh, t: usize) {
    let tri = &arena.faces[tm.face[t]];
    let tri_plane = &arena.planes[tri.plane.unwrap()];
    let v0 = prepare_need_vert(cd, &arena.verts[tri.vert[0]].co_exact);
    let v1 = prepare_need_vert(cd, &arena.verts[tri.vert[1]].co_exact);
    let v2 = prepare_need_vert(cd, &arena.verts[tri.vert[2]].co_exact);
    let n_axis = &tri_plane.norm_exact[cd.proj_axis];
    let rev = if !n_axis.is_zero() && n_axis.is_positive() {
        cd.proj_axis == 1
    } else {
        cd.proj_axis != 1
    };
    cd.face_offsets.push(cd.face_vert_indices.len());
    cd.face_vert_indices.push(v0);
    if rev {
        cd.face_vert_indices.push(v2);
        cd.face_vert_indices.push(v1);
    } else {
        cd.face_vert_indices.push(v1);
        cd.face_vert_indices.push(v2);
    }
    cd.input_face.push(t);
    cd.is_reversed.push(rev);
}

fn prepare_cdt_input(arena: &IMeshArena, tm: &IMesh, t: usize, itts: &[IttValue]) -> CdtData {
    let face = &arena.faces[tm.face[t]];
    let t_plane = face.plane.unwrap();
    let proj_axis = dominant_axis_mpq(&arena.planes[t_plane].norm_exact);
    let mut ans = CdtData {
        t_plane,
        vert: Vec::new(),
        edge: Vec::new(),
        face_offsets: Vec::new(),
        face_vert_indices: Vec::new(),
        input_face: Vec::new(),
        is_reversed: Vec::new(),
        cdt_out: CdtResult::default(),
        verts_to_edge: HashMap::new(),
        proj_axis,
    };
    prepare_need_tri(&mut ans, arena, tm, t);
    for itt in itts {
        match itt.kind {
            IttKind::None => {}
            IttKind::Point => {
                prepare_need_vert(&mut ans, &itt.p1);
            }
            IttKind::Segment => {
                prepare_need_edge(&mut ans, &itt.p1, &itt.p2);
            }
            IttKind::Coplanar => {
                prepare_need_tri(&mut ans, arena, tm, itt.t_source as usize);
            }
        }
    }
    ans.face_offsets.push(ans.face_vert_indices.len());
    ans
}

fn prepare_cdt_input_for_cluster(
    arena: &IMeshArena,
    tm: &IMesh,
    clinfo: &CoplanarClusterInfo,
    c: usize,
    itts: &[IttValue],
) -> CdtData {
    let cl = clinfo.cluster(c);
    let t0 = cl.tris[0];
    let t_plane = arena.faces[tm.face[t0]].plane.unwrap();
    let proj_axis = dominant_axis_mpq(&arena.planes[t_plane].norm_exact);
    let mut ans = CdtData {
        t_plane,
        vert: Vec::new(),
        edge: Vec::new(),
        face_offsets: Vec::new(),
        face_vert_indices: Vec::new(),
        input_face: Vec::new(),
        is_reversed: Vec::new(),
        cdt_out: CdtResult::default(),
        verts_to_edge: HashMap::new(),
        proj_axis,
    };
    for &t in &cl.tris {
        prepare_need_tri(&mut ans, arena, tm, t);
    }
    for itt in itts {
        match itt.kind {
            IttKind::Point => {
                prepare_need_vert(&mut ans, &itt.p1);
            }
            IttKind::Segment => {
                prepare_need_edge(&mut ans, &itt.p1, &itt.p2);
            }
            _ => {}
        }
    }
    ans.face_offsets.push(ans.face_vert_indices.len());
    ans
}

/// `OrderedEdge` key.
fn ordered_edge(e: [i32; 2]) -> (i32, i32) {
    if e[0] < e[1] {
        (e[0], e[1])
    } else {
        (e[1], e[0])
    }
}

/// `populate_cdt_edge_map`.
fn populate_cdt_edge_map(verts_to_edge: &mut HashMap<(i32, i32), i32>, cdt_out: &CdtResult<Mpq>) {
    verts_to_edge.reserve(cdt_out.edge.len());
    for (e, edge) in cdt_out.edge.iter().enumerate() {
        verts_to_edge.insert(ordered_edge(*edge), e as i32);
    }
}

/// `do_cdt` — run the CDT on mpq2 input.
fn do_cdt(cd: &mut CdtData) {
    let cdt_in = CdtInput::<Mpq> {
        vert: &cd.vert,
        edge: &cd.edge,
        face_offsets: &cd.face_offsets,
        face_vert_indices: &cd.face_vert_indices,
        epsilon: mpq_i(0),
        needed_ids: CDT_ORIG_VERTS | CDT_ORIG_EDGES | CDT_ORIG_FACES,
    };
    cd.cdt_out = delaunay_2d_calc(&cdt_in, crate::delaunay_2d::CdtOutputType::Inside);
    const MAKE_EDGE_MAP_THRESHOLD: usize = 15;
    if cd.cdt_out.edge.len() >= MAKE_EDGE_MAP_THRESHOLD {
        populate_cdt_edge_map(&mut cd.verts_to_edge, &cd.cdt_out);
    }
}

/// `get_cdt_edge_orig` — original edge index for a CDT output edge.
fn get_cdt_edge_orig(
    i0: i32,
    i1: i32,
    cd: &CdtData,
    arena: &IMeshArena,
    r_is_intersect: &mut bool,
) -> i32 {
    let foff = cd.cdt_out.face_edge_offset;
    *r_is_intersect = false;
    let mut e = NO_INDEX;
    if !cd.verts_to_edge.is_empty() {
        e = *cd
            .verts_to_edge
            .get(&ordered_edge([i0, i1]))
            .unwrap_or(&NO_INDEX);
    } else {
        for (ee, edge) in cd.cdt_out.edge.iter().enumerate() {
            if (edge[0] == i0 && edge[1] == i1) || (edge[0] == i1 && edge[1] == i0) {
                e = ee as i32;
                break;
            }
        }
    }
    if e == NO_INDEX {
        return NO_INDEX;
    }
    let mut face_eorig = NO_INDEX;
    let mut have_non_face_eorig = false;
    for &orig_index in &cd.cdt_out.edge_orig[e as usize] {
        if orig_index >= foff {
            if face_eorig == NO_INDEX {
                let in_face_index = (orig_index / foff) - 1;
                let pos = (orig_index % foff) as usize;
                let in_tm_face_index = cd.input_face[in_face_index as usize];
                let facep = &arena.faces[in_tm_face_index];
                debug_assert!(pos < facep.size());
                let is_rev = cd.is_reversed[in_face_index as usize];
                let eorig = if is_rev {
                    facep.edge_orig[2 - pos]
                } else {
                    facep.edge_orig[pos]
                };
                if eorig != NO_INDEX {
                    face_eorig = eorig;
                }
            }
        } else {
            have_non_face_eorig = true;
            if face_eorig != NO_INDEX && have_non_face_eorig {
                break;
            }
        }
    }
    if face_eorig != NO_INDEX {
        return face_eorig;
    }
    if have_non_face_eorig {
        *r_is_intersect = true;
        return NO_INDEX;
    }
    NO_INDEX
}

/// `cdt_tri_as_imesh_face`.
fn cdt_tri_as_imesh_face(
    cdt_out_t: usize,
    cdt_in_t: usize,
    cd: &CdtData,
    arena: &mut IMeshArena,
    tm: &IMesh,
) -> usize {
    let t_orig = arena.faces[cd.input_face[cdt_in_t]].orig;
    debug_assert_eq!(cd.cdt_out.face[cdt_out_t].len(), 3);
    let i0 = cd.cdt_out.face[cdt_out_t][0];
    let i1 = cd.cdt_out.face[cdt_out_t][1];
    let i2 = cd.cdt_out.face[cdt_out_t][2];
    let v0co = unproject_cdt_vert(cd, arena, &cd.cdt_out.vert[i0 as usize]);
    let v1co = unproject_cdt_vert(cd, arena, &cd.cdt_out.vert[i1 as usize]);
    let v2co = unproject_cdt_vert(cd, arena, &cd.cdt_out.vert[i2 as usize]);
    let v0 = arena.add_or_find_vert_mpq(&v0co, NO_INDEX);
    let v1 = arena.add_or_find_vert_mpq(&v1co, NO_INDEX);
    let v2 = arena.add_or_find_vert_mpq(&v2co, NO_INDEX);
    let mut is_isect0 = false;
    let mut is_isect1 = false;
    let mut is_isect2 = false;
    let facep;
    if cd.is_reversed[cdt_in_t] {
        let oe0 = get_cdt_edge_orig(i0, i2, cd, arena, &mut is_isect0);
        let oe1 = get_cdt_edge_orig(i2, i1, cd, arena, &mut is_isect1);
        let oe2 = get_cdt_edge_orig(i1, i0, cd, arena, &mut is_isect2);
        facep = arena.add_face_full(
            vec![v0, v2, v1],
            t_orig,
            vec![oe0, oe1, oe2],
            vec![is_isect0, is_isect1, is_isect2],
        );
    } else {
        let oe0 = get_cdt_edge_orig(i0, i1, cd, arena, &mut is_isect0);
        let oe1 = get_cdt_edge_orig(i1, i2, cd, arena, &mut is_isect1);
        let oe2 = get_cdt_edge_orig(i2, i0, cd, arena, &mut is_isect2);
        facep = arena.add_face_full(
            vec![v0, v1, v2],
            t_orig,
            vec![oe0, oe1, oe2],
            vec![is_isect0, is_isect1, is_isect2],
        );
    }
    let mut f = arena.faces[facep].clone();
    f.populate_plane(arena, false);
    arena.faces[facep] = f;
    facep
}

/// `is_quad_flip_v3_first_third_fast` on double3.
fn is_quad_flip_first_third(v1: [f64; 3], v2: [f64; 3], v3: [f64; 3], v4: [f64; 3]) -> bool {
    let d_12 = sub_d(v2, v1);
    let d_13 = sub_d(v3, v1);
    let d_14 = sub_d(v4, v1);
    let cross_a = cross_d(d_12, d_13);
    let cross_b = cross_d(d_14, d_13);
    dot_d(cross_a, cross_b) > 0.0
}

/// `polyfill_triangulate_poly` — fast polyfill triangulation.
fn polyfill_triangulate_poly(f: usize, arena: &mut IMeshArena) -> Vec<usize> {
    let flen = arena.faces[f].size();
    debug_assert!(flen >= 4);
    if arena.faces[f].plane.is_none() {
        let mut fc = arena.faces[f].clone();
        fc.populate_plane(arena, false);
        arena.faces[f] = fc;
    }
    let poly_normal = arena.planes[arena.faces[f].plane.unwrap()].norm;
    let mut no = [
        poly_normal[0] as f32,
        poly_normal[1] as f32,
        poly_normal[2] as f32,
    ];
    blenders_math::vector_inline::normalize_v3(&mut no);
    if flen == 4 {
        let v0 = arena.faces[f].vert[0];
        let v1 = arena.faces[f].vert[1];
        let v2 = arena.faces[f].vert[2];
        let v3 = arena.faces[f].vert[3];
        let eo_01 = arena.faces[f].edge_orig[0];
        let eo_12 = arena.faces[f].edge_orig[1];
        let eo_23 = arena.faces[f].edge_orig[2];
        let eo_30 = arena.faces[f].edge_orig[3];
        let orig = arena.faces[f].orig;
        let (f0, f1);
        if is_quad_flip_first_third(
            arena.verts[v0].co,
            arena.verts[v1].co,
            arena.verts[v2].co,
            arena.verts[v3].co,
        ) {
            f0 = arena.add_face_full(
                vec![v0, v1, v3],
                orig,
                vec![eo_01, -1, eo_30],
                vec![false; 3],
            );
            f1 = arena.add_face_full(
                vec![v1, v2, v3],
                orig,
                vec![eo_12, eo_23, -1],
                vec![false; 3],
            );
        } else {
            f0 = arena.add_face_full(
                vec![v0, v1, v2],
                orig,
                vec![eo_01, eo_12, -1],
                vec![false; 3],
            );
            f1 = arena.add_face_full(
                vec![v0, v2, v3],
                orig,
                vec![-1, eo_23, eo_30],
                vec![false; 3],
            );
        }
        return vec![f0, f1];
    }
    /* Project along negative face normal. */
    let axis_mat = axis_dominant_to_m3_negate(blenders_math::vector::Vec3::new(
        no[0], no[1], no[2],
    ));
    let mut projverts: Vec<[f32; 2]> = Vec::with_capacity(flen);
    for j in 0..flen {
        let dco = arena.verts[arena.faces[f].vert[j]].co;
        let co = blenders_math::vector::Vec3::new(
            dco[0] as f32,
            dco[1] as f32,
            dco[2] as f32,
        );
        /* `mul_v2_m3v3`: project 3d co to 2d with 3x3 axis matrix. */
        let r = axis_mat * co;
        projverts.push([r.x, r.y]);
    }
    let tris = polyfill_calc_pub(&projverts, 0);
    let mut ans = Vec::with_capacity(tris.len());
    for tri in &tris {
        let mut eo = [NO_INDEX; 3];
        let mut v = [0usize; 3];
        for k in 0..3 {
            v[k] = arena.faces[f].vert[tri[k] as usize];
            if (tri[k] + 1) % (flen as u32) == tri[(k + 1) % 3] {
                eo[k] = arena.faces[f].edge_orig[tri[k] as usize];
            }
        }
        let orig = arena.faces[f].orig;
        let nf = arena.add_face_full(
            v.to_vec(),
            orig,
            eo.to_vec(),
            vec![false; 3],
        );
        ans.push(nf);
    }
    ans
}

/// `exact_triangulate_poly` — CDT-based triangulation, polyfill fallback
/// when Steiner verts appear.
fn exact_triangulate_poly(f: usize, arena: &mut IMeshArena) -> Vec<usize> {
    let flen = arena.faces[f].size();
    if arena.faces[f].plane.is_none() {
        let mut fc = arena.faces[f].clone();
        fc.populate_plane(arena, false);
        arena.faces[f] = fc;
    }
    let poly_normal = arena.planes[arena.faces[f].plane.unwrap()].norm;
    let axis = dominant_axis_d(poly_normal);
    let rev1 = axis == 1;
    let rev2 = poly_normal[axis] < 0.0;
    let rev = rev1 ^ rev2;
    let mut in_verts: Vec<Mpq2> = vec![Mpq2::zero(); flen];
    for i in 0..flen {
        let ii = if rev { flen - i - 1 } else { i };
        let co_e = &arena.verts[arena.faces[f].vert[ii]].co_exact;
        let mut k = 0;
        let mut p2d = Mpq2::zero();
        for j in 0..3 {
            if j != axis {
                if k == 0 {
                    p2d.x = co_e[j].clone();
                } else {
                    p2d.y = co_e[j].clone();
                }
                k += 1;
            }
        }
        in_verts[ii] = p2d;
    }
    let face_offsets: Vec<usize> = vec![0, flen];
    let face_vert_indices: Vec<i32> = (0..flen as i32).collect();
    let empty_edges: Vec<[i32; 2]> = Vec::new();
    let cdt_in = CdtInput::<Mpq> {
        vert: &in_verts,
        edge: &empty_edges,
        face_offsets: &face_offsets,
        face_vert_indices: &face_vert_indices,
        epsilon: mpq_i(0),
        needed_ids: CDT_ORIG_VERTS | CDT_ORIG_EDGES | CDT_ORIG_FACES,
    };
    let cdt_out = delaunay_2d_calc(&cdt_in, crate::delaunay_2d::CdtOutputType::Inside);
    let n_tris = cdt_out.face.len();
    let mut ans = Vec::with_capacity(n_tris);
    for t in 0..n_tris {
        let mut i_v_out = [0i32; 3];
        let mut v = [0usize; 3];
        let mut eo = [NO_INDEX; 3];
        let mut needs_steiner = false;
        for i in 0..3 {
            i_v_out[i] = cdt_out.face[t][i];
            if cdt_out.vert_orig[i_v_out[i] as usize].is_empty() {
                needs_steiner = true;
                break;
            }
            v[i] = arena.faces[f].vert[cdt_out.vert_orig[i_v_out[i] as usize][0] as usize];
        }
        if needs_steiner {
            return polyfill_triangulate_poly(f, arena);
        }
        let mut verts_to_edge = HashMap::new();
        populate_cdt_edge_map(&mut verts_to_edge, &cdt_out);
        let foff = cdt_out.face_edge_offset;
        for i in 0..3 {
            let e_out = *verts_to_edge
                .get(&ordered_edge([i_v_out[i], i_v_out[(i + 1) % 3]]))
                .unwrap_or(&NO_INDEX);
            debug_assert!(e_out != NO_INDEX);
            eo[i] = NO_INDEX;
            for &orig in &cdt_out.edge_orig[e_out as usize] {
                if orig >= foff {
                    let pos = (orig % foff) as usize;
                    debug_assert!(pos < arena.faces[f].size());
                    eo[i] = arena.faces[f].edge_orig[pos];
                    break;
                }
            }
        }
        let orig = arena.faces[f].orig;
        if rev {
            ans.push(arena.add_face_full(
                vec![v[0], v[2], v[1]],
                orig,
                vec![eo[2], eo[1], eo[0]],
                vec![false; 3],
            ));
        } else {
            ans.push(arena.add_face_full(
                vec![v[0], v[1], v[2]],
                orig,
                vec![eo[0], eo[1], eo[2]],
                vec![false; 3],
            ));
        }
    }
    ans
}

/// `face_is_degenerate`.
fn face_is_degenerate(arena: &IMeshArena, f: usize) -> bool {
    let face = &arena.faces[f];
    let v0 = face.vert[0];
    let v1 = face.vert[1];
    let v2 = face.vert[2];
    if v0 == v1 || v0 == v2 || v1 == v2 {
        return true;
    }
    let da = sub_d(arena.verts[v2].co, arena.verts[v0].co);
    let db = sub_d(arena.verts[v2].co, arena.verts[v1].co);
    let dab = cross_d(da, db);
    let dab_length_squared = dot_d(dab, dab);
    let err_bound = supremum_dot_cross(dab, dab) * (INDEX_DOT_CROSS as f64) * DBL_EPSILON;
    if dab_length_squared > err_bound {
        return false;
    }
    let a = &arena.verts[v2].co_exact - &arena.verts[v0].co_exact;
    let b = &arena.verts[v2].co_exact - &arena.verts[v1].co_exact;
    let ab = Mpq3::cross(&a, &b);
    ab.x.is_zero() && ab.y.is_zero() && ab.z.is_zero()
}

/// `any_degenerate_tris_fast`.
fn any_degenerate_tris_fast(arena: &IMeshArena, triangulation: &[usize]) -> bool {
    for &f in triangulation {
        let face = &arena.faces[f];
        let v0 = face.vert[0];
        let v1 = face.vert[1];
        let v2 = face.vert[2];
        if v0 == v1 || v0 == v2 || v1 == v2 {
            return true;
        }
        let da = sub_d(arena.verts[v2].co, arena.verts[v0].co);
        let db = sub_d(arena.verts[v2].co, arena.verts[v1].co);
        let da_length_squared = dot_d(da, da);
        let db_length_squared = dot_d(db, db);
        if da_length_squared == 0.0 || db_length_squared == 0.0 {
            return true;
        }
        let dab = cross_d(da, db);
        let dab_length_squared = dot_d(dab, dab);
        let sin_squared_t = dab_length_squared / (da_length_squared * db_length_squared);
        if sin_squared_t < 1e-8 {
            return true;
        }
    }
    false
}

/// `triangulate_poly` — polyfill first, exact CDT if degenerate tris appear.
fn triangulate_poly(f: usize, arena: &mut IMeshArena) -> Vec<usize> {
    let ans = polyfill_triangulate_poly(f, arena);
    if any_degenerate_tris_fast(arena, &ans) {
        return exact_triangulate_poly(f, arena);
    }
    ans
}

/// `triangulate_polymesh` — public: tessellate all faces into triangles.
pub fn triangulate_polymesh(arena: &mut IMeshArena, imesh: &IMesh) -> IMesh {
    let mut face_tris: Vec<usize> = Vec::with_capacity(3 * imesh.face_size());
    for &f in &imesh.face {
        let flen = arena.faces[f].size();
        if flen >= 4 && arena.faces[f].plane.is_none() {
            let mut fc = arena.faces[f].clone();
            fc.populate_plane(arena, false);
            arena.faces[f] = fc;
        }
        if flen == 3 {
            face_tris.push(f);
        } else {
            for tri in triangulate_poly(f, arena) {
                face_tris.push(tri);
            }
        }
    }
    IMesh::from_faces(face_tris)
}

/// `extract_subdivided_tri`.
fn extract_subdivided_tri(
    cd: &CdtData,
    arena: &mut IMeshArena,
    tm: &IMesh,
    t: usize,
) -> IMesh {
    let mut t_in_cdt = -1i32;
    for (i, &inp) in cd.input_face.iter().enumerate() {
        if inp == t {
            t_in_cdt = i as i32;
        }
    }
    if t_in_cdt == -1 {
        debug_assert!(false, "Could not find {t} in cdt input tris");
        return IMesh::new();
    }
    let mut faces = Vec::new();
    for (f, orig_set) in cd.cdt_out.face_orig.iter().enumerate() {
        if orig_set.contains(&(t_in_cdt as u32)) {
            faces.push(cdt_tri_as_imesh_face(f, t_in_cdt as usize, cd, arena, tm));
        }
    }
    IMesh::from_faces(faces)
}

/// `TriOverlaps` — BVH overlap machinery (serial port).
struct TriOverlaps {
    overlap: Vec<BVHTreeOverlap>,
    first_overlap: Vec<i32>,
}
impl TriOverlaps {
    fn new(
        arena: &IMeshArena,
        tm: &IMesh,
        tri_bb: &[BoundingBox],
        nshapes: i32,
        shape_fn: &dyn Fn(i32) -> i32,
        use_self: bool,
    ) -> Self {
        /* tree_type 8 (octree), axis 6 (XYZ). */
        let mut tree = bvhtree_new(tm.face_size() as i32, f32::EPSILON, 8, 6);
        let two_trees_no_self = nshapes == 2 && !use_self;
        let mut tree_b = two_trees_no_self.then(|| {
            bvhtree_new(tm.face_size() as i32, f32::EPSILON, 8, 6)
        });
        let shapes: Vec<i32> = (0..tm.face_size())
            .map(|t| shape_fn(arena.faces[tm.face[t]].orig))
            .collect();
        for t in 0..tm.face_size() {
            let bb = &tri_bb[t];
            let bbpts = [
                bb.min[0] as f32,
                bb.max[0] as f32,
                bb.min[1] as f32,
                bb.max[1] as f32,
                bb.min[2] as f32,
                bb.max[2] as f32,
            ];
            let shape = shapes[t];
            if two_trees_no_self {
                if shape == 0 {
                    bvhtree_insert(&mut tree, t as i32, &bbpts, 2);
                } else if shape == 1 {
                    bvhtree_insert(tree_b.as_mut().unwrap(), t as i32, &bbpts, 2);
                }
            } else if shape != -1 {
                bvhtree_insert(&mut tree, t as i32, &bbpts, 2);
            }
        }
        bvhtree_balance(&mut tree);
        let mut overlap;
        if two_trees_no_self {
            let tb = tree_b.as_mut().unwrap();
            bvhtree_balance(tb);
            overlap = bvhtree_overlap(&tree, tb, None);
            /* Repeat with indexA/indexB reversed. */
            let n = overlap.len();
            for i in 0..n {
                overlap.push(BVHTreeOverlap {
                    index_a: overlap[i].index_b,
                    index_b: overlap[i].index_a,
                });
            }
        } else if nshapes == 1 {
            overlap = bvhtree_overlap_self(&tree, None);
        } else {
            /* only_different_shapes callback. */
            let tm_faces: Vec<i32> = (0..tm.face_size())
                .map(|t| arena.faces[tm.face[t]].orig)
                .collect();
            let mut cb = |a: i32, b: i32, _thread: i32| -> bool {
                tm_faces[a as usize] != tm_faces[b as usize]
            };
            overlap = bvhtree_overlap(&tree, &tree, Some(&mut cb));
        }
        overlap.sort_by(|a, b| {
            a.index_a
                .cmp(&b.index_a)
                .then(a.index_b.cmp(&b.index_b))
        });
        let mut first_overlap = vec![-1i32; tm.face_size()];
        for (i, ov) in overlap.iter().enumerate() {
            if first_overlap[ov.index_a as usize] == -1 {
                first_overlap[ov.index_a as usize] = i as i32;
            }
        }
        TriOverlaps {
            overlap,
            first_overlap,
        }
    }
    fn first_overlap_index(&self, t: usize) -> i32 {
        self.first_overlap[t]
    }
}

fn canon_int_pair(a: usize, b: usize) -> (usize, usize) {
    if a <= b {
        (a, b)
    } else {
        (b, a)
    }
}

/// `calc_overlap_itts` (serial).
fn calc_overlap_itts(
    itt_map: &mut HashMap<(usize, usize), IttValue>,
    arena: &IMeshArena,
    tm: &IMesh,
    ov: &TriOverlaps,
) {
    let mut intersect_pairs: Vec<(usize, usize)> = Vec::new();
    for olap in &ov.overlap {
        let key = canon_int_pair(olap.index_a as usize, olap.index_b as usize);
        if !itt_map.contains_key(&key) {
            itt_map.insert(key, IttValue::none());
            intersect_pairs.push(key);
        }
    }
    for (a, b) in intersect_pairs {
        let itt = intersect_tri_tri(arena, tm, a, b);
        itt_map.insert((a, b), itt);
    }
}

/// `calc_subdivided_non_cluster_tris` (serial).
fn calc_subdivided_non_cluster_tris(
    r_tri_subdivided: &mut Vec<IMesh>,
    arena: &mut IMeshArena,
    tm: &IMesh,
    itt_map: &HashMap<(usize, usize), IttValue>,
    clinfo: &CoplanarClusterInfo,
    ov: &TriOverlaps,
) {
    let overlap = &ov.overlap;
    let overlap_num = overlap.len();
    let mut overlap_tri_range: Vec<(usize, usize, usize)> = Vec::with_capacity(overlap_num);
    let mut overlap_index = 0usize;
    while overlap_index < overlap_num {
        let t = overlap[overlap_index].index_a as usize;
        let mut i = overlap_index;
        while i + 1 < overlap_num && overlap[i + 1].index_a as usize == t {
            i += 1;
        }
        if clinfo.tri_cluster(t) == NO_INDEX {
            let len = i - overlap_index + 1;
            if !(len == 1 && overlap[overlap_index].index_b as usize == t) {
                overlap_tri_range.push((t, overlap_index, len));
            }
        }
        overlap_index = i + 1;
    }
    for &(t, start, len) in &overlap_tri_range {
        let mut itts: Vec<IttValue> = Vec::with_capacity(len);
        for j in start..start + len {
            let t_other = overlap[j].index_b as usize;
            let key = canon_int_pair(t, t_other);
            if let Some(itt) = itt_map.get(&key) {
                if itt.kind != IttKind::None {
                    itts.push(itt.clone());
                }
            }
        }
        if !itts.is_empty() {
            let mut cd = prepare_cdt_input(arena, tm, t, &itts);
            do_cdt(&mut cd);
            r_tri_subdivided[t] = extract_subdivided_tri(&cd, arena, tm, t);
        }
    }
    /* Triangles with no subdivision and not in clusters stay as-is. */
    for t in 0..tm.face_size() {
        if r_tri_subdivided[t].face.is_empty() && clinfo.tri_cluster(t) == NO_INDEX {
            r_tri_subdivided[t] = IMesh::from_faces(vec![tm.face[t]]);
        }
    }
}

/// `calc_cluster_tris`.
fn calc_cluster_tris(
    tri_subdivided: &mut Vec<IMesh>,
    arena: &mut IMeshArena,
    tm: &IMesh,
    clinfo: &CoplanarClusterInfo,
    cluster_subdivided: &[CdtData],
) {
    for c in 0..clinfo.tot_cluster() {
        let cl = clinfo.cluster(c);
        let cd = &cluster_subdivided[c];
        let n_cluster_tris = cl.tot_tri();
        let cdt_out = &cd.cdt_out;
        debug_assert_eq!(cd.input_face.len(), n_cluster_tris);
        let mut face_vec: Vec<Vec<usize>> = vec![Vec::new(); n_cluster_tris];
        for cdt_out_t in 0..cdt_out.face.len() {
            for &cdt_in_t in &cdt_out.face_orig[cdt_out_t] {
                let f = cdt_tri_as_imesh_face(
                    cdt_out_t,
                    cdt_in_t as usize,
                    cd,
                    arena,
                    tm,
                );
                face_vec[cdt_in_t as usize].push(f);
            }
        }
        for cdt_in_t in 0..cd.input_face.len() {
            let tm_t = cd.input_face[cdt_in_t];
            debug_assert!(tri_subdivided[tm_t].face.is_empty());
            tri_subdivided[tm_t] = IMesh::from_faces(face_vec[cdt_in_t].clone());
        }
    }
}

/// `calc_cluster_subdivided`.
fn calc_cluster_subdivided(
    clinfo: &CoplanarClusterInfo,
    c: usize,
    arena: &IMeshArena,
    tm: &IMesh,
    ov: &TriOverlaps,
    itt_map: &HashMap<(usize, usize), IttValue>,
) -> CdtData {
    let cl = clinfo.cluster(c);
    let mut itts: Vec<IttValue> = Vec::new();
    for &t in &cl.tris {
        let first_i = ov.first_overlap_index(t);
        if first_i == -1 {
            continue;
        }
        let mut i = first_i as usize;
        while i < ov.overlap.len() && ov.overlap[i].index_a as usize == t {
            let t_other = ov.overlap[i].index_b as usize;
            if clinfo.tri_cluster(t_other) != c as i32 {
                let key = canon_int_pair(t, t_other);
                if let Some(itt) = itt_map.get(&key) {
                    if itt.kind != IttKind::None && itt.kind != IttKind::Coplanar {
                        itts.push(itt.clone());
                    }
                }
            }
            i += 1;
        }
    }
    let mut cd_data = prepare_cdt_input_for_cluster(arena, tm, clinfo, c, &itts);
    do_cdt(&mut cd_data);
    cd_data
}

/// `union_tri_subdivides`.
fn union_tri_subdivides(tri_subdivided: &[IMesh]) -> IMesh {
    let tot_tri: usize = tri_subdivided.iter().map(|m| m.face.len()).sum();
    let mut faces = Vec::with_capacity(tot_tri);
    for m in tri_subdivided {
        faces.extend_from_slice(&m.face);
    }
    IMesh::from_faces(faces)
}

/// `find_clusters` — coplanar cluster detection.
fn find_clusters(
    arena: &IMeshArena,
    tm: &IMesh,
    tri_bb: &[BoundingBox],
    itt_map: &HashMap<(usize, usize), IttValue>,
) -> CoplanarClusterInfo {
    let mut ans = CoplanarClusterInfo::new(tm.face_size());
    /* Stable order via sorted unique set. */
    let mut maybe_coplanar_tris: Vec<usize> = Vec::with_capacity(2 * itt_map.len());
    for (key, value) in itt_map {
        if value.kind == IttKind::Coplanar {
            maybe_coplanar_tris.push(key.0);
            maybe_coplanar_tris.push(key.1);
        }
    }
    maybe_coplanar_tris.sort_unstable();
    maybe_coplanar_tris.dedup();
    if maybe_coplanar_tris.is_empty() {
        return ans;
    }
    /* plane bucket -> clusters. Key on canonical plane via hash buckets. */
    let mut plane_hash: HashMap<u64, Vec<usize>> = HashMap::new();
    let mut plane_vals: Vec<Plane> = Vec::new();
    let mut plane_cls: Vec<Vec<CoplanarCluster>> = Vec::new();
    for &t in &maybe_coplanar_tris {
        let mut tplane = arena.planes[arena.faces[tm.face[t]].plane.unwrap()].clone();
        tplane.make_canonical();
        let h = tplane.hash();
        let mut pidx = usize::MAX;
        if let Some(cands) = plane_hash.get(&h) {
            for &ci in cands {
                if plane_vals[ci].eq_plane(&tplane) {
                    pidx = ci;
                    break;
                }
            }
        }
        if pidx == usize::MAX {
            pidx = plane_vals.len();
            plane_vals.push(tplane);
            plane_hash.entry(h).or_default().push(pidx);
            plane_cls.push(Vec::new());
        }
        let curcls = &mut plane_cls[pidx];
        /* Partition curcls into intersecting vs non-intersecting. */
        let mut int_idx: Vec<usize> = Vec::new();
        for (i, cl) in curcls.iter().enumerate() {
            if bbs_might_intersect(&tri_bb[t], &cl.bounding_box()) {
                int_idx.push(i);
            }
        }
        if int_idx.is_empty() {
            let mut cl = CoplanarCluster::default();
            cl.add_tri(t, tri_bb[t]);
            curcls.push(cl);
        } else if int_idx.len() == 1 {
            curcls[int_idx[0]].add_tri(t, tri_bb[t]);
        } else {
            /* Merge t + all intersecting clusters. */
            let mut mergecl = CoplanarCluster::default();
            mergecl.add_tri(t, tri_bb[t]);
            for &i in &int_idx {
                for &t2 in &curcls[i].tris {
                    mergecl.add_tri(t2, tri_bb[t2]);
                }
            }
            let mut newvec = Vec::with_capacity(curcls.len());
            newvec.push(mergecl);
            for (i, cl) in curcls.iter().enumerate() {
                if !int_idx.contains(&i) {
                    newvec.push(cl.clone());
                }
            }
            *curcls = newvec;
        }
    }
    for cls in &plane_cls {
        for cl in cls {
            if cl.tot_tri() > 1 {
                ans.add_cluster(cl.clone());
            }
        }
    }
    ans
}

/// `has_degenerate_tris` (serial).
fn has_degenerate_tris(arena: &IMeshArena, tm: &IMesh) -> bool {
    (0..tm.face_size()).any(|t| face_is_degenerate(arena, tm.face[t]))
}

/// `remove_degenerate_tris`.
fn remove_degenerate_tris(arena: &IMeshArena, tm_in: &IMesh) -> IMesh {
    let mut new_faces = Vec::with_capacity(tm_in.face_size());
    for &f in &tm_in.face {
        if !face_is_degenerate(arena, f) {
            new_faces.push(f);
        }
    }
    IMesh::from_faces(new_faces)
}

/// `trimesh_self_intersect`.
pub fn trimesh_self_intersect(arena: &mut IMeshArena, tm_in: &IMesh) -> IMesh {
    trimesh_nary_intersect(arena, tm_in, 1, &|_t| 0, true)
}

/// `trimesh_nary_intersect` — the main entry point.
pub fn trimesh_nary_intersect(
    arena: &mut IMeshArena,
    tm_in: &IMesh,
    nshapes: i32,
    shape_fn: &dyn Fn(i32) -> i32,
    use_self: bool,
) -> IMesh {
    let tm_cleaned;
    let tm_clean: &IMesh = if has_degenerate_tris(arena, tm_in) {
        tm_cleaned = remove_degenerate_tris(arena, tm_in);
        &tm_cleaned
    } else {
        tm_in
    };
    let tri_bb = calc_face_bounding_boxes(arena, tm_clean);
    let tri_ov = TriOverlaps::new(arena, tm_clean, &tri_bb, nshapes, shape_fn, use_self);
    let mut tri_subdivided: Vec<IMesh> = Vec::with_capacity(tm_clean.face_size());
    for _ in 0..tm_clean.face_size() {
        tri_subdivided.push(IMesh::new());
    }
    for t in 0..tm_clean.face_size() {
        if tri_ov.first_overlap_index(t) != -1 {
            let mut fc = arena.faces[tm_clean.face[t]].clone();
            fc.populate_plane(arena, true);
            arena.faces[tm_clean.face[t]] = fc;
        }
    }
    /* itt_map[(a,b)] = intersection of tris a,b (a<b). */
    let mut itt_map: HashMap<(usize, usize), IttValue> =
        HashMap::with_capacity(tri_ov.overlap.len());
    calc_overlap_itts(&mut itt_map, arena, tm_clean, &tri_ov);
    let clinfo = find_clusters(arena, tm_clean, &tri_bb, &itt_map);
    calc_subdivided_non_cluster_tris(
        &mut tri_subdivided,
        arena,
        tm_clean,
        &itt_map,
        &clinfo,
        &tri_ov,
    );
    let mut cluster_subdivided: Vec<CdtData> = Vec::with_capacity(clinfo.tot_cluster());
    for c in 0..clinfo.tot_cluster() {
        cluster_subdivided.push(calc_cluster_subdivided(
            &clinfo, c, arena, tm_clean, &tri_ov, &itt_map,
        ));
    }
    calc_cluster_tris(&mut tri_subdivided, arena, tm_clean, &clinfo, &cluster_subdivided);
    union_tri_subdivides(&tri_subdivided)
}

/// `write_obj_mesh` — debug .obj writer (`/tmp/` on unix).
pub fn write_obj_mesh(arena: &IMeshArena, m: &mut IMesh, objname: &str) {
    if m.face_size() == 0 {
        return;
    }
    let objdir = "/tmp/";
    if !m.has_verts() {
        m.populate_vert(arena);
    }
    let mut s = String::new();
    for &v in &m.vert {
        let dv = arena.verts[v].co;
        s.push_str(&format!("v {} {} {}\n", dv[0], dv[1], dv[2]));
    }
    for &fi in &m.face {
        let face = &arena.faces[fi];
        s.push_str("f ");
        for &v in &face.vert {
            let i = m.lookup_vert(v);
            s.push_str(&format!("{} ", i + 1));
        }
        s.push('\n');
    }
    let _ = std::fs::write(format!("{}{}{}", objdir, objname, ".obj"), s);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Build a tetrahedron-vs-tetrahedron scenario: two overlapping triangles
    /// in different planes should produce a segment intersection.
    #[test]
    fn trimesh_self_intersect_crossing_tris() {
        let mut arena = IMeshArena::new();
        /* Tri 1 in z=0 plane, tri 2 crossing through it. */
        let a = arena.add_or_find_vert_d([0.0, 0.0, 0.0], 0);
        let b = arena.add_or_find_vert_d([2.0, 0.0, 0.0], 0);
        let c = arena.add_or_find_vert_d([0.0, 2.0, 0.0], 0);
        let f0 = arena.add_face(vec![a, b, c], 0);
        let d = arena.add_or_find_vert_d([0.5, 0.5, -1.0], 1);
        let e = arena.add_or_find_vert_d([0.5, 0.5, 1.0], 1);
        let g = arena.add_or_find_vert_d([1.5, 0.5, 1.0], 1);
        let f1 = arena.add_face(vec![d, e, g], 1);
        let tm = IMesh::from_faces(vec![f0, f1]);
        let out = trimesh_self_intersect(&mut arena, &tm);
        /* f0 should be subdivided (more faces than input). */
        assert!(out.face_size() >= 2);
        /* Every output face should be a triangle. */
        for &fi in &out.face {
            assert_eq!(arena.faces[fi].size(), 3);
        }
    }

    #[test]
    fn triangulate_polymesh_quad() {
        let mut arena = IMeshArena::new();
        let a = arena.add_or_find_vert_d([0.0, 0.0, 0.0], 0);
        let b = arena.add_or_find_vert_d([1.0, 0.0, 0.0], 1);
        let c = arena.add_or_find_vert_d([1.0, 1.0, 0.0], 2);
        let d = arena.add_or_find_vert_d([0.0, 1.0, 0.0], 3);
        let f = arena.add_face(vec![a, b, c, d], 0);
        let tm = IMesh::from_faces(vec![f]);
        let out = triangulate_polymesh(&mut arena, &tm);
        assert_eq!(out.face_size(), 2);
        for &fi in &out.face {
            assert_eq!(arena.faces[fi].size(), 3);
        }
    }
}
