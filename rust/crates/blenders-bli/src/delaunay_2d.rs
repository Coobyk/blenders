//! Port of `delaunay_2d.cc` — constrained Delaunay triangulation (2D CDT).
//!
//! Double specialization only (`T = double`); the `mpq_class`/`WITH_GMP`
//! specialization is cfg-gated upstream and marked PORT_PENDING.
//! Pointer graph -> index arenas: `SymEdge` ids are `edge_id * 2 + side`,
//! `CDTVert`/`CDTEdge`/`CDTFace` are `usize` arena indices, `NIL` is -1/none.
//! `VectorSet<uint32_t>` input_ids -> sorted-unique `Vec<u32>` (`IdSet`).
//! Filtered exact predicates -> `robust` crate (the same Shewchuk
//! arithmetic upstream uses via `orient2d(double2,...)`/`incircle(...)`).

pub const NIL: usize = usize::MAX;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CdtOutputType {
    Full,
    Inside,
    InsideWithHoles,
    InsideWithHolesNonzero,
    Constraints,
    ConstraintsValidBmesh,
    ConstraintsValidBmeshWithHoles,
    ConstraintsValidBmeshWithHolesNonzero,
}

pub const CDT_NO_ORIG_IDS: u32 = 0;
pub const CDT_ORIG_VERTS: u32 = 1 << 0;
pub const CDT_INTERSECTED_EDGES: u32 = 1 << 1;
pub const CDT_ORIG_EDGES: u32 = 1 << 2;
pub const CDT_ORIG_FACES: u32 = 1 << 3;
pub const CDT_CW_ORIG_FACES: u32 = 1 << 4;
pub const CDT_ONLY_ONE_ORIG: u32 = 1 << 5;

fn output_uses_evenodd_holes(t: CdtOutputType) -> bool {
    matches!(
        t,
        CdtOutputType::InsideWithHoles | CdtOutputType::ConstraintsValidBmeshWithHoles
    )
}
fn output_uses_nonzero_holes(t: CdtOutputType) -> bool {
    matches!(
        t,
        CdtOutputType::InsideWithHolesNonzero | CdtOutputType::ConstraintsValidBmeshWithHolesNonzero
    )
}

/// `CDT_input`, generic over scalar T (upstream `CDT_input<T>`):
/// `f64` -> `CDT_input<double>`; `BigRational` -> `CDT_input<mpq_class>`.
pub struct CdtInput<'a, S: CdtScalar> {
    pub vert: &'a [S::Co],
    pub edge: &'a [[i32; 2]],
    /// `OffsetIndices` — face f spans `face_vert_indices[offsets[f]..offsets[f+1]]`.
    pub face_offsets: &'a [usize],
    pub face_vert_indices: &'a [i32],
    pub epsilon: S,
    pub needed_ids: u32,
}

/// `CDT_result`.
pub struct CdtResult<S: CdtScalar> {
    pub vert: Vec<S::Co>,
    pub edge: Vec<[i32; 2]>,
    pub face: Vec<Vec<i32>>,
    pub vert_orig: Vec<Vec<u32>>,
    pub edge_orig: Vec<Vec<u32>>,
    pub intersected_edges_orig: Vec<[i32; 2]>,
    pub face_orig: Vec<Vec<u32>>,
    pub face_edge_offset: u32,
}
impl<S: CdtScalar> Default for CdtResult<S> {
    fn default() -> Self {
        CdtResult {
            vert: Vec::new(),
            edge: Vec::new(),
            face: Vec::new(),
            vert_orig: Vec::new(),
            edge_orig: Vec::new(),
            intersected_edges_orig: Vec::new(),
            face_orig: Vec::new(),
            face_edge_offset: 0,
        }
    }
}

/// Scalar + 2-vector types behind `template<typename T>`:
/// `f64` -> `VecBase<double,2>` = `[f64;2]`; `Mpq` -> `mpq2` = `Mpq2`.
pub trait CdtScalar: Clone + PartialEq + std::fmt::Debug {
    type Co: Clone + PartialEq + Default + std::fmt::Debug;
    /// `co.approx` — double approximation for FatCo.
    fn approx(c: &Self::Co) -> [f64; 2];
    /// `orient2d` on exact coords (robust/Shewchuk for f64, exact det for mpq).
    fn orient2d(a: &Self::Co, b: &Self::Co, c: &Self::Co) -> i32;
    fn incircle(a: &Self::Co, b: &Self::Co, c: &Self::Co, d: &Self::Co) -> i32;
    fn in_line(a: &Self::Co, b: &Self::Co, c: &Self::Co) -> bool;
    /// `isect_seg_seg` — returns (kind, lambda-along-ab).
    fn isect_seg_seg(a: &Self::Co, b: &Self::Co, c: &Self::Co, d: &Self::Co)
        -> (LineLineKind, Self);
    fn interpolate(a: &Self::Co, b: &Self::Co, t: &Self) -> Self::Co;
    fn zero() -> Self;
    /// `lambda = -1` 'deleted' sentinel.
    fn deleted() -> Self;
    fn is_zero(&self) -> bool;
    fn is_one(&self) -> bool;
    fn is_deleted(&self) -> bool;
    /// `lambda <= middle_lambda (0.5)`.
    fn below_half(&self) -> bool;
    /// `fill_crossdata_for_intersect` CROSS epsilon snapping:
    /// -1 -> snap to va; +1 -> snap to vb; 0 -> interior.
    /// Always 0 for exact arithmetic (upstream skips the check for mpq).
    fn eps_snap(&self, len_ab: f64, epsilon: &Self) -> i32;
    /// Lexicographic compare for site sorting (`site_lexicographic_sort`).
    fn cmp_co(a: &Self::Co, b: &Self::Co) -> std::cmp::Ordering;
}

/* Sorted-unique u32 set — `VectorSet<uint32_t>`. */
#[derive(Clone, Default)]
struct IdSet(Vec<u32>);
impl IdSet {
    fn add(&mut self, id: u32) {
        if !self.0.contains(&id) {
            self.0.push(id);
            self.0.sort_unstable();
        }
    }
    fn add_all(&mut self, src: &IdSet) {
        for &id in &src.0 {
            self.add(id);
        }
    }
    fn len(&self) -> usize {
        self.0.len()
    }
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    fn first(&self) -> u32 {
        self.0[0]
    }
    fn contains(&self, id: u32) -> bool {
        self.0.binary_search(&id).is_ok()
    }
}

/// `FatCo<T>` — exact + approx coords.
#[derive(Clone)]
struct FatCo<S: CdtScalar> {
    exact: S::Co,
    approx: [f64; 2],
    abs_approx: [f64; 2],
}
impl<S: CdtScalar> FatCo<S> {
    fn new(v: S::Co) -> Self {
        let approx = S::approx(&v);
        FatCo {
            exact: v,
            approx,
            abs_approx: [approx[0].abs(), approx[1].abs()],
        }
    }
}

#[derive(Clone)]
struct CdtVert<S: CdtScalar> {
    co: FatCo<S>,
    symedge: usize,
    input_ids: IdSet,
    index: i32,
    merge_to_index: i32,
    visit_index: i32,
    intersected_edges: [i32; 2],
}

#[derive(Clone, Default)]
struct CdtEdge {
    input_ids: IdSet,
}

#[derive(Clone, Copy)]
struct SymEdgeRec {
    next: usize,
    rot: usize,
    vert: usize,
    edge: usize,
    face: usize,
}
const SE_NIL: SymEdgeRec = SymEdgeRec {
    next: NIL,
    rot: NIL,
    vert: NIL,
    edge: NIL,
    face: NIL,
};

#[derive(Clone, Default)]
struct CdtFace {
    symedge: usize,
    input_ids: IdSet,
    visit_index: i32,
    deleted: bool,
    hole: bool,
}

struct CdtArrangement<S: CdtScalar> {
    verts: Vec<CdtVert<S>>,
    edges: Vec<CdtEdge>,
    symedges: Vec<SymEdgeRec>,
    faces: Vec<CdtFace>,
    outer_face: usize,
}

struct CdtState<S: CdtScalar> {
    cdt: CdtArrangement<S>,
    input_vert_num: i32,
    visit_count: i32,
    face_edge_offset: u32,
    epsilon: S,
    needed_ids: u32,
    edge_winding_map: Option<std::collections::HashMap<usize, i32>>,
    polygon_boundary_count_map: Option<std::collections::HashMap<usize, i32>>,
}

impl<S: CdtScalar> CdtArrangement<S> {
    fn se(&self, id: usize) -> SymEdgeRec {
        self.symedges[id]
    }
    fn sem(&mut self, id: usize) -> &mut SymEdgeRec {
        &mut self.symedges[id]
    }
    /// `sym(se)` — other symedge for same edge.
    fn sym(&self, se: usize) -> usize {
        let r = self.se(se);
        self.se(r.next).rot
    }
    /// `prev(se)` — symedge whose next is se.
    fn prev(&self, se: usize) -> usize {
        let r = self.se(se);
        self.se(self.se(r.rot).next).rot
    }

    fn add_vert(&mut self, pt: S::Co) -> usize {
        let index = self.verts.len() as i32;
        self.verts.push(CdtVert {
            co: FatCo::new(pt),
            symedge: NIL,
            input_ids: IdSet::default(),
            index,
            merge_to_index: -1,
            visit_index: 0,
            intersected_edges: [-1, -1],
        });
        index as usize
    }

    fn add_edge(&mut self, v1: usize, v2: usize, fleft: usize, fright: usize) -> usize {
        let e = self.edges.len();
        self.edges.push(CdtEdge::default());
        self.symedges.push(SE_NIL);
        self.symedges.push(SE_NIL);
        let se = e * 2;
        let sesym = e * 2 + 1;
        self.sem(se).edge = e;
        self.sem(sesym).edge = e;
        self.sem(se).face = fleft;
        self.sem(sesym).face = fright;
        self.sem(se).vert = v1;
        if self.verts[v1].symedge == NIL {
            self.verts[v1].symedge = se;
        }
        self.sem(sesym).vert = v2;
        if self.verts[v2].symedge == NIL {
            self.verts[v2].symedge = sesym;
        }
        e
    }

    fn add_face(&mut self) -> usize {
        self.faces.push(CdtFace {
            symedge: NIL,
            ..Default::default()
        });
        self.faces.len() - 1
    }

    /// `add_diagonal(s1, s2)` — split the face containing s1,s2 with an edge
    /// `s1->vert` -> `s2->vert`; returns new edge id.
    fn add_diagonal(&mut self, s1: usize, s2: usize) -> usize {
        let fold = self.se(s1).face;
        let fnew = self.add_face();
        let s1prev = self.prev(s1);
        let s1prevsym = self.sym(s1prev);
        let s2prev = self.prev(s2);
        let s2prevsym = self.sym(s2prev);
        let v1 = self.se(s1).vert;
        let v2 = self.se(s2).vert;
        let ediag = self.add_edge(v1, v2, fnew, fold);
        let sdiag = ediag * 2;
        let sdiagsym = ediag * 2 + 1;
        self.sem(sdiag).next = s2;
        self.sem(sdiagsym).next = s1;
        self.sem(s2prev).next = sdiagsym;
        self.sem(s1prev).next = sdiag;
        self.sem(s1).rot = sdiag;
        self.sem(sdiag).rot = s1prevsym;
        self.sem(s2).rot = sdiagsym;
        self.sem(sdiagsym).rot = s2prevsym;
        let mut se = s2;
        while se != sdiag {
            self.sem(se).face = fnew;
            se = self.se(se).next;
        }
        let ids = self.faces[fold].input_ids.clone();
        self.faces[fnew].input_ids.add_all(&ids);
        ediag
    }

