/// Port of `engines/eevee/eevee_hizbuffer.cc`
/// — HiZ mip chain for culling.
/// `eevee_hiz_mips` (verbatim — level count).
pub fn eevee_hiz_mips(w: i32, h: i32) -> i32 {
    let m = w.max(h);
    31 - (m.max(1) as u32).leading_zeros() as i32 + 1
}
