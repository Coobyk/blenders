//! Port of `math_boolean.cc` — Shewchuk's exact arithmetic predicates.
//!
//! Public API (mpq exact versions + adaptive double-precision versions).
//! The mpq (arbitrary-precision) variants map to the same adaptive
//! result; the expansion-arithmetic internals (`exactinit`,
//! `*adapt`, `insphereexact`, `fast_expansion_sum_zeroelim`,
//! `scale_expansion_zeroelim`, `estimate`) are pending — marked
//! PORT_PENDING. Current predicates are f64-accurate, not yet the
//! exact-arithmetic fallbacks.

use crate::vector::{Vec2, Vec3};

fn sgn(x: f64) -> i32 {
    if x > 0.0 {
        1
    } else if x < 0.0 {
        -1
    } else {
        0
    }
}

/* Error bounds (exactinit, IEEE double, no FP contraction). */
const EPSILON: f64 = f64::EPSILON / 2.0;
const SPLITTER: f64 = 4096.0;
const _RESULTERRBOUND: f64 = (3.0 + 8.0 * EPSILON) * EPSILON;
const CCWERRBOUND_A: f64 = (3.0 + 16.0 * EPSILON) * EPSILON;
const _CCWERRBOUND_B: f64 = (2.0 + 12.0 * EPSILON) * EPSILON;
const _CCWERRBOUND_C: f64 = (9.0 + 64.0 * EPSILON) * EPSILON * EPSILON;
const O3DERRBOUND_A: f64 = (7.0 + 56.0 * EPSILON) * EPSILON;
const _O3DERRBOUND_B: f64 = (3.0 + 28.0 * EPSILON) * EPSILON;
const _O3DERRBOUND_C: f64 = (26.0 + 288.0 * EPSILON) * EPSILON * EPSILON;
const ICCERRBOUND_A: f64 = (10.0 + 96.0 * EPSILON) * EPSILON;
const _ICCERRBOUND_B: f64 = (4.0 + 48.0 * EPSILON) * EPSILON;
const _ICCERRBOUND_C: f64 = (44.0 + 576.0 * EPSILON) * EPSILON * EPSILON;
const ISPERRBOUND_A: f64 = (16.0 + 224.0 * EPSILON) * EPSILON;
const _ISPERRBOUND_B: f64 = (5.0 + 72.0 * EPSILON) * EPSILON;
const _ISPERRBOUND_C: f64 = (71.0 + 1408.0 * EPSILON) * EPSILON * EPSILON;

/// `orient2d(mpq2,mpq2,mpq2)` / `orient2d(double2,...)` — sign of
/// cross((b-a),(c-a)).
pub fn orient2d(a: Vec2, b: Vec2, c: Vec2) -> i32 {
    let d = (b.x as f64 - a.x as f64) * (c.y as f64 - a.y as f64)
        - (b.y as f64 - a.y as f64) * (c.x as f64 - a.x as f64);
    sgn(d)
}

/// `orient2d_fast` — same test without exact fallback.
pub fn orient2d_fast(a: Vec2, b: Vec2, c: Vec2) -> i32 {
    orient2d(a, b, c)
}

/// `incircle` — sign of incircle(a,b,c,d).
pub fn incircle(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> i32 {
    let adx = a.x as f64 - d.x as f64;
    let ady = a.y as f64 - d.y as f64;
    let bdx = b.x as f64 - d.x as f64;
    let bdy = b.y as f64 - d.y as f64;
    let cdx = c.x as f64 - d.x as f64;
    let cdy = c.y as f64 - d.y as f64;
    let det = (adx * adx + ady * ady) * (bdx * cdy - bdy * cdx)
        - (bdx * bdx + bdy * bdy) * (adx * cdy - ady * cdx)
        + (cdx * cdx + cdy * cdy) * (adx * bdy - ady * bdx);
    sgn(det)
}

/// `incircle_fast`.
pub fn incircle_fast(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> i32 {
    incircle(a, b, c, d)
}

/// `orient3d` — sign of det(b-a, c-a, d-a).
pub fn orient3d(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> i32 {
    let ax = a.x as f64 - d.x as f64;
    let ay = a.y as f64 - d.y as f64;
    let az = a.z as f64 - d.z as f64;
    let bx = b.x as f64 - d.x as f64;
    let by = b.y as f64 - d.y as f64;
    let bz = b.z as f64 - d.z as f64;
    let cx = c.x as f64 - d.x as f64;
    let cy = c.y as f64 - d.y as f64;
    let cz = c.z as f64 - d.z as f64;
    sgn(ax * (by * cz - bz * cy) - ay * (bx * cz - bz * cx) + az * (bx * cy - by * cx))
}

/// `orient3d_fast`.
pub fn orient3d_fast(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> i32 {
    orient3d(a, b, c, d)
}

/// `insphere` — sign of insphere(a,b,c,d,e) (a-d define sphere, e tested).
pub fn insphere(a: Vec3, b: Vec3, c: Vec3, d: Vec3, e: Vec3) -> i32 {
    // Determinant-based test: |e inside sphere through a,b,c,d|.
    let ax = a.x as f64 - e.x as f64;
    let ay = a.y as f64 - e.y as f64;
    let az = a.z as f64 - e.z as f64;
    let bx = b.x as f64 - e.x as f64;
    let by = b.y as f64 - e.y as f64;
    let bz = b.z as f64 - e.z as f64;
    let cx = c.x as f64 - e.x as f64;
    let cy = c.y as f64 - e.y as f64;
    let cz = c.z as f64 - e.z as f64;
    let dx = d.x as f64 - e.x as f64;
    let dy = d.y as f64 - e.y as f64;
    let dz = d.z as f64 - e.z as f64;
    let a2 = ax * ax + ay * ay + az * az;
    let b2 = bx * bx + by * by + bz * bz;
    let c2 = cx * cx + cy * cy + cz * cz;
    let d2 = dx * dx + dy * dy + dz * dz;
    let cdx = cy * dz - cz * dy;
    let cdy = cz * dx - cx * dz;
    let cdz = cx * dy - cy * dx;
    let det = a2 * (bx * cdy - by * cdx + bz * cdz - dx * (by * cz - bz * cy)
        + dy * (bx * cz - bz * cx) - dz * (bx * cy - by * cx))
        - b2 * (ax * (cy * dz - cz * dy) - ay * (cx * dz - cz * dx) + az * (cx * dy - cy * dx))
        + c2 * (ax * (by * dz - bz * dy) - ay * (bx * dz - bz * dx) + az * (bx * dy - by * dx))
        - d2 * (ax * (by * cz - bz * cy) - ay * (bx * cz - bz * cx) + az * (bx * cy - by * cx));
    sgn(det)
}

/// `insphere_fast`.
pub fn insphere_fast(a: Vec3, b: Vec3, c: Vec3, d: Vec3, e: Vec3) -> i32 {
    insphere(a, b, c, d, e)
}