    /// `add_vert_to_symedge_edge` — new edge from v to se->vert, spliced in.
    fn add_vert_to_symedge_edge(&mut self, v: usize, se: usize) -> usize {
        let se_rot = self.se(se).rot;
        let se_rotsym = self.sym(se_rot);
        let se_face = self.se(se).face;
        let se_vert = self.se(se).vert;
        let e = self.add_edge(v, se_vert, se_face, se_face);
        let new_se = e * 2;
        let new_se_sym = e * 2 + 1;
        self.sem(new_se).next = se;
        self.sem(new_se_sym).next = new_se;
        self.sem(new_se).rot = new_se;
        self.sem(new_se_sym).rot = se_rot;
        self.sem(se).rot = new_se_sym;
        self.sem(se_rotsym).next = new_se_sym;
        e
    }

    /// `connect_separate_parts`.
    fn connect_separate_parts(&mut self, se1: usize, se2: usize) -> usize {
        debug_assert_eq!(self.se(se1).face, self.outer_face);
        debug_assert_eq!(self.se(se2).face, self.outer_face);
        let se1_rot = self.se(se1).rot;
        let se1_rotsym = self.sym(se1_rot);
        let se2_rot = self.se(se2).rot;
        let se2_rotsym = self.sym(se2_rot);
        let v1 = self.se(se1).vert;
        let v2 = self.se(se2).vert;
        let of = self.outer_face;
        let e = self.add_edge(v1, v2, of, of);
        let new_se = e * 2;
        let new_se_sym = e * 2 + 1;
        self.sem(new_se).next = se2;
        self.sem(new_se_sym).next = se1;
        self.sem(new_se).rot = se1_rot;
        self.sem(new_se_sym).rot = se2_rot;
        self.sem(se1).rot = new_se;
        self.sem(se2).rot = new_se_sym;
        self.sem(se1_rotsym).next = new_se;
        self.sem(se2_rotsym).next = new_se_sym;
        e
    }

    /// `split_edge(se, lambda, ...)` — returns new edge (second half).
    fn split_edge(
        &mut self,
        se: usize,
        lambda: S,
        edge_winding_map: Option<&mut std::collections::HashMap<usize, i32>>,
        polygon_boundary_count_map: Option<&mut std::collections::HashMap<usize, i32>>,
    ) -> usize {
        let a = self.verts[self.se(se).vert].co.exact.clone();
        let b = self.verts[self.se(self.se(se).next).vert].co.exact.clone();
        let sesym = self.sym(se);
        let sesymprev = self.prev(sesym);
        let sesymprevsym = self.sym(sesymprev);
        let senext = self.se(se).next;
        let pt = S::interpolate(&a, &b, &lambda);
        let v = self.add_vert(pt);
        let se_face = self.se(se).face;
        let sesym_face = self.se(sesym).face;
        let dst = self.se(self.se(se).next).vert;
        let e = self.add_edge(v, dst, se_face, sesym_face);
        debug_assert_eq!(self.edges.len() - 1, e);
        self.sem(sesym).vert = v;
        let newse = e * 2;
        let newsesym = e * 2 + 1;
        self.sem(se).next = newse;
        self.sem(newsesym).next = sesym;
        self.sem(newse).next = senext;
        self.sem(newse).rot = sesym;
        self.sem(sesym).rot = newse;
        self.sem(senext).rot = newsesym;
        self.sem(newsesym).rot = sesymprevsym;
        self.sem(sesymprev).next = newsesym;
        let nv = self.se(newsesym).vert;
        if self.verts[nv].symedge == sesym {
            self.verts[nv].symedge = newsesym;
        }
        let ids = self.edges[self.se(se).edge].input_ids.clone();
        self.edges[e].input_ids.add_all(&ids);

        if let Some(map) = polygon_boundary_count_map {
            let count = *map.get(&self.se(se).edge).unwrap_or(&0);
            if count != 0 {
                map.insert(e, count);
            }
        }
        if let Some(map) = edge_winding_map {
            let mut winding = *map.get(&self.se(se).edge).unwrap_or(&0);
            if winding != 0 {
                if se == self.se(se).edge * 2 + 1 {
                    winding = -winding;
                }
                map.insert(e, winding);
            }
        }
        e
    }

    /// `delete_edge`.
    fn delete_edge(&mut self, se: usize) {
        let sesym = self.sym(se);
        let v1 = self.se(se).vert;
        let v2 = self.se(sesym).vert;
        let aface = self.se(se).face;
        let bface = self.se(sesym).face;
        let f = self.se(se).next;
        let h = self.prev(se);
        let i = self.se(sesym).next;
        let j = self.prev(sesym);
        let jsym = self.sym(j);
        let hsym = self.sym(h);
        let v1_isolated = i == se;
        let v2_isolated = f == sesym;

        if !v1_isolated {
            self.sem(h).next = i;
            self.sem(i).rot = hsym;
        }
        if !v2_isolated {
            self.sem(j).next = f;
            self.sem(f).rot = jsym;
        }
        if !v1_isolated && !v2_isolated && aface != bface {
            let mut k = i;
            while k != f {
                self.sem(k).face = aface;
                k = self.se(k).next;
            }
        }
        if v1_isolated {
            self.verts[v1].symedge = NIL;
        } else if self.verts[v1].symedge == se {
            self.verts[v1].symedge = i;
        }
        if v2_isolated {
            self.verts[v2].symedge = NIL;
        } else if self.verts[v2].symedge == sesym {
            self.verts[v2].symedge = f;
        }
        self.sem(se).next = NIL;
        self.sem(se).rot = NIL;
        self.sem(sesym).next = NIL;
        self.sem(sesym).rot = NIL;
        if !v1_isolated && !v2_isolated && aface != bface {
            self.faces[bface].deleted = true;
            if self.outer_face == bface {
                self.outer_face = aface;
            }
        } else if v1_isolated && v2_isolated {
            debug_assert_eq!(aface, bface);
            if aface == self.outer_face {
                let os = self.faces[self.outer_face].symedge;
                if os == se || os == sesym {
                    self.faces[self.outer_face].symedge = NIL;
                }
            } else {
                self.faces[aface].deleted = true;
            }
        }
    }

    fn get_vert_resolve_merge(&self, i: usize) -> usize {
        let mut v = i;
        if self.verts[v].merge_to_index != -1 {
            v = self.verts[v].merge_to_index as usize;
        }
        v
    }
}

/* Predicates — `filtered_orient2d<T>`/`filtered_incircle<T>`:
 * exact `S::orient2d`/`S::incircle` (robust for f64, exact det for mpq). */
fn filtered_orient2d<S: CdtScalar>(a: &FatCo<S>, b: &FatCo<S>, c: &FatCo<S>) -> i32 {
    S::orient2d(&a.exact, &b.exact, &c.exact)
}
fn filtered_incircle<S: CdtScalar>(a: &FatCo<S>, b: &FatCo<S>, c: &FatCo<S>, d: &FatCo<S>) -> i32 {
    S::incircle(&a.exact, &b.exact, &c.exact, &d.exact)
}
fn dot2(a: [f64; 2], b: [f64; 2]) -> f64 {
    a[0] * b[0] + a[1] * b[1]
}
fn sub2(a: [f64; 2], b: [f64; 2]) -> [f64; 2] {
    [a[0] - b[0], a[1] - b[1]]
}
fn distance2(a: [f64; 2], b: [f64; 2]) -> f64 {
    (sub2(a, b)[0].powi(2) + sub2(a, b)[1].powi(2)).sqrt()
}
fn distance_squared2(a: [f64; 2], b: [f64; 2]) -> f64 {
    let d = sub2(a, b);
    d[0] * d[0] + d[1] * d[1]
}

/* `isect_result` kind (math_vec.cc). */
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LineLineKind {
    Colinear = -1,
    None = 0,
    Exact = 1,
    Cross = 2,
}

/* `CdtScalar` for f64 — `CDT_input<double>` specialization. */
impl CdtScalar for f64 {
    type Co = [f64; 2];
    fn approx(c: &Self::Co) -> [f64; 2] {
        *c
    }
    fn orient2d(a: &Self::Co, b: &Self::Co, c: &Self::Co) -> i32 {
        let r = robust::orient2d(
            robust::Coord { x: a[0], y: a[1] },
            robust::Coord { x: b[0], y: b[1] },
            robust::Coord { x: c[0], y: c[1] },
        );
        if r > 0.0 {
            1
        } else if r < 0.0 {
            -1
        } else {
            0
        }
    }
    fn incircle(a: &Self::Co, b: &Self::Co, c: &Self::Co, d: &Self::Co) -> i32 {
        let r = robust::incircle(
            robust::Coord { x: a[0], y: a[1] },
            robust::Coord { x: b[0], y: b[1] },
            robust::Coord { x: c[0], y: c[1] },
            robust::Coord { x: d[0], y: d[1] },
        );
        if r > 0.0 {
            1
        } else if r < 0.0 {
            -1
        } else {
            0
        }
    }
    fn in_line(a: &Self::Co, b: &Self::Co, c: &Self::Co) -> bool {
        let ab = sub2(*b, *a);
        let ac = sub2(*c, *a);
        if dot2(ab, ac) < 0.0 {
            return false;
        }
        let bc = sub2(*c, *b);
        dot2(bc, ac) >= 0.0
    }
    fn isect_seg_seg(
        v1: &Self::Co,
        v2: &Self::Co,
        v3: &Self::Co,
        v4: &Self::Co,
    ) -> (LineLineKind, Self) {
        let div = (v2[0] - v1[0]) * (v4[1] - v3[1]) - (v2[1] - v1[1]) * (v4[0] - v3[0]);
        if div == 0.0 {
            return (LineLineKind::Colinear, 0.0);
        }
        let lambda =
            ((v1[1] - v3[1]) * (v4[0] - v3[0]) - (v1[0] - v3[0]) * (v4[1] - v3[1])) / div;
        let mu = ((v1[1] - v3[1]) * (v2[0] - v1[0]) - (v1[0] - v3[0]) * (v2[1] - v1[1])) / div;
        let kind = if (0.0..=1.0).contains(&lambda) && (0.0..=1.0).contains(&mu) {
            if lambda == 0.0 || lambda == 1.0 || mu == 0.0 || mu == 1.0 {
                LineLineKind::Exact
            } else {
                LineLineKind::Cross
            }
        } else {
            LineLineKind::None
        };
        (kind, lambda)
    }
    fn interpolate(a: &Self::Co, b: &Self::Co, t: &Self) -> Self::Co {
        [a[0] + t * (b[0] - a[0]), a[1] + t * (b[1] - a[1])]
    }
    fn zero() -> Self {
        0.0
    }
    fn deleted() -> Self {
        -1.0
    }
    fn is_zero(&self) -> bool {
        *self == 0.0
    }
    fn is_one(&self) -> bool {
        *self == 1.0
    }
    fn is_deleted(&self) -> bool {
        *self == -1.0
    }
    fn below_half(&self) -> bool {
        *self <= 0.5
    }
    fn eps_snap(&self, len_ab: f64, epsilon: &Self) -> i32 {
        if *self * len_ab <= *epsilon {
            -1
        } else if (1.0 - *self) * len_ab <= *epsilon {
            1
        } else {
            0
        }
    }
    fn cmp_co(a: &Self::Co, b: &Self::Co) -> std::cmp::Ordering {
        a[0].partial_cmp(&b[0])
            .unwrap()
            .then(a[1].partial_cmp(&b[1]).unwrap())
    }
}

