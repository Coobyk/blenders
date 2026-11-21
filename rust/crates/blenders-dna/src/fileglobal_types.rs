//! Port of `DNA_fileglobal_types.h` — `FileGlobal` (file header data).

use crate::id::NIL;

/// `FileGlobal`.
#[derive(Clone)]
pub struct FileGlobal {
    /// `char subvstr[4]`.
    pub subvstr: [u8; 4],
    pub subversion: i16,
    pub minversion: i16,
    pub minsubversion: i16,
    /// `bScreen *curscreen`, `Scene *curscene`, `ViewLayer *cur_view_layer`.
    pub curscreen: usize,
    pub curscene: usize,
    pub cur_view_layer: usize,
    pub fileflags: i32,
    pub globalf: i32,
    pub build_commit_timestamp: u64,
    /// `char build_hash[16]`.
    pub build_hash: [u8; 16],
    /// `char filepath[1024]`.
    pub filepath: [u8; 1024],
    /// `char colorspace_scene_linear_name[64]`.
    pub colorspace_scene_linear_name: [u8; 64],
    pub colorspace_scene_linear_to_xyz: [[f32; 3]; 3],
}
impl Default for FileGlobal {
    fn default() -> Self {
        Self {
            subvstr: [0; 4],
            subversion: 0,
            minversion: 0,
            minsubversion: 0,
            curscreen: NIL,
            curscene: NIL,
            cur_view_layer: NIL,
            fileflags: 0,
            globalf: 0,
            build_commit_timestamp: 0,
            build_hash: [0; 16],
            filepath: [0; 1024],
            colorspace_scene_linear_name: [0; 64],
            colorspace_scene_linear_to_xyz: [[0.0; 3]; 3],
        }
    }
}
