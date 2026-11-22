//! Port of `intern/depsgraph_update.cc` — editor-update
/// callback registry.

use std::sync::Mutex;

/// `DEG_EditorUpdateIDCb`.
pub type DegEditorUpdateIdCb = Box<dyn FnMut(usize) + Send>;
/// `DEG_EditorUpdateSceneCb`.
pub type DegEditorUpdateSceneCb = Box<dyn FnMut(bool) + Send>;

static EDITOR_UPDATE_ID_CB: Mutex<Option<DegEditorUpdateIdCb>> = Mutex::new(None);
static EDITOR_UPDATE_SCENE_CB: Mutex<Option<DegEditorUpdateSceneCb>> = Mutex::new(None);

/// `deg_editors_id_update`.
pub fn deg_editors_id_update(id: usize) {
    if let Some(cb) = EDITOR_UPDATE_ID_CB.lock().unwrap().as_mut() {
        cb(id);
    }
}

/// `deg_editors_scene_update`.
pub fn deg_editors_scene_update(updated: bool) {
    if let Some(cb) = EDITOR_UPDATE_SCENE_CB.lock().unwrap().as_mut() {
        cb(updated);
    }
}

/// `DEG_editors_set_update_cb`.
pub fn deg_editors_set_update_cb(
    id_func: Option<DegEditorUpdateIdCb>,
    scene_func: Option<DegEditorUpdateSceneCb>,
) {
    *EDITOR_UPDATE_ID_CB.lock().unwrap() = id_func;
    *EDITOR_UPDATE_SCENE_CB.lock().unwrap() = scene_func;
}