/* `CdtScalar` for `Mpq` (BigRational) — `CDT_input<mpq_class>` specialization
 * (`WITH_GMP` path upstream). All predicates exact — no filtering needed. */
pub mod mpq_cdt {
    use super::{CdtScalar, LineLineKind};
    use blenders_math::mpq::{mpq_to_f64, Mpq2, Mpq};
    use num_bigint::BigInt;
    use num_rational::BigRational;
    use num_traits::{One, Zero};

    fn dsub(a: &Mpq, b: &Mpq) -> Mpq {
        a - b
    }
    impl CdtScalar for Mpq {
        type Co = Mpq2;
        fn approx(c: &Self::Co) -> [f64; 2] {
            c.get_d()
        }
        /// `orient2d(const mpq2&, mpq2&, mpq2&)` — exact determinant sign.
        fn orient2d(a: &Self::Co, b: &Self::Co, c: &Self::Co) -> i32 {
            let detleft = dsub(&a.x, &c.x) * dsub(&b.y, &c.y);
            let detright = dsub(&a.y, &c.y) * dsub(&b.x, &c.x);
            blenders_math::mpq::mpq_sgn(&(detleft - detright))
        }
        /// `incircle(mpq2...)` — lifted determinant sign, exact.
        fn incircle(a: &Self::Co, b: &Self::Co, c: &Self::Co, d: &Self::Co) -> i32 {
            let adx = dsub(&a.x, &d.x);
            let bdx = dsub(&b.x, &d.x);
            let cdx = dsub(&c.x, &d.x);
            let ady = dsub(&a.y, &d.y);
            let bdy = dsub(&b.y, &d.y);
            let cdy = dsub(&c.y, &d.y);
            let bdxcdy = &bdx * &cdy;
            let cdxbdy = &cdx * &bdy;
            let alift = &adx * &adx + &ady * &ady;
            let cdxady = &cdx * &ady;
            let adxcdy = &adx * &cdy;
            let blift = &bdx * &bdx + &bdy * &bdy;
            let adxbdy = &adx * &bdy;
            let bdxady = &bdx * &ady;
            let clift = &cdx * &cdx + &cdy * &cdy;
            let det = alift * (bdxcdy - cdxbdy) + blift * (cdxady - adxcdy)
                + clift * (adxbdy - bdxady);
            blenders_math::mpq::mpq_sgn(&det)
        }
        /// `in_line<mpq_class>` — exact dot products.
        fn in_line(a: &Self::Co, b: &Self::Co, c: &Self::Co) -> bool {
            let ab = b - a;
            let ac = c - a;
            if ab.dot(&ac) < num_traits::Zero::zero() {
                return false;
            }
            let bc = c - b;
            bc.dot(&ac) >= num_traits::Zero::zero()
        }
        /// `isect_seg_seg(mpq2...)` — exact; avoids dividing mudiv by div.
        fn isect_seg_seg(
            v1: &Self::Co,
            v2: &Self::Co,
            v3: &Self::Co,
            v4: &Self::Co,
        ) -> (LineLineKind, Self) {
            let div = (&v2.x - &v1.x) * (&v4.y - &v3.y) - (&v2.y - &v1.y) * (&v4.x - &v3.x);
            let zero: Mpq = num_traits::Zero::zero();
            if div == zero {
                return (LineLineKind::Colinear, zero);
            }
            let lambda = ((&v1.y - &v3.y) * (&v4.x - &v3.x) - (&v1.x - &v3.x) * (&v4.y - &v3.y))
                / &div;
            let mudiv =
                (&v1.y - &v3.y) * (&v2.x - &v1.x) - (&v1.x - &v3.x) * (&v2.y - &v1.y);
            let one: Mpq = num_traits::One::one();
            let in01 = |q: &Mpq, hi: &Mpq| -> bool { *q >= zero && *q <= *hi };
            let seg = in01(&lambda, &one)
                && ((div > zero && in01(&mudiv, &div))
                    || (div < zero && mudiv <= zero && mudiv >= div));
            let kind = if seg {
                if lambda == zero || lambda == one || mudiv == zero || mudiv == div {
                    LineLineKind::Exact
                } else {
                    LineLineKind::Cross
                }
            } else {
                LineLineKind::None
            };
            (kind, lambda)
        }
        /// `interpolate(a, b, t)` = a + t*(b - a).
        fn interpolate(a: &Self::Co, b: &Self::Co, t: &Self) -> Self::Co {
            a + &(&(b - a) * t)
        }
        fn zero() -> Self {
            num_traits::Zero::zero()
        }
        fn deleted() -> Self {
            BigRational::from_integer(BigInt::from(-1))
        }
        fn is_zero(&self) -> bool {
            *self == num_traits::Zero::zero()
        }
        fn is_one(&self) -> bool {
            *self == num_traits::One::one()
        }
        fn is_deleted(&self) -> bool {
            *self == Self::deleted()
        }
        fn below_half(&self) -> bool {
            *self <= BigRational::new(BigInt::one(), BigInt::from(2))
        }
        fn eps_snap(&self, _len_ab: f64, _epsilon: &Self) -> i32 {
            0 /* Upstream: no epsilon snapping for exact T. */
        }
        fn cmp_co(a: &Self::Co, b: &Self::Co) -> std::cmp::Ordering {
            a.x.cmp(&b.x).then(a.y.cmp(&b.y))
        }
    }
}

/* Search helpers. */

fn is_border_edge<S: CdtScalar>(cdt: &CdtArrangement<S>, e: usize) -> bool {
    cdt.symedges[e * 2].face == cdt.outer_face || cdt.symedges[e * 2 + 1].face == cdt.outer_face
}
fn is_constrained_edge<S: CdtScalar>(cdt: &CdtArrangement<S>, e: usize) -> bool {
    !cdt.edges[e].input_ids.is_empty()
}
fn is_deleted_edge<S: CdtScalar>(cdt: &CdtArrangement<S>, e: usize) -> bool {
    cdt.symedges[e * 2].next == NIL
}
fn is_original_vert<S: CdtScalar>(st: &CdtState<S>, v: usize) -> bool {
    (st.cdt.verts[v].index) < st.input_vert_num
}

fn find_symedge_between_verts<S: CdtScalar>(cdt: &CdtArrangement<S>, v1: usize, v2: usize) -> usize {
    let mut t = cdt.verts[v1].symedge;
    let tstart = t;
    loop {
        if cdt.se(cdt.se(t).next).vert == v2 {
            return t;
        }
        t = cdt.se(t).rot;
        if t == tstart {
            break;
        }
    }
    NIL
}
fn find_symedge_with_face<S: CdtScalar>(cdt: &CdtArrangement<S>, v: usize, f: usize) -> usize {
    let mut t = cdt.verts[v].symedge;
    let tstart = t;
    loop {
        if cdt.se(t).face == f {
            return t;
        }
        t = cdt.se(t).rot;
        if t == tstart {
            break;
        }
    }
    NIL
}
fn exists_edge<S: CdtScalar>(cdt: &CdtArrangement<S>, v1: usize, v2: usize) -> bool {
    find_symedge_between_verts(cdt, v1, v2) != NIL
}

/* Site sorting and merging. */

#[derive(Clone, Copy)]
struct SiteInfo {
    v: usize,
    orig_index: i32,
}

fn site_lexicographic_sort<S: CdtScalar>(cdt: &CdtArrangement<S>, a: &SiteInfo, b: &SiteInfo) -> std::cmp::Ordering {
    let ca = &cdt.verts[a.v].co.exact;
    let cb = &cdt.verts[b.v].co.exact;
    S::cmp_co(ca, cb).then(a.orig_index.cmp(&b.orig_index))
}

fn find_site_merges<S: CdtScalar>(cdt: &mut CdtArrangement<S>, sites: &mut [SiteInfo]) {
    let n = sites.len();
    let mut i = 0;
    while i + 1 < n {
        let mut j = i + 1;
        while j < n && cdt.verts[sites[j].v].co.exact == cdt.verts[sites[i].v].co.exact {
            let m = sites[i].orig_index;
            cdt.verts[sites[j].v].merge_to_index = m;
            j += 1;
        }
        if j - i > 1 {
            i = j - 1;
        }
        i += 1;
    }
}

fn vert_left_of_symedge<S: CdtScalar>(cdt: &CdtArrangement<S>, v: usize, se: usize) -> bool {
    let a = &cdt.verts[v].co;
    let b = &cdt.verts[cdt.se(se).vert].co;
    let c = &cdt.verts[cdt.se(cdt.se(se).next).vert].co;
    filtered_orient2d(a, b, c) > 0
}
fn vert_right_of_symedge<S: CdtScalar>(cdt: &CdtArrangement<S>, v: usize, se: usize) -> bool {
    let a = &cdt.verts[v].co;
    let b = &cdt.verts[cdt.se(cdt.se(se).next).vert].co;
    let c = &cdt.verts[cdt.se(se).vert].co;
    filtered_orient2d(a, b, c) > 0
}
fn dc_tri_valid<S: CdtScalar>(cdt: &CdtArrangement<S>, se: usize, basel: usize, basel_sym: usize) -> bool {
    let a = &cdt.verts[cdt.se(cdt.se(se).next).vert].co;
    let b = &cdt.verts[cdt.se(basel_sym).vert].co;
    let c = &cdt.verts[cdt.se(basel).vert].co;
    filtered_orient2d(a, b, c) > 0
}

