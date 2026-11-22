//! Port of `intern/bonecolor.cc` — `BoneColor` palette
//! index vs custom color, equality + hash.

/// `BoneColor` verbatim.
#[derive(Clone, Copy, Default)]
pub struct BoneColor {
    /// `palette_index`: 0 = bone default, -1 = custom,
    /// >0 = theme index.
    pub palette_index: i8,
    /// `custom` (solid/select/active RGBA).
    pub custom_solid: [u8; 4],
    pub custom_select: [u8; 4],
    pub custom_active: [u8; 4],
    /// `custom.flag`.
    pub custom_flag: u8,
}

impl BoneColor {
    /// `BoneColor::effective_color` decision (verbatim —
    /// 0→none, <0→custom, >0→theme tarm[idx-1]).
    pub fn effective_color_index(&self) -> Option<i32> {
        let color_index = self.palette_index as i32;
        if color_index == 0 {
            None
        } else if color_index < 0 {
            Some(-1)
        } else {
            Some(color_index - 1)
        }
    }
}

impl PartialEq for BoneColor {
    /// `operator==` (verbatim — same index; custom
    /// compares first 3 bytes + flag).
    fn eq(&self, other: &Self) -> bool {
        if self.palette_index != other.palette_index {
            return false;
        }
        if self.palette_index == -1 {
            return self.custom_solid[..3] == other.custom_solid[..3]
                && self.custom_select[..3] == other.custom_select[..3]
                && self.custom_active[..3] == other.custom_active[..3]
                && self.custom_flag == other.custom_flag;
        }
        true
    }
}

impl BoneColor {
    /// `BoneColor::hash` (verbatim — index or rgb
    /// triples + flag).
    pub fn hash(&self) -> u64 {
        if self.palette_index >= 0 {
            self.palette_index as u64
        } else {
            let h_solid = u64::from_be_bytes([
                0, 0, 0, 0, 0,
                self.custom_solid[0], self.custom_solid[1], self.custom_solid[2],
            ]);
            let h_select = u64::from_be_bytes([
                0, 0, 0, 0, 0,
                self.custom_select[0], self.custom_select[1], self.custom_select[2],
            ]);
            let h_active = u64::from_be_bytes([
                0, 0, 0, 0, 0,
                self.custom_active[0], self.custom_active[1], self.custom_active[2],
            ]);
            h_solid ^ h_select.wrapping_mul(0x9e3779b97f4a7c15) ^ h_active.wrapping_mul(2) ^ self.custom_flag as u64
        }
    }
}

/// `ANIM_bonecolor_posebone_get` (verbatim — pchan color
/// 0 → bone's).
pub fn anim_bonecolor_posebone_get<'a>(
    pchan_color: &'a BoneColor,
    bone_color: &'a BoneColor,
) -> &'a BoneColor {
    if pchan_color.palette_index == 0 {
        bone_color
    } else {
        pchan_color
    }
}
