//! Port of `intern/packedFile.cc` — `PackedFile` seek/read,
//! count, write-to-file, compare-to-file, unpack path generation,
//! `BKE_packedfile_id_check`.
//!
//! `Main` iteration (pack/unpack all) is expressed over injected
//! ID lists; `BLI_path_abs`/image-type helpers are injected.

use blenders_dna::packedfile_types::PackedFile;

/// libc `SEEK_*`.
pub const SEEK_SET: i32 = 0;
/// `SEEK_CUR`.
pub const SEEK_CUR: i32 = 1;
/// `SEEK_END`.
pub const SEEK_END: i32 = 2;

/// `BKE_packedfile_seek`.
pub fn bke_packedfile_seek(pf: &mut PackedFile, offset: i32, whence: i32) -> i32 {
    let oldseek = pf.seek;
    let mut seek = match whence {
        SEEK_CUR => oldseek + offset,
        SEEK_END => pf.size + offset,
        SEEK_SET => offset,
        _ => return -1,
    };
    if seek < 0 {
        seek = 0;
    } else if seek > pf.size {
        seek = pf.size;
    }
    pf.seek = seek;
    oldseek
}

/// `BKE_packedfile_rewind`.
pub fn bke_packedfile_rewind(pf: &mut PackedFile) {
    bke_packedfile_seek(pf, 0, SEEK_SET);
}

/// `BKE_packedfile_read` — returns the bytes read (None on error).
pub fn bke_packedfile_read(pf: &mut PackedFile, size: i32) -> Option<Vec<u8>> {
    if size < 0 {
        return None;
    }
    let size = (size + pf.seek > pf.size)
        .then(|| pf.size - pf.seek)
        .unwrap_or(size)
        .max(0);
    let start = pf.seek as usize;
    let end = (pf.seek + size) as usize;
    let out = pf.data.get(start..end)?.to_vec();
    pf.seek += size;
    Some(out)
}

/// `PackedFileCount`.
#[derive(Clone, Copy, Default)]
pub struct PackedFileCount {
    /// `individual_files`.
    pub individual_files: i32,
    /// `bakes`.
    pub bakes: i32,
}

/// `BKE_packedfile_count_all` — counts packed files over injected
/// per-list iterators: (has_packedfile, is_linked) per ID, plus
/// node-modifier bake packed flags.
pub fn bke_packedfile_count_all(
    ids: &dyn Fn() -> Vec<(bool, bool)>,
    node_modifier_bakes: &dyn Fn() -> Vec<bool>,
) -> PackedFileCount {
    let mut count = PackedFileCount::default();
    for (has_packedfile, is_linked) in ids() {
        if has_packedfile && !is_linked {
            count.individual_files += 1;
        }
    }
    for packed in node_modifier_bakes() {
        if packed {
            count.bakes += 1;
        }
    }
    count
}

/// `BKE_packedfile_free`.
pub fn bke_packedfile_free(pf: &mut PackedFile) {
    pf.data = Vec::new();
    pf.size = 0;
    pf.seek = 0;
    pf.sharing_info = 0;
}

/// `BKE_packedfile_write_to_file` — `path_abs` resolves `//`
/// relative paths; `reports` collects (level, msg).
pub fn bke_packedfile_write_to_file(
    reports: &mut dyn FnMut(&str, &str),
    ref_file_name: &str,
    filepath_rel: &str,
    path_abs: &dyn Fn(&str, &str) -> String,
    pf: &PackedFile,
) -> i32 {
    let filepath = path_abs(filepath_rel, ref_file_name);
    let mut remove_tmp = false;
    let mut filepath_temp = String::new();

    if std::path::Path::new(&filepath).exists() {
        for number in 1..=999 {
            filepath_temp = format!("{}.{:03}_", filepath, number);
            if !std::path::Path::new(&filepath_temp).exists() {
                if std::fs::copy(&filepath, &filepath_temp).is_ok() {
                    remove_tmp = true;
                }
                break;
            }
        }
    }

    if let Some(parent) = std::path::Path::new(&filepath).parent() {
        let _ = std::fs::create_dir_all(parent);
    }

    let mut ret_value = 0;
    match std::fs::write(&filepath, &pf.data[..pf.size as usize]) {
        Ok(()) => {
            reports("info", &format!("Saved packed file to: {}", filepath));
        }
        Err(_) => {
            reports("error", &format!("Error creating file '{}'", filepath));
            ret_value = -1;
        }
    }

    if remove_tmp {
        if ret_value == -1 {
            if std::fs::rename(&filepath_temp, &filepath).is_err() {
                reports(
                    "error",
                    &format!(
                        "Error restoring temp file (check files '{}' '{}')",
                        filepath_temp, filepath
                    ),
                );
            }
        } else if std::fs::remove_file(&filepath_temp).is_err() {
            reports("error", &format!("Error deleting '{}' (ignored)", filepath_temp));
        }
    }
    ret_value
}

