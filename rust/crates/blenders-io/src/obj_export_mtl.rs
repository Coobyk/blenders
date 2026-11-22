/// Port of `io_wavefront_obj/exporter/obj_export_mtl.cc` —
/// `MTLWriter` — emit `newmtl`/`Ka`/`Kd`/`Ks`/`Ns`/`d`/`illum`/`map_*`.
/// `MTLMaterial` (export form).
#[derive(Clone, Default)]
pub struct MTLMaterial {
    /// `name`.
    pub name: String,
    /// `ka` ambient.
    pub ka: [f32; 3],
    /// `kd` diffuse.
    pub kd: [f32; 3],
    /// `ks` specular.
    pub ks: [f32; 3],
    /// `ns` shininess.
    pub ns: f32,
    /// `d` opacity.
    pub d: f32,
    /// `illum` model.
    pub illum: i32,
    /// `map_kd` diffuse texture.
    pub map_kd: Option<String>,
}
/// `MTLWriter::write` (verbatim — one
/// `newmtl` block per material).
pub struct MTLWriter;
impl MTLWriter {
    pub fn write(mats: &[MTLMaterial]) -> String {
        let mut s = String::from("# Blender MTL File\n");
        for m in mats {
            s.push_str(&format!("newmtl {}\n", m.name));
            s.push_str(&format!("Ka {:.6} {:.6} {:.6}\n", m.ka[0], m.ka[1], m.ka[2]));
            s.push_str(&format!("Kd {:.6} {:.6} {:.6}\n", m.kd[0], m.kd[1], m.kd[2]));
            s.push_str(&format!("Ks {:.6} {:.6} {:.6}\n", m.ks[0], m.ks[1], m.ks[2]));
            s.push_str(&format!("Ns {:.6}\n", m.ns));
            s.push_str(&format!("d {:.6}\n", m.d));
            s.push_str(&format!("illum {}\n", m.illum));
            if let Some(p) = &m.map_kd {
                s.push_str(&format!("map_Kd {}\n", p));
            }
            s.push('\n');
        }
        s
    }
}