/// `dc_tri` — Guibas-Stolfi divide & conquer merge step.
fn dc_tri<S: CdtScalar>(
    cdt: &mut CdtArrangement<S>,
    sites: &mut [SiteInfo],
    start: usize,
    end: usize,
) -> (usize, usize) {
    let n = end - start;
    if n <= 1 {
        return (NIL, NIL);
    }
    if n <= 3 {
        let v1 = sites[start].v;
        let v2 = sites[start + 1].v;
        let of = cdt.outer_face;
        let ea = cdt.add_edge(v1, v2, of, of);
        let (e0, e1) = (ea * 2, ea * 2 + 1);
        cdt.sem(e0).next = e1;
        cdt.sem(e1).next = e0;
        cdt.sem(e0).rot = e0;
        cdt.sem(e1).rot = e1;
        if n == 2 {
            return (e0, e1);
        }
        let v3 = sites[start + 2].v;
        let eb = cdt.add_vert_to_symedge_edge(v3, e1);
        let orient = filtered_orient2d(
            &cdt.verts[v1].co,
            &cdt.verts[v2].co,
            &cdt.verts[v3].co,
        );
        if orient > 0 {
            cdt.add_diagonal(eb * 2, e0);
            (e0, eb * 2)
        } else if orient < 0 {
            cdt.add_diagonal(e0, eb * 2);
            (cdt.se(e0).rot, cdt.se(eb * 2).rot)
        } else {
            (e0, eb * 2)
        }
    } else {
        let n2 = n / 2;
        debug_assert!(n2 >= 2 && end - (start + n2) >= 2);
        let (mut ldo, mut ldi) = dc_tri(cdt, sites, start, start + n2);
        let (mut rdi, mut rdo) = dc_tri(cdt, sites, start + n2, end);
        /* Find lower common tangent of L and R. */
        loop {
            if vert_left_of_symedge(cdt, cdt.se(rdi).vert, ldi) {
                ldi = cdt.se(ldi).next;
            } else if vert_right_of_symedge(cdt, cdt.se(ldi).vert, rdi) {
                rdi = cdt.se(cdt.sym(rdi)).rot;
            } else {
                break;
            }
        }
        let s1 = cdt.se(cdt.sym(rdi)).next;
        let ebasel = cdt.connect_separate_parts(s1, ldi);
        let mut basel = ebasel * 2;
        let mut basel_sym = ebasel * 2 + 1;
        if cdt.se(ldi).vert == cdt.se(ldo).vert {
            ldo = basel_sym;
        }
        if cdt.se(rdi).vert == cdt.se(rdo).vert {
            rdo = basel;
        }
        /* Merge loop. */
        loop {
            let mut lcand = cdt.se(basel_sym).rot;
            let mut rcand = cdt.se(basel_sym).next;
            if dc_tri_valid(cdt, lcand, basel, basel_sym) {
                loop {
                    let d = &cdt.verts[cdt.se(cdt.se(cdt.se(lcand).rot).next).vert].co;
                    let a = &cdt.verts[cdt.se(basel_sym).vert].co;
                    let b = &cdt.verts[cdt.se(basel).vert].co;
                    let c = &cdt.verts[cdt.se(cdt.se(lcand).next).vert].co;
                    if filtered_incircle(a, b, c, d) <= 0 {
                        break;
                    }
                    let t = cdt.se(lcand).rot;
                    let e = cdt.sym(lcand);
                    cdt.delete_edge(e);
                    lcand = t;
                }
            }
            if dc_tri_valid(cdt, rcand, basel, basel_sym) {
                loop {
                    let s_rc = cdt.sym(rcand);
                    let d = &cdt.verts[cdt.se(cdt.se(cdt.se(s_rc).next).next).vert].co;
                    let a = &cdt.verts[cdt.se(basel_sym).vert].co;
                    let b = &cdt.verts[cdt.se(basel).vert].co;
                    let c = &cdt.verts[cdt.se(cdt.se(rcand).next).vert].co;
                    if filtered_incircle(a, b, c, d) <= 0 {
                        break;
                    }
                    let t = cdt.se(s_rc).next;
                    cdt.delete_edge(rcand);
                    rcand = t;
                }
            }
            let valid_lcand = dc_tri_valid(cdt, lcand, basel, basel_sym);
            let valid_rcand = dc_tri_valid(cdt, rcand, basel, basel_sym);
            if !valid_lcand && !valid_rcand {
                break;
            }
            let ebasel = if !valid_lcand
                || (valid_rcand
                    && filtered_incircle(
                        &cdt.verts[cdt.se(cdt.se(lcand).next).vert].co,
                        &cdt.verts[cdt.se(lcand).vert].co,
                        &cdt.verts[cdt.se(rcand).vert].co,
                        &cdt.verts[cdt.se(cdt.se(rcand).next).vert].co,
                    ) > 0)
            {
                let s2 = cdt.se(rcand).next;
                cdt.add_diagonal(s2, basel_sym)
            } else {
                let s2 = cdt.se(basel_sym).next;
                let s1 = cdt.sym(lcand);
                cdt.add_diagonal(s2, s1)
            };
            basel = ebasel * 2;
            basel_sym = ebasel * 2 + 1;
            debug_assert_eq!(cdt.se(basel_sym).face, cdt.outer_face);
        }
        (ldo, rdo)
    }
}

fn dc_triangulate<S: CdtScalar>(cdt: &mut CdtArrangement<S>, sites: &mut [SiteInfo]) {
    /* Compress merged verts in place. */
    let mut i = 0;
    let mut j = 0;
    let nsites = sites.len();
    while j < nsites {
        let s = sites[j];
        sites[i] = s;
        j += 1;
        if cdt.verts[s.v].merge_to_index < 0 {
            i += 1;
        }
    }
    let _ = i;
    let n = i;
    if n == 0 {
        return;
    }
    dc_tri(cdt, &mut sites[..n], 0, n);
}

/// `initial_triangulation` — Delaunay triangulation of all verts.
fn initial_triangulation<S: CdtScalar>(cdt: &mut CdtArrangement<S>) {
    let n = cdt.verts.len();
    if n <= 1 {
        return;
    }
    let mut sites: Vec<SiteInfo> = (0..n)
        .map(|i| SiteInfo {
            v: i,
            orig_index: i as i32,
        })
        .collect();
    sites.sort_by(|a, b| site_lexicographic_sort(cdt, a, b));
    find_site_merges(cdt, &mut sites);
    dc_triangulate(cdt, &mut sites);
}

/// `re_delaunay_triangulate` — Anglada's restricted re-triangulation of the
/// pseudo-polygon on se's left face.
fn re_delaunay_triangulate<S: CdtScalar>(cdt: &mut CdtArrangement<S>, se: usize) {
    if cdt.se(se).face == cdt.outer_face || cdt.se(cdt.sym(se)).face == cdt.outer_face {
        return;
    }
    let mut count = 1;
    let mut ss = cdt.se(se).next;
    while ss != se {
        count += 1;
        ss = cdt.se(ss).next;
    }
    if count <= 3 {
        return;
    }
    let first = cdt.se(cdt.se(se).next).next;
    let a = cdt.se(se).vert;
    let b = cdt.se(cdt.se(se).next).vert;
    let mut c = cdt.se(first).vert;
    let mut cse = first;
    let mut ss = cdt.se(first).next;
    while ss != se {
        let v = cdt.se(ss).vert;
        if filtered_incircle(
            &cdt.verts[a].co,
            &cdt.verts[b].co,
            &cdt.verts[c].co,
            &cdt.verts[v].co,
        ) > 0
        {
            c = v;
            cse = ss;
        }
        ss = cdt.se(ss).next;
    }
    let mut ebc = NIL;
    let mut eca = NIL;
    if !exists_edge(cdt, b, c) {
        let s1 = cdt.se(se).next;
        ebc = cdt.add_diagonal(s1, cse);
    }
    if !exists_edge(cdt, c, a) {
        eca = cdt.add_diagonal(cse, se);
    }
    if ebc != NIL {
        re_delaunay_triangulate(cdt, ebc * 2 + 1);
    }
    if eca != NIL {
        re_delaunay_triangulate(cdt, eca * 2 + 1);
    }
}

fn tri_orient<S: CdtScalar>(cdt: &CdtArrangement<S>, t: usize) -> i32 {
    let a = &cdt.verts[cdt.se(t).vert].co;
    let b = &cdt.verts[cdt.se(cdt.se(t).next).vert].co;
    let c = &cdt.verts[cdt.se(cdt.se(cdt.se(t).next).next).vert].co;
    filtered_orient2d(a, b, c)
}

/* CrossData — crossing point while inserting an edge constraint. */
#[derive(Clone)]
struct CrossData<S: CdtScalar> {
    lambda: S,
    vert: usize,
    se_in: usize,
    se_out: usize,
}
fn cd_nil<S: CdtScalar>() -> CrossData<S> {
    CrossData {
        lambda: S::zero(),
        vert: NIL,
        se_in: NIL,
        se_out: NIL,
    }
}

/// `fill_crossdata_for_through_vert`.
fn fill_crossdata_for_through_vert<S: CdtScalar>(
    cdt: &CdtArrangement<S>,
    v: usize,
    cd_out: usize,
    cd: &mut CrossData<S>,
    cd_next: &mut CrossData<S>,
) {
    cd_next.lambda = S::zero();
    cd_next.vert = v;
    cd_next.se_in = NIL;
    cd_next.se_out = NIL;
    if cd.lambda.is_zero() {
        cd.se_out = cd_out;
    } else {
        /* One of the edges in the triangle with edge sym(cd.in) contains v. */
        let mut se = cdt.sym(cd.se_in);
        if cdt.se(se).vert != v {
            se = cdt.se(se).next;
            if cdt.se(se).vert != v {
                se = cdt.se(se).next;
            }
        }
        debug_assert_eq!(cdt.se(se).vert, v);
        cd_next.se_in = se;
    }
}

/// `fill_crossdata_for_intersect`.
fn fill_crossdata_for_intersect<S: CdtScalar>(
    cdt: &mut CdtArrangement<S>,
    curco: &FatCo<S>,
    v2: usize,
    t: usize,
    cd: &mut CrossData<S>,
    cd_next: &mut CrossData<S>,
    epsilon: &S,
) {
    let va = cdt.se(t).vert;
    let vb = cdt.se(cdt.se(t).next).vert;
    let se_vcvb = cdt.sym(cdt.se(t).next);
    let se_vcva = cdt.se(cdt.se(t).next).next;
    let (kind, lambda) = S::isect_seg_seg(
        &cdt.verts[va].co.exact,
        &cdt.verts[vb].co.exact,
        &curco.exact,
        &cdt.verts[v2].co.exact,
    );
    match kind {
        LineLineKind::Cross => {
            let len_ab = distance2(cdt.verts[va].co.approx, cdt.verts[vb].co.approx);
            match lambda.eps_snap(len_ab, epsilon) {
                -1 => fill_crossdata_for_through_vert(cdt, va, se_vcva, cd, cd_next),
                1 => fill_crossdata_for_through_vert(cdt, vb, se_vcvb, cd, cd_next),
                _ => {
                    *cd_next = CrossData {
                        lambda,
                        vert: NIL,
                        se_in: t,
                        se_out: NIL,
                    };
                    if cd.lambda.is_zero() {
                        cd.se_out = se_vcva;
                    }
                }
            }
        }
        LineLineKind::Exact => {
            if lambda.is_zero() {
                fill_crossdata_for_through_vert(cdt, va, se_vcva, cd, cd_next);
            } else if lambda.is_one() {
                fill_crossdata_for_through_vert(cdt, vb, se_vcvb, cd, cd_next);
            } else {
                *cd_next = CrossData {
                    lambda,
                    vert: NIL,
                    se_in: t,
                    se_out: NIL,
                };
                if cd.lambda.is_zero() {
                    cd.se_out = se_vcva;
                }
            }
        }
        LineLineKind::None => {
            if lambda.below_half() {
                fill_crossdata_for_through_vert(cdt, va, se_vcva, cd, cd_next);
            } else {
                fill_crossdata_for_through_vert(cdt, vb, se_vcvb, cd, cd_next);
            }
        }
        LineLineKind::Colinear => {
            if distance_squared2(cdt.verts[va].co.approx, cdt.verts[v2].co.approx)
                <= distance_squared2(cdt.verts[vb].co.approx, cdt.verts[v2].co.approx)
            {
                fill_crossdata_for_through_vert(cdt, va, se_vcva, cd, cd_next);
            } else {
                fill_crossdata_for_through_vert(cdt, vb, se_vcvb, cd, cd_next);
            }
        }
    }
}

