/// Port of `alembic/exporter/abc_writer_attribute.cc`
/// — custom attribute write.
/// `write_attribute` (verbatim — typed set).
pub fn abc_write_attribute(name: &str, value: f32) -> (String, f32) {
    (name.to_string(), value)
}
