/// Port of `blendthumb_win32.cc` —
/// Windows shell thumbnail provider.
/// `BlendThumbWin32` (verbatim — COM
/// IThumbnailProvider impl).
#[derive(Default)]
pub struct BlendThumbWin32 {
    /// `handle`.
    pub handle: usize,
}
