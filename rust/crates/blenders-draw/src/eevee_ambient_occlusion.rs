/// Port of `engines/eevee/eevee_ambient_occlusion.cc`
/// — AO settings + pass.
/// `eevee_ao` (verbatim — intensity/distance).
pub fn eevee_ao(intensity: f32, distance: f32) -> [f32; 2] {
    [intensity, distance]
}