/// `get_next_crossing_from_vert`.
fn get_next_crossing_from_vert<S: CdtScalar>(
    st: &mut CdtState<S>,
    cd: &mut CrossData<S>,
    cd_next: &mut CrossData<S>,
    v2: usize,
) -> bool {
    let tstart = st.cdt.verts[cd.vert].symedge;
    let mut t = tstart;
    let vcur = cd.vert;
    loop {
        let va = st.cdt.se(st.cdt.se(t).next).vert;
        let vb = st.cdt.se(st.cdt.se(st.cdt.se(t).next).next).vert;
        let orient1 = filtered_orient2d(
            &st.cdt.verts[st.cdt.se(t).vert].co,
            &st.cdt.verts[va].co,
            &st.cdt.verts[v2].co,
        );
        if orient1 == 0
            && S::in_line(&st.cdt.verts[vcur].co.exact, &st.cdt.verts[va].co.exact, &st.cdt.verts[v2].co.exact)
        {
            fill_crossdata_for_through_vert(&st.cdt, va, t, cd, cd_next);
            return true;
        }
        if st.cdt.se(t).face != st.cdt.outer_face {
            let orient2 = filtered_orient2d(
                &st.cdt.verts[vcur].co,
                &st.cdt.verts[vb].co,
                &st.cdt.verts[v2].co,
            );
            if orient1 > 0 && orient2 < 0 {
                let t2 = st.cdt.se(t).next;
                let curco = st.cdt.verts[vcur].co.clone();
                fill_crossdata_for_intersect(
                    &mut st.cdt,
                    &curco,
                    v2,
                    t2,
                    cd,
                    cd_next,
                    &st.epsilon,
                );
                return true;
            }
        }
        t = st.cdt.se(t).rot;
        if t == tstart {
            break;
        }
    }
    false
}

/// `get_next_crossing_from_edge`.
fn get_next_crossing_from_edge<S: CdtScalar>(
    st: &mut CdtState<S>,
    cd: &mut CrossData<S>,
    cd_next: &mut CrossData<S>,
    v2: usize,
    epsilon: &S,
) {
    let va = st.cdt.se(cd.se_in).vert;
    let vb = st.cdt.se(st.cdt.se(cd.se_in).next).vert;
    let curco = S::interpolate(
        &st.cdt.verts[va].co.exact,
        &st.cdt.verts[vb].co.exact,
        &cd.lambda,
    );
    let fat_curco = FatCo::<S>::new(curco);
    let se_ac = st.cdt.se(st.cdt.sym(cd.se_in)).next;
    let vc = st.cdt.se(st.cdt.se(se_ac).next).vert;
    let orient = filtered_orient2d(&fat_curco, &st.cdt.verts[v2].co, &st.cdt.verts[vc].co);
    if orient < 0 {
        let t = st.cdt.se(se_ac).next;
        fill_crossdata_for_intersect(&mut st.cdt, &fat_curco, v2, t, cd, cd_next, epsilon);
    } else if orient > 0 {
        fill_crossdata_for_intersect(&mut st.cdt, &fat_curco, v2, se_ac, cd, cd_next, epsilon);
    } else {
        *cd_next = CrossData {
            lambda: S::zero(),
            vert: vc,
            se_in: st.cdt.se(se_ac).next,
            se_out: NIL,
        };
    }
}

/* ---- Constraint insertion ---- */

fn get_two_mut<T>(v: &mut Vec<T>, a: usize, b: usize) -> (&mut T, &mut T) {
    debug_assert!(a < b);
    let (lo, hi) = v.split_at_mut(b);
    (&mut lo[a], &mut hi[0])
}

/// `add_edge_constraint` — insert constrained edge v1->v2; collect generated
/// edges (in order) into `r_edges` when requested.
fn add_edge_constraint<S: CdtScalar>(
    st: &mut CdtState<S>,
    v1: usize,
    v2: usize,
    input_id: u32,
    r_edges: Option<&mut Vec<usize>>,
) {
    let mut r_edges = r_edges;
    let mut edge_list: Vec<usize> = Vec::new();
    if v1 == v2 {
        return;
    }
    let t = find_symedge_between_verts(&st.cdt, v1, v2);
    if t != NIL {
        let e = st.cdt.se(t).edge;
        st.cdt.edges[e].input_ids.add(input_id);
        if r_edges.is_some() {
            edge_list.push(e);
        }
        if let Some(r) = r_edges.as_mut() {
            r.extend(edge_list);
        }
        return;
    }

    /* Fill crossings array. */
    let visit = {
        st.visit_count += 1;
        st.visit_count
    };
    let mut crossings: Vec<CrossData<S>> = Vec::with_capacity(128);
    crossings.push(CrossData {
        lambda: S::zero(),
        vert: v1,
        se_in: NIL,
        se_out: NIL,
    });
    loop {
        let n = crossings.len();
        if n > 0 && crossings[n - 1].vert == v2 {
            break;
        }
        crossings.push(cd_nil());
        let prev_lambda = crossings[n - 1].lambda.clone();
        let (cd, cd_next) = get_two_mut(&mut crossings, n - 1, n);
        let ok = if prev_lambda.is_zero() {
            get_next_crossing_from_vert(st, cd, cd_next, v2)
        } else {
            let eps = st.epsilon.clone();
            get_next_crossing_from_edge(st, cd, cd_next, v2, &eps);
            true
        };
        const UNREASONABLY_LARGE_CROSSINGS: usize = 100000;
        if !ok || crossings.len() == UNREASONABLY_LARGE_CROSSINGS {
            debug_assert!(false);
            return;
        }
        if crossings[n].lambda.is_zero() {
            let vv = crossings[n].vert;
            if st.cdt.verts[vv].visit_index == visit {
                debug_assert!(false);
                return;
            }
            st.cdt.verts[vv].visit_index = visit;
        }
    }

    /* Post-process crossings: mark "deleted" ones with lambda = -1. */
    let ncrossings = crossings.len();
    for i in 2..ncrossings {
        if crossings[i].lambda.is_zero() {
            let v = crossings[i].vert;
            let mut j = i - 1;
            loop {
                if j == 0 {
                    break;
                }
                let cd_prev = crossings[j].clone();
                let done = (cd_prev.lambda.is_zero() && cd_prev.vert != v)
                    || (!cd_prev.lambda.is_zero()
                        && st.cdt.se(cd_prev.se_in).vert != v
                        && st.cdt.se(st.cdt.se(cd_prev.se_in).next).vert != v);
                if done {
                    break;
                }
                crossings[j].lambda = S::deleted();
                j -= 1;
            }
            if j < i - 1 {
                /* Fix in/out across the gap. */
                if crossings[j].lambda.is_zero() {
                    let se = find_symedge_between_verts(&st.cdt, crossings[j].vert, v);
                    if se == NIL {
                        return;
                    }
                    crossings[j].se_out = se;
                    crossings[i].se_in = NIL;
                } else {
                    let f = st.cdt.se(st.cdt.sym(crossings[j].se_in)).face;
                    let se = find_symedge_with_face(&st.cdt, v, f);
                    if se == NIL {
                        return;
                    }
                    crossings[i].se_in = se;
                }
            }
        }
    }

    /* Insert intersection points on constrained edges. */
    for i in 0..ncrossings {
        let cd = &mut crossings[i];
        if !cd.lambda.is_zero()
            && !cd.lambda.is_deleted()
            && is_constrained_edge(&st.cdt, st.cdt.se(cd.se_in).edge)
        {
            let in_edge = st.cdt.se(cd.se_in).edge;
            let edge = st.cdt.split_edge(
                cd.se_in,
                cd.lambda.clone(),
                st.edge_winding_map.as_mut(),
                st.polygon_boundary_count_map.as_mut(),
            );
            cd.vert = st.cdt.se(edge * 2).vert;
            if st.needed_ids & CDT_INTERSECTED_EDGES != 0 {
                let edge1_id = if !st.cdt.edges[in_edge].input_ids.is_empty() {
                    st.cdt.edges[in_edge].input_ids.first() as i32
                } else {
                    -1
                };
                st.cdt.verts[cd.vert].intersected_edges = [edge1_id, input_id as i32];
            }
        }
    }

    /* Remove crossed, non-intersected edges. */
    for i in 0..ncrossings {
        let cd = &crossings[i];
        if !cd.lambda.is_zero()
            && !cd.lambda.is_deleted()
            && !is_constrained_edge(&st.cdt, st.cdt.se(cd.se_in).edge)
        {
            st.cdt.delete_edge(cd.se_in);
        }
    }

    /* Insert segments for v1->v2. */
    let mut tstart = crossings[0].se_out;
    for i in 1..ncrossings {
        let cd = &crossings[i];
        if cd.lambda.is_deleted() {
            continue;
        }
        let mut t = NIL;
        let mut tnext = NIL;
        if !cd.lambda.is_zero() {
            if is_constrained_edge(&st.cdt, st.cdt.se(cd.se_in).edge) {
                t = st.cdt.verts[cd.vert].symedge;
                tnext = st.cdt.se(st.cdt.sym(t)).next;
            }
        } else {
            t = cd.se_in;
            tnext = cd.se_out;
            if t == NIL {
                /* Previous non-deleted crossing must also have been a vert,
                 * and segment should exist. */
                let mut j = i - 1;
                let mut cd_prev = crossings[0].clone();
                loop {
                    cd_prev = crossings[j].clone();
                    if !cd_prev.lambda.is_deleted() {
                        break;
                    }
                    if j == 0 {
                        break;
                    }
                    j -= 1;
                }
                debug_assert!(cd_prev.lambda.is_zero());
                debug_assert_eq!(st.cdt.se(st.cdt.se(cd_prev.se_out).next).vert, cd.vert);
                let edge = st.cdt.se(cd_prev.se_out).edge;
                st.cdt.edges[edge].input_ids.add(input_id);
                edge_list.push(edge);
            }
        }
        if t != NIL {
            let edge = if st.cdt.se(st.cdt.se(tstart).next).vert == st.cdt.se(t).vert {
                st.cdt.se(tstart).edge
            } else {
                st.cdt.add_diagonal(tstart, t)
            };
            st.cdt.edges[edge].input_ids.add(input_id);
            edge_list.push(edge);
            /* Re-triangulate upper and lower gaps. */
            re_delaunay_triangulate(&mut st.cdt, edge * 2);
            re_delaunay_triangulate(&mut st.cdt, edge * 2 + 1);
        }
        if i < ncrossings - 1 && tnext != NIL {
            tstart = tnext;
        }
    }
    if let Some(r) = r_edges {
        r.extend(edge_list);
    }
}

/// `add_edge_constraints`.
fn add_edge_constraints<S: CdtScalar>(st: &mut CdtState<S>, input: &CdtInput<S>) {
    let ne = input.edge.len() as u32;
    let nv = input.vert.len() as i32;
    for i in 0..ne as usize {
        let iv1 = input.edge[i][0];
        let iv2 = input.edge[i][1];
        if iv1 < 0 || iv1 >= nv || iv2 < 0 || iv2 >= nv {
            continue;
        }
        let v1 = st.cdt.get_vert_resolve_merge(iv1 as usize);
        let v2 = st.cdt.get_vert_resolve_merge(iv2 as usize);
        let id = if st.needed_ids & (CDT_ORIG_EDGES | CDT_INTERSECTED_EDGES) != 0 {
            i as u32
        } else {
            0
        };
        add_edge_constraint(st, v1, v2, id, None);
    }
    st.face_edge_offset = ne;
}

