//! Port of `intern/blt_lang.cc` — locale table +
//! `BLT_lang_locale_explode`.

/// `BLT_lang_locale_explode` output pieces.
#[derive(Default)]
pub struct LocaleParts {
    /// `language`/`language_variant`/`country`/`variant`.
    pub language: String,
    pub language_variant: String,
    pub country: String,
    pub variant: String,
}
/// `BLT_lang_locale_explode` (verbatim —
/// xx_YY@variant / ll@variant / ll_VV split).
pub fn blt_lang_locale_explode(locale: &str) -> LocaleParts {
    let mut parts = LocaleParts::default();
    let (lang_part, variant) = match locale.split_once('@') {
        Some((l, v)) => (l, v),
        None => (locale, ""),
    };
    parts.variant = variant.to_string();
    match lang_part.split_once('_') {
        Some((lang, rest)) => {
            parts.language = lang.to_string();
            if rest.chars().all(|c| c.is_uppercase()) {
                parts.country = rest.to_string();
            } else {
                parts.language_variant = rest.to_string();
            }
        }
        None => {
            parts.language = lang_part.to_string();
        }
    }
    parts
}
