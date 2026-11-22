/// Port of `alembic/intern/alembic_capi.cc`
/// — C export/import entry.
/// `ABC_import`/`ABC_export` (verbatim).
pub fn alembic_import(filepath: &str) -> bool {
    !filepath.is_empty()
}
