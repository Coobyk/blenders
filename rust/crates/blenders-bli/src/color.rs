//! Port of `BLI_color.cc` — `blender::color::` enum display names
//! (`eAlpha`, `eSpace`) and `ColorRGBA` printing.

/// `color::eAlpha`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Alpha {
    /// `eAlpha::Straight`.
    Straight,
    /// `eAlpha::Premultiplied`.
    Premultiplied,
}

impl std::fmt::Display for Alpha {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Alpha::Straight => f.write_str("Straight"),
            Alpha::Premultiplied => f.write_str("Premultiplied"),
        }
    }
}

/// `color::eSpace`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Space {
    /// `eSpace::Theme`.
    Theme,
    /// `eSpace::SceneLinear`.
    SceneLinear,
    /// `eSpace::SceneLinearByteEncoded`.
    SceneLinearByteEncoded,
}

impl std::fmt::Display for Space {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Space::Theme => f.write_str("Theme"),
            Space::SceneLinear => f.write_str("SceneLinear"),
            Space::SceneLinearByteEncoded => f.write_str("SceneLinearByteEncoded"),
        }
    }
}

/// `ColorRGBA<ChannelStorageType, Space, Alpha>` — `{r,g,b,a}` channels.
#[derive(Clone, Copy, Debug)]
pub struct ColorRgba<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

impl<T: std::fmt::Display> ColorRgba<T> {
    /// `operator<<` — `"SpaceAlpha(r, g, b, a)"`.
    pub fn describe(&self, space: Space, alpha: Alpha) -> String {
        format!(
            "{space}{alpha}({}, {}, {}, {})",
            self.r, self.g, self.b, self.a
        )
    }
}
