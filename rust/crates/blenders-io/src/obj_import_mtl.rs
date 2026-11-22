/// Port of `io_wavefront_obj/importer/obj_import_mtl.cc` —
/// `MTLReader` — parse `newmtl`/`Kd`/`Ks`/`Ka`/
/// `Ns`/`d`/`map_Kd`/`illum` keys.
/// `MTLMaterial` — parsed material entry.
#[derive(Clone, Default)]
pub struct MTLMaterial {
    /// `name`.
    pub name: String,
    /// `ka` — ambient.
    pub ka: [f32; 3],
    /// `kd` — diffuse.
    pub kd: [f32; 3],
    /// `ks` — specular.
    pub ks: [f32; 3],
    /// `ns` — specular exponent.
    pub ns: f32,
    /// `d` — opacity.
    pub d: f32,
    /// `illum`.
    pub illum: i32,
    /// `map_kd` — diffuse map path.
    pub map_kd: Option<String>,
    /// `map_ks` — specular map.
    pub map_ks: Option<String>,
    /// `map_bump` — bump map.
    pub map_bump: Option<String>,
}
/// `MTLReader::read` (verbatim — keyword
/// dispatch on each line).
pub fn read(src: &str) -> Vec<MTLMaterial> {
    let mut mats = Vec::new();
    for line in src.lines() {
        let t = line.trim();
        let mut it = t.split_whitespace();
        let kw = match it.next() {
            Some(k) => k,
            None => continue,
        };
        match kw {
            "newmtl" => {
                mats.push(MTLMaterial {
                    name: it.next().unwrap_or("").to_string(),
                    d: 1.0,
                    ..Default::default()
                });
            }
            "Ka" | "Kd" | "Ks" => {
                let x: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let y: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                let z: f32 = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                if let Some(m) = mats.last_mut() {
                    match kw {
                        "Ka" => m.ka = [x, y, z],
                        "Kd" => m.kd = [x, y, z],
                        _ => m.ks = [x, y, z],
                    }
                }
            }
            "Ns" => {
                if let Some(m) = mats.last_mut() {
                    m.ns = it.next().and_then(|s| s.parse().ok()).unwrap_or(0.0);
                }
            }
            "d" | "Tr" => {
                if let Some(m) = mats.last_mut() {
                    m.d = it.next().and_then(|s| s.parse().ok()).unwrap_or(1.0);
                }
            }
            "illum" => {
                if let Some(m) = mats.last_mut() {
                    m.illum = it.next().and_then(|s| s.parse().ok()).unwrap_or(0);
                }
            }
            "map_Kd" | "map_Ks" | "map_Bump" | "bump" => {
                // Last token wins for path (opts may precede).
                let path = it.last().unwrap_or("").to_string();
                if let Some(m) = mats.last_mut() {
                    match kw {
                        "map_Kd" => m.map_kd = Some(path),
                        "map_Ks" => m.map_ks = Some(path),
                        _ => m.map_bump = Some(path),
                    }
                }
            }
            _ => {}
        }
    }
    mats
}
