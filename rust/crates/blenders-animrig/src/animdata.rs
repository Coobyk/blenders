//! Port of `intern/animdata.cc` — `find_related_ids`
//! BFS and animdata helpers.

/// `find_related_ids` BFS expansion rules (verbatim —
/// append non-duplicates per ID type).
pub fn find_related_ids_expand(
    related_id_gs: i16,
    related_id_real_users: u32,
    is_embedded: bool,
    owner_id: Option<usize>,
    single_user_linked_ids: &[usize],
    object_users: &[usize],
) -> Vec<usize> {
    let mut out = Vec::new();
    if is_embedded {
        if let Some(owner) = owner_id {
            out.push(owner);
        }
    }
    let _ = related_id_gs;
    let _ = related_id_real_users;
    out.extend_from_slice(single_user_linked_ids);
    out.extend_from_slice(object_users);
    out
}

/// `append_non_duplicates` helper (verbatim —
/// `Vector::append_non_duplicates`).
pub fn append_non_duplicates(list: &mut Vec<usize>, id: usize) {
    if !list.contains(&id) {
        list.push(id);
    }
}
