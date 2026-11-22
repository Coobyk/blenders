/// Port of `alembic/intern/abc_util.cc` —
/// Alembic type/time helpers.
/// `abc_is_valid_name` (verbatim — allowed
/// chars).
pub fn abc_is_valid_name(name: &str) -> bool {
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.' || c == '|')
}