/// `id_range_in_list` — is any id in [range_start, range_end] in id_list?
fn id_range_in_list(id_list: &IdSet, range_start: u32, range_end: u32) -> bool {
    id_list.0.iter().any(|&id| id >= range_start && id <= range_end)
}

/// `add_face_ids` — flood-fill face_id to interior faces.
fn add_face_ids<S: CdtScalar>(st: &mut CdtState<S>, face_symedge: usize, face_id: u32, fedge_start: u32, fedge_end: u32) {
    st.visit_count += 1;
    let visit = st.visit_count;
    let mut stack: Vec<usize> = vec![face_symedge];
    while let Some(se0) = stack.pop() {
        let face = st.cdt.se(se0).face;
        if st.cdt.faces[face].visit_index == visit {
            continue;
        }
        st.cdt.faces[face].visit_index = visit;
        st.cdt.faces[face].input_ids.add(face_id);
        let se_start = se0;
        let mut se = st.cdt.se(se0).next;
        while se != se_start {
            let se_sym = st.cdt.sym(se);
            let face_other = st.cdt.se(se_sym).face;
            if st.cdt.faces[face_other].visit_index != visit {
                let e = st.cdt.se(se).edge;
                if !id_range_in_list(&st.cdt.edges[e].input_ids, fedge_start, fedge_end) {
                    stack.push(se_sym);
                }
            }
            se = st.cdt.se(se).next;
        }
    }
}

/// `add_face_constraints` — returns number of face-edges added.
fn add_face_constraints<S: CdtScalar>(
    st: &mut CdtState<S>,
    input: &CdtInput<S>,
    output_type: CdtOutputType,
    need_winding: bool,
) -> i32 {
    let nv = input.vert.len() as i32;
    let num_faces = input.face_offsets.len().saturating_sub(1);
    let face_vert = |f: usize| -> &[i32] {
        &input.face_vert_indices[input.face_offsets[f]..input.face_offsets[f + 1]]
    };
    let mut maxflen: u32 = 0;
    for f in 0..num_faces {
        maxflen = maxflen.max(face_vert(f).len() as u32);
    }
    st.face_edge_offset = maxflen.max(st.face_edge_offset);
    let mut faces_added = 0i32;

    let need_orig_face_ids = (st.needed_ids & CDT_ORIG_FACES) != 0
        || output_type == CdtOutputType::ConstraintsValidBmesh;
    let skip_cw_ids = need_orig_face_ids
        && (st.needed_ids & CDT_CW_ORIG_FACES) == 0
        && !matches!(
            output_type,
            CdtOutputType::ConstraintsValidBmesh
                | CdtOutputType::ConstraintsValidBmeshWithHoles
                | CdtOutputType::ConstraintsValidBmeshWithHolesNonzero
        );

    for f in 0..num_faces {
        let flen = face_vert(f).len();
        if flen <= 2 {
            continue;
        }
        let mut signed_area = 0.0f64;
        let fedge_start = (f as u32 + 1) * st.face_edge_offset;
        let mut face_symedge0 = NIL;
        for i in 0..flen {
            let face_edge_id = fedge_start + i as u32;
            let iv1 = face_vert(f)[i];
            let iv2 = face_vert(f)[(i + 1) % flen];
            if iv1 < 0 || iv1 >= nv || iv2 < 0 || iv2 >= nv {
                continue;
            }
            faces_added += 1;
            let v1 = st.cdt.get_vert_resolve_merge(iv1 as usize);
            let v2 = st.cdt.get_vert_resolve_merge(iv2 as usize);
            if skip_cw_ids {
                let (a, b) = (st.cdt.verts[v1].co.approx, st.cdt.verts[v2].co.approx);
                signed_area += a[0] * b[1] - a[1] * b[0];
            }
            let mut edge_list: Vec<usize> = Vec::new();
            add_edge_constraint(st, v1, v2, face_edge_id, Some(&mut edge_list));
            if !edge_list.is_empty() {
                let face_edge = edge_list[0];
                let mut fse = face_edge * 2;
                if st.cdt.se(fse).vert != v1 {
                    fse = face_edge * 2 + 1;
                    debug_assert_eq!(st.cdt.se(fse).vert, v1);
                }
                face_symedge0 = fse;
                if st.polygon_boundary_count_map.is_some() || need_winding {
                    let mut curr_vert = v1;
                    for &e in &edge_list {
                        let (next_vert, winding_delta) =
                            if st.cdt.se(e * 2).vert == curr_vert {
                                (st.cdt.se(e * 2 + 1).vert, 1)
                            } else {
                                debug_assert_eq!(st.cdt.se(e * 2 + 1).vert, curr_vert);
                                (st.cdt.se(e * 2).vert, -1)
                            };
                        if let Some(map) = st.polygon_boundary_count_map.as_mut() {
                            *map.entry(e).or_insert(0) += 1;
                        }
                        if need_winding {
                            if let Some(map) = st.edge_winding_map.as_mut() {
                                *map.entry(e).or_insert(0) += winding_delta;
                            }
                        }
                        curr_vert = next_vert;
                    }
                }
            }
        }
        let fedge_end = fedge_start + flen as u32 - 1;
        if face_symedge0 != NIL && need_orig_face_ids && (!skip_cw_ids || signed_area >= 0.0) {
            let id = if st.needed_ids & CDT_ORIG_FACES != 0 {
                f as u32
            } else {
                0
            };
            add_face_ids(st, face_symedge0, id, fedge_start, fedge_end);
        }
    }
    faces_added
}

fn vert_touches_face<S: CdtScalar>(cdt: &CdtArrangement<S>, v: usize, f: usize) -> bool {
    let mut se = cdt.verts[v].symedge;
    let start = se;
    loop {
        if cdt.se(se).face == f {
            return true;
        }
        se = cdt.se(se).rot;
        if se == start {
            break;
        }
    }
    false
}

/// `dissolve_symedge` — delete_edge preserving outer face / face symedges.
fn dissolve_symedge<S: CdtScalar>(st: &mut CdtState<S>, se_in: usize) {
    let cdt = &mut st.cdt;
    let mut se = se_in;
    let mut symse = cdt.sym(se);
    if cdt.se(symse).face == cdt.outer_face {
        se = cdt.sym(se);
        symse = cdt.sym(se);
    }
    let os = cdt.faces[cdt.outer_face].symedge;
    if os == se || os == symse {
        let nn = cdt.se(cdt.se(se).next).next;
        if nn == se {
            cdt.faces[cdt.outer_face].symedge = NIL;
        } else {
            cdt.faces[cdt.outer_face].symedge = nn;
        }
    } else {
        let v1_isolated = cdt.se(symse).next == se;
        let v2_isolated = cdt.se(se).next == symse;
        let f_se = cdt.se(se).face;
        let f_sym = cdt.se(symse).face;
        if cdt.faces[f_se].symedge == se {
            cdt.faces[f_se].symedge = if v2_isolated { cdt.prev(se) } else { cdt.se(se).next };
        }
        if cdt.faces[f_sym].symedge == symse {
            cdt.faces[f_sym].symedge =
                if v1_isolated { cdt.prev(symse) } else { cdt.se(symse).next };
        }
    }
    cdt.delete_edge(se);
}

/// `remove_non_constraint_edges`.
fn remove_non_constraint_edges<S: CdtScalar>(st: &mut CdtState<S>) {
    let n = st.cdt.edges.len();
    for e in 0..n {
        if !is_deleted_edge(&st.cdt, e) && !is_constrained_edge(&st.cdt, e) {
            dissolve_symedge(st, e * 2);
        }
    }
}

/// `remove_non_constraint_edges_leave_valid_bmesh`.
fn remove_non_constraint_edges_leave_valid_bmesh<S: CdtScalar>(st: &mut CdtState<S>, use_hole_status: bool) {
    struct EdgeToSort {
        len_squared: f64,
        e: usize,
    }
    let nedges = st.cdt.edges.len();
    if nedges == 0 {
        return;
    }
    let mut dissolvable: Vec<EdgeToSort> = Vec::with_capacity(nedges);
    for e in 0..nedges {
        if !is_deleted_edge(&st.cdt, e) && !is_constrained_edge(&st.cdt, e) {
            let co1 = st.cdt.verts[st.cdt.se(e * 2).vert].co.approx;
            let co2 = st.cdt.verts[st.cdt.se(e * 2 + 1).vert].co.approx;
            dissolvable.push(EdgeToSort {
                len_squared: distance_squared2(co1, co2),
                e,
            });
        }
    }
    dissolvable.sort_by(|a, b| {
        let c = a.len_squared.partial_cmp(&b.len_squared).unwrap();
        if c != std::cmp::Ordering::Equal {
            return c;
        }
        let ev = |cdt: &CdtArrangement<S>, e: usize| -> (i32, i32) {
            let v0 = cdt.verts[cdt.se(e * 2).vert].index;
            let v1 = cdt.verts[cdt.se(e * 2 + 1).vert].index;
            if v0 < v1 {
                (v0, v1)
            } else {
                (v1, v0)
            }
        };
        ev(&st.cdt, a.e).cmp(&ev(&st.cdt, b.e))
    });
    let is_output_face = |cdt: &CdtArrangement<S>, f: usize| {
        if use_hole_status {
            !cdt.faces[f].hole
        } else {
            !cdt.faces[f].input_ids.is_empty()
        }
    };
    for ets in &dissolvable {
        let e = ets.e;
        if is_deleted_edge(&st.cdt, e) {
            continue;
        }
        let se = e * 2;
        let mut dissolve = true;
        let fleft = st.cdt.se(se).face;
        let fright = st.cdt.se(st.cdt.sym(se)).face;
        if fleft != st.cdt.outer_face
            && fright != st.cdt.outer_face
            && (is_output_face(&st.cdt, fleft) || is_output_face(&st.cdt, fright))
        {
            let mut se2 = st.cdt.se(se).next;
            while dissolve && se2 != se {
                if st.cdt.se(st.cdt.sym(se2)).face == fright
                    || (st.cdt.se(se2).vert != st.cdt.se(st.cdt.se(se).next).vert
                        && vert_touches_face(&st.cdt, st.cdt.se(se2).vert, fright))
                {
                    dissolve = false;
                }
                se2 = st.cdt.se(se2).next;
            }
        }
        if dissolve {
            dissolve_symedge(st, se);
        }
    }
}

