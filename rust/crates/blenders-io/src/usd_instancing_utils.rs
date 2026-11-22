/// Port of `usd/intern/usd_instancing_utils.cc`
/// — mark prototypes + instanceable refs.
/// `mark_as_instance` (verbatim — flag).
pub fn usd_mark_as_instance(prim_path: &str) -> String {
    format!("{}_proto", prim_path)
}
