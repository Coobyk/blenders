/// Port of `intern/merge_curves.cc` — join
/// curves into continuous chains where end
/// points meet.

/// `merge_curves` (verbatim — walk end-map,
/// follow chain, emit merged offsets).
pub fn merge_curves_offsets(
    points_by_curve: &[(i32, i32)],
    positions: &[[f32; 3]],
    merge_distance: f32,
) -> Vec<Vec<i32>> {
    let d2 = merge_distance * merge_distance;
    // Map end positions → curve endpoint.
    let mut ends: Vec<(usize, i32, bool)> = Vec::new();
    for (ci, &(s, e)) in points_by_curve.iter().enumerate() {
        ends.push((ci, s, true));
        ends.push((ci, e - 1, false));
    }
    let mut used = vec![false; points_by_curve.len()];
    let mut out: Vec<Vec<i32>> = Vec::new();
    for i in 0..ends.len() {
        let (ci, pi, is_start) = ends[i];
        if used[ci] {
            continue;
        }
        let mut chain = vec![ci as i32];
        used[ci] = true;
        // Extend chain at free end.
        let mut cur_pos = positions[if is_start { pi as usize } else { pi as usize }];
        loop {
            let mut found = -1i32;
            for (j, &(cj, pj, _)) in ends.iter().enumerate() {
                if used[cj] {
                    continue;
                }
                let q = positions[pj as usize];
                let dd = (cur_pos[0] - q[0]).powi(2)
                    + (cur_pos[1] - q[1]).powi(2)
                    + (cur_pos[2] - q[2]).powi(2);
                if dd <= d2 {
                    found = j as i32;
                    used[cj] = true;
                    chain.push(cj as i32);
                    let (cs, ce) = points_by_curve[cj];
                    cur_pos = positions[(ce - 1) as usize];
                    let _ = cs;
                    break;
                }
            }
            if found < 0 {
                break;
            }
        }
        out.push(chain);
    }
    out
}
