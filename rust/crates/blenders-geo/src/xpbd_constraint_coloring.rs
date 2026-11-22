//! Port of `xpbd/intern/xpbd_constraint_coloring.cc`
//! — graph color constraints for parallel XPBD.

/// `color_constraints` (verbatim — greedy
// vertex coloring over constraint graph).
pub fn xpbd_color_constraints(
    constraint_points: &[Vec<i32>],
    num_points: i32,
) -> Vec<i32> {
    let mut color = vec![-1i32; num_points as usize];
    for (ci, pts) in constraint_points.iter().enumerate() {
        let mut used = vec![false; num_points as usize];
        for &p in pts {
            if color[p as usize] >= 0 {
                used[color[p as usize] as usize] = true;
            }
        }
        let mut c = 0i32;
        while c < num_points && used[c as usize] {
            c += 1;
        }
        for &p in pts {
            color[p as usize] = c;
        }
        let _ = ci;
    }
    color
}
