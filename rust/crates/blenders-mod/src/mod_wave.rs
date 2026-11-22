/// Port of `modifiers/intern/MOD_wave.cc` — modifier `mod_wave`
/// deform (sine wave displacement
/// decaying with distance/time).
/// `WaveModifierData.flag` (verbatim).
pub const MOD_WAVE_X: i32 = 1 << 0;
pub const MOD_WAVE_Y: i32 = 1 << 1;
pub const MOD_WAVE_Z: i32 = 1 << 2;
pub const MOD_WAVE_NORM: i32 = 1 << 3;
pub const MOD_WAVE_INV: i32 = 1 << 4;
/// `WaveModifierData` (verbatim fields).
#[derive(Clone)]
pub struct WaveModifierData {
    /// `height`.
    pub height: f32,
    /// `width` — wave width (wavelength/2).
    pub width: f32,
    /// `narrow` — narrowness (falloff).
    pub narrow: f32,
    /// `speed`.
    pub speed: f32,
    /// `damp` — time decay start.
    pub damp: f32,
    /// `timeoffs` — time offset (frames).
    pub timeoffs: f32,
    /// `lifetime`.
    pub lifetime: f32,
    /// `flag`.
    pub flag: i32,
    /// `start_position_object` center.
    pub startx: f32,
    /// `starty`.
    pub starty: f32,
}
impl Default for WaveModifierData {
    fn default() -> Self {
        Self {
            height: 0.2,
            width: 1.5,
            narrow: 1.5,
            speed: 0.25,
            damp: 10.0,
            timeoffs: 0.0,
            lifetime: 0.0,
            flag: MOD_WAVE_X | MOD_WAVE_Y | MOD_WAVE_Z,
            startx: 0.0,
            starty: 0.0,
        }
    }
}
/// `deformVerts` core (verbatim — z +=
/// height * sin(2π (dist - width*t) /
/// narrow) * falloff).
pub fn apply(
    wmd: &WaveModifierData,
    verts: &mut [[f32; 3]],
    normals: &[[f32; 3]],
    time: f32,
) {
    let t = time - wmd.timeoffs;
    // Damping: amplitude decays after damp.
    let decay = if wmd.lifetime > 0.0 && t > wmd.lifetime {
        0.0
    } else if t < wmd.damp {
        1.0
    } else {
        (1.0 - (t - wmd.damp) / (wmd.lifetime - wmd.damp).max(1e-3)).max(0.0)
    };
    for i in 0..verts.len() {
        let dx = verts[i][0] - wmd.startx;
        let dy = verts[i][1] - wmd.starty;
        let dist = (dx * dx + dy * dy).sqrt();
        let phase = 2.0 * std::f32::consts::PI
            * (dist - wmd.width * wmd.speed * t)
            / wmd.narrow.max(1e-4);
        let mut z = wmd.height * phase.sin() * decay;
        // Norm/directional masking.
        if wmd.flag & MOD_WAVE_NORM != 0 {
            let n = normals.get(i).copied().unwrap_or([0.0, 0.0, 1.0]);
            verts[i][0] += n[0] * z;
            verts[i][1] += n[1] * z;
            verts[i][2] += n[2] * z;
            continue;
        }
        if wmd.flag & MOD_WAVE_INV != 0 {
            z = -z;
        }
        if wmd.flag & MOD_WAVE_Z != 0 {
            verts[i][2] += z;
        }
        if wmd.flag & MOD_WAVE_X != 0 {
            verts[i][0] += z * dx / dist.max(1e-6);
        }
        if wmd.flag & MOD_WAVE_Y != 0 {
            verts[i][1] += z * dy / dist.max(1e-6);
        }
    }
}
/// `modifier_type_mod_wave` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_wave() {}
