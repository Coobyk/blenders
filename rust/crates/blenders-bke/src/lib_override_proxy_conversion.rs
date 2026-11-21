//! Port of `intern/lib_override_proxy_conversion.cc` —
//! proxy → library-override conversion.
//!
//! Everything is ID/Object handle manipulation; `FOREACH_MAIN_ID`,
//! `FOREACH_SCENE_OBJECT`, `BKE_lib_override_library_*`,
//! `BKE_scene_collections_object_remove`, `id_us_min`, `DEG` tags are
//! injected. `LinkNodePair` → `Vec<usize>`.

use blenders_dna::id::NIL;

/// `ID_TAG_DOIT` (DNA_ID.h).
pub const ID_TAG_DOIT: i32 = 1 << 2;

/// The `Object`/`ID` fields the conversion touches.
#[derive(Default)]
pub struct ProxyObjectView {
    /// `ob->proxy_group` — object handle.
    pub proxy_group: usize,
    /// `ob->proxy` — object handle.
    pub proxy: usize,
    /// `ob->id` — own ID handle.
    pub id: usize,
    /// `ob->id.lib` — library handle (`NIL` = local).
    pub lib: usize,
}

/// `BKE_lib_override_library_proxy_convert` — `id_root`/
/// `id_instance_hint` computed per `is_override_instancing_object`;
/// `is_overridable`, `lib_override_init`, `override_flag_clear`,
/// `proxy_from_clear`, `us_min`, `deg_tag`, `foreach_main_id`,
/// `lib_override_create` injected.
#[allow(clippy::too_many_arguments)]
pub fn bke_lib_override_library_proxy_convert(
    ob_proxy: usize,
    view: &mut dyn FnMut(usize) -> ProxyObjectView,
    set_proxy: &mut dyn FnMut(usize, usize, usize), /* (ob, proxy, proxy_group) */
    instance_collection_of: &dyn Fn(usize) -> usize,
    id_of_object: &dyn Fn(usize) -> usize,
    proxy_of_object: &dyn Fn(usize) -> usize,
    is_overridable_library_hierarchy: &dyn Fn(usize) -> bool,
    lib_override_init: &mut dyn FnMut(usize, usize),
    override_flag_clear_system_defined: &mut dyn FnMut(usize),
    proxy_from_clear: &mut dyn FnMut(usize),
    id_us_min: &mut dyn FnMut(usize),
    id_tag_or: &mut dyn FnMut(usize, i32),
    id_newid_set: &mut dyn FnMut(usize, usize),
    deg_tag: &mut dyn FnMut(usize),
    foreach_main_id: &mut dyn FnMut(&mut dyn FnMut(usize)),
    id_is_linked: &dyn Fn(usize) -> bool,
    id_lib_of: &dyn Fn(usize) -> usize,
    lib_override_create: &mut dyn FnMut(usize, usize, usize, usize) -> bool, /* (lib, root, hint) */
) -> bool {
    let ob_view = view(ob_proxy);
    let ob_proxy_group = ob_view.proxy_group;
    let is_override_instancing_object =
        ob_proxy_group != NIL && instance_collection_of(ob_proxy_group) != NIL;
    let ob_proxy_proxy = ob_view.proxy;
    let id_root = if is_override_instancing_object {
        instance_collection_of(ob_proxy_group)
    } else {
        id_of_object(proxy_of_object(ob_proxy))
    };
    let id_instance_hint = if is_override_instancing_object {
        id_of_object(ob_proxy_group)
    } else {
        id_of_object(ob_proxy)
    };
    if !is_overridable_library_hierarchy(id_root) {
        if ob_proxy_proxy != NIL {
            proxy_from_clear(ob_proxy_proxy);
        }
        id_us_min(id_of_object(ob_proxy_proxy));
        set_proxy(ob_proxy, NIL, NIL);
        return false;
    }
    id_tag_or(id_of_object(ob_proxy_proxy), ID_TAG_DOIT);
    id_newid_set(id_of_object(ob_proxy_proxy), ob_view.id);
    lib_override_init(ob_view.id, id_of_object(ob_proxy_proxy));
    override_flag_clear_system_defined(ob_view.id);
    proxy_from_clear(ob_proxy_proxy);
    set_proxy(ob_proxy, NIL, NIL);
    deg_tag(ob_view.id);
    let ob_lib = ob_view.lib;
    foreach_main_id(&mut |id_iter| {
        if !id_is_linked(id_iter) || id_lib_of(id_iter) == ob_lib {
            id_tag_or(id_iter, ID_TAG_DOIT);
        }
    });
    lib_override_create(ob_lib, id_root, id_root, id_instance_hint)
}

