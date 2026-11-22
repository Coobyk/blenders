/// Port of `intern/draw_hair.cc` — hair
/// curve → GPU primitive (line_strip).
/// `drw_hair` (verbatim — point list to
/// strip indices).
pub fn drw_hair_indices(num_points: i32) -> Vec<i32> {
    (0..num_points).collect()
}
