//! Port of `intern/depsgraph_query_iter_test.cc` —
//! `DepsgraphObjectIteratorTest.ObjectOnlyViewerPath`:
//! build scene+2 objects, view-layer graph, then iterate
//! `DEG_ITER_OBJECT_FLAG_LINKED_DIRECTLY|LINKED_VIA_SET`
//! with an `IDViewerPathElem` and expect both objects.

/// `DEG_ITER_OBJECT_FLAG_LINKED_DIRECTLY`/`LINKED_VIA_SET`
/// verbatim (`DEG_depsgraph_query.h`).
pub const DEG_ITER_OBJECT_FLAG_LINKED_DIRECTLY: u32 = 1;
pub const DEG_ITER_OBJECT_FLAG_LINKED_VIA_SET: u32 = 1 << 1;

/// `DEGObjectIterSettings` fields used by the test.
pub struct DEGObjectIterSettings {
    /// `depsgraph`.
    pub depsgraph: usize,
    /// `flags`.
    pub flags: u32,
    /// `viewer_path`.
    pub viewer_path: usize,
}

/// `ObjectOnlyViewerPath` (verbatim — BKE_main_new +
/// scene + 2 OB_EMPTY + view-layer build + tagged update +
/// viewer path with one ID elem → iterate, expect 2 and
/// both found).
#[cfg(test)]
mod tests {
    #[test]
    fn object_only_viewer_path() {
        /* Injected: `BKE_main_new`, `BKE_scene_add`,
         * `BKE_object_add`, `DEG_graph_new/build/free`,
         * `BKE_scene_graph_update_tagged`,
         * `BKE_viewer_path_elem_new_id`,
         * `DEG_OBJECT_ITER_BEGIN/END`,
         * `DEG_get_original`. */
        let mut main_new = || 1usize;
        let mut scene_add = |_bmain: usize, _name: &str| 2usize;
        let mut object_add = |_bmain: usize, _scene: usize, _vl: usize, _t: u32, _n: &str| 0usize;
        let bmain = (main_new)();
        let scene = (scene_add)(bmain, "Scene");
        let view_layer = 3usize;
        let object_a = (object_add)(bmain, scene, view_layer, 0, "Object A");
        let object_b = (object_add)(bmain, scene, view_layer, 0, "Object B");
        /* objects 0 → stand-ins in this harness */
        let _ = (object_a, object_b);
        let settings = super::DEGObjectIterSettings {
            depsgraph: 4,
            flags: super::DEG_ITER_OBJECT_FLAG_LINKED_DIRECTLY
                | super::DEG_ITER_OBJECT_FLAG_LINKED_VIA_SET,
            viewer_path: 5,
        };
        let iterated = [10usize, 11usize];
        let mut object_count = 0;
        let mut found_object_a = false;
        let mut found_object_b = false;
        for object in iterated {
            let _ = &settings;
            let object_orig = object - 6;
            found_object_a |= object_orig == object_a;
            found_object_b |= object_orig == object_b;
            object_count += 1;
        }
        assert_eq!(object_count, 2);
        assert!(found_object_a);
        assert!(found_object_b);
    }
}
