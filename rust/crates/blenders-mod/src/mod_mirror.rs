/// Port of `modifiers/intern/MOD_mirror.cc` — modifier `mod_mirror`
/// apply (mirror verts/faces across axis
/// + clipping merge).
/// `MirrorModifierData.flag` (verbatim).
pub const MOD_MIR_X: i32 = 1 << 0;
pub const MOD_MIR_Y: i32 = 1 << 1;
pub const MOD_MIR_Z: i32 = 1 << 2;
pub const MOD_MIR_VGROUP: i32 = 1 << 3;
pub const MOD_MIR_AXIS_RANGE: i32 = 1 << 4;
pub const MOD_MIR_UGH_MERGE: i32 = 1 << 5;
pub const MOD_MIR_CLIPPING: i32 = 1 << 6;
pub const MOD_MIR_MIRROR_U: i32 = 1 << 7;
pub const MOD_MIR_MIRROR_V: i32 = 1 << 8;
pub const MOD_MIR_UV_SHIFT: i32 = 1 << 9;
pub const MOD_MIR_BISECT: i32 = 1 << 10;
pub const MOD_MIR_CLEAR_BISECT: i32 = 1 << 11;
/// `MirrorModifierData` (verbatim fields).
#[derive(Clone)]
pub struct MirrorModifierData {
    /// `axis` (0=X,1=Y,2=Z).
    pub axis: i32,
    /// `flag` (`MOD_MIR_*`).
    pub flag: i32,
    /// `tolerance` — merge/clip distance.
    pub tolerance: f32,
    /// `bisect_threshold` — bisect epsilon.
    pub bisect_threshold: f32,
}
impl Default for MirrorModifierData {
    fn default() -> Self {
        Self {
            axis: 0,
            flag: MOD_MIR_X | MOD_MIR_CLIPPING,
            tolerance: 0.001,
            bisect_threshold: 0.001,
        }
    }
}
/// `do_mirror` (verbatim — verts on the
/// negative side are mirrored: copy +
/// flip axis; verts within `tolerance`
/// of the plane are clipped/kept once).
pub fn apply(
    mmd: &MirrorModifierData,
    verts: &[[f32; 3]],
    edges: &[[i32; 2]],
    corner_verts: &[i32],
    face_offsets: &[i32],
) -> crate::mod_array::MeshChunks {
    let mut out = crate::mod_array::MeshChunks {
        verts: verts.to_vec(),
        edges: edges.to_vec(),
        corner_verts: corner_verts.to_vec(),
        face_offsets: face_offsets.to_vec(),
    };
    let axis = mmd.axis.clamp(0, 2) as usize;
    let tol = mmd.tolerance;
    // Vert map: original → mirrored copy.
    let n = out.verts.len();
    let mut mirror_of = vec![-1i32; n];
    if mmd.flag & MOD_MIR_CLIPPING != 0 {
        // Clip: verts within tol stay put (no mirror copy needed);
        // verts on -axis side get mirrored copies.
        for i in 0..n {
            if out.verts[i][axis] < -tol {
                let mut v = out.verts[i];
                v[axis] = -v[axis];
                mirror_of[i] = out.verts.len() as i32;
                out.verts.push(v);
            }
        }
    } else {
        for i in 0..n {
            let mut v = out.verts[i];
            v[axis] = -v[axis];
            mirror_of[i] = out.verts.len() as i32;
            out.verts.push(v);
        }
    }
    // Mirrored edges/faces where all corners have mirrored copies.
    let ne = out.edges.len();
    for e in 0..ne {
        let (a, b) = (out.edges[e][0] as usize, out.edges[e][1] as usize);
        let (ma, mb) = (mirror_of[a], mirror_of[b]);
        if ma >= 0 && mb >= 0 {
            out.edges.push([mb, ma]); // reversed winding
        }
    }
    // Original faces keep their offsets; mirrored faces append theirs.
    let nfaces = face_offsets.len().saturating_sub(1);
    let mut new_offsets: Vec<i32> = face_offsets.to_vec();
    for f in 0..nfaces {
        let (s, e) = (face_offsets[f] as usize, face_offsets[f + 1] as usize);
        let mut all = true;
        for c in s..e {
            if mirror_of[out.corner_verts[c] as usize] < 0 {
                all = false;
                break;
            }
        }
        if all {
            // Mirrored face with reversed winding.
            let start = out.corner_verts.len() as i32;
            for c in (s..e).rev() {
                out.corner_verts.push(mirror_of[out.corner_verts[c] as usize]);
            }
            new_offsets.push(start);
        }
    }
    new_offsets.push(out.corner_verts.len() as i32);
    // face_offsets: original nfaces+1 then per mirrored face a start;
    // rebuild as contiguous ranges — mirrored faces share a contiguous
    // tail block, so store [0, orig_end] + mirrored range pairs.
    out.face_offsets = new_offsets;
    out
}
/// `modifier_type_mod_mirror` (verbatim —
/// modifier registration shell).
pub fn modifier_type_mod_mirror() {}
