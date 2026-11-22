/// Port of `bpy_app_handlers.cc` — `bpy.app.handlers`
/// (persistent/depsgraph/frame callbacks).
/// `HANDLER_*` callback ordinals (verbatim).
pub const HANDLER_RENDER_INIT: i32 = 0;
pub const HANDLER_RENDER_PRE: i32 = 1;
pub const HANDLER_RENDER_POST: i32 = 2;
pub const HANDLER_RENDER_WRITE: i32 = 3;
pub const HANDLER_RENDER_STATS: i32 = 4;
pub const HANDLER_COMPLETE: i32 = 5;
pub const HANDLER_CANCEL: i32 = 6;
pub const HANDLER_DEPSGRAPH_UPDATE_PRE: i32 = 7;
pub const HANDLER_DEPSGRAPH_UPDATE_POST: i32 = 8;
pub const HANDLER_VERSION_UPDATE: i32 = 9;
pub const HANDLER_LOAD_FACTORY_STARTUP_PRE: i32 = 10;
pub const HANDLER_LOAD_FACTORY_STARTUP_POST: i32 = 11;
pub const HANDLER_LOAD_PRE: i32 = 12;
pub const HANDLER_LOAD_POST: i32 = 13;
pub const HANDLER_SAVE_PRE: i32 = 14;
pub const HANDLER_SAVE_POST: i32 = 15;
pub const HANDLER_UNDO_PRE: i32 = 16;
pub const HANDLER_UNDO_POST: i32 = 17;
pub const HANDLER_REDO_PRE: i32 = 18;
pub const HANDLER_REDO_POST: i32 = 19;
pub const HANDLER_EXTENSION_REPOS_UPDATE_PRE: i32 = 20;
pub const HANDLER_EXTENSION_REPOS_UPDATE_POST: i32 = 21;
pub const HANDLER_EXTENSION_REPOS_UPDATE_SYNC: i32 = 22;
