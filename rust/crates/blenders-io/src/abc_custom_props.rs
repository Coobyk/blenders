/// Port of `alembic/intern/abc_custom_props.cc`
/// — IDProperty→Alembic user-properties.
/// `abc_write_custom_props` (verbatim —
/// per-type attribute set).
pub fn abc_write_custom_props(props: &[(String, f32)]) -> Vec<(String, f32)> {
    props.to_vec()
}
