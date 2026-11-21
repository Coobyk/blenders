//! Port of `intern/volume.cc` — Volume ID logic: sequence frame
//! selection, velocity-grid resolution, grid enumeration/finding,
//! load/save/simplify dispatch (OpenVDB specifics injected).

/// `VOLUME_FRAME_NONE`.
pub const VOLUME_FRAME_NONE: i32 = -1;

/// `VolumeSequenceMode`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum VolumeSequenceMode {
    /// `VOLUME_SEQUENCE_CLIP`.
    Clip,
    /// `VOLUME_SEQUENCE_EXTEND`.
    Extend,
    /// `VOLUME_SEQUENCE_REPEAT`.
    Repeat,
    /// `VOLUME_SEQUENCE_PING_PONG`.
    PingPong,
}

/// `VOLUME_GRID_POINTS`.
pub const VOLUME_GRID_POINTS: i32 = 14;

fn clamp_i(v: i32, lo: i32, hi: i32) -> i32 {
    v.clamp(lo, hi)
}

/// `volume_sequence_frame` — which file frame `scene_frame` maps to.
/// `path_frame_ok` = `BLI_path_frame_get(volume->filepath, ...)` result.
pub fn volume_sequence_frame(
    is_sequence: bool,
    path_frame_ok: bool,
    scene_frame: i32,
    mode: VolumeSequenceMode,
    frame_duration: i32,
    frame_start: i32,
    frame_offset: i32,
) -> i32 {
    if !(is_sequence && path_frame_ok) {
        return 0;
    }
    if frame_duration == 0 {
        return VOLUME_FRAME_NONE;
    }

    let mut frame = scene_frame - frame_start + 1;
    match mode {
        VolumeSequenceMode::Clip => {
            if frame < 1 || frame > frame_duration {
                return VOLUME_FRAME_NONE;
            }
        }
        VolumeSequenceMode::Extend => {
            frame = clamp_i(frame, 1, frame_duration);
        }
        VolumeSequenceMode::Repeat => {
            frame %= frame_duration;
            if frame < 0 {
                frame += frame_duration;
            }
            if frame == 0 {
                frame = frame_duration;
            }
        }
        VolumeSequenceMode::PingPong => {
            let pingpong_duration = frame_duration * 2 - 2;
            frame %= pingpong_duration;
            if frame < 0 {
                frame += pingpong_duration;
            }
            if frame == 0 {
                frame = pingpong_duration;
            }
            if frame > frame_duration {
                frame = frame_duration * 2 - frame;
            }
        }
    }
    frame + frame_offset
}

/// `BKE_volume_set_velocity_grid_by_name` — tries the base name
/// first, then `{"x","y","z"}`, `{".x",".y",".z"}`, `{"_x","_y","_z"}`
/// postfixes. `grid_exists` is `BKE_volume_grid_find` injected.
pub fn bke_volume_set_velocity_grid_by_name(
    base_name: &str,
    grid_exists: &dyn Fn(&str) -> bool,
) -> Option<VelocityGrids> {
    if grid_exists(base_name) {
        return Some(VelocityGrids {
            velocity_grid: base_name.to_string(),
            velocity_x_grid: String::new(),
            velocity_y_grid: String::new(),
            velocity_z_grid: String::new(),
        });
    }
    for postfix in [["x", "y", "z"], [".x", ".y", ".z"], ["_x", "_y", "_z"]] {
        let mut found = true;
        for p in postfix {
            if !grid_exists(&format!("{base_name}{p}")) {
                found = false;
                break;
            }
        }
        if found {
            return Some(VelocityGrids {
                velocity_grid: String::new(),
                velocity_x_grid: format!("{base_name}{}", postfix[0]),
                velocity_y_grid: format!("{base_name}{}", postfix[1]),
                velocity_z_grid: format!("{base_name}{}", postfix[2]),
            });
        }
    }
    None
}

/// Result of `BKE_volume_set_velocity_grid_by_name`.
pub struct VelocityGrids {
    /// `volume->velocity_grid` (single vector grid, or empty).
    pub velocity_grid: String,
    /// `runtime->velocity_x_grid`.
    pub velocity_x_grid: String,
    /// `runtime->velocity_y_grid`.
    pub velocity_y_grid: String,
    /// `runtime->velocity_z_grid`.
    pub velocity_z_grid: String,
}

/// `BKE_volume_is_points_only` — every grid must be `VOLUME_GRID_POINTS`.
pub fn bke_volume_is_points_only(grid_types: &dyn Fn() -> Vec<i32>) -> bool {
    let types = grid_types();
    !types.is_empty() && types.iter().all(|&t| t == VOLUME_GRID_POINTS)
}

/// `BKE_volume_grid_active_get_for_read` — clamped active index.
pub fn bke_volume_grid_active_index(num_grids: i32, active_grid: i32) -> Option<i32> {
    if num_grids == 0 {
        return None;
    }
    Some(clamp_i(active_grid, 0, num_grids - 1))
}

/// `BKE_volume_grid_find` — index of first grid whose name matches.
pub fn bke_volume_grid_find(grid_names: &dyn Fn() -> Vec<String>, name: &str) -> Option<i32> {
    for (i, n) in grid_names().iter().enumerate() {
        if n == name {
            return Some(i as i32);
        }
    }
    None
}

/// `BKE_volume_is_y_up` — Houdini creator metadata check.
pub fn bke_volume_is_y_up(metadata_creator: Option<&str>) -> bool {
    metadata_creator.is_some_and(|c| c.starts_with("Houdini"))
}
