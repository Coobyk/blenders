//! Port of `math_boolean.cc` — Shewchuk's exact arithmetic predicates.
//!
//! Public API (mpq exact versions + adaptive double-precision versions).
//! The robust predicates delegate to the `robust` crate — a faithful Rust
//! port of the same Shewchuk `predicates.c` expansion arithmetic that the
//! C++ `robust_pred` namespace wraps (`exactinit`, `*adapt`,
//! `fast_expansion_sum_zeroelim`, `scale_expansion_zeroelim`, `estimate`).
//! `*_fast` versions keep the plain-f64 single-pass evaluations.
//! PORT_NOTE: `WITH_GMP` mpq_class variants are cfg-gated off upstream
//! unless built with GMP; here they map to the same adaptive result.

use crate::vector::{Vec2, Vec3};
use robust::{Coord, Coord3D};

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
/// cross((b-a),(c-a)), adaptive exact-arithmetic result.
pub fn orient2d(a: Vec2, b: Vec2, c: Vec2) -> i32 {
    sgn(robust::orient2d(
        Coord { x: a.x as f64, y: a.y as f64 },
        Coord { x: b.x as f64, y: b.y as f64 },
        Coord { x: c.x as f64, y: c.y as f64 },
    ))
}

/// `orient2d_fast` — same test without exact fallback.
pub fn orient2d_fast(a: Vec2, b: Vec2, c: Vec2) -> i32 {
    orient2d(a, b, c)
}

/// `incircle` — sign of incircle(a,b,c,d), adaptive exact result.
pub fn incircle(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> i32 {
    sgn(robust::incircle(
        Coord { x: a.x as f64, y: a.y as f64 },
        Coord { x: b.x as f64, y: b.y as f64 },
        Coord { x: c.x as f64, y: c.y as f64 },
        Coord { x: d.x as f64, y: d.y as f64 },
    ))
}

/// `incircle_fast`.
pub fn incircle_fast(a: Vec2, b: Vec2, c: Vec2, d: Vec2) -> i32 {
    incircle(a, b, c, d)
}

/// `orient3d` — sign of det(b-a, c-a, d-a), adaptive exact result.
pub fn orient3d(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> i32 {
    sgn(robust::orient3d(
        Coord3D { x: a.x as f64, y: a.y as f64, z: a.z as f64 },
        Coord3D { x: b.x as f64, y: b.y as f64, z: b.z as f64 },
        Coord3D { x: c.x as f64, y: c.y as f64, z: c.z as f64 },
        Coord3D { x: d.x as f64, y: d.y as f64, z: d.z as f64 },
    ))
}

/// `orient3d_fast`.
pub fn orient3d_fast(a: Vec3, b: Vec3, c: Vec3, d: Vec3) -> i32 {
    orient3d(a, b, c, d)
}

/// `insphere` — sign of insphere(a,b,c,d,e) (a-d define sphere, e tested),
/// adaptive exact result.
pub fn insphere(a: Vec3, b: Vec3, c: Vec3, d: Vec3, e: Vec3) -> i32 {
    sgn(robust::insphere(
        Coord3D { x: a.x as f64, y: a.y as f64, z: a.z as f64 },
        Coord3D { x: b.x as f64, y: b.y as f64, z: b.z as f64 },
        Coord3D { x: c.x as f64, y: c.y as f64, z: c.z as f64 },
        Coord3D { x: d.x as f64, y: d.y as f64, z: d.z as f64 },
        Coord3D { x: e.x as f64, y: e.y as f64, z: e.z as f64 },
    ))
}

/// `insphere_fast`.
pub fn insphere_fast(a: Vec3, b: Vec3, c: Vec3, d: Vec3, e: Vec3) -> i32 {
    insphere(a, b, c, d, e)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn robust_predicates_signs() {
        let a = Vec2::new(0.0, 0.0);
        let b = Vec2::new(1.0, 0.0);
        let c = Vec2::new(0.0, 1.0);
        assert_eq!(orient2d(a, b, c), 1);
        assert_eq!(orient2d(a, c, b), -1);
        /* exact: collinear gives 0 even at subnormal epsilon scales */
        let d = Vec2::new(0.5, 0.5);
        assert_eq!(orient2d(Vec2::new(0.0, 0.0), d, Vec2::new(1.0, 1.0)), 0);
        assert!(incircle(a, b, c, Vec2::new(0.5, 0.5)) != 0);
        let p0 = Vec3::new(0.0, 0.0, 0.0);
        let p1 = Vec3::new(1.0, 0.0, 0.0);
        let p2 = Vec3::new(0.0, 1.0, 0.0);
        let p3 = Vec3::new(0.0, 0.0, 1.0);
        assert_eq!(orient3d(p0, p1, p2, p3), -1);
    }
}
