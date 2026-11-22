/// Port of `engines/select/select_debug_engine.cc`
/// — select-id debug visualizer.
/// `select_debug` (verbatim — color ramp).
pub fn select_debug(id: u32) -> [f32; 3] {
    [
        ((id >> 0) & 0xff) as f32 / 255.0,
        ((id >> 8) & 0xff) as f32 / 255.0,
        ((id >> 16) & 0xff) as f32 / 255.0,
    ]
}
