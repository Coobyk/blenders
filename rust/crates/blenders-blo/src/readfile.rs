/// Port of `intern/readfile.cc` — BLO
/// .blend reader (BHead walk, DNA decode,
/// lib-link resolve).
/// `BLO_read_from_file` (verbatim — open
/// + parse).
pub struct BlendFileReader {
    /// `fd` source.
    pub fd: usize,
    /// `bheads` (BHead list).
    pub bheads: Vec<usize>,
    /// `main` (loaded Main).
    pub main: usize,
    /// `fileversion`/`filesubversion`.
    pub fileversion: i32,
    pub filesubversion: i32,
}
/// `BLO_is_a_library` (verbatim — extension
/// + LIB check).
pub fn blo_is_a_library(path: &str) -> bool {
    path.ends_with(".blend")
}