/// `BlendFileReadReport::count` fields used.
#[derive(Default)]
pub struct ProxyConvertCount {
    pub proxies_to_lib_overrides_success: i32,
    pub proxies_to_lib_overrides_failures: i32,
}

/// `lib_override_library_proxy_convert_do`.
fn lib_override_library_proxy_convert_do(
    ob_proxy: usize,
    convert: &mut dyn FnMut(usize) -> bool,
    remove_object: &mut dyn FnMut(usize),
    ob_proxy_group_of: &mut dyn FnMut(usize) -> usize,
    reports: &mut ProxyConvertCount,
) {
    let ob_proxy_group = ob_proxy_group_of(ob_proxy);
    let is_override_instancing_object = ob_proxy_group != NIL;
    if convert(ob_proxy) {
        if is_override_instancing_object {
            remove_object(ob_proxy_group); /* `BKE_scene_collections_object_remove`. */
        }
        reports.proxies_to_lib_overrides_success += 1;
    }
}

/// `BKE_lib_override_library_main_proxy_convert` — `scenes` yields
/// scene handles; `scene_objects(scene)` yields object handles;
/// `main_objects` yields every Main object. `view`/`set_proxy` as in
/// `bke_lib_override_library_proxy_convert`.
#[allow(clippy::too_many_arguments)]
pub fn bke_lib_override_library_main_proxy_convert(
    scenes: &mut dyn FnMut(&mut dyn FnMut(usize)),
    scene_objects: &mut dyn FnMut(usize, &mut dyn FnMut(usize)),
    main_objects: &mut dyn FnMut(&mut dyn FnMut(usize)),
    view: &mut dyn FnMut(usize) -> ProxyObjectView,
    set_proxy: &mut dyn FnMut(usize, usize, usize),
    convert: &mut dyn FnMut(usize) -> bool,
    remove_object: &mut dyn FnMut(usize),
    proxy_from_clear: &mut dyn FnMut(usize),
    id_us_min: &mut dyn FnMut(usize),
    id_of_object: &dyn Fn(usize) -> usize,
    reports: &mut ProxyConvertCount,
    logs: &mut Vec<String>,
) {
    scenes(&mut |scene| {
        let mut proxy_objects: Vec<usize> = Vec::new();
        scene_objects(scene, &mut |object| {
            if view(object).proxy_group != NIL {
                proxy_objects.push(object);
            }
        });
        scene_objects(scene, &mut |object| {
            let v = view(object);
            if v.proxy != NIL && v.proxy_group == NIL {
                proxy_objects.push(object);
            }
        });
        for &proxy_object in &proxy_objects {
            lib_override_library_proxy_convert_do(
                proxy_object,
                convert,
                remove_object,
                &mut |o| view(o).proxy_group,
                reports,
            );
        }
    });
    main_objects(&mut |object| {
        let v = view(object);
        if v.proxy_group != NIL || v.proxy != NIL {
            if v.lib != NIL {
                logs.push(format!(
                    "Linked proxy object '{}' failed to be converted to library override",
                    v.id
                ));
            } else {
                logs.push(format!(
                    "Proxy object '{}' failed to be converted to library override",
                    v.id
                ));
            }
            reports.proxies_to_lib_overrides_failures += 1;
            if v.proxy != NIL {
                proxy_from_clear(v.proxy);
            }
            id_us_min(id_of_object(v.proxy));
            set_proxy(object, NIL, NIL);
        }
    });
}