/// `remove_outer_edges_until_constraints`.
fn remove_outer_edges_until_constraints<S: CdtScalar>(st: &mut CdtState<S>) {
    st.visit_count += 1;
    let visit = st.visit_count;
    st.cdt.faces[st.cdt.outer_face].visit_index = visit;
    let mut fstack: Vec<usize> = Vec::new();
    let se_start = st.cdt.faces[st.cdt.outer_face].symedge;
    let mut se = se_start;
    if se != NIL {
        loop {
            let e = st.cdt.se(se).edge;
            if !is_constrained_edge(&st.cdt, e) {
                let fsym = st.cdt.se(st.cdt.sym(se)).face;
                if st.cdt.faces[fsym].visit_index != visit {
                    fstack.push(fsym);
                }
            }
            se = st.cdt.se(se).next;
            if se == se_start {
                break;
            }
        }
    }
    while let Some(f) = fstack.pop() {
        if st.cdt.faces[f].visit_index == visit {
            continue;
        }
        debug_assert_ne!(f, st.cdt.outer_face);
        st.cdt.faces[f].visit_index = visit;
        let se_start = st.cdt.faces[f].symedge;
        if se_start == NIL {
            continue;
        }
        let mut se = se_start;
        let mut to_dissolve: Vec<usize> = Vec::new();
        loop {
            let e = st.cdt.se(se).edge;
            if !is_constrained_edge(&st.cdt, e) {
                let fsym = st.cdt.se(st.cdt.sym(se)).face;
                if st.cdt.faces[fsym].visit_index != visit {
                    fstack.push(fsym);
                } else {
                    to_dissolve.push(se);
                }
            }
            se = st.cdt.se(se).next;
            if se == se_start {
                break;
            }
        }
        for se in to_dissolve {
            if st.cdt.se(se).next != NIL {
                dissolve_symedge(st, se);
            }
        }
    }
}

/// `remove_faces_in_holes`.
fn remove_faces_in_holes<S: CdtScalar>(st: &mut CdtState<S>) {
    let n = st.cdt.faces.len();
    for i in 0..n {
        if !st.cdt.faces[i].deleted && st.cdt.faces[i].hole {
            st.cdt.faces[i].deleted = true;
            let mut se = st.cdt.faces[i].symedge;
            let se_start = se;
            loop {
                debug_assert_ne!(se, NIL);
                let se_next = st.cdt.se(se).next;
                if !is_constrained_edge(&st.cdt, st.cdt.se(se).edge) {
                    st.cdt.sem(se).next = NIL;
                }
                se = se_next;
                if se == se_start {
                    break;
                }
            }
        }
    }
}

const VISIT_INDEX_UNVISITED: i32 = -1;
const VISIT_INDEX_OUTER_FACE: i32 = -2;

/// `flood_fill_region_values` — generic over parity/winding domain.
fn flood_fill_region_values<V: Copy + PartialEq>(
    region_pair_value: &std::collections::HashMap<(i32, i32), V>,
    region_value: &mut [V],
    unset_value: V,
    combine: impl Fn(V, V) -> V,
) {
    let num_regions = region_value.len();
    /* CSR adjacency. */
    let mut offsets = vec![0i64; num_regions + 1];
    for &(src, _dst) in region_pair_value.keys() {
        offsets[src as usize] += 1;
    }
    let mut total = 0i64;
    for o in offsets.iter_mut().take(num_regions) {
        total += *o;
        *o = total - *o;
    }
    offsets[num_regions] = total;
    let mut adjacency: Vec<(i32, V)> = vec![(0, unset_value); total as usize];
    let mut write_position = vec![0i64; num_regions];
    for (&(src, dst), &val) in region_pair_value.iter() {
        let idx = (offsets[src as usize] + write_position[src as usize]) as usize;
        adjacency[idx] = (dst, val);
        write_position[src as usize] += 1;
    }
    let mut stack: Vec<usize> = (0..num_regions)
        .filter(|&r| region_value[r] != unset_value)
        .collect();
    while let Some(region) = stack.pop() {
        let cur = region_value[region];
        for i in offsets[region] as usize..offsets[region + 1] as usize {
            let (neighbor, val) = adjacency[i];
            if region_value[neighbor as usize] == unset_value {
                region_value[neighbor as usize] = combine(cur, val);
                stack.push(neighbor as usize);
            }
        }
    }
}

/// `detect_holes_with_fillrule_even_odd`.
fn detect_holes_with_fillrule_even_odd<S: CdtScalar>(st: &mut CdtState<S>) {
    debug_assert!(st.polygon_boundary_count_map.is_some());
    let boundary_count = st.polygon_boundary_count_map.take().unwrap();

    let mut boundary_regions: Vec<(i32, i8)> = Vec::new();
    let mut fstack: Vec<usize> = Vec::with_capacity(st.cdt.faces.len());
    for f in st.cdt.faces.iter_mut() {
        f.visit_index = VISIT_INDEX_UNVISITED;
    }
    st.cdt.faces[st.cdt.outer_face].visit_index = VISIT_INDEX_OUTER_FACE;

    let mut cur_region: i32 = -1;
    for f_init in 0..st.cdt.faces.len() {
        if st.cdt.faces[f_init].deleted
            || st.cdt.faces[f_init].symedge == NIL
            || st.cdt.faces[f_init].visit_index != VISIT_INDEX_UNVISITED
        {
            continue;
        }
        cur_region += 1;
        st.cdt.faces[f_init].visit_index = cur_region;
        fstack.push(f_init);
        let mut outer_parity: i8 = -1;
        while let Some(f) = fstack.pop() {
            debug_assert_eq!(st.cdt.faces[f].visit_index, cur_region);
            let se_start = st.cdt.faces[f].symedge;
            let mut se = se_start;
            loop {
                let e = st.cdt.se(se).edge;
                let se_sym = st.cdt.sym(se);
                let neighbor = st.cdt.se(se_sym).face;
                let constrained = is_constrained_edge(&st.cdt, e);
                if neighbor == st.cdt.outer_face {
                    let flip = (*boundary_count.get(&e).unwrap_or(&0) & 1) as i8;
                    if outer_parity == -1 {
                        outer_parity = flip;
                    } else if outer_parity != flip {
                        outer_parity = 1;
                    }
                } else if !constrained
                    && !st.cdt.faces[neighbor].deleted
                    && st.cdt.faces[neighbor].visit_index == VISIT_INDEX_UNVISITED
                {
                    st.cdt.faces[neighbor].visit_index = cur_region;
                    fstack.push(neighbor);
                }
                se = st.cdt.se(se).next;
                if se == se_start {
                    break;
                }
            }
        }
        if outer_parity != -1 {
            boundary_regions.push((cur_region, outer_parity));
        }
    }
    let num_regions = cur_region + 1;
    st.visit_count = num_regions;
    if num_regions == 0 {
        st.polygon_boundary_count_map = Some(boundary_count);
        return;
    }

    let mut region_pair_flip: std::collections::HashMap<(i32, i32), i8> =
        std::collections::HashMap::with_capacity(6 * num_regions as usize);
    for e in 0..st.cdt.edges.len() {
        if is_deleted_edge(&st.cdt, e) || !is_constrained_edge(&st.cdt, e) {
            continue;
        }
        let region0 = st.cdt.faces[st.cdt.se(e * 2).face].visit_index;
        let region1 = st.cdt.faces[st.cdt.se(e * 2 + 1).face].visit_index;
        if region0 == region1 || region0 < 0 || region1 < 0 {
            continue;
        }
        let flip = (*boundary_count.get(&e).unwrap_or(&0) & 1) as i8;
        region_pair_flip.entry((region0, region1)).or_insert(flip);
        region_pair_flip.entry((region1, region0)).or_insert(flip);
    }
    let mut region_parity = vec![-1i8; num_regions as usize];
    for &(region, parity) in &boundary_regions {
        if region_parity[region as usize] == -1 {
            region_parity[region as usize] = parity;
        }
    }
    flood_fill_region_values(&region_pair_flip, &mut region_parity, -1i8, |cur, flip| {
        cur ^ flip
    });
    for f in st.cdt.faces.iter_mut() {
        let region = f.visit_index;
        if region >= 0 {
            f.hole = region_parity[region as usize] != 1;
        }
    }
    st.polygon_boundary_count_map = Some(boundary_count);
}

/// `detect_holes_with_fillrule_nonzero`.
fn detect_holes_with_fillrule_nonzero<S: CdtScalar>(st: &mut CdtState<S>) {
    let winding_map = st.edge_winding_map.take().unwrap_or_default();

    let mut boundary_regions: Vec<(i32, i32)> = Vec::new();
    let mut fstack: Vec<usize> = Vec::with_capacity(st.cdt.faces.len());
    for f in st.cdt.faces.iter_mut() {
        f.visit_index = VISIT_INDEX_UNVISITED;
    }
    st.cdt.faces[st.cdt.outer_face].visit_index = VISIT_INDEX_OUTER_FACE;

    let mut cur_region: i32 = -1;
    for f_init in 0..st.cdt.faces.len() {
        if st.cdt.faces[f_init].deleted
            || st.cdt.faces[f_init].symedge == NIL
            || st.cdt.faces[f_init].visit_index != VISIT_INDEX_UNVISITED
        {
            continue;
        }
        cur_region += 1;
        st.cdt.faces[f_init].visit_index = cur_region;
        fstack.push(f_init);
        let mut found_constrained_outer = false;
        let mut found_any_outer = false;
        let mut outer_winding = 0i32;
        while let Some(f) = fstack.pop() {
            debug_assert_eq!(st.cdt.faces[f].visit_index, cur_region);
            let se_start = st.cdt.faces[f].symedge;
            let mut se = se_start;
            loop {
                let e = st.cdt.se(se).edge;
                let se_sym = st.cdt.sym(se);
                let neighbor = st.cdt.se(se_sym).face;
                if is_constrained_edge(&st.cdt, e) {
                    if neighbor == st.cdt.outer_face && !found_constrained_outer {
                        found_constrained_outer = true;
                        found_any_outer = true;
                        let winding = *winding_map.get(&e).unwrap_or(&0);
                        if se == e * 2 {
                            outer_winding = winding;
                        } else {
                            outer_winding = -winding;
                        }
                    }
                } else if neighbor == st.cdt.outer_face {
                    found_any_outer = true;
                } else if !st.cdt.faces[neighbor].deleted
                    && st.cdt.faces[neighbor].visit_index == VISIT_INDEX_UNVISITED
                {
                    st.cdt.faces[neighbor].visit_index = cur_region;
                    fstack.push(neighbor);
                }
                se = st.cdt.se(se).next;
                if se == se_start {
                    break;
                }
            }
        }
        if found_any_outer {
            boundary_regions.push((cur_region, outer_winding));
        }
    }
    let num_regions = cur_region + 1;
    st.visit_count = num_regions;
    if num_regions == 0 {
        st.edge_winding_map = Some(winding_map);
        return;
    }

    let mut region_pair_winding: std::collections::HashMap<(i32, i32), i32> =
        std::collections::HashMap::with_capacity(6 * num_regions as usize);
    for e in 0..st.cdt.edges.len() {
        if is_deleted_edge(&st.cdt, e) || !is_constrained_edge(&st.cdt, e) {
            continue;
        }
        let region0 = st.cdt.faces[st.cdt.se(e * 2).face].visit_index;
        let region1 = st.cdt.faces[st.cdt.se(e * 2 + 1).face].visit_index;
        if region0 == region1 || region0 < 0 || region1 < 0 {
            continue;
        }
        let winding = *winding_map.get(&e).unwrap_or(&0);
        region_pair_winding
            .entry((region0, region1))
            .or_insert(-winding);
        region_pair_winding
            .entry((region1, region0))
            .or_insert(winding);
    }
    let mut region_winding = vec![i32::MIN; num_regions as usize];
    for &(region, winding) in &boundary_regions {
        if region_winding[region as usize] == i32::MIN {
            region_winding[region as usize] = winding;
        }
    }
    flood_fill_region_values(&region_pair_winding, &mut region_winding, i32::MIN, |cur, d| {
        cur + d
    });
    for f in st.cdt.faces.iter_mut() {
        let region = f.visit_index;
        if region >= 0 {
            let winding = region_winding[region as usize];
            f.hole = winding == 0 || winding == i32::MIN;
        }
    }
    st.edge_winding_map = Some(winding_map);
}

