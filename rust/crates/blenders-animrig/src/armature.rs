//! Port of `intern/armature.cc` — pose-bone descendant
//! iterators.

/// `pose_bone_descendent_iterator` (verbatim — BFS via
/// growing queue).
pub fn pose_bone_descendent_iterator(
    pchan_root: usize,
    child_pchan_of: &mut dyn FnMut(usize) -> Vec<usize>,
    callback: &mut dyn FnMut(usize),
) {
    let mut descendants = vec![pchan_root];
    let mut i = 0;
    while i < descendants.len() {
        let descendant = descendants[i];
        i += 1;
        (callback)(descendant);
        for child in (child_pchan_of)(descendant) {
            descendants.push(child);
        }
    }
}

/// `pose_bone_descendent_depth_iterator` (verbatim —
/// recursive DFS, short-circuit &&).
pub fn pose_bone_descendent_depth_iterator(
    pchan_root: usize,
    child_pchan_of: &mut dyn FnMut(usize) -> Vec<usize>,
    callback: &mut dyn FnMut(usize) -> bool,
) -> bool {
    fn recurse(
        pchan: usize,
        child_pchan_of: &mut dyn FnMut(usize) -> Vec<usize>,
        callback: &mut dyn FnMut(usize) -> bool,
    ) -> bool {
        if !(callback)(pchan) {
            return false;
        }
        let mut success = true;
        for child in (child_pchan_of)(pchan) {
            success &= recurse(child, child_pchan_of, callback);
        }
        success
    }
    recurse(pchan_root, child_pchan_of, callback)
}
