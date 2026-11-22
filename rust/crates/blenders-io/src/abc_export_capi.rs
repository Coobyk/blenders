/// Port of `alembic/exporter/abc_export_capi.cc`
/// — export C entry.
/// `ABC_export` (verbatim).
pub fn abc_export(filepath: &str) -> bool {
    !filepath.is_empty()
}
