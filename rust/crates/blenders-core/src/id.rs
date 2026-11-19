//! Port of `ID` (DNA_ID.h / BKE_lib_id) — the base datablock type.
//!
//! Blender `ID` is a polymorphic base: every datablock (Object, Mesh,
//! Material, …) embeds an `ID` header with a unique 66-char name whose
//! first 2 bytes encode the type ("OB", "ME", …).

/// `MAX_ID_NAME` — Blender keeps 64 usable chars + 2-byte type prefix.
pub const MAX_ID_NAME: usize = 66;

/// Datablock kind — port of `ID_Type`/`ID.code` (`ID_OB`, `ID_ME`, …).
/// The two-letter code is exactly Blender's `ID.name` prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IdKind {
    /// `OB` — object.
    Object,
    /// `ME` — mesh.
    Mesh,
    /// `MA` — material.
    Material,
    /// `TE` — texture.
    Texture,
    /// `IM` — image.
    Image,
    /// `LA` — light (lamp).
    Light,
    /// `CA` — camera.
    Camera,
    /// `WO` — world.
    World,
    /// `SCE` — scene (3 letters).
    Scene,
    /// `CU` — curve.
    Curve,
    /// `AR` — armature.
    Armature,
    /// `AC` — action.
    Action,
    /// `NT` — node tree.
    NodeTree,
    /// `BR` — brush.
    Brush,
    /// `PK` — grease pencil palette (placeholder slot).
    Palette,
    /// `WM` — window manager.
    WindowManager,
    /// `LI` — library.
    Library,
    /// `GR` — collection.
    Collection,
    /// `SP` — speaker.
    Speaker,
    /// `MB` — metaball.
    MetaBall,
    /// `LT` — lattice.
    Lattice,
    /// `KE` — shape key.
    Key,
    /// `SCR` — screen.
    Screen,
    /// `MC` — movie clip.
    MovieClip,
    /// `MSK` — mask.
    Mask,
    /// `CF` — cache file.
    CacheFile,
    /// `CV` — cache volume (placeholder).
    Volume,
    /// `PT` — point cloud.
    PointCloud,
    /// `GP` — grease pencil.
    GreasePencil,
    /// `WAV`/`SND` — sound.
    Sound,
    /// `TXT` — text block.
    Text,
    /// `VSE` — vfont.
    VFont,
    /// `WS` — workspace.
    Workspace,
    /// `MOV` — movie data.
    Movie,
}

impl IdKind {
    /// `BKE_idcode_to_name` — the 2-3 letter code stored in `ID.name`.
    pub fn code(self) -> &'static str {
        match self {
            IdKind::Object => "OB",
            IdKind::Mesh => "ME",
            IdKind::Material => "MA",
            IdKind::Texture => "TE",
            IdKind::Image => "IM",
            IdKind::Light => "LA",
            IdKind::Camera => "CA",
            IdKind::World => "WO",
            IdKind::Scene => "SCE",
            IdKind::Curve => "CU",
            IdKind::Armature => "AR",
            IdKind::Action => "AC",
            IdKind::NodeTree => "NT",
            IdKind::Brush => "BR",
            IdKind::Palette => "PK",
            IdKind::WindowManager => "WM",
            IdKind::Library => "LI",
            IdKind::Collection => "GR",
            IdKind::Speaker => "SP",
            IdKind::MetaBall => "MB",
            IdKind::Lattice => "LT",
            IdKind::Key => "KE",
            IdKind::Screen => "SCR",
            IdKind::MovieClip => "MC",
            IdKind::Mask => "MSK",
            IdKind::CacheFile => "CF",
            IdKind::Volume => "CV",
            IdKind::PointCloud => "PT",
            IdKind::GreasePencil => "GP",
            IdKind::Sound => "SND",
            IdKind::Text => "TXT",
            IdKind::VFont => "VF",
            IdKind::Workspace => "WS",
            IdKind::Movie => "MOV",
        }
    }

    /// `BKE_idcode_from_name` — parse a code back.
    pub fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "OB" => IdKind::Object,
            "ME" => IdKind::Mesh,
            "MA" => IdKind::Material,
            "TE" => IdKind::Texture,
            "IM" => IdKind::Image,
            "LA" => IdKind::Light,
            "CA" => IdKind::Camera,
            "WO" => IdKind::World,
            "SCE" => IdKind::Scene,
            "CU" => IdKind::Curve,
            "AR" => IdKind::Armature,
            "AC" => IdKind::Action,
            "NT" => IdKind::NodeTree,
            "BR" => IdKind::Brush,
            "PK" => IdKind::Palette,
            "WM" => IdKind::WindowManager,
            "LI" => IdKind::Library,
            "GR" => IdKind::Collection,
            "SP" => IdKind::Speaker,
            "MB" => IdKind::MetaBall,
            "LT" => IdKind::Lattice,
            "KE" => IdKind::Key,
            "SCR" => IdKind::Screen,
            "MC" => IdKind::MovieClip,
            "MSK" => IdKind::Mask,
            "CF" => IdKind::CacheFile,
            "CV" => IdKind::Volume,
            "PT" => IdKind::PointCloud,
            "GP" => IdKind::GreasePencil,
            "SND" | "SO" => IdKind::Sound,
            "TXT" => IdKind::Text,
            "VF" => IdKind::VFont,
            "WS" => IdKind::Workspace,
            "MOV" => IdKind::Movie,
            _ => return None,
        })
    }
}