/// `prepare_cdt_for_output`.
fn prepare_cdt_for_output<S: CdtScalar>(st: &mut CdtState<S>, output_type: CdtOutputType) {
    if st.cdt.edges.is_empty() {
        return;
    }
    /* All non-deleted faces get a symedge. */
    for e in 0..st.cdt.edges.len() {
        if !is_deleted_edge(&st.cdt, e) {
            let f0 = st.cdt.se(e * 2).face;
            if st.cdt.faces[f0].symedge == NIL {
                st.cdt.faces[f0].symedge = e * 2;
            }
            let f1 = st.cdt.se(e * 2 + 1).face;
            if st.cdt.faces[f1].symedge == NIL {
                st.cdt.faces[f1].symedge = e * 2 + 1;
            }
        }
    }
    if output_uses_evenodd_holes(output_type) {
        detect_holes_with_fillrule_even_odd(st);
    } else if output_uses_nonzero_holes(output_type) {
        detect_holes_with_fillrule_nonzero(st);
    }
    match output_type {
        CdtOutputType::Constraints => remove_non_constraint_edges(st),
        CdtOutputType::ConstraintsValidBmesh => {
            remove_non_constraint_edges_leave_valid_bmesh(st, false)
        }
        CdtOutputType::Inside => remove_outer_edges_until_constraints(st),
        CdtOutputType::InsideWithHoles | CdtOutputType::InsideWithHolesNonzero => {
            remove_outer_edges_until_constraints(st);
            remove_faces_in_holes(st);
        }
        CdtOutputType::ConstraintsValidBmeshWithHoles
        | CdtOutputType::ConstraintsValidBmeshWithHolesNonzero => {
            remove_outer_edges_until_constraints(st);
            remove_non_constraint_edges_leave_valid_bmesh(st, true);
            remove_faces_in_holes(st);
        }
        _ => {}
    }
}

/// `get_cdt_output`.
fn get_cdt_output<S: CdtScalar>(st: &mut CdtState<S>, output_type: CdtOutputType) -> CdtResult<S> {
    prepare_cdt_for_output(st, output_type);
    let mut result = CdtResult {
        face_edge_offset: st.face_edge_offset,
        ..Default::default()
    };
    let verts_size = st.cdt.verts.len();
    let mut vert_to_output_map = vec![0usize; verts_size];
    let mut nv = 0usize;
    for i in 0..verts_size {
        if st.cdt.verts[i].merge_to_index == -1 {
            vert_to_output_map[i] = nv;
            nv += 1;
        }
    }
    if nv == 0 {
        return result;
    }
    if nv < verts_size {
        for i in 0..verts_size {
            let m = st.cdt.verts[i].merge_to_index;
            if m != -1 {
                if st.needed_ids & CDT_ORIG_VERTS != 0 && (i as i32) < st.input_vert_num {
                    let mt = m as usize;
                    let tmp = IdSet(vec![i as u32]);
                    st.cdt.verts[mt].input_ids.add_all(&tmp);
                }
                vert_to_output_map[i] = vert_to_output_map[m as usize];
            }
        }
    }
    result.vert = Vec::with_capacity(nv);
    if st.needed_ids & CDT_ORIG_VERTS != 0 {
        result.vert_orig = vec![Vec::new(); nv];
    }
    if st.needed_ids & CDT_INTERSECTED_EDGES != 0 {
        result.intersected_edges_orig = vec![[-1, -1]; nv];
    }
    let mut i_out = 0;
    for i in 0..verts_size {
        let v = &st.cdt.verts[i];
        if v.merge_to_index == -1 {
            result.vert.push(v.co.exact.clone());
            if st.needed_ids & CDT_ORIG_VERTS != 0 {
                if (i as i32) < st.input_vert_num {
                    result.vert_orig[i_out].push(i as u32);
                }
                result.vert_orig[i_out].extend_from_slice(&v.input_ids.0);
            }
            if st.needed_ids & CDT_INTERSECTED_EDGES != 0 {
                result.intersected_edges_orig[i_out] = v.intersected_edges;
            }
            i_out += 1;
        }
    }

    /* Edges. */
    let ne = (0..st.cdt.edges.len())
        .filter(|&e| !is_deleted_edge(&st.cdt, e))
        .count();
    result.edge = Vec::with_capacity(ne);
    if st.needed_ids & CDT_ORIG_EDGES != 0 {
        result.edge_orig = vec![Vec::new(); ne];
    }
    for e in 0..st.cdt.edges.len() {
        if is_deleted_edge(&st.cdt, e) {
            continue;
        }
        let vo1 = vert_to_output_map[st.cdt.verts[st.cdt.se(e * 2).vert].index as usize];
        let vo2 = vert_to_output_map[st.cdt.verts[st.cdt.se(e * 2 + 1).vert].index as usize];
        result.edge.push([vo1 as i32, vo2 as i32]);
        if st.needed_ids & CDT_ORIG_EDGES != 0 {
            result.edge_orig[result.edge.len() - 1].extend_from_slice(&st.cdt.edges[e].input_ids.0);
        }
    }

    /* Faces. */
    let nf = (0..st.cdt.faces.len())
        .filter(|&f| !st.cdt.faces[f].deleted && f != st.cdt.outer_face)
        .count();
    result.face = Vec::with_capacity(nf);
    if st.needed_ids & CDT_ORIG_FACES != 0 {
        result.face_orig = vec![Vec::new(); nf];
    }
    for f in 0..st.cdt.faces.len() {
        if st.cdt.faces[f].deleted || f == st.cdt.outer_face {
            continue;
        }
        let mut face_verts: Vec<i32> = Vec::new();
        let se_start = st.cdt.faces[f].symedge;
        debug_assert_ne!(se_start, NIL);
        let mut se = se_start;
        loop {
            face_verts.push(
                vert_to_output_map[st.cdt.verts[st.cdt.se(se).vert].index as usize] as i32,
            );
            se = st.cdt.se(se).next;
            if se == se_start {
                break;
            }
        }
        result.face.push(face_verts);
        if st.needed_ids & CDT_ORIG_FACES != 0 {
            result.face_orig[result.face.len() - 1]
                .extend_from_slice(&st.cdt.faces[f].input_ids.0);
        }
    }
    result
}

fn add_input_verts<S: CdtScalar>(st: &mut CdtState<S>, input: &CdtInput<S>) {
    for i in 0..st.input_vert_num as usize {
        st.cdt.add_vert(input.vert[i].clone());
    }
}

/// `delaunay_calc` -> public `delaunay_2d_calc` (f64 specialization only;
/// mpq/WITH_GMP variant is PORT_PENDING).
pub fn delaunay_2d_calc<S: CdtScalar>(input: &CdtInput<S>, output_type: CdtOutputType) -> CdtResult<S> {
    let nv = input.vert.len();
    let ne = input.edge.len();
    let nf = input.face_offsets.len().saturating_sub(1);
    let mut st = CdtState {
        cdt: CdtArrangement {
            verts: Vec::with_capacity(2 * nv),
            edges: Vec::with_capacity(3 * nv + 2 * ne + 6 * nf),
            symedges: Vec::with_capacity(2 * (3 * nv + 2 * ne + 6 * nf)),
            faces: Vec::with_capacity(2 * nv + 2 * ne + 2 * nf),
            outer_face: 0,
        },
        input_vert_num: nv as i32,
        visit_count: 0,
        face_edge_offset: 0,
        epsilon: input.epsilon.clone(),
        needed_ids: input.needed_ids,
        edge_winding_map: None,
        polygon_boundary_count_map: None,
    };
    st.cdt.outer_face = st.cdt.add_face();
    let need_winding = output_uses_nonzero_holes(output_type);
    let need_polygon_boundary_count = output_uses_evenodd_holes(output_type);
    if need_winding {
        st.edge_winding_map = Some(std::collections::HashMap::new());
    }
    if need_polygon_boundary_count {
        st.polygon_boundary_count_map = Some(std::collections::HashMap::new());
    }
    add_input_verts(&mut st, input);
    initial_triangulation(&mut st.cdt);
    add_edge_constraints(&mut st, input);
    let actual_nf = add_face_constraints(&mut st, input, output_type, need_winding);
    let mut output_type = output_type;
    if actual_nf == 0
        && !matches!(
            output_type,
            CdtOutputType::Full | CdtOutputType::Inside | CdtOutputType::Constraints
        )
    {
        output_type = CdtOutputType::Inside;
    }
    get_cdt_output(&mut st, output_type)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn triangulate_square() {
        let verts = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let edges = [[0, 1], [1, 2], [2, 3], [3, 0]];
        let input = CdtInput {
            vert: &verts,
            edge: &edges,
            face_offsets: &[0],
            face_vert_indices: &[],
            epsilon: 0.0,
            needed_ids: CDT_NO_ORIG_IDS,
        };
        let out = delaunay_2d_calc(&input, CdtOutputType::Full);
        assert_eq!(out.vert.len(), 4);
        /* A triangulated quad: 4 hull edges + 1 diagonal = 5 edges, 2 faces. */
        assert_eq!(out.edge.len(), 5);
        assert_eq!(out.face.len(), 2);
    }

    #[test]
    fn constrained_square_face_inside() {
        let verts = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
        let input = CdtInput {
            vert: &verts,
            edge: &[],
            face_offsets: &[0, 4],
            face_vert_indices: &[0, 1, 2, 3],
            epsilon: 0.0,
            needed_ids: CDT_ORIG_FACES,
        };
        let out = delaunay_2d_calc(&input, CdtOutputType::Inside);
        assert_eq!(out.vert.len(), 4);
        assert_eq!(out.face.len(), 2); /* Two triangles inside the square. */
        assert!(out
            .face_orig
            .iter()
            .all(|orig| orig.contains(&0)));
    }

    #[test]
    fn mpq_cdt_square() {
        use blenders_math::mpq::{mpq, Mpq, Mpq2};
        let verts = [
            Mpq2::new(mpq(0.0), mpq(0.0)),
            Mpq2::new(mpq(1.0), mpq(0.0)),
            Mpq2::new(mpq(1.0), mpq(1.0)),
            Mpq2::new(mpq(0.0), mpq(1.0)),
        ];
        let edges = [[0, 1], [1, 2], [2, 3], [3, 0]];
        let input = CdtInput::<Mpq> {
            vert: &verts,
            edge: &edges,
            face_offsets: &[0],
            face_vert_indices: &[],
            epsilon: Mpq::default(),
            needed_ids: CDT_NO_ORIG_IDS,
        };
        let out = delaunay_2d_calc(&input, CdtOutputType::Full);
        assert_eq!(out.vert.len(), 4);
        assert_eq!(out.edge.len(), 5);
        assert_eq!(out.face.len(), 2);
    }
}