/// `ePF_FileCompare`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PF_FileCompare {
    /// `PF_CMP_EQUAL`.
    Equal,
    /// `PF_CMP_DIFFERS`.
    Differs,
    /// `PF_CMP_NOFILE`.
    NoFile,
}

/// `BKE_packedfile_compare_to_file`.
pub fn bke_packedfile_compare_to_file(
    ref_file_name: &str,
    filepath_rel: &str,
    path_abs: &dyn Fn(&str, &str) -> String,
    pf: &PackedFile,
) -> PF_FileCompare {
    let filepath = path_abs(filepath_rel, ref_file_name);
    let Ok(meta) = std::fs::metadata(&filepath) else {
        return PF_FileCompare::NoFile;
    };
    if meta.len() as i32 != pf.size {
        return PF_FileCompare::Differs;
    }
    let data = match std::fs::read(&filepath) {
        Ok(d) => d,
        Err(_) => return PF_FileCompare::NoFile,
    };
    if data == pf.data[..pf.size as usize] {
        PF_FileCompare::Equal
    } else {
        PF_FileCompare::Differs
    }
}

/// `unpack_generate_paths` — returns (abspath, relpath).
/// `split_dir_file`/`path_ext_ensure` are injected so image tiling
/// and extension rules stay in their modules.
pub fn unpack_generate_paths(
    filepath: &str,
    id_name: &str,
    id_type: u16,
    mut tile_suffix: impl FnMut() -> String,
    mut ext_ensure: impl FnMut(&mut String),
    path_abs: &dyn Fn(&str, &str) -> String,
    path_rel: &dyn Fn(&str, &str) -> String,
) -> (String, String) {
    let split = filepath.rfind('/').map(|i| i + 1).unwrap_or(0);
    let (dir, mut filename) = (filepath[..split].to_string(), filepath[split..].to_string());
    if filename.is_empty() {
        /* `ID_IM` == GS "IM". */
        filename = id_name.get(2..).unwrap_or(id_name).to_string();
        if id_type == (('I' as u16) << 8 | 'M' as u16) {
            filename.push_str(&tile_suffix());
            ext_ensure(&mut filename);
        }
    }
    let abspath = path_abs(&format!("{}{}", dir, filename), "");
    let relpath = path_rel(&abspath, "");
    (abspath, relpath)
}

/// `BKE_packedfile_id_check` — returns true when the ID type
/// supports packing (`ELEM` over the C switch).
pub fn bke_packedfile_id_check(id_type: u16) -> bool {
    /* GS(id->name): 'IM','VF','SO','VO','LI' + 'GR' (grease pencil)
     * + any ID with `id->library` (LI handled upstream). */
    matches!(
        id_type,
        t if t == (b'I' as u16) << 8 | b'M' as u16
            || t == (b'V' as u16) << 8 | b'F' as u16
            || t == (b'S' as u16) << 8 | b'O' as u16
            || t == (b'V' as u16) << 8 | b'O' as u16
            || t == (b'L' as u16) << 8 | b'I' as u16
            || t == (b'G' as u16) << 8 | b'D' as u16
    )
}

/// `BKE_packedfile_blend_write` — writes {size, data} through the
/// injected writer fns.
pub fn bke_packedfile_blend_write(
    pf: &PackedFile,
    write_int: &mut dyn FnMut(i32),
    write_raw: &mut dyn FnMut(&[u8]),
) {
    write_int(pf.size);
    write_raw(&pf.data[..pf.size as usize]);
}

/// `BKE_packedfile_blend_read` — reads through injected reader
/// fns; `filepath` is retained for parity with C error paths.
pub fn bke_packedfile_blend_read(
    read_int: &mut dyn FnMut() -> i32,
    read_raw: &mut dyn FnMut(i32) -> Vec<u8>,
) -> PackedFile {
    let size = read_int();
    let data = read_raw(size);
    PackedFile {
        size,
        seek: 0,
        data,
        sharing_info: 0,
    }
}
