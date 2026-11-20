//! Port of `lasso_2d.cc` — lasso selection helpers over int2 coords.

use crate::rct::Rcti;
use blenders_math::geom::{isect_point_poly_v2_int, isect_seg_seg_v2_int, Isect};

/// `BLI_lasso_boundbox`.
pub fn lasso_boundbox(mcoords: &[[i32; 2]]) -> Option<Rcti> {
    if mcoords.is_empty() {
        return None;
    }
    let mut rect = Rcti {
        xmin: mcoords[0][0],
        xmax: mcoords[0][0],
        ymin: mcoords[0][1],
        ymax: mcoords[0][1],
    };
    for c in &mcoords[1..] {
        if c[0] < rect.xmin {
            rect.xmin = c[0];
        } else if c[0] > rect.xmax {
            rect.xmax = c[0];
        }
        if c[1] < rect.ymin {
            rect.ymin = c[1];
        } else if c[1] > rect.ymax {
            rect.ymax = c[1];
        }
    }
    Some(rect)
}

/// `BLI_lasso_is_point_inside`.
pub fn lasso_is_point_inside(mcoords: &[[i32; 2]], sx: i32, sy: i32, error_value: i32) -> bool {
    if sx == error_value || mcoords.is_empty() {
        return false;
    }
    isect_point_poly_v2_int([sx, sy], mcoords)
}

/// `BLI_lasso_is_edge_inside`.
pub fn lasso_is_edge_inside(
    mcoords: &[[i32; 2]],
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
    error_value: i32,
) -> bool {
    if x0 == error_value || x1 == error_value || mcoords.is_empty() {
        return false;
    }
    let v1 = [x0, y0];
    let v2 = [x1, y1];
    if lasso_is_point_inside(mcoords, v1[0], v1[1], error_value) {
        return true;
    }
    if lasso_is_point_inside(mcoords, v2[0], v2[1], error_value) {
        return true;
    }
    let last = mcoords.len() - 1;
    if isect_seg_seg_v2_int(mcoords[0], mcoords[last], v1, v2) == Isect::Exact
        || isect_seg_seg_v2_int(mcoords[0], mcoords[last], v1, v2) == Isect::Cross
    {
        return true;
    }
    for i in 0..last {
        let r = isect_seg_seg_v2_int(mcoords[i], mcoords[i + 1], v1, v2);
        if r == Isect::Exact || r == Isect::Cross {
            return true;
        }
    }
    false
}