/// `ID.name` — fixed-capacity name with embedded kind code.
/// Stored as `"<CODE><name>"` (e.g. "OBMyCube"), matching Blender's
/// packed `char name[66]`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct IdName(String);

impl IdName {
    /// `BKE_id_new_name_validate` — build `"OB"+"MyCube"`-style name.
    pub fn new(kind: IdKind, name: &str) -> Self {
        let mut s = String::with_capacity(MAX_ID_NAME.min(name.len() + 3));
        s.push_str(kind.code());
        s.push_str(name);
        s.truncate(MAX_ID_NAME);
        Self(s)
    }

    /// Kind prefix decoded from the stored name (`BKE_id_type_from_name`).
    pub fn kind(&self) -> Option<IdKind> {
        // Try 3-letter codes first ("SCE", "TXT", …), then 2-letter.
        let end = self.0.len().min(3);
        if let Some(k) = IdKind::from_code(&self.0[..end]) {
            return Some(k);
        }
        IdKind::from_code(&self.0[..self.0.len().min(2)])
    }

    /// Human part after the code (`id->name + BKE_idcode_len`).
    pub fn short_name(&self) -> &str {
        let code_len = self
            .kind()
            .map(|k| k.code().len())
            .unwrap_or(0);
        &self.0[code_len..]
    }
}

impl std::fmt::Display for IdName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// Port of `struct ID` — common datablock header.
#[derive(Debug)]
pub struct Id {
    /// `ID.name[66]` — kind code + unique name.
    pub name: IdName,
    /// `ID.us` — user/reference count (`id_us_plus`/`id_us_min`).
    pub users: i16,
    /// `ID.flag` — generic flags.
    pub flag: u32,
    /// `ID.icon_id` — preview icon index.
    pub icon_id: i32,
    /// `ID.tag` — dirty/eval tags.
    pub tag: u32,
    /// `LIB_TAG_*` library tag.
    pub lib_tag: u32,
}

impl Id {
    /// `BKE_id_new` — construct header with kind + name.
    pub fn new(kind: IdKind, name: &str) -> Self {
        Self {
            name: IdName::new(kind, name),
            users: 0,
            flag: 0,
            icon_id: 0,
            tag: 0,
            lib_tag: 0,
        }
    }

    /// `id_us_plus` — increment user count.
    pub fn us_plus(&mut self) {
        self.users += 1;
    }

    /// `id_us_min` — decrement user count (clamped at 0 like BKE_libblock).
    pub fn us_min(&mut self) {
        if self.users > 0 {
            self.users -= 1;
        }
    }

    /// Datablock kind (`GS(id->name)`).
    pub fn kind(&self) -> Option<IdKind> {
        self.name.kind()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_roundtrip() {
        let id = Id::new(IdKind::Object, "Cube");
        assert_eq!(id.name.to_string(), "OBCube");
        assert_eq!(id.kind(), Some(IdKind::Object));
        assert_eq!(id.name.short_name(), "Cube");
    }

    #[test]
    fn users() {
        let mut id = Id::new(IdKind::Mesh, "Grid");
        id.us_plus();
        id.us_plus();
        id.us_min();
        assert_eq!(id.users, 1);
        id.us_min();
        id.us_min();
        assert_eq!(id.users, 0);
    }
}
