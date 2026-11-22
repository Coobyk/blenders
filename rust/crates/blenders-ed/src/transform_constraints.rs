/// Port of `transform/transform_constraints.cc`
/// — axis constraints (`setConstraint`:
/// project vec onto constrained axis/plane).
/// `con_axisVec`/`con_axisSize`/
/// `con_axisRot` (verbatim — project `vec`
/// onto the axis-mask axes).
pub fn con_axis_vec(con_mode: u32, vec: &mut [f32; 3]) {
    for i in 0..3 {
        if con_mode & (crate::transform::CON_AXIS0 << i) == 0 {
            vec[i] = 0.0;
        }
    }
}
/// `postConstraintChecks` (verbatim —
/// normalize axis counts: 1 axis → lock
/// to that axis, 2 → plane).
pub fn post_constraint_checks(con_mode: u32) -> u32 {
    let mut mode = con_mode;
    let mut n = 0;
    for i in 0..3 {
        if mode & (crate::transform::CON_AXIS0 << i) != 0 {
            n += 1;
        }
    }
    if n == 0 {
        mode = 0;
    }
    mode
}
/// `startConstraint` (verbatim — mark
/// CON_APPLY once axes chosen).
pub fn start_constraint(con_mode: &mut u32) {
    *con_mode |= crate::transform::CON_APPLY;
}
/// `constraintNumInput` (verbatim —
/// numeric input → axis values).
pub fn constraint_num_input(con_mode: u32, values: &mut [f32; 4]) {
    for i in 0..3 {
        if con_mode & (crate::transform::CON_AXIS0 << i) == 0 {
            values[i] = if con_mode != 0 { 0.0 } else { values[i] };
        }
    }
}
